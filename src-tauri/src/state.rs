use crate::frontend_message::{send_to_frontend, NewTrafficHeadData, Payload, SendData, Status};
use crate::server::PrintMode;
use crate::traffic::{self, wrap_entries, Body as TrafficBody, Header, Traffic, TrafficHead};
use anyhow::{anyhow, bail, Context, Result};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full};
use indexmap::IndexMap;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};
use tokio::sync::{broadcast, Mutex, Notify};
use uuid::Uuid;

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
    pub traffic: Traffic,
    pub body: Option<TrafficBody>,
    pub traffic_type: String,
}

#[derive(Debug)]
pub struct PausedTrafficInfo {
    pub traffic: Traffic,
    pub body: Option<Bytes>,
    pub notify: Arc<Notify>,
}

#[derive(Debug)]
pub struct State {
    print_mode: PrintMode,
    pub traffics: Mutex<IndexMap<usize, Traffic>>,
    pub traffics_notifier: broadcast::Sender<TrafficHead>,
    pub monitor_running: AtomicBool,
    // 添加断点相关字段
    breakpoints: Mutex<HashMap<String, Breakpoint>>,
    pub paused_traffic: Mutex<HashMap<String, PausedTrafficInfo>>,
    pub pause_notifier: broadcast::Sender<(String, TrafficData)>,
    app_handle: tauri::AppHandle,
    pub monitor_traffic: AtomicBool,
}

impl State {
    pub fn new(print_mode: PrintMode, app_handle: tauri::AppHandle) -> Self {
        let (traffics_notifier, _) = broadcast::channel(128);
        // let (pause_notifier, _) = broadcast::channel(32);
        let (pause_notifier, _) = broadcast::channel(32);
        Self {
            print_mode,
            traffics: Default::default(),
            traffics_notifier,
            monitor_running: AtomicBool::new(false),
            breakpoints: Mutex::new(HashMap::new()),
            paused_traffic: Mutex::new(HashMap::new()),
            pause_notifier,
            app_handle,
            monitor_traffic: AtomicBool::new(false),
        }
    }

    pub async fn resend_traffic(&self, id: usize) -> Result<(), anyhow::Error> {
        let traffics = self.traffics.lock().await;
        let uu_id = Uuid::new_v4().to_string();
        // 获取指定 ID 的流量
        let traffic = traffics
            .get(&id)
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
        println!("on_resend_traffic: {:?}", traffic);

        // 验证必需的字段
        let url = traffic
            .url
            .ok_or_else(|| anyhow::anyhow!("URL is required"))?;
        let method = traffic
            .method
            .ok_or_else(|| anyhow::anyhow!("Method is required"))?;

        let proxy_url = format!("http://127.0.0.1:{}", current_port);

        // 创建代理客户端
        let client = Client::builder().proxy(Proxy::http(&proxy_url)?).build()?;

        // 构建请求
        let mut request_builder = client.request(method.parse()?, &url);

        // 添加请求头
        if let Some(headers) = traffic.modified_headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // 添加请求体
        if let Some(body_content) = traffic.modified_body {
            request_builder = request_builder.body(body_content);
        }

        // 发送请求
        tokio::spawn(async move {
            if let Err(e) = request_builder.send().await {
                println!("请求失败: {}", e);
            }
        });
        //     .await
        //     .context("Failed to send request")?;

        // // 处理响应
        // println!("Response status: {}", response.status());

        // // 读取响应体
        // let body = response.bytes().await?;
        // println!("Response body length: {}", body.len());

        Ok(())
    }

    // 添加方法来迁移数据
    pub async fn migrate_from(&self, old_state: &Arc<State>) {
        // 迁移 traffics
        {
            let mut new_traffics = self.traffics.lock().await;
            let old_traffics = old_state.traffics.lock().await;
            *new_traffics = old_traffics.clone();
        }

        // 迁移断点数据
        {
            let mut new_breakpoints = self.breakpoints.lock().await;
            let old_breakpoints = old_state.breakpoints.lock().await;
            *new_breakpoints = old_breakpoints.clone();
        }

        // let  rx = old_state.traffics_notifier.subscribe();
        // d
    }

    // 断点管理方法
    pub async fn update_breakpoint(
        &self,
        breakpoints_items: Vec<Breakpoint>,
    ) -> Result<(), anyhow::Error> {
        let mut breakpoints: tokio::sync::MutexGuard<'_, HashMap<String, Breakpoint>> =
            self.breakpoints.lock().await;
        for breakpoint_item in breakpoints_items {
            breakpoints.insert(breakpoint_item.id.clone(), breakpoint_item);
        }
        Ok(())
    }

    pub async fn remove_breakpoint(&self, ids: &Vec<String>) -> Result<(), anyhow::Error> {
        let mut breakpoints = self.breakpoints.lock().await;
        for id in ids {
            breakpoints.remove(*&id);
        }
        Ok(())
    }

