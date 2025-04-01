// #[macro_use]
// extern crate log;
pub mod frontend_message;
pub mod models;
pub mod rewind;
pub mod server;
pub mod state;
pub mod traffic;
pub mod utils;

use crate::models::{charles, charles::CharlesConverter};
use crate::{
    cert::CertificateAuthority,
    server::{PrintMode, Server, ServerBuilder},
    state::{DebuggerCommand, State as TrafficState},
    traffic::{Body, SearchQuery, Traffic, TrafficHead},
    utils::serialize_option_datetime,
};
use anyhow::Result;

use chrono::{Datelike, Local};

use indexmap::IndexMap;
use log::info;
use models::ExternalProxy;
use serde::Serialize;
use state::{SearchResult, TrafficModification};

use std::{
    fs,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    vec,
};
use tauri::Manager;
use tauri::State;
use tauri_plugin_log::{fern, Target, TargetKind};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreBuilder;

use time::OffsetDateTime;
use tokio::sync::{oneshot, Mutex};
use tokio::{net::TcpListener, time::Duration};
use traffic::{extract_mime, BodyHex, TransactionState};

const APP_NAME: &str = "ez-shark";

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub url: String,
    pub method: String,
    pub status: TransactionState,
    pub code: Option<u16>,
    pub protocol: Option<String>,
    #[serde(serialize_with = "serialize_option_datetime")]
    pub start_time: Option<OffsetDateTime>,
    #[serde(serialize_with = "serialize_option_datetime")]
    pub end_time: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficDetail {
    pub overview: Overview,
    pub req_head_json: Option<String>,
    pub res_head_json: Option<String>,
    pub req_body_hex: Option<Vec<BodyHex>>,
    pub res_body_hex: Option<Vec<BodyHex>>,
    pub req_body: Option<Body>,
    pub res_body: Option<Body>,
}

pub struct ProxyServer {
    ca: Arc<CertificateAuthority>,
    current_port: u16,
    server: Option<Arc<Server>>,
    stop_sender: Option<oneshot::Sender<()>>,
    old_state: Option<Arc<TrafficState>>,
    app_handle: tauri::AppHandle,
}

impl ProxyServer {
    pub fn new(ca: CertificateAuthority, app_handle: tauri::AppHandle) -> Self {
        ProxyServer {
            ca: Arc::new(ca),
            current_port: 0,
            server: None,
            stop_sender: None,
            old_state: None,
            app_handle,
        }
    }

    // 获取当前的 state
    pub fn get_state(&self) -> Option<Arc<TrafficState>> {
        self.server.as_ref().map(|server| server.state())
    }

