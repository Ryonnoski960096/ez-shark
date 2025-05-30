use crate::extract_mime;
use crate::frontend_message::{send_to_frontend, NewTrafficHeadData, Payload, SendData, Status};
use crate::server::{PrintMode, Server};
use crate::traffic::{
    self, string_to_body_hex, wrap_entries, Body as TrafficBody, Header, Headers, SearchQuery,
    Traffic, TrafficHead, TransactionState,
};
use crate::utils::to_ext_name;
use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose;
use bytes::Bytes;
use http::{Method, Uri};
use http_body_util::Full;
use hyper_proxy2::{Intercept, Proxy, ProxyConnector};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use log::{debug, error};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};
use tauri_plugin_store::StoreBuilder;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::sync::{broadcast, Mutex, Notify};
use uuid::Uuid;

use base64::Engine;
use std::io::Write;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrafficModification {
    pub id: String, // 对应被暂停的流量 ID
    pub modified_type: String,
    pub url: Option<String>,
    pub method: Option<String>,
    pub modified_headers: Option<HashMap<String, String>>,
    pub modified_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Res {
    pub body: Option<String>,
    pub headers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Req {
    pub body: Option<String>,
    pub headers: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BreakpointsConfig {
    breakpoints: HashMap<String, Breakpoint>,
    #[serde(rename = "toolEnabled")]
    tool_enabled: bool,
}
impl BreakpointsConfig {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            tool_enabled: false,
        }
    }
}

impl Default for BreakpointsConfig {
    fn default() -> Self {
        Self {
            breakpoints: HashMap::new(),
            tool_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointConditions {
    pub url: Option<String>,
    pub method: Option<String>,
    pub request: Option<Req>,
    pub response: Option<Res>,
    pub req_enable: bool,
    pub res_enable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub enabled: bool,
    pub conditions: BreakpointConditions,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficData {
    pub traffic: Arc<Traffic>,
    pub body: Option<TrafficBody>,
    pub traffic_type: String,
}

#[derive(Debug)]
pub struct PausedTrafficInfo {
    pub traffic: Arc<Traffic>,
    pub body: Option<Bytes>,
    pub notify: Arc<Notify>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub position: String,
    pub content: String,
    pub keyword_byte_index: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchData {
    pub id: u64,
    pub url: String,
    pub search_item: Vec<SearchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub text: String,
    pub search_data: Vec<SearchData>,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct State {
    print_mode: PrintMode,
    // pub session: RwLock<HashMap<String, Mutex<IndexMap<usize, Arc<Traffic>>>>>,
    pub traffics: Cache<u64, Arc<Traffic>>,
    pub traffics_notifier: broadcast::Sender<TrafficHead>,
    pub monitor_running: AtomicBool,
    // 添加断点相关字段
    pub paused_traffic: Mutex<HashMap<String, PausedTrafficInfo>>,
    pub pause_notifier: broadcast::Sender<(String, TrafficData)>,
    app_handle: tauri::AppHandle,
    pub monitor_traffic: Mutex<String>,
}

impl State {
    pub fn new(print_mode: PrintMode, app_handle: tauri::AppHandle) -> Self {
        let (traffics_notifier, _) = broadcast::channel(128);
        // let (pause_notifier, _) = broadcast::channel(32);
        let (pause_notifier, _) = broadcast::channel(32);
        Self {
            print_mode,
            traffics: Cache::builder().max_capacity(10_000).build(),
            // session: Default::default(),
            traffics_notifier,
            monitor_running: AtomicBool::new(false),
            paused_traffic: Mutex::new(HashMap::new()),
            pause_notifier,
            app_handle,
            monitor_traffic: Mutex::new(String::new()),
        }
    }

    pub async fn set_monitor_traffic(&self, traffic: String) -> Result<(), String> {
        let mut current = self.monitor_traffic.lock().await;
        *current = traffic;
        Ok(())
    }

    pub async fn is_monitor_traffic(&self) -> bool {
        let current = self.monitor_traffic.lock().await;
        // debug!("current: {:#?}", current);
        !current.is_empty()
    }

    pub fn get_current_session(&self) -> String {
        let store_path: PathBuf = PathBuf::from("settings.json");

        let store = StoreBuilder::new(&self.app_handle, store_path)
            .build()
            .expect("store build failed");

        // 从存储中获取 current_session 字符串
        let current_session: String = store
            .get("currentListenSession")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default(); // 如果没有值则返回空字符串
                                  // debug!("currentListenSession: {:#?}", current_session);
        current_session
    }

    pub async fn ez_search_traffic(
        &self,
        key_word: &String,
        session_id: &String,
    ) -> Result<Vec<String>, anyhow::Error> {
        let mut results: Vec<String> = Vec::new();
        // debug!("search_traffic: {:#?}", data);
        if session_id.trim().is_empty() {
            return Ok(results);
        }
        let keyword = key_word.to_string().to_lowercase();

        for (id, traffic) in self.traffics.iter() {
            if traffic.session_id != session_id.to_string() {
                continue;
            }
            let traffic_clone = traffic;
            let uri = traffic_clone.uri.to_lowercase();

            if uri.contains(&keyword) {
                results.push(id.to_string());
            }
        }
        return Ok(results);
    }

    pub async fn search_traffic(
        &self,
        data: SearchQuery,
        session_id: String,
    ) -> Result<Vec<SearchData>, anyhow::Error> {
        let mut results: Vec<SearchData> = Vec::new();
        debug!("search_traffic: {:#?}", data);
        if session_id.trim().is_empty() {
            return Ok(results);
        }

        if data.text.is_empty() {
            return Ok(results);
        }

        // 修改为查找所有匹配项的函数
        let text_matches_all = |field: &str| -> Vec<usize> {
            let lowercase_field = field.to_lowercase();
            let lowercase_search = data.text.to_lowercase();

            lowercase_field
                .match_indices(&lowercase_search)
                .map(|(index, _)| index)
                .collect()
        };

        for (id, traffic) in self.traffics.iter() {
            if traffic.session_id != session_id {
                continue;
            }
            let mut search_item: Vec<SearchItem> = Vec::new();
            // 检查请求URL
            if data.position.request_url {
                let mut byte_index_list: Vec<usize> = Vec::new();
                for byte_index in text_matches_all(&traffic.uri) {
                    byte_index_list.push(byte_index);
                }
                if !byte_index_list.is_empty() {
                    search_item.push(SearchItem {
                        position: "Request URL".to_string(),
                        content: traffic.uri.clone(),
                        keyword_byte_index: byte_index_list,
                    });
                }
            }

            // 检查请求头
            if data.position.request_header {
                if let Some(headers) = &traffic.req_headers {
                    let headers_json = headers.to_json();
                    if headers_json
                        .to_lowercase()
                        .contains(&data.text.to_lowercase())
                    {
                        let mut byte_index_list: Vec<usize> = Vec::new();

                        for byte_index in text_matches_all(&headers_json) {
                            byte_index_list.push(byte_index);
                        }
                        if !byte_index_list.is_empty() {
                            search_item.push(SearchItem {
                                position: "Request Header".to_string(),
                                content: headers_json,
                                keyword_byte_index: byte_index_list,
                            });
                        }
                    }
                }
            }

            // 检查响应头
            if data.position.response_header {
                if let Some(headers) = &traffic.res_headers {
                    let headers_json = headers.to_json();
                    if headers_json
                        .to_lowercase()
                        .contains(&data.text.to_lowercase())
                    {
                        let mut byte_index_list: Vec<usize> = Vec::new();
                        for byte_index in text_matches_all(&headers_json) {
                            byte_index_list.push(byte_index);
                        }
                        if !byte_index_list.is_empty() {
                            search_item.push(SearchItem {
                                position: "Response Header".to_string(),
                                content: headers_json,
                                keyword_byte_index: byte_index_list,
                            });
                        }
                    }
                }
            }

            let (req_body, res_body) = traffic.bodies(false).await;

            // 检查请求体
            if data.position.request_body {
                if let Some(body) = req_body {
                    let mut byte_index_list: Vec<usize> = Vec::new();
                    for byte_index in text_matches_all(&body.value) {
                        byte_index_list.push(byte_index);
                    }
                    if !byte_index_list.is_empty() {
                        search_item.push(SearchItem {
                            position: "Request Body".to_string(),
                            content: body.value,
                            keyword_byte_index: byte_index_list,
                        });
                    }
                }
            }

            // 检查响应体
            if data.position.response_body {
                if let Some(body) = res_body {
                    let mut byte_index_list: Vec<usize> = Vec::new();
                    for byte_index in text_matches_all(&body.value) {
                        byte_index_list.push(byte_index);
                    }
                    if !byte_index_list.is_empty() {
                        search_item.push(SearchItem {
                            position: "Response Body".to_string(),
                            content: body.value,
                            keyword_byte_index: byte_index_list,
                        });
                    }
                }
            }

            if !search_item.is_empty() {
                results.push(SearchData {
                    id: *id,
                    url: traffic.uri.clone(),
                    search_item,
                });
            }
        }

        Ok(results)
    }
    pub async fn resend_traffic(&self, id: u64) -> Result<(), anyhow::Error> {
        let uu_id = Uuid::new_v4().to_string();

        // 获取指定 ID 的流量

        let traffic: Arc<Traffic> = self
            .traffics
            .get(&id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Traffic with id {} not found", id))?;

        let (req_body, _) = traffic.bodies(false).await;

        let traffic_data = TrafficData {
            traffic: traffic.clone(),
            body: req_body,
            traffic_type: String::from("resend"),
        };

        // println!("resend_traffic：{:?}", traffic_data);

        let payload: Payload<(String, TrafficData)> = Payload {
            status: Status::Success,
            message: "重发".to_string(),
            data: Some((uu_id, traffic_data)),
        };

        let send_data = SendData {
            event_name: "resend-traffic".to_string(),
            payload,
        };

        send_to_frontend(send_data, &self.app_handle);

        Ok(())
    }

    pub async fn on_resend_traffic(
        &self,
        traffic: TrafficModification,
        current_port: u16,
    ) -> Result<(), anyhow::Error> {
        // debug!("on_resend_traffic: {:?}", traffic);

        // 验证必需的字段
        let url = traffic
            .url
            .ok_or_else(|| anyhow::anyhow!("URL is required"))?;
        let method = traffic
            .method
            .ok_or_else(|| anyhow::anyhow!("Method is required"))?;

        // 解析请求方法
        let http_method = match method.to_lowercase().as_str() {
            "get" => Method::GET,
            "post" => Method::POST,
            "put" => Method::PUT,
            "delete" => Method::DELETE,
            "head" => Method::HEAD,
            "options" => Method::OPTIONS,
            "patch" => Method::PATCH,
            "trace" => Method::TRACE,
            _ => return Err(anyhow::anyhow!("Invalid method")),
        };

        // 创建 HTTPS 连接器
        let https = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_all_versions()
            .build();

        // 设置代理
        let proxy_uri: Uri = format!("http://127.0.0.1:{}", current_port).parse()?;
        let proxy = Proxy::new(Intercept::All, proxy_uri);

        // 创建代理连接器
        let connector = ProxyConnector::from_proxy(https, proxy)?;

        // 创建客户端
        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .build(connector);

        // 构建请求
        let mut request_builder = hyper::Request::builder().uri(&url).method(http_method);

        // 添加请求头
        if let Some(headers) = &traffic.modified_headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        let request = if let Some(body_content) = traffic.modified_body {
            let body_vec = body_content.as_bytes().to_vec();

            // 确保请求头中的 content-length 与实际长度匹配
            request_builder = request_builder.header("content-length", body_vec.len().to_string());

            // 创建请求
            request_builder.body(Full::new(Bytes::from(body_vec)))?
        } else {
            request_builder.body(Full::<Bytes>::new(Bytes::new()))?
        };

        debug!("发送请求: {:?}", request);

        tokio::spawn(async move {
            match client.request(request).await {
                Ok(response) => {
                    let status = response.status();
                    debug!("请求发送成功，状态码: {}", status);
                }
                Err(e) => error!("请求发送失败: {}", e),
            }
        });

        Ok(())
    }

    // 迁移数据
    pub async fn migrate_from(&self, other_state: &Arc<State>) {
        // 获取对旧结构中 session 的读锁
        let other_traffics = &other_state.traffics;

        // 遍历旧结构中的所有流量
        for (key, other_traffic) in other_traffics {
            self.traffics.insert(*key, other_traffic).await;
        }
    }

    // 检查流量是否匹配断点
    pub async fn check_breakpoints(
        &self,
        breakpoints: BreakpointsConfig,
        traffic: &Traffic,
        traffic_type: String,
    ) -> Option<(Vec<Breakpoint>, BreakpointMatchResult)> {
        if !breakpoints.tool_enabled {
            return None;
        }

        let mut true_breakpoints = Vec::new();

        for breakpoint in breakpoints.breakpoints.values() {
            if !breakpoint.enabled {
                continue;
            }
            let match_result = self.matches_breakpoint(breakpoint, traffic, traffic_type.clone());
            match match_result {
                BreakpointMatchResult::FullMatch => {
                    true_breakpoints.clear();
                    true_breakpoints.push(breakpoint.clone());
                    return Some((true_breakpoints, match_result));
                }
                BreakpointMatchResult::HeaderOnlyMatch => {
                    true_breakpoints.push(breakpoint.clone());
                    continue;
                }
                BreakpointMatchResult::NoMatch => {
                    continue;
                }
            }
        }

        if true_breakpoints.len() > 0 {
            return Some((true_breakpoints, BreakpointMatchResult::HeaderOnlyMatch));
        } else {
            None
        }
    }

    pub async fn check_body_breakpoints(
        &self,
        body: TrafficBody,
        breakpoints: Vec<Breakpoint>,
        breakpoint_type: String,
    ) -> bool {
        for breakpoint in breakpoints {
            if breakpoint_type == String::from("request")
                && body
                    .value
                    .contains(&breakpoint.conditions.request.unwrap().body.unwrap())
            {
                return true;
            } else if breakpoint_type == String::from("response")
                && body
                    .value
                    .contains(&breakpoint.conditions.response.unwrap().body.unwrap())
            {
                return true;
            }
        }
        false
    }

    pub async fn modify_paused_traffic(
        &self,
        modification: TrafficModification,
    ) -> Result<(), anyhow::Error> {
        let mut paused_traffic = self.paused_traffic.lock().await;
        match paused_traffic.get_mut(&modification.id) {
            Some(info) => {
                let traffic = &mut info.traffic;
                let body = &mut info.body;
                let mut traffic_clone = Traffic::clone(&traffic);

                if let Some(new_headers) = modification.modified_headers {
                    if modification.modified_type.as_str() == "request" {
                        if let Some(req_headers) = &mut traffic_clone.req_headers {
                            // 更新或添加新的请求头
                            for (name, value) in new_headers {
                                // 检查是否已存在，存在则更新，不存在则添加
                                if let Some(header) = req_headers
                                    .items
                                    .iter_mut()
                                    .find(|h| h.name.to_lowercase() == name.to_lowercase())
                                {
                                    header.value = value;
                                } else {
                                    req_headers.items.push(Header { name, value });
                                }
                            }
                        };
                    } else {
                        if let Some(res_headers) = &mut traffic_clone.res_headers {
                            // 更新或添加新的请求头
                            for (name, value) in new_headers {
                                // 检查是否已存在，存在则更新，不存在则添加
                                if let Some(header) = res_headers
                                    .items
                                    .iter_mut()
                                    .find(|h| h.name.to_lowercase() == name.to_lowercase())
                                {
                                    header.value = value;
                                } else {
                                    res_headers.items.push(Header { name, value });
                                }
                            }
                        };
                    }
                }

                if let Some(new_url) = modification.url {
                    // 修改 URL
                    traffic_clone.uri = new_url;
                }

                if let Some(new_method) = modification.method {
                    // 修改 请求方法
                    traffic_clone.method = new_method;
                }
                *traffic = Arc::new(traffic_clone);

                if let Some(new_body) = modification.modified_body {
                    // 假设你有一个方法来更新请求体
                    // 这里需要根据你的 Body 类型具体实现
                    *body = Some(Bytes::from(new_body.into_bytes()));
                }

                Ok(())
            }
            None => {
                return Err(anyhow!("Traffic not found"));
            }
        }
    }

    pub async fn create_traffic_head(
        &self,
        traffic: &Traffic,
        id: u64,
        session_id: String,
    ) -> Result<TrafficHead, String> {
        if !traffic.valid {
            return Err("Invalid traffic".to_string());
        }
        let head = traffic.head(id, session_id);
        let traffic_head_data = NewTrafficHeadData::new(&head);

        send_to_frontend(traffic_head_data.send_data, &self.app_handle);
        std::result::Result::Ok(head)
    }

    pub async fn add_traffic(
        &self,
        traffic: Arc<Traffic>,
        session_id: &String,
    ) -> Result<TrafficHead, String> {
        // 创建head
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        // 创建traffic head
        let head = self
            .create_traffic_head(&traffic, id, session_id.to_string())
            .await?;
        self.traffics.insert(id, traffic).await;

        Ok(head)
    }

    // 删除流量
    pub async fn delete_traffic(&self, id: u64) -> Result<(), anyhow::Error> {
        let traffic = self.traffics.remove(&id).await;
        if traffic.is_none() {
            return Err(anyhow::anyhow!("Traffic not found"));
        }
        Ok(())
    }

    // 继续执行被暂停的流量
    pub async fn continue_traffic(&self, id: &str) -> Result<(), anyhow::Error> {
        debug!("开始执行 continue_traffic");

        let notify = {
            let paused = self.paused_traffic.lock().await;
            paused.get(id).map(|info| info.notify.clone())
        };

        if let Some(notify) = notify {
            notify.notify_one();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Traffic not found"))
        }
    }

    // 断点匹配逻辑
    fn matches_breakpoint(
        &self,
        breakpoint: &Breakpoint,
        traffic: &Traffic,
        traffic_type: String,
    ) -> BreakpointMatchResult {
        let conditions = &breakpoint.conditions;

        // 检查是否启用了该类型的断点匹配
        let is_enabled = match traffic_type.as_str() {
            "request" => conditions.req_enable,
            "response" => conditions.res_enable,
            _ => false,
        };

        if !is_enabled {
            return BreakpointMatchResult::NoMatch;
        }

        // 根据流量类型选择匹配的条件和数据
        let (url_match, method_match, headers, condition_headers, condition_body) =
            match traffic_type.as_str() {
                "request" => (
                    conditions
                        .url
                        .as_ref()
                        .map(|url| traffic.uri.contains(url))
                        .unwrap_or(true),
                    conditions
                        .method
                        .as_ref()
                        .map(|method| method == &traffic.method)
                        .unwrap_or(true),
                    traffic.req_headers.clone(),
                    conditions.request.as_ref().and_then(|r| r.headers.clone()),
                    conditions.request.as_ref().and_then(|r| r.body.clone()),
                ),
                "response" => (
                    conditions
                        .url
                        .as_ref()
                        .map(|url| traffic.uri.contains(url))
                        .unwrap_or(true),
                    conditions
                        .method
                        .as_ref()
                        .map(|method| method == &traffic.method)
                        .unwrap_or(true),
                    traffic.res_headers.clone(),
                    conditions.response.as_ref().and_then(|r| r.headers.clone()),
                    conditions.response.as_ref().and_then(|r| r.body.clone()),
                ),
                _ => (false, false, None, None, None),
            };

        // URL 和方法匹配
        if !url_match || !method_match {
            return BreakpointMatchResult::NoMatch;
        }

        // 检查请求/响应头匹配
        let header_match = if let Some(condition_header) = condition_headers {
            headers
                .as_ref()
                .map(|headers| {
                    headers.items.iter().any(|h| {
                        h.name
                            .to_lowercase()
                            .contains(&condition_header.to_lowercase())
                            || h.value
                                .to_lowercase()
                                .contains(&condition_header.to_lowercase())
                    })
                })
                .unwrap_or(false)
        } else {
            true // 没有指定头条件，认为匹配
        };

        // 检查请求/响应体匹配
        let body_match = if let Some(_condition_body) = condition_body {
            // 如果配置了体条件，则返回仅头匹配
            return BreakpointMatchResult::HeaderOnlyMatch;
        } else {
            true // 没有配置体条件，认为匹配
        };

        // 根据匹配结果返回
        match (header_match, body_match) {
            (true, true) => BreakpointMatchResult::FullMatch,
            (true, false) => BreakpointMatchResult::HeaderOnlyMatch,
            _ => BreakpointMatchResult::NoMatch,
        }
    }

    pub async fn done_traffic(&self, head_id: u64, raw_size: u64) {
        let current_session = self.get_current_session();
        let Some(traffic) = self.traffics.get(&head_id).await else {
            error!("流量不存在");
            return;
        };
        let mut traffic_clone = Traffic::clone(&traffic);
        // 设置事务状态
        match traffic.status {
            Some(status) => {
                if status >= 400 {
                    traffic_clone.set_transaction_state(traffic::TransactionState::Failed);
                } else {
                    traffic_clone.set_transaction_state(traffic::TransactionState::Completed);
                }
            }
            None => {
                traffic_clone.set_transaction_state(traffic::TransactionState::Failed);
            }
        }
        traffic_clone.uncompress_res_file().await;
        traffic_clone.done_res_body(raw_size);

        if let Err(e) = self
            .create_traffic_head(&traffic_clone, head_id, current_session)
            .await
        {
            error!("Failed to create traffic head: {}", e);
        }

        // 打印日志
        match self.print_mode {
            PrintMode::Nothing => {}
            PrintMode::Oneline => {
                debug!("# {}", traffic_clone.oneline());
            }
            PrintMode::Markdown => {
                debug!("{}", traffic_clone.markdown().await);
            }
        }
    }

    pub async fn get_traffic(&self, id: u64) -> Result<Arc<Traffic>> {
        Ok(self
            .traffics
            .get(&id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Traffic with id {} not found", id))?)
    }

    pub fn subscribe_traffics(&self) -> broadcast::Receiver<TrafficHead> {
        self.traffics_notifier.subscribe()
    }

    pub async fn list_heads(&self) -> Result<Vec<TrafficHead>> {
        let current_session = self.get_current_session();

        // 将每个流量转换为TrafficHead并收集到Vec中
        Ok(self
            .traffics
            .iter()
            .map(|(id, traffic)| traffic.head(*id, current_session.clone()))
            .collect())
    }

    pub async fn export_traffic(&self, id: u64, format: &str) -> Result<(String, &'static str)> {
        let traffic = self
            .get_traffic(id)
            .await
            .map_err(|_| anyhow!("Not found traffic {id}"))?;
        traffic.export(format).await
    }

    pub async fn export_all_traffics(
        &self,
        format: &str,
        session_id: String,
    ) -> Result<(String, &'static str)> {
        match format {
            "markdown" => {
                let output = futures_util::future::join_all(self.traffics.iter().map(
                    |(_, traffic)| async move {
                        // 使用 async move 和 to_string() 解决生命周期问题
                        traffic.markdown().await.to_string()
                    },
                ))
                .await
                .into_iter()
                .collect::<Vec<String>>()
                .join("\n\n");
                Ok((output, "text/markdown; charset=UTF-8"))
            }
            "har" => {
                let values: Vec<Value> = futures_util::future::join_all(
                    self.traffics
                        .iter()
                        .map(|(_, traffic)| async move { traffic.har_entry().await }),
                )
                .await
                .into_iter()
                .flatten()
                .collect();
                let json_output = wrap_entries(values);
                let output = serde_json::to_string_pretty(&json_output)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize HAR JSON: {}", e))?;
                Ok((output, "application/json; charset=UTF-8"))
            }
            "curl" => {
                let output = futures_util::future::join_all(
                    self.traffics
                        .iter()
                        .map(|(_, traffic)| async move { traffic.curl().await }),
                )
                .await
                .into_iter()
                .collect::<Vec<String>>()
                .join("\n\n");
                Ok((output, "text/plain; charset=UTF-8"))
            }
            "json" => {
                let values = futures_util::future::join_all(
                    self.traffics
                        .iter()
                        .map(|(_, traffic)| async move { traffic.json().await }),
                )
                .await
                .into_iter()
                .collect::<Vec<Value>>();
                let output = serde_json::to_string_pretty(&values)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize JSON: {}", e))?;
                Ok((output, "application/json; charset=UTF-8"))
            }
            "" => {
                let values = self
                    .traffics
                    .iter()
                    .map(|(id, traffic)| traffic.head(*id, session_id.clone()))
                    .collect::<Vec<TrafficHead>>();
                let output = serde_json::to_string_pretty(&values)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize TrafficHead JSON: {}", e))?;
                Ok((output, "application/json; charset=UTF-8"))
            }
            _ => bail!("Unsupported format: {}", format),
        }
    }

    pub async fn import_har(
        &self,
        har_json: serde_json::Value,
        server: &Arc<Server>,
        session_id: String,
    ) -> Result<Vec<TrafficHead>, String> {
        if let Some(entries) = har_json["log"]["entries"].as_array() {
            let mut result_array = Vec::new();

            for (gid, entry) in entries.iter().enumerate() {
                // 处理请求头
                let req_headers = entry["request"]["headers"].as_array().map(|headers| {
                    let items = headers
                        .iter()
                        .map(|h| Header {
                            name: h["name"].as_str().unwrap_or("").to_string(),
                            value: h["value"].as_str().unwrap_or("").to_string(),
                        })
                        .collect::<Vec<Header>>();
                    let size = items.len() as u64;
                    Headers { items, size }
                });

                // 处理响应头
                let res_headers = entry["response"]["headers"].as_array().map(|headers| {
                    let items = headers
                        .iter()
                        .map(|h| Header {
                            name: h["name"].as_str().unwrap_or("").to_string(),
                            value: h["value"].as_str().unwrap_or("").to_string(),
                        })
                        .collect::<Vec<Header>>();
                    let size = items.len() as u64;

                    Headers { items, size }
                });

                // 处理请求体
                let req_body: Option<String> = entry["request"]["postData"]["text"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| {
                        // 如果text为空，尝试处理params
                        entry["request"]["postData"]["params"]
                            .as_array()
                            .map(|params| {
                                params
                                    .iter()
                                    .map(|p| {
                                        format!(
                                            "{}={}",
                                            p["name"].as_str().unwrap_or(""),
                                            p["value"].as_str().unwrap_or("")
                                        )
                                    })
                                    .collect::<Vec<String>>()
                                    .join("&")
                            })
                    });

                // 处理响应体
                let res_body = entry["response"]["content"]["text"]
                    .as_str()
                    .and_then(|text| {
                        // 处理base64编码
                        if entry["response"]["content"]["encoding"]
                            .as_str()
                            .unwrap_or("")
                            == "base64"
                        {
                            // 尝试解码并转换为字符串
                            general_purpose::STANDARD
                                .decode(text)
                                .ok()
                                .and_then(|decoded| String::from_utf8(decoded).ok())
                        } else {
                            Some(text.to_string())
                        }
                    });

                // 解析时间
                let start_time = entry["startedDateTime"]
                    .as_str()
                    .and_then(|t| OffsetDateTime::parse(t, &Rfc3339).ok());

                let end_time: Option<OffsetDateTime> = start_time.map(|start| {
                    start + Duration::from_millis(entry["time"].as_u64().unwrap_or(0))
                });

                // 构建请求URL（包含查询参数）
                let uri = entry["request"]["url"].as_str().unwrap_or("").to_string();

                let req_body_hex = match &req_body {
                    Some(body) => Some(string_to_body_hex(body)),
                    None => None,
                };

                let res_body_hex = match &res_body {
                    Some(body) => Some(string_to_body_hex(body)),
                    None => None,
                };

                // 构建Traffic对象
                let mut traffic = Traffic {
                    gid: gid as u64,
                    session_id: session_id.clone(),
                    uri,
                    method: entry["request"]["method"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    transaction_state: TransactionState::Completed,
                    req_headers,
                    req_body_file: None,
                    req_body_hex,
                    status: entry["response"]["status"].as_u64().map(|s| s as u16),
                    http_version: entry["request"]["httpVersion"]
                        .as_str()
                        .map(|s| s.to_string()),
                    res_headers,
                    res_body_file: None,
                    res_body_hex,
                    res_body_size: entry["response"]["content"]["size"].as_u64(),
                    websocket_id: None,
                    start_time,
                    end_time,
                    error: None,
                    valid: true,
                };

                let s = &*server;
                let temp_dir: &std::path::PathBuf = &s.temp_dir;
                let mime = extract_mime(&traffic.req_headers);
                let ext_name = to_ext_name(mime);
                let path = temp_dir.join(format!("{:05}-req{ext_name}", gid));
                let file = File::create(&path).with_context(|| {
                    format!(
                        "Failed to create file '{}' to store request body",
                        path.display()
                    )
                });
                if let Ok(mut file) = file {
                    if let Some(body) = &req_body {
                        let _ = file.write_all(body.as_bytes()).with_context(|| {
                            format!("Failed to write to file '{}'", path.display())
                        });
                        traffic.set_req_body_file(&path);
                    }
                }

                let mime = extract_mime(&traffic.res_headers);
                let ext_name = to_ext_name(mime);
                let path = temp_dir.join(format!("{:05}-res{ext_name}", gid));
                let file = File::create(&path).with_context(|| {
                    format!(
                        "Failed to create file '{}' to store response body",
                        path.display()
                    )
                });
                if let Ok(mut file) = file {
                    if let Some(body) = &res_body {
                        let _ = file.write_all(body.as_bytes()).with_context(|| {
                            format!("Failed to write to file '{}'", path.display())
                        });
                        traffic.set_res_body_file(&path);
                    }
                }

                traffic.valid = true;

                let id: u64 = self.traffics.entry_count() + 1;
                let head = traffic.head(id, session_id.to_string());
                self.traffics.insert(id, Arc::new(traffic)).await;
                result_array.push(head);
            }

            debug!("Imported {:#?} traffics", result_array);
            return Ok(result_array);
        }
        Err("Invalid HAR file format".to_string())
    }
}

// 调试器命令
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DebuggerCommand {
    #[serde(rename = "continue")]
    Continue { id: String },
    #[serde(rename = "traffic_modification")]
    ModifyTraffic(TrafficModification),
}

#[derive(Debug, Deserialize)]
// 新增一个枚举来表示匹配状态
pub enum BreakpointMatchResult {
    FullMatch,       // 请求头和请求体都匹配
    HeaderOnlyMatch, // 只有请求头匹配
    NoMatch,         // 没有匹配
}