    // 检查流量是否匹配断点
    pub async fn check_breakpoints(
        &self,
        traffic: &Traffic,
        traffic_type: String,
    ) -> Option<(Vec<Breakpoint>, BreakpointMatchResult)> {
        let breakpoints = self.breakpoints.lock().await;
        let mut true_breakpoints = Vec::new();

        for breakpoint in breakpoints.values() {
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

    pub async fn get_breakpoints(&self) -> Vec<Breakpoint> {
        let breakpoints = self.breakpoints.lock().await;
        breakpoints.values().cloned().collect()
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

                if let Some(new_headers) = modification.modified_headers {
                    if modification.modified_type.as_str() == "request" {
                        if let Some(req_headers) = &mut traffic.req_headers {
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
                        if let Some(res_headers) = &mut traffic.res_headers {
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
                    traffic.uri = new_url;
                }

                if let Some(new_method) = modification.method {
                    // 修改 请求方法
                    traffic.method = new_method;
                }

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
        id: usize,
    ) -> Result<TrafficHead, String> {
        if !traffic.valid {
            return Err("Invalid traffic".to_string());
        }
        let head = traffic.head(id);
        let traffic_head_data = NewTrafficHeadData::new(&head);

        send_to_frontend(traffic_head_data.send_data, &self.app_handle);
        std::result::Result::Ok(head)
    }

    pub async fn add_traffic(&self, traffic: Traffic) -> Result<TrafficHead, String> {
        let mut traffics = self.traffics.lock().await;
        let id = traffics.len() + 1;
        let head = self.create_traffic_head(&traffic, id).await?;
        traffics.insert(id, traffic);

        std::result::Result::Ok(head)
    }

    // 继续执行被暂停的流量
    pub async fn continue_traffic(&self, id: &str) -> Result<(), anyhow::Error> {
        println!("开始执行 continue_traffic");

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

    pub async fn done_traffic(&self, head_id: usize, raw_size: u64) {
        let mut traffics = self.traffics.lock().await;
        // let Some((id, traffic)) = traffics.iter_mut().find(|(_, v)| v.gid == gid) else {
        //     return;
        // };
        let traffic = traffics.get_mut(&head_id).unwrap();

        match traffic.status {
            Some(status) => {
                if status >= 400 {
                    traffic.set_transaction_state(traffic::TransactionState::Failed);
                } else {
                    traffic.set_transaction_state(traffic::TransactionState::Completed);
                }
            }
            None => {
                traffic.set_transaction_state(traffic::TransactionState::Failed);
            }
        }

        traffic.uncompress_res_file().await;
        traffic.done_res_body(raw_size);

        self.create_traffic_head(&traffic, head_id).await.unwrap();

        // let traffic_head_data = NewTrafficHeadData::new(&head);

        // send_to_frontend(traffic_head_data.send_data, &self.app_handle);
        // // let _ = self.traffics_notifier.send(head);
        match self.print_mode {
            PrintMode::Nothing => {}
            PrintMode::Oneline => {
                println!("# {}", traffic.oneline());
            }
            PrintMode::Markdown => {
                println!("{}", traffic.markdown().await);
            }
        }
    }

    pub async fn get_traffic(&self, id: usize) -> Option<Traffic> {
        let traffics = self.traffics.lock().await;
        traffics.get(&id).cloned()
    }

    pub fn subscribe_traffics(&self) -> broadcast::Receiver<TrafficHead> {
        self.traffics_notifier.subscribe()
    }

    pub async fn list_heads(&self) -> Vec<TrafficHead> {
        let traffics = self.traffics.lock().await;
        traffics
            .iter()
            .map(|(id, traffic)| traffic.head(*id))
            .collect()
    }

    pub async fn export_traffic(&self, id: usize, format: &str) -> Result<(String, &'static str)> {
        let traffic = self
            .get_traffic(id)
            .await
            .ok_or_else(|| anyhow!("Not found traffic {id}"))?;
        // println!("traffic2222:{:#?}", traffic);
        traffic.export(format).await
    }

    pub async fn export_all_traffics(&self, format: &str) -> Result<(String, &'static str)> {
        let traffics = self.traffics.lock().await;
        match format {
            "markdown" => {
                let output =
                    futures_util::future::join_all(traffics.iter().map(|(_, v)| v.markdown()))
                        .await
                        .into_iter()
                        .collect::<Vec<String>>()
                        .join("\n\n");
                Ok((output, "text/markdown; charset=UTF-8"))
            }
            "har" => {
                let values: Vec<Value> =
                    futures_util::future::join_all(traffics.iter().map(|(_, v)| v.har_entry()))
                        .await
                        .into_iter()
                        .flatten()
                        .collect();
                let json_output = wrap_entries(values);
                let output = serde_json::to_string_pretty(&json_output)?;
                Ok((output, "application/json; charset=UTF-8"))
            }
            "curl" => {
                let output = futures_util::future::join_all(traffics.iter().map(|(_, v)| v.curl()))
                    .await
                    .into_iter()
                    .collect::<Vec<String>>()
                    .join("\n\n");
                Ok((output, "text/plain; charset=UTF-8"))
            }
            "json" => {
                let values = futures_util::future::join_all(traffics.iter().map(|(_, v)| v.json()))
                    .await
                    .into_iter()
                    .collect::<Vec<Value>>();
                let output = serde_json::to_string_pretty(&values)?;
                Ok((output, "application/json; charset=UTF-8"))
            }
            "" => {
                let values = traffics
                    .iter()
                    .map(|(id, traffic)| traffic.head(*id))
                    .collect::<Vec<TrafficHead>>();
                let output = serde_json::to_string_pretty(&values)?;
                Ok((output, "application/json; charset=UTF-8"))
            }
            _ => bail!("Unsupported format: {}", format),
        }
    }
}

// 调试器命令
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DebuggerCommand {
    #[serde(rename = "update_breakpoint")]
    UpdateBreakpoint { breakpoints: Vec<Breakpoint> },
    #[serde(rename = "remove_breakpoint")]
    RemoveBreakpoint { ids: Vec<String> },
    #[serde(rename = "continue")]
    Continue { id: String },
    #[serde(rename = "list_breakpoints")]
    ListBreakpoints,
    // 可以添加更多命令...
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