    // 停止之前的服务
    pub async fn stop_previous_server(&mut self) -> Result<Option<Arc<TrafficState>>, String> {
        // 保存当前状态用于后续恢复
        let old_state = self.server.as_ref().map(|server| server.state());
        self.old_state = old_state.clone();
        // 如果有正在运行的服务，停止它
        if let Some(stop_tx) = self.stop_sender.take() {
            // 克隆 stop_tx 用于错误恢复
            let send_result = stop_tx.send(());

            match send_result {
                Ok(_) => {
                    // 使用动态方式等待服务停止
                    let max_wait_time = Duration::from_secs(5); // 最大等待时间
                    let poll_interval = Duration::from_millis(100); // 轮询间隔
                    let start_time = std::time::Instant::now();

                    // 先将服务实例置为 None，这样弱引用检测才能正常工作
                    let server_instance = self.server.take();
                    self.current_port = 0;

                    // 如果有服务实例，创建一个弱引用来监控它
                    if let Some(server_ref) = server_instance {
                        let weak_server = Arc::downgrade(&server_ref);

                        // 创建一个通知器，用于服务停止时发出信号
                        let shutdown_complete = Arc::new(tokio::sync::Notify::new());
                        let shutdown_complete_clone = shutdown_complete.clone();

                        // 启动一个任务来监控服务引用计数
                        tokio::spawn(async move {
                            // 定期检查服务是否已被释放
                            loop {
                                if weak_server.upgrade().is_none() {
                                    // 服务已被释放，发出通知
                                    shutdown_complete_clone.notify_one();
                                    break;
                                }

                                // 如果引用计数为1（只有我们自己持有），也可以认为服务已停止
                                if weak_server.strong_count() <= 1 {
                                    // 手动释放最后一个引用
                                    drop(server_ref);
                                    shutdown_complete_clone.notify_one();
                                    break;
                                }

                                tokio::time::sleep(poll_interval).await;
                            }
                        });

                        // 等待服务停止或超时
                        tokio::select! {
                            _ = shutdown_complete.notified() => {
                                // 服务已停止
                                log::debug!("服务已正常停止，耗时: {:?}", start_time.elapsed());
                            }
                            _ = tokio::time::sleep(max_wait_time) => {
                                // 超时
                                log::warn!("等待服务停止超时，强制清理资源，耗时: {:?}", start_time.elapsed());
                            }
                        }
                    } else {
                        log::debug!("没有活跃的服务实例需要停止");
                    }

                    Ok(old_state)
                }
                Err(_) => {
                    // 发送失败，说明接收端可能已经关闭
                    // 清理状态
                    self.server = None;
                    self.current_port = 0;

                    Err("Failed to send stop signal: receiver dropped".to_string())
                }
            }
        } else {
            // 没有运行中的服务
            Ok(old_state)
        }
    }
    // 启动新服务
    pub async fn start_new_server(
        &mut self,
        port: u16,
        old_state: Option<Arc<TrafficState>>,
    ) -> Result<String, String> {
        // 检查端口是否被占用
        let ip = IpAddr::from([127, 0, 0, 1]);
        let listener = match TcpListener::bind(SocketAddr::new(ip, port)).await {
            Ok(listener) => listener,
            Err(_) => {
                return Err("Please check if the port is occupied.".to_string());
            }
        };

        // 创建新的服务器
        let server = ServerBuilder::new(Arc::clone(&self.ca), self.app_handle.clone())
            .print_mode(PrintMode::Oneline)
            .build();

        // 迁移旧状态
        if let Some(old_state) = old_state {
            server.state().migrate_from(&old_state).await;
        }

        // 保存新服务器实例
        self.server = Some(server.clone());

        // 启动服务器
        match server.run(listener).await {
            Ok(stop_tx) => {
                info!("HTTP(S) proxy listening at {ip}:{port}");
                self.stop_sender = Some(stop_tx);
                self.current_port = port;

                {
                    let log_dir = self
                        .app_handle
                        .path()
                        .app_log_dir()
                        .expect("Failed to get log directory");

                    info!("Log file path: {:?}", log_dir);
                }

                Ok("Success".to_string())
            }
            Err(e) => {
                self.server = None;
                Err(format!("Error: Failed to start server: {}", e))
            }
        }
    }

    // 重启服务
    pub async fn restart_server(&mut self, port: u16) -> Result<String, String> {
        // 如果端口相同且服务已存在，直接返回
        if self.current_port == port && self.server.is_some() {
            return Ok("Success".to_string());
        }

        // 停止当前服务并获取状态
        let old_state = match self.stop_previous_server().await {
            Ok(state) => state,
            Err(e) => return Err(e),
        };

        // 启动新服务
        self.start_new_server(port, old_state).await
    }

    // 临时暂停服务（保留状态）
    pub async fn pause_server(&mut self) -> Result<Option<Arc<TrafficState>>, String> {
        self.stop_previous_server().await
    }

    // 恢复暂停的服务
    pub async fn resume_server(&mut self, port: u16) -> Result<String, String> {
        let state = self.old_state.clone();
        self.start_new_server(port, state).await
        // match self.old_state {
        //     Some(old_state) => {
        //     }
        //     None => {
        //       Err("No saved state to resume".to_string())
        //     }
        // }
    }

    // 完全停止服务（不保留状态）
    pub async fn shutdown_server(&mut self) -> Result<(), String> {
        match self.stop_previous_server().await {
            Ok(_) => {
                self.server = None;
                self.current_port = 0;
                self.stop_sender = None;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // 停止服务器
    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(stop_tx) = self.stop_sender.take() {
            let _ = stop_tx.send(());
            self.server = None;
            self.current_port = 0;
        }
        Ok(())
    }
}

#[tauri::command]
async fn setting_port(
    port: u16,
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
) -> Result<String, String> {
    if port < 1024 {
        return Err("port is error".to_string());
    }
    let mut proxy_server = proxy_server.lock().await;
    proxy_server
        .restart_server(port)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn change_monitor_traffic(
    monitor_traffic: String,
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let mut current_monitor_traffic = state.monitor_traffic.lock().await;
        *current_monitor_traffic = monitor_traffic;
        return Ok("Success".to_string());
    }
    Err("Not found state".to_string())
}

#[tauri::command]
async fn get_monitor_session_id(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let session_id = state.monitor_traffic.lock().await;
        return Ok(session_id.to_string());
    }
    Err("Not found state".to_string())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command]
//  async fn start_traffic_monitor(
//     window: Window,
//     proxy_server: State<'_, Arc<Mutex<ProxyServer>>>
// ) -> Result<(), String> {
//     let proxy_server = proxy_server.lock().await;
//     if let Some(state) = proxy_server.get_state() {
//         // 检查是否已经运行
//         if state.monitor_running.load(Ordering::SeqCst) {
//             return Ok(());  // 如果已经运行，直接返回
//         }
//         // 设置运行标志
//         state.monitor_running.store(true, Ordering::SeqCst);
//         // 订阅流量通知
//         let mut rx = state.traffics_notifier.subscribe();
//         let window = window.clone();
//         let mut pause_rx = state.pause_notifier.subscribe();
//         // 启动后台任务转发流量
//         tauri::async_runtime::spawn(async move {
//             loop {
//                 tokio::select! {
//                     // 处理普通流量消息
//                     result = rx.recv() => {
//                         match result {
//                             Ok(traffic_head) => {
//                                 if let Err(e) = window.emit("new-traffic", traffic_head) {
//                                     eprintln!("Failed to emit traffic: {}", e);
//                                 }
//                             }
//                             Err(e) => {
//                                 eprintln!("Traffic channel error: {}", e);
//                                 break;
//                             }
//                         }
//                     }
//                     // 处理暂停消息
//                     result = pause_rx.recv() => {
//                         match result {
//                             Ok(traffic) => {
//                                 if let Err(e) = window.emit("pause-traffic", traffic) {
//                                     eprintln!("Failed to emit pause traffic: {}", e);
//                                 }
//                             }
//                             Err(e) => {
//                                 eprintln!("Pause channel error: {}", e);
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//         });
//         return Ok(());
//     }
//     Err("Not found state".to_string())

// }

#[tauri::command]
async fn get_traffic_detail(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    session_id: String,
    id: usize,
) -> Result<TrafficDetail, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let traffic = state
            .get_traffic(id, session_id)
            .await
            .map_err(|e| e.to_string())?;

        let (req_body, res_body) = traffic.bodies(false).await;

        let traffic_detail = TrafficDetail {
            overview: Overview {
                url: traffic.uri.clone(),
                method: traffic.method.clone(),
                code: traffic.status.clone(),
                status: traffic.transaction_state.clone(),
                protocol: traffic.http_version.clone(),
                start_time: traffic.start_time.clone(),
                end_time: traffic.end_time.clone(),
            },
            req_head_json: traffic.req_head_json(),
            res_head_json: traffic.res_head_json(),
            req_body_hex: traffic.req_body_hex.clone(),
            res_body_hex: traffic.res_body_hex.clone(),
            req_body,
            res_body,
        };
        return Ok(traffic_detail);
    }
    Err("Not found state".to_string())
}
#[tauri::command]
async fn handle_debugger_command(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    command: DebuggerCommand,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        match command {
            DebuggerCommand::Continue { id } => match state.continue_traffic(&id).await {
                Ok(_) => return Ok("Success".to_string()),
                Err(e) => return Err(e.to_string()),
            },
            DebuggerCommand::ModifyTraffic(modification) => {
                match state.modify_paused_traffic(modification).await {
                    Ok(_) => return Ok("Success".to_string()),
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
    }
    Err("Not found state".to_string())
    // Ok(())
}

#[tauri::command]
async fn handle_export_traffic(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    session_id: String,
    path: String,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let format = Path::new(&path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "md" => "markdown",
                "har" => "har",
                "sh" => "curl",
                "json" => "json",
                _ => "txt",
            })
            .unwrap_or("txt")
            .to_string();
        let (content, _) = state
            .export_all_traffics(&format, session_id)
            .await
            .map_err(|e| e.to_string())?;

        // 确保父目录存在
        if let Some(parent) = Path::new(&path).parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // 直接写入文件
        fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;

        return Ok("Success".to_string());
    }
    Err("Not found state".to_string())
}

#[tauri::command]
async fn handle_copy_traffic(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    id: usize,
    format: String,
    session_id: String,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let (data, _) = state
            .export_traffic(id, &format, session_id)
            .await
            .map_err(|e| format!("Failed to copy traffic: {}", e))?;

        return Ok(data.to_string());
    }
    Err("Not found state".to_string())
}

#[tauri::command]
async fn open_config_dir(
    app_handle: tauri::AppHandle,
    config_dir: State<'_, PathBuf>,
) -> Result<String, String> {
    // app_handle.m;
    // 打开证书文件
    if !config_dir.exists() {
        return Err("The file does not exist.".into());
    }

    match app_handle
        .opener()
        .open_path(config_dir.to_str().unwrap(), None::<&str>)
    {
        Ok(_) => Ok("Success".to_string()),
        Err(_) => Err("Failed to open config directory".to_string()),
    }
}

#[tauri::command]
async fn import_session(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    session_id: String,
    path: String,
) -> Result<Vec<TrafficHead>, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let file_path = std::path::Path::new(&path);
        if !file_path.exists() {
            return Err("File json not exist".to_string());
        }

        // 读取文件内容
        let file_content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // 直接解析成 Traffic 数组
        let traffic_array: Vec<Traffic> = serde_json::from_str(&file_content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut result_array = Vec::new();

        // 获取session的写锁
        let mut sessions = state.session.write().await;

        // 确保会话存在
        if !sessions.contains_key(&session_id) {
            sessions.insert(session_id.clone(), Mutex::new(IndexMap::new()));
        }

        // 获取指定会话的锁
        let session_traffics = sessions.get(&session_id).unwrap();
        let mut session_traffics_locked = session_traffics.lock().await;

        // 处理每个 Traffic 对象
        for mut traffic in traffic_array {
            traffic.valid = true;
            let id = session_traffics_locked.len() + 1;
            let head = traffic.head(id, session_id.to_string());
            session_traffics_locked.insert(id, Arc::new(traffic));
            result_array.push(head);
        }
        return Ok(result_array);
    } else {
        Err("Not found state".to_string())
    }
}

#[tauri::command]
async fn import_har(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    session_id: String,
    path: String,
) -> Result<Vec<TrafficHead>, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let file_path = std::path::Path::new(&path);

        if !file_path.exists() {
            return Err("HAR 文件不存在".to_string());
        }

        // 读取HAR文件内容
        let file_content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| format!("读取文件失败: {}", e))?;

        // 解析HAR文件
        let har: serde_json::Value =
            serde_json::from_str(&file_content).map_err(|e| format!("解析HAR JSON失败: {}", e))?;

        if let Some(server) = &proxy_server.server {
            Ok(state
                .import_har(har, server, session_id)
                .await
                .map_err(|e| e.to_string())?)
        } else {
            Err("Not found server".to_string())
        }
    } else {
        Err("未找到状态".to_string())
    }
}

#[tauri::command]
async fn import_charles(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    app_handle: tauri::AppHandle,
    session_id: String,
    path: String,
) -> Result<Vec<TrafficHead>, String> {
    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(format!("chls 文件不存在, file_path:{:#?}", file_path));
    }
    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");

    // 创建 StoreBuilder 实例
    let store = StoreBuilder::new(&app_handle, PathBuf::from("settings.json"))
        .build()
        .map_err(|e| e.to_string())?;

    // 获取 charlesPath
    if let Some(charles_path) = store.get("charlesPath") {
        if let Some(charles_path_str) = charles_path.as_str() {
            let charles_converter =
                CharlesConverter::new(charles_path_str).map_err(|err| err.to_string())?;
            let path_resolver = app_handle.path();
            let temp_dir = path_resolver.temp_dir().map_err(|e| e.to_string())?;
            let output_filename = {
                let parts: Vec<&str> = file_name.split('.').collect();
                if !parts.is_empty() {
                    format!("{}.har", parts[parts.len() - 2])
                } else {
                    "output.har".to_string()
                }
            };

            let output_path: PathBuf = temp_dir.join(output_filename);
            if output_path.exists() {
                fs::remove_file(&output_path).map_err(|e| e.to_string())?;
            }
            let har_path = charles_converter
                .convert_to_har(&path, output_path.to_str())
                .map_err(|e| e.to_string())?;
            let result = import_har(
                proxy_server,
                session_id,
                har_path.to_str().unwrap().to_string(),
            )
            .await;
            // 清理临时文件
            if let Err(e) = fs::remove_file(&har_path) {
                eprintln!("Failed to remove temporary file: {}", e);
            }
            if let Ok(res) = result {
                return Ok(res);
            } else {
                return Err(result.unwrap_err());
            }
        }
    }

    Err("Charles 路径未找到".to_string()) // 如果没有找到 charlesPath，返回错误
}

fn get_log_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, tauri::Error> {
    Ok(app_handle.path().app_log_dir()?)
}

fn get_log_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, tauri::Error> {
    let log_dir = get_log_dir(app_handle)?;
    Ok(log_dir.join(format!("{}.log", APP_NAME)))
}

fn cleanup_old_logs(app_handle: &tauri::AppHandle) {
    let today = Local::now().format("%Y%m%d").to_string();

    if let Ok(log_dir) = app_handle.path().app_log_dir() {
        if let Ok(entries) = fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(APP_NAME) && file_name.ends_with(".log") {
                        // 从文件名中提取日期数字部分 (20250223)
                        if let Some(date_str) =
                            file_name.get(APP_NAME.len() + 1..APP_NAME.len() + 9)
                        {
                            // 如果日期小于今天，就删除
                            if date_str < today.as_str() {
                                let _ = fs::remove_file(entry.path());
                                log::info!("Removed old log file: {}", file_name);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tauri::command]
async fn get_log_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let log_path = get_log_file_path(&app_handle).map_err(|e| e.to_string())?;
    Ok(log_path.to_string_lossy().replace('\\', "/"))
}

#[tauri::command]
async fn resend(
    id: usize,
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let _ = state.resend_traffic(id).await;
        return Ok("Success".to_string());
    } else {
        Err("Not found state".to_string())
    }
}

#[tauri::command]
async fn on_resend(
    data: TrafficModification,
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let current_port = proxy_server.current_port;
        let _ = state.on_resend_traffic(data, current_port).await;

        return Ok("Success".to_string());
    } else {
        Err("Not found state".to_string())
    }
}

#[tauri::command]
async fn search(
    data: SearchQuery,
    session_id: String,
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
) -> Result<SearchResult, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        let text = data.text.clone();
        match state.search_traffic(data, session_id).await {
            Ok(search_result) => {
                return Ok(SearchResult {
                    text,
                    search_data: search_result,
                });
            }
            Err(_) => {
                return Err("Search traffic failed".to_string());
            }
        }
    } else {
        Err("Not found state".to_string())
    }
}

#[tauri::command]
async fn delete_traffic(
    proxy_server: State<'_, Arc<Mutex<ProxyServer>>>,
    session_id: String,
    ids: Vec<usize>,
) -> Result<String, String> {
    let proxy_server = proxy_server.lock().await;
    if let Some(state) = proxy_server.get_state() {
        for id in ids {
            state
                .delete_traffic(id, &session_id)
                .await
                .expect("delete traffic failed");
        }
        return Ok("Success".to_string());
    }
    Err("Not found state".to_string())
}

#[tauri::command]
async fn is_charles_running() -> bool {
    charles::is_charles_running()
}

#[tauri::command]
async fn kill_charles() -> Result<bool, String> {
    charles::kill_charles_async().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(ca: CertificateAuthority, config_dir: PathBuf) {
    let date = Local::now();
    // 生成带日期的日志文件名
    let log_file_name = format!(
        "{}_{}{:02}{:02}",
        APP_NAME,
        date.year(),
        date.month(),
        date.day()
    );

    tauri::Builder::default()
        // .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let path: PathBuf = PathBuf::from("settings.json");
            let store = StoreBuilder::new(app.handle(), path).build()?;

            if store.get("externalProxy").is_none() {
                let initial_config = ExternalProxy::new();
                if let Ok(json_value) = serde_json::to_value(initial_config) {
                    let _ = store.set("externalProxy", json_value);
                    let _ = store.save();
                }
            }

            let store = Arc::new(Mutex::new(store));
            app.manage(store.clone()); // 确保管理 store 状态

            let proxy_server = ProxyServer::new(ca, app.handle().clone());
            let proxy_server = Arc::new(Mutex::new(proxy_server));
            app.manage(proxy_server);

            cleanup_old_logs(&app.handle());
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir {
                        file_name: Some(log_file_name),
                    }),
                ])
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .with_colors(
                    fern::colors::ColoredLevelConfig::new()
                        .info(fern::colors::Color::Green)
                        .warn(fern::colors::Color::Yellow)
                        .error(fern::colors::Color::Red)
                        .debug(fern::colors::Color::Blue)
                        .trace(fern::colors::Color::White),
                )
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(config_dir)
        // .manage(proxy_server)
        .invoke_handler(tauri::generate_handler![
            // start_traffic_monitor,
            get_traffic_detail,
            handle_debugger_command,
            handle_export_traffic,
            handle_copy_traffic,
            open_config_dir,
            setting_port,
            import_session,
            import_har,
            import_charles,
            change_monitor_traffic,
            get_monitor_session_id,
            get_log_path,
            resend,
            on_resend,
            search,
            delete_traffic,
            is_charles_running,
            kill_charles
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod cert;
