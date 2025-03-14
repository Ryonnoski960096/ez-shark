use crate::frontend_message::{send_to_frontend, Payload, SendData, Status};
use crate::models::crypto::CRYPTO_SERVICE;
use crate::models::external_proxy::check_proxy_config;
use crate::models::{get_proxy_config, ExternalProxy};
use crate::traffic::{self, bytes_to_hex_structs, TrafficHead};
use crate::{
    cert::CertificateAuthority,
    rewind::Rewind,
    state::{BreakpointMatchResult, PausedTrafficInfo, State, TrafficData},
    traffic::{extract_mime, Body as TrafficBody, Header, Traffic, TransactionState},
    utils::*,
};
use anyhow::{anyhow, Context as _, Result};
use async_compression::tokio::{
    bufread::{BrotliDecoder, DeflateDecoder, GzipDecoder, ZstdDecoder},
    write::{BrotliEncoder, DeflateEncoder, GzipEncoder, ZstdEncoder},
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bytes::Bytes;
use headers::Authorization;
use http::HeaderMap;
use http::{
    header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE},
    uri::{Authority, Scheme},
    HeaderValue,
};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    body::{Body, Frame, Incoming},
    header::HeaderName,
    service::service_fn,
    Method, StatusCode, Uri,
};
use hyper_proxy2::{Intercept, Proxy, ProxyConnector};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::connect::{Connect, HttpConnector};
use hyper_util::{
    client::legacy::Client,
    rt::{TokioExecutor, TokioIo},
};
use log::{debug, error, info};
use pin_project_lite::pin_project;
use serde::ser::StdError;
use serde::Serialize;
use serde_json::de;
use std::error::Error;
use std::marker::Unpin;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{
    collections::HashMap,
    convert::Infallible,
    fs::File,
    io::Write,
    net::SocketAddr,
    path::PathBuf,
    pin::Pin,
    process,
    sync::Arc,
    task::{Context, Poll},
};
use time::OffsetDateTime;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{oneshot, Mutex, Notify},
};
use tokio_graceful::Shutdown;
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;
// type TrafficDoneSender = mpsc::UnboundedSender<(usize, u64)>;
type Request = hyper::Request<Incoming>;
type Response = hyper::Response<BoxBody<Bytes, anyhow::Error>>;

pub type TrafficTuple = (Option<String>, TrafficData);

#[derive(Debug, Clone, Serialize)]
pub struct NewPauseTrafficData {
    pub send_data: SendData<TrafficTuple>,
}

impl NewPauseTrafficData {
    pub fn new(traffic_tuple: TrafficTuple) -> Self {
        // 先创建 Payload
        let payload = Payload {
            status: Status::Success,
            message: "暂停规则匹配".to_string(),
            data: Some(traffic_tuple),
        };

        // 创建完整的结构体
        Self {
            send_data: SendData {
                event_name: "pause-traffic".to_string(),
                payload,
            },
        }
    }
}

pub struct ServerBuilder {
    ca: Arc<CertificateAuthority>,
    print_mode: PrintMode,
    app_handle: tauri::AppHandle,
}

impl ServerBuilder {
    pub fn new(ca: Arc<CertificateAuthority>, app_handle: tauri::AppHandle) -> Self {
        Self {
            ca,
            print_mode: PrintMode::Markdown,
            app_handle,
        }
    }

    // pub fn reverse_proxy_url(mut self, reverse_proxy_url: Option<String>) -> Self {
    //     self.reverse_proxy_url = reverse_proxy_url;
    //     self
    // }

    // pub fn title_filters(mut self, filters: Vec<TitleFilter>) -> Self {
    //     self.title_filters = filters;
    //     self
    // }
    // pub fn mime_filters(mut self, mime_filters: Vec<String>) -> Self {
    //     self.mime_filters = mime_filters;
    //     self
    // }

    // pub fn web(mut self, web: bool) -> Self {
    //     self.web = web;
    //     self
    // }

    pub fn print_mode(mut self, print_mode: PrintMode) -> Self {
        self.print_mode = print_mode;
        self
    }

    pub fn build(self) -> Arc<Server> {
        let temp_dir = std::env::temp_dir().join(format!("ezshark-{}", process::id()));
        info!("temp_dir={}", temp_dir.display(),);
        Arc::new(Server {
            ca: self.ca,
            // reverse_proxy_url: self.reverse_proxy_url,
            // title_filters: self.title_filters,
            // mime_filters: self.mime_filters,
            // web: self.web,
            state: Arc::new(State::new(self.print_mode, self.app_handle.clone())),
            temp_dir,
            app_handle: self.app_handle,
        })
    }
}

pub struct Server {
    ca: Arc<CertificateAuthority>,
    state: Arc<State>,
    temp_dir: PathBuf,
    app_handle: tauri::AppHandle,
}

impl Server {
    pub async fn run(self: Arc<Self>, listener: TcpListener) -> Result<oneshot::Sender<()>> {
        info!("Starting HTTP(S) proxy server");

        std::fs::create_dir_all(&self.temp_dir)
            .with_context(|| format!("Failed to create temp dir '{}'", self.temp_dir.display()))?;

        let (stop_tx, stop_rx) = oneshot::channel();
        // let (traffic_done_tx, mut traffic_done_rx) = mpsc::unbounded_channel();

        let active_connections: Arc<Mutex<HashMap<SocketAddr, TcpStream>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let server_cloned = self.clone();

        let active_connections_clone = active_connections.clone();
        tokio::spawn(async move {
            let shutdown = Shutdown::new(async { stop_rx.await.unwrap_or_default() });
            let guard = shutdown.guard_weak();

            loop {
                tokio::select! {
                    res = listener.accept() => {
                        let Ok((cnx, addr)) = res else {
                            continue;
                        };

                        // 转换为标准库的 TcpStream
                        let std_stream = match cnx.into_std() {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Failed to convert to std stream: {}", e);
                                continue;
                            }
                        };

                        // 克隆标准库的 stream
                        let stream_for_hyper = match std_stream.try_clone() {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Failed to clone stream: {}", e);
                                continue;
                            }
                        };

                        // 两个流都转回 tokio 的 TcpStream
                        let cnx = match tokio::net::TcpStream::from_std(std_stream) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Failed to convert back to tokio stream: {}", e);
                                continue;
                            }
                        };

                        let stream_for_hyper = match tokio::net::TcpStream::from_std(stream_for_hyper) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Failed to convert stream for hyper: {}", e);
                                continue;
                            }
                        };

                        // let stream = Arc::new(cnx);
                        active_connections_clone.lock().await.insert(addr, cnx);

                        // let traffic_done_tx = traffic_done_tx.clone();
                        let server_cloned = server_cloned.clone();
                        // let active_connections = active_connections_clone.clone();

                        shutdown.spawn_task(async move {
                            let io = TokioIo::new(stream_for_hyper);

                            let hyper_service = service_fn(move |request: hyper::Request<Incoming>| {
                                server_cloned.clone().handle(request)
                            });

                            let res = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                                .serve_connection_with_upgrades(io, hyper_service)
                                .await;

                            if let Err(e) = res {
                                error!("Connection error: {}", e);
                            }

                            // active_connections.lock().await.remove(&addr);
                        });
                    }
                    _ = guard.cancelled() => {
                        let mut connections = active_connections_clone.lock().await;
                        // println!("connections:{:?}", connections);

                    for (addr, stream) in connections.iter_mut() {
                        info!("Closing connection to {}", addr);
                        // let mut stream_guard = stream;
                        // 使用异步 shutdown，需要 .await
                        if let Err(e) = stream.shutdown().await {
                            error!("Error shutting down connection to {}: {}", addr, e);
                        }
                    }
                    connections.clear();
                    break;
                    }
                }
            }
        });

        // tokio::spawn(async move {
        //     while let Some((gid, raw_size)) = traffic_done_rx.recv().await {
        //         let state = self.state.clone();
        //         tokio::spawn(async move {
        //             state.done_traffic(gid, raw_size).await;
        //         });
        //     }
        // });

        Ok(stop_tx)
    }

    pub fn state(&self) -> Arc<State> {
        self.state.clone()
    }

    async fn handle_body(
        &self,
        mut bytes: Bytes,
        content_encoding: String,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        bytes = match content_encoding.as_str() {
            "gzip" => {
                let mut decoder = GzipDecoder::new(&bytes[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).await.map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Gzip decompression error: {}", e),
                    ))
                })?;
                Bytes::from(decompressed)
            }
            "deflate" => {
                let mut decoder = DeflateDecoder::new(&bytes[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).await.map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Deflate decompression error: {}", e),
                    ))
                })?;
                Bytes::from(decompressed)
            }
            "br" => {
                let mut decoder = BrotliDecoder::new(&bytes[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).await.map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Brotli decompression error: {}", e),
                    ))
                })?;
                Bytes::from(decompressed)
            }
            "zstd" => {
                let mut decoder = ZstdDecoder::new(&bytes[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).await.map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Brotli decompression error: {}", e),
                    ))
                })?;
                Bytes::from(decompressed)
            }
            _ => bytes,
        };
        // }

        Ok(bytes)
    }

    async fn repack_body(&self, body: Bytes, original_encoding: Option<String>) -> Result<Bytes> {
        if let Some(original_encoding) = original_encoding {
            match original_encoding.as_str() {
                "gzip" => {
                    let mut compressed = Vec::new();
                    let mut encoder = GzipEncoder::new(&mut compressed);
                    encoder.write_all(&body).await?;
                    encoder.shutdown().await?;
                    Ok(Bytes::from(compressed))
                }
                "deflate" => {
                    let mut compressed = Vec::new();
                    let mut encoder = DeflateEncoder::new(&mut compressed);
                    encoder.write_all(&body).await?;
                    encoder.shutdown().await?;
                    Ok(Bytes::from(compressed))
                }
                "br" => {
                    let mut compressed = Vec::new();
                    let mut encoder = BrotliEncoder::new(&mut compressed);
                    encoder.write_all(&body).await?;
                    encoder.shutdown().await?;
                    Ok(Bytes::from(compressed))
                }
                "zstd" => {
                    let mut compressed = Vec::new();
                    let mut encoder = ZstdEncoder::new(&mut compressed);
                    encoder.write_all(&body).await?;
                    encoder.shutdown().await?;
                    Ok(Bytes::from(compressed))
                }
                _ => Ok(body),
            }
        } else {
            Ok(body)
        }
    }

    async fn continue_request<B>(
        &self,
        bytes: B,
        traffic: Traffic,
        head_id: Option<usize>,
    ) -> Result<Response, hyper::Error>
    where
        B: Body + Send + Unpin + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn StdError + Send + Sync>>,
    {
        let method = match traffic.method.to_lowercase().as_str() {
            "get" => Method::GET,
            "post" => Method::POST,
            "put" => Method::PUT,
            "delete" => Method::DELETE,
            "head" => Method::HEAD,
            "options" => Method::OPTIONS,
            "patch" => Method::PATCH,
            "trace" => Method::TRACE,
            _ => {
                return self
                    .internal_server_error("Invalid method", traffic, head_id)
                    .await
            }
        };

        // 获取代理配置并检查是否需要使用代理
        match get_proxy_config(&self.app_handle) {
            Ok(proxy_config) => {
                let mut builder = hyper::Request::builder().uri(&traffic.uri).method(method);

                if let Some(req_headers) = &traffic.req_headers {
                    for header in &req_headers.items {
                        builder = builder.header(&header.name, &header.value);
                    }
                }

                let proxy_req = match builder.body(bytes) {
                    Ok(v) => v,
                    Err(err) => {
                        return self.internal_server_error(err, traffic, head_id).await;
                    }
                };

                let https = HttpsConnectorBuilder::new()
                    .with_webpki_roots()
                    .https_or_http()
                    .enable_all_versions()
                    .build();

                let need_proxy = check_proxy_config(&proxy_config, traffic.uri.clone());
                debug!("need_proxy={}", need_proxy);
                if need_proxy {
                    self.send_request_with_proxy(&proxy_config, https, proxy_req, traffic, head_id)
                        .await
                } else {
                    self.send_request_direct(https, proxy_req, traffic, head_id)
                        .await
                }
            }
            Err(err) => {
                error!("Failed to get proxy config: {}", err);
                return self.internal_server_error(err, traffic, head_id).await;
            }
        }
    }

    // 使用代理发送请求
    async fn send_request_with_proxy<B>(
        &self,
        proxy_config: &ExternalProxy,
        https: HttpsConnector<HttpConnector>,
        mut proxy_req: hyper::Request<B>,
        traffic: Traffic,
        head_id: Option<usize>,
    ) -> Result<Response, hyper::Error>
    where
        B: Body + Send + Unpin + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn StdError + Send + Sync>>,
    {
        let uri: Uri = traffic.uri.parse().unwrap();
        let proxy = {
            let mutable_external_proxy_configuration = &proxy_config
                .configurations
                .entry
                .iter()
                .find(|entry| entry.string == proxy_config.proxy_type)
                .unwrap()
                .mutable_external_proxy_configuration;
            debug!(
                "mutable_external_proxy_configuration={:?}",
                mutable_external_proxy_configuration
            );
            let host = &mutable_external_proxy_configuration.host;

            let port = mutable_external_proxy_configuration.port;

            let proxy_uri: Uri = format!("http://{}:{}", host, port).parse().unwrap();
            let mut proxy = Proxy::new(Intercept::All, proxy_uri.clone());

            let username = &mutable_external_proxy_configuration.username;
            let encrypted_password = &mutable_external_proxy_configuration.encrypted_password;
            // 创建一个变量来存储解密后的密码，确保它的生命周期足够长
            let decrypted_password = match CRYPTO_SERVICE.decrypt(encrypted_password) {
                Ok(decrypted_text) => decrypted_text,
                Err(e) => {
                    return self.internal_server_error(e, traffic, head_id).await;
                }
            };

            proxy.set_authorization(Authorization::basic(username, &decrypted_password));
            proxy
        };

        let connector = ProxyConnector::from_proxy(https, proxy).unwrap();
        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .build(connector.clone());
        // 对 HTTP 请求添加代理头
        let is_https = uri.scheme_str() == Some("https");
        if !is_https {
            if let Some(headers) = connector.http_headers(&uri) {
                proxy_req.headers_mut().extend(headers.clone().into_iter());
            }
        }

        self.send_and_process_request(client, proxy_req, traffic, head_id)
            .await
    }

    // 直接发送请求（不使用代理）
    async fn send_request_direct<B>(
        &self,
        https: HttpsConnector<HttpConnector>,
        proxy_req: hyper::Request<B>,
        traffic: Traffic,
        head_id: Option<usize>,
    ) -> Result<Response, hyper::Error>
    where
        B: Body + Send + Unpin + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn StdError + Send + Sync>>,
    {
        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .build(https);

        self.send_and_process_request(client, proxy_req, traffic, head_id)
            .await
    }

    // 公共的请求发送和处理逻辑
    async fn send_and_process_request<C, B>(
        &self,
        client: Client<C, B>,
        req: hyper::Request<B>,
        mut traffic: Traffic,
        head_id: Option<usize>,
    ) -> Result<Response, hyper::Error>
    where
        C: Connect + Clone + Send + Sync + 'static,
        B: Body + Send + Unpin + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn StdError + Send + Sync>>,
    {
        let proxy_res = client.request(req).await;

        if let Some(hd_id) = head_id {
            traffic.end_time = Some(OffsetDateTime::now_utc());
            traffic.set_transaction_state(TransactionState::Responding);
            let _ = self.state.create_traffic_head(&traffic, hd_id).await;
        }

        let proxy_res = match proxy_res {
            Ok(v) => v,
            Err(err) => {
                error!("Request Error: {:#?}", err);
                return self.internal_server_error(err, traffic, head_id).await;
            }
        };

        self.process_proxy_res(proxy_res, traffic, head_id).await
    }

    // 断点处理和暂停方法
    async fn handle_request_breakpoint_and_pause(
        &self,
        mut traffic: Traffic,
        mut body_bytes: Bytes,
        content_encoding: String,
        head_id: Option<usize>,
        // traffic_done_tx: TrafficDoneSender
    ) -> Result<Response, hyper::Error> {
        let id = Uuid::new_v4().to_string();
        let req_body_content = TrafficBody::bytes(&body_bytes);

        let traffic_data = TrafficData {
            traffic: traffic.clone(),
            body: Some(req_body_content),
            traffic_type: String::from("request"),
        };

        debug!("命中断点，准备暂停");

        let notify = Arc::new(Notify::new());

        {
            let mut paused = self.state.paused_traffic.lock().await;
            paused.insert(
                id.clone(),
                PausedTrafficInfo {
                    traffic: traffic.clone(),
                    body: Some(body_bytes.clone()),
                    notify: notify.clone(),
                },
            );
        }

        // 通知调试器
        // let _ = self.state.pause_notifier.send((id.clone(), traffic_data));
        let new_pause_traffic_data = NewPauseTrafficData::new((Some(id.clone()), traffic_data));
        send_to_frontend(new_pause_traffic_data.send_data, &self.app_handle);

        // 等待继续信号
        println!("等待继续信号");
        // self.state.continue_notify.notified().await;
        notify.notified().await;
        println!("收到请求继续信号");

        // 获取可能被修改的流量信息
        let (modified_traffic, modified_body) = {
            let mut paused = self.state.paused_traffic.lock().await;
            match paused.get(&id) {
                Some(info) => {
                    let result = (info.traffic.clone(), info.body.clone());
                    paused.remove(&id);
                    result
                }
                None => {
                    println!("未找到流量信息");
                    (traffic.clone(), Some(body_bytes.clone()))
                }
            }
        };

        // 更新 traffic
        traffic = modified_traffic;
        if let Some(modifier_body) = modified_body {
            body_bytes = modifier_body;
        }

        body_bytes = match self.repack_body(body_bytes, Some(content_encoding)).await {
            Ok(body) => body,
            Err(e) => {
                println!("Error repacking body: {:?}", e);
                Bytes::new()
            }
        };

        if let Some(ref mut headers) = traffic.req_headers {
            let content_length = headers
                .items
                .iter_mut()
                .find(|predicate| predicate.name == "content-length");
            match content_length {
                Some(header) => {
                    // 如果找到，更新值
                    header.value = body_bytes.len().to_string();
                }
                None => {
                    // 如果没找到，添加新的 content-length 头
                    headers.items.push(Header {
                        name: "content-length".to_string(),
                        value: body_bytes.len().to_string(),
                    });
                }
            }
        }
        // let req_body = BodyWrapper::new(Full::new(body_bytes))
        let req_body_file = if traffic.valid {
            match self.req_body_file(&mut traffic) {
                Ok(v) => Some(v),
                Err(err) => {
                    return self.internal_server_error(err, traffic, head_id).await;
                }
            }
        } else {
            None
        };

        let req_body = BodyWrapper::new(Full::new(body_bytes.clone()), req_body_file, None, None);
        return self.continue_request(req_body, traffic, head_id).await;
    }

    async fn get_body_bytes<B>(&self, body: Option<B>) -> Result<Bytes, String>
    where
        B: Body<Data = Bytes> + Send + 'static,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        if let Some(body) = body {
            match body.collect().await {
                Ok(collected) => Ok(collected.to_bytes()),
                Err(_) => Err(String::from("Error collecting body")),
            }
        } else {
            Ok(Bytes::from(""))
        }
    }

    async fn get_body_data(
        &self,
        req_body: Option<Incoming>,
        res_body: Option<BoxBody<Bytes, anyhow::Error>>,
        traffic: Traffic,
    ) -> Result<(Bytes, TrafficBody, String), String> {
        // 先处理 body 并获取相应的 headers
        let (body_bytes, content_encoding) = if req_body.is_some() {
            // 处理请求体
            let bytes = self.get_body_bytes(req_body).await?;
            let content_encoding = traffic
                .req_headers
                .as_ref()
                .and_then(|headers| {
                    headers
                        .items
                        .iter()
                        .find(|predicate| predicate.name == "content-encoding")
                        .map(|encoding_value| encoding_value.value.clone())
                })
                .unwrap_or_default();
            (bytes, content_encoding)
        } else if res_body.is_some() {
            // 处理响应体
            let bytes = self.get_body_bytes(res_body).await?;
            let content_encoding = traffic
                .res_headers
                .as_ref()
                .and_then(|headers| {
                    headers
                        .items
                        .iter()
                        .find(|predicate| predicate.name == "content-encoding")
                        .map(|encoding_value| encoding_value.value.clone())
                })
                .unwrap_or_default();
            (bytes, content_encoding)
        } else {
            (Bytes::from(""), String::new())
        };

        // 处理内容编码
        let body_bytes = if !content_encoding.is_empty() {
            match self.handle_body(body_bytes, content_encoding.clone()).await {
                Ok(result) => result,
                Err(e) => {
                    println!("Error handling request body: {:?}", e);
                    Bytes::from("")
                }
            }
        } else {
            body_bytes
        };

        let body_content = TrafficBody::bytes(&body_bytes);
        Ok((body_bytes, body_content, content_encoding))
    }

    async fn handle(self: Arc<Self>, req: Request) -> Result<Response, hyper::Error> {
        let req_uri = req.uri().to_string();
        let headers = req.headers().clone();
        let method = req.method().clone();
        let uri = if !req_uri.starts_with('/') {
            req_uri.clone()
        } else {
            let mut res = Response::default();
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            set_res_body(&mut res, "No reserver proxy url");
            return Ok(res);
        };

        // let path = match uri.split_once('?') {
        //     Some((v, _)) => v,
        //     None => uri.as_str(),
        // };

        let mut traffic = Traffic::new(&uri, method.as_str());

        let mut head: Option<TrafficHead>;

        // 基础检查和设置
        traffic.set_start_time();
        traffic.check_match();
        traffic.set_req_headers(req.headers());
        // println!("{:?}", req.headers());

        if method == Method::CONNECT {
            traffic.check_match();
            return self.handle_connect(req, traffic);
        }

        // let body_bytes = req.into_body().collect().await?.to_bytes();

        // traffic.req_body = Some(TrafficBody::bytes(&body_bytes));

        // 根据 monitor_traffic 状态处理
        if self.state.monitor_traffic.load(Ordering::SeqCst) {
            // 只有在监控模式下才需要 add_traffic
            match self.state.add_traffic(traffic.clone()).await {
                Ok(traffic_head) => {
                    head = Some(traffic_head);
                    traffic.end_time = Some(OffsetDateTime::now_utc());
                    traffic.set_transaction_state(TransactionState::Requesting);

                    // 如果有head，创建流量头
                    if let Some(hd) = head {
                        match self.state.create_traffic_head(&traffic, hd.id).await {
                            Ok(new_head) => head = Some(new_head),
                            Err(err) => {
                                return self.internal_server_error(err, traffic, None).await
                            }
                        }
                    }
                }
                Err(err) => return self.internal_server_error(err, traffic, None).await,
            };
        } else {
            // 非监控模式下，处理请求体并继续
            let req_body_file = if traffic.valid {
                match self.req_body_file(&mut traffic) {
                    Ok(v) => Some(v),
                    Err(err) => return self.internal_server_error(err, traffic, None).await,
                }
            } else {
                None
            };
            let req_body = BodyWrapper::new(req.into_body(), req_body_file, None, Some(headers));
            return self.continue_request(req_body, traffic, None).await;
        }
        // 继续处理
        let header_breakpoint_result = self
            .state
            .check_breakpoints(&traffic, String::from("request"))
            .await;
        let head_id = head.map(|hd| hd.id);

        match header_breakpoint_result {
            Some((breakpoints, match_result)) => {
                match match_result {
                    BreakpointMatchResult::HeaderOnlyMatch => {
                        let (req_body_bytes, req_body_content, content_encoding) = match self
                            .get_body_data(Some(req.into_body()), None, traffic.clone())
                            .await
                        {
                            Ok(result) => result,
                            Err(err) => {
                                return self.internal_server_error(err, traffic, head_id).await;

                                //  self.internal_server_error(err, traffic, head_id).await;
                            }
                        };
                        traffic.req_body_hex = Some(bytes_to_hex_structs(&req_body_bytes));
                        // 检查请求体是否匹配
                        if self
                            .state
                            .check_body_breakpoints(
                                req_body_content,
                                breakpoints,
                                String::from("request"),
                            )
                            .await
                        {
                            return self
                                .handle_request_breakpoint_and_pause(
                                    traffic,
                                    req_body_bytes,
                                    content_encoding,
                                    head_id,
                                )
                                .await;
                        } else {
                            let req_body_file = if traffic.valid {
                                match self.req_body_file(&mut traffic) {
                                    Ok(v) => Some(v),
                                    Err(err) => {
                                        return self
                                            .internal_server_error(err, traffic, head_id)
                                            .await;
                                    }
                                }
                            } else {
                                None
                            };

                            let req_body = BodyWrapper::new(
                                Full::new(req_body_bytes.clone()),
                                req_body_file,
                                None,
                                Some(headers),
                            );
                            return self.continue_request(req_body, traffic, head_id).await;
                        }
                    }
                    BreakpointMatchResult::FullMatch => {
                        let (req_body_bytes, _, content_encoding) = match self
                            .get_body_data(Some(req.into_body()), None, traffic.clone())
                            .await
                        {
                            Ok(result) => result,
                            Err(err) => {
                                return self.internal_server_error(err, traffic, head_id).await;
                            }
                        };
                        traffic.req_body_hex = Some(bytes_to_hex_structs(&req_body_bytes));

                        return self
                            .handle_request_breakpoint_and_pause(
                                traffic,
                                req_body_bytes,
                                content_encoding,
                                head_id,
                            )
                            .await;
                    }
                    _ => {
                        // 没有匹配，直接转发
                        let req_body_file = if traffic.valid {
                            match self.req_body_file(&mut traffic) {
                                Ok(v) => Some(v),
                                Err(err) => {
                                    return self.internal_server_error(err, traffic, head_id).await;
                                }
                            }
                        } else {
                            None
                        };
                        match self.get_body_bytes(Some(req.into_body())).await {
                            Ok(bytes) => {
                                let hex_structs = bytes_to_hex_structs(&bytes);
                                // 设置 16进制数据
                                traffic.req_body_hex = Some(hex_structs);
                                let req_body = BodyWrapper::new(
                                    Full::new(bytes.clone()),
                                    req_body_file,
                                    None,
                                    Some(headers),
                                );
                                return self.continue_request(req_body, traffic, head_id).await;
                            }
                            Err(e) => {
                                return self.internal_server_error(e, traffic, head_id).await;
                            }
                        }
                    }
                }
            }
            None => {
                let req_body_file = if traffic.valid {
                    match self.req_body_file(&mut traffic) {
                        Ok(v) => Some(v),
                        Err(err) => {
                            return self.internal_server_error(err, traffic, head_id).await;
                        }
                    }
                } else {
                    None
                };

                // let req_body = BodyWrapper::new(req.into_body(), req_body_file, None);

                match self.get_body_bytes(Some(req.into_body())).await {
                    Ok(bytes) => {
                        let hex_structs = bytes_to_hex_structs(&bytes);
                        // 设置 16进制数据
                        traffic.req_body_hex = Some(hex_structs);
                        let req_body = BodyWrapper::new(
                            Full::new(bytes.clone()),
                            req_body_file,
                            None,
                            Some(headers),
                        );
                        return self.continue_request(req_body, traffic, head_id).await;
                    }
                    Err(e) => {
                        return self.internal_server_error(e, traffic, head_id).await;
                    }
                }
            }
        }
    }

    // async fn handle_cert_index(&self, res: &mut Response, path: &str) -> Result<()> {
    //     if path.is_empty() {
    //         set_res_body(res, CERT_INDEX);
    //         res.headers_mut().insert(
    //             CONTENT_TYPE,
    //             HeaderValue::from_static("text/html; charset=UTF-8"),
    //         );
    //     } else if path == "proxyfor-ca-cert.cer" || path == "proxyfor-ca-cert.pem" {
    //         let body = self.ca.ca_cert_pem();
    //         set_res_body(res, body);
    //         res.headers_mut().insert(
    //             CONTENT_TYPE,
    //             HeaderValue::from_static("application/x-x509-ca-cert"),
    //         );
    //         res.headers_mut().insert(
    //             CONTENT_DISPOSITION,
    //             HeaderValue::from_str(&format!(r#"attachment; filename="{path}""#))?,
    //         );
    //     } else {
    //         *res.status_mut() = StatusCode::NOT_FOUND;
    //     }
    //     Ok(())
    // }
    // async fn handle_subscribe_traffics(&self, res: &mut Response) -> Result<()> {
    //     let (init_data, receiver) = (
    //         self.state.list_heads().await,
    //         self.state.subscribe_traffics(),
    //     );
    //     let stream = BroadcastStream::new(receiver);
    //     let stream = stream
    //         .map_ok(|head| ndjson_frame(&head))
    //         .map_err(|err| anyhow!("{err}"));
    //     let body = if init_data.is_empty() {
    //         BodyExt::boxed(StreamBody::new(stream))
    //     } else {
    //         let init_stream =
    //             stream::iter(init_data.into_iter().map(|head| Ok(ndjson_frame(&head))));
    //         let combined_stream = init_stream.chain(stream);
    //         BodyExt::boxed(StreamBody::new(combined_stream))
    //     };
    //     *res.body_mut() = body;
    //     res.headers_mut().insert(
    //         CONTENT_TYPE,
    //         HeaderValue::from_static("application/x-ndjson; charset=UTF-8"),
    //     );
    //     res.headers_mut()
    //         .insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    //     Ok(())
    // }

    // async fn handle_list_traffics(&self, res: &mut Response, format: &str) -> Result<()> {
    //     let (data, content_type) = self.state.export_all_traffics(format).await?;
    //     set_res_body(res, data);
    //     res.headers_mut()
    //         .insert(CONTENT_TYPE, HeaderValue::from_str(content_type)?);
    //     res.headers_mut()
    //         .insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    //     Ok(())
    // }

    // async fn handle_get_traffic(&self, res: &mut Response, id: &str, format: &str) -> Result<()> {
    //     let Ok(id) = id.parse() else {
    //         *res.status_mut() = StatusCode::BAD_REQUEST;
    //         set_res_body(res, "Invalid id");
    //         return Ok(());
    //     };
    //     let (data, content_type) = self.state.export_traffic(id, format).await?;
    //     set_res_body(res, data);
    //     res.headers_mut()
    //         .insert(CONTENT_TYPE, HeaderValue::from_str(content_type)?);
    //     res.headers_mut()
    //         .insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    //     Ok(())
    // }

    fn handle_connect(
        self: Arc<Self>,
        mut req: Request,
        mut traffic: Traffic,
    ) -> Result<Response, hyper::Error> {
        let mut res = Response::default();
        let authority = match req.uri().authority().cloned() {
            Some(authority) => authority,
            None => {
                *res.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(res);
            }
        };
        let server = self.clone();
        let fut = async move {
            match hyper::upgrade::on(&mut req).await {
                Ok(upgraded) => {
                    let mut upgraded = TokioIo::new(upgraded);

                    let mut buffer = [0; 4];
                    let bytes_read = match upgraded.read_exact(&mut buffer).await {
                        Ok(bytes_read) => bytes_read,
                        Err(err) => {
                            traffic.add_error(format!(
                                "Failed to read from upgraded connection: {err}"
                            ));
                            return;
                        }
                    };

                    let mut upgraded = Rewind::new_buffered(
                        upgraded,
                        bytes::Bytes::copy_from_slice(buffer[..bytes_read].as_ref()),
                    );
                    if buffer == *b"GET " {
                        if let Err(err) = self
                            .serve_connect_stream(upgraded, Scheme::HTTP, authority)
                            .await
                        {
                            traffic.add_error(format!("Websocket connect error: {err}"));
                        }
                    } else if buffer[..2] == *b"\x16\x03" {
                        let server_config = match self.ca.gen_server_config(&authority).await {
                            Ok(server_config) => server_config,
                            Err(err) => {
                                println!("Failed to build server config");
                                traffic.add_error(format!("Failed to build server config: {err}"));
                                return;
                            }
                        };
                        // println!("server_config:{:#?}", server_config);
                        let stream = match TlsAcceptor::from(server_config).accept(upgraded).await {
                            Ok(stream) => stream,
                            Err(err) => {
                                println!("Failed to establish TLS Connection");

                                traffic.add_error(format!(
                                    "Failed to establish TLS Connection: {err}"
                                ));
                                return;
                            }
                        };

                        if let Err(err) = self
                            .serve_connect_stream(stream, Scheme::HTTPS, authority)
                            .await
                        {
                            if !err
                                .to_string()
                                .starts_with("error shutting down connection")
                            {
                                traffic.add_error(format!("HTTPS connect error: {err}"));
                            }
                        }
                    } else {
                        traffic.add_error(format!(
                            "Unknown protocol, read '{:02X?}' from upgraded connection",
                            &buffer[..bytes_read]
                        ));

                        let mut server = match TcpStream::connect(authority.as_str()).await {
                            Ok(server) => server,
                            Err(err) => {
                                traffic
                                    .add_error(format! {"Failed to connect to {authority}: {err}"});
                                return;
                            }
                        };

                        if let Err(err) =
                            tokio::io::copy_bidirectional(&mut upgraded, &mut server).await
                        {
                            traffic.add_error(format!(
                                "Failed to tunnel unknown protocol to {}: {}",
                                authority, err
                            ));
                        }
                    }
                }
                Err(err) => {
                    println!("err!!:{:#?}", err);
                    traffic.add_error(format!("Upgrade error: {err}"));
                }
            };
            // let state = server.state.clone();
            if !server.state.monitor_traffic.load(Ordering::SeqCst) {
                return;
            }
            let _ = server.state.add_traffic(traffic).await;
        };

        tokio::spawn(fut);
        Ok(Response::default())
    }

    async fn serve_connect_stream<I>(
        self: Arc<Self>,
        stream: I,
        scheme: Scheme,
        authority: Authority,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>>
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let service = service_fn(|mut req| {
            if req.version() == hyper::Version::HTTP_10 || req.version() == hyper::Version::HTTP_11
            {
                let (mut parts, body) = req.into_parts();

                parts.uri = {
                    let mut parts = parts.uri.into_parts();
                    parts.scheme = Some(scheme.clone());
                    parts.authority = Some(authority.clone());
                    Uri::from_parts(parts).expect("Failed to build URI")
                };

                req = Request::from_parts(parts, body);
            };

            self.clone().handle(req)
        });

        hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
            .serve_connection_with_upgrades(TokioIo::new(stream), service)
            .await
    }

    async fn handle_response_breakpoint_and_pause(
        &self,
        mut traffic: Traffic,
        mut body_bytes: Bytes,
        content_encoding: String,
        head_id: Option<usize>,
    ) -> Result<Response, hyper::Error> {
        let id = Uuid::new_v4().to_string();
        let res_body_content = TrafficBody::bytes(&body_bytes);
        let traffic_data = TrafficData {
            traffic: traffic.clone(),
            body: Some(res_body_content.clone()),
            traffic_type: String::from("response"),
        };

        println!("命中响应断点,准备暂停");

        let notify = Arc::new(Notify::new());
        {
            let mut paused = self.state.paused_traffic.lock().await;
            paused.insert(
                id.clone(),
                PausedTrafficInfo {
                    traffic: traffic.clone(),
                    body: Some(body_bytes.clone()),
                    notify: notify.clone(),
                },
            );
        }

        // let _ = self.state.pause_notifier.send((id.clone(), traffic_data));
        let new_pause_traffic_data = NewPauseTrafficData::new((Some(id.clone()), traffic_data));
        send_to_frontend(new_pause_traffic_data.send_data, &self.app_handle);
        println!("等待继续信号");
        notify.notified().await;
        println!("继续响应执行");

        let (modified_traffic, modified_body) = {
            let mut paused = self.state.paused_traffic.lock().await;
            match paused.get(&id) {
                Some(info) => {
                    let result = (info.traffic.clone(), info.body.clone());
                    paused.remove(&id);
                    result
                }
                None => {
                    println!("未找到流量信息");
                    (traffic.clone(), Some(body_bytes.clone()))
                }
            }
        };

        // 使用可能被修改的流量信息继续处理
        traffic = modified_traffic;

        // 更新响应体
        if let Some(modifier_body) = modified_body {
            // let content_modifier_body = TrafficBody::bytes(&modifier_body);
            body_bytes = modifier_body;
        }
        let final_body = match self
            .repack_body(body_bytes, Some(content_encoding.clone()))
            .await
        {
            Ok(body) => body,
            Err(e) => {
                println!("Error repacking body: {:?}", e);
                Bytes::new()
            }
        };

        if let Some(ref mut headers) = traffic.res_headers {
            let content_length = headers
                .items
                .iter_mut()
                .find(|predicate| predicate.name == "content-length");
            match content_length {
                Some(header) => {
                    header.value = final_body.len().to_string();
                }
                None => headers.items.push(Header {
                    name: "content-length".to_string(),
                    value: final_body.len().to_string(),
                }),
            }
        }
        let body: BoxBody<Bytes, anyhow::Error> = Full::new(final_body)
            .map_err(|never: Infallible| -> anyhow::Error {
                match never {} // 这永远不会发生，因为 Infallible 是空类型
            })
            .boxed();
        return self
            .continue_response(body, traffic, content_encoding, head_id)
            .await;
    }

    async fn continue_response(
        &self,
        body: BoxBody<Bytes, anyhow::Error>,
        mut traffic: Traffic,
        encoding: String,
        head_id: Option<usize>,
    ) -> Result<Response, hyper::Error> {
        let mut res = Response::default();
        let status_code = match StatusCode::from_u16(traffic.status.clone().unwrap()) {
            Ok(value) => value,
            Err(e) => {
                println!("Invalid status code: {}", e);
                return self
                    .internal_server_error("Invalid status code", traffic, head_id)
                    .await;
            }
        };
        *res.status_mut() = status_code;

        let new_header = traffic.res_headers.clone().unwrap();
        for header in &new_header.items {
            if let Ok(name) = HeaderName::from_bytes(header.name.as_bytes()) {
                if let Ok(value) = header.value.parse() {
                    res.headers_mut().insert(name, value);
                }
            }
        }

        let res_body_file = if traffic.valid {
            match self.res_body_file(&mut traffic, &encoding) {
                Ok(v) => Some(v),
                Err(err) => {
                    return self.internal_server_error(err, traffic, head_id).await;
                }
            }
        } else {
            None
        };

        let res_body: BodyWrapper<BoxBody<Bytes, anyhow::Error>> = BodyWrapper::new(
            body,
            res_body_file,
            Some((head_id, self.state.clone())),
            Some(res.headers().clone()),
        );

        *res.body_mut() = BoxBody::new(res_body);

        match head_id {
            Some(hd_id) => {
                traffic.end_time = Some(OffsetDateTime::now_utc());

                traffic.set_transaction_state(TransactionState::ResponseDone);
                let _ = self.state.create_traffic_head(&traffic, hd_id).await;
                // println!("响应完成-----------------{:?}----------------------",traffic);
                {
                    let mut traffics = self.state.traffics.lock().await;
                    traffics.insert(hd_id, traffic);
                }
            }
            None => {}
        }

        Ok(res)
    }

    async fn process_proxy_res<T: Body<Data = Bytes> + Send + Sync + 'static>(
        &self,
        proxy_res: hyper::Response<T>,
        mut traffic: Traffic,
        head_id: Option<usize>,
    ) -> Result<Response, hyper::Error> {
        // let before = time::Instant::now();
        let proxy_res = {
            let (parts, body) = proxy_res.into_parts();
            Response::from_parts(parts, body.map_err(|_| anyhow!("Invalid response")).boxed())
        };

        let proxy_res_version = proxy_res.version();
        let proxy_res_status = proxy_res.status();
        let proxy_res_headers = proxy_res.headers().clone();

        traffic
            .set_res_status(proxy_res_status)
            .set_http_version(&proxy_res_version)
            .set_res_headers(&proxy_res_headers);

        let _content_type = proxy_res_headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();

        traffic.check_match();

        let mut encoding: String = String::new();
        for (key, value) in proxy_res_headers.iter() {
            if key == CONTENT_ENCODING {
                encoding = value.to_str().map(|v| v.to_string()).unwrap_or_default();
            }
        }

        if !self.state.monitor_traffic.load(Ordering::SeqCst) {
            match head_id {
                Some(_) => {}
                None => match self.get_body_bytes(Some(proxy_res.into_body())).await {
                    Ok(bytes) => {
                        let body = Full::new(bytes)
                            .map_err(|_: Infallible| -> anyhow::Error { unreachable!() });
                        return self
                            .continue_response(BoxBody::new(body), traffic, encoding, head_id)
                            .await;
                    }
                    Err(e) => {
                        return self.internal_server_error(e, traffic, head_id).await;
                    }
                },
            }
        }

        let header_breakpoint_result = self
            .state
            .check_breakpoints(&traffic, String::from("response"))
            .await;
        match header_breakpoint_result {
            Some((breakpoints, match_result)) => {
                match match_result {
                    BreakpointMatchResult::HeaderOnlyMatch => {
                        let (res_body_bytes, res_body_content, content_encoding) = match self
                            .get_body_data(None, Some(proxy_res.into_body()), traffic.clone())
                            .await
                        {
                            Ok(result) => result,
                            Err(err) => {
                                return self.internal_server_error(err, traffic, head_id).await;
                            }
                        };
                        traffic.res_body_hex = Some(bytes_to_hex_structs(&res_body_bytes));

                        if self
                            .state
                            .check_body_breakpoints(
                                res_body_content,
                                breakpoints,
                                String::from("response"),
                            )
                            .await
                        {
                            return self
                                .handle_response_breakpoint_and_pause(
                                    traffic,
                                    res_body_bytes,
                                    content_encoding,
                                    head_id,
                                )
                                .await;
                        } else {
                            let body: BoxBody<Bytes, anyhow::Error> = Full::new(res_body_bytes)
                                .map_err(|never: Infallible| -> anyhow::Error {
                                    match never {} // 这永远不会发生，因为 Infallible 是空类型
                                })
                                .boxed();
                            return self
                                .continue_response(body, traffic, content_encoding, head_id)
                                .await;
                        }
                    }
                    BreakpointMatchResult::FullMatch => {
                        let (res_body_bytes, _res_body_content, content_encoding) = match self
                            .get_body_data(None, Some(proxy_res.into_body()), traffic.clone())
                            .await
                        {
                            Ok(result) => result,
                            Err(err) => {
                                return self.internal_server_error(err, traffic, head_id).await;
                            }
                        };
                        traffic.res_body_hex = Some(bytes_to_hex_structs(&res_body_bytes));

                        return self
                            .handle_response_breakpoint_and_pause(
                                traffic,
                                res_body_bytes,
                                content_encoding,
                                head_id,
                            )
                            .await;
                    }
                    _ => {
                        match self.get_body_bytes(Some(proxy_res.into_body())).await {
                            Ok(bytes) => {
                                let hex_structs = bytes_to_hex_structs(&bytes);
                                // 设置 16进制数据
                                traffic.res_body_hex = Some(hex_structs);
                                let body = Full::new(bytes)
                                    .map_err(|_: Infallible| -> anyhow::Error { unreachable!() });
                                return self
                                    .continue_response(
                                        BoxBody::new(body),
                                        traffic,
                                        encoding,
                                        head_id,
                                    )
                                    .await;
                            }
                            Err(e) => {
                                return self.internal_server_error(e, traffic, head_id).await;
                            }
                        }
                        // return self
                        //     .continue_response(proxy_res.into_body(), traffic, encoding, head_id)
                        //     .await;
                    }
                }
            }
            None => {
                match self.get_body_bytes(Some(proxy_res.into_body())).await {
                    Ok(bytes) => {
                        let hex_structs = bytes_to_hex_structs(&bytes);
                        // 设置 16进制数据
                        traffic.res_body_hex = Some(hex_structs);
                        let body = Full::new(bytes)
                            .map_err(|_: Infallible| -> anyhow::Error { unreachable!() });
                        return self
                            .continue_response(BoxBody::new(body), traffic, encoding, head_id)
                            .await;
                    }
                    Err(e) => {
                        return self.internal_server_error(e, traffic, head_id).await;
                    }
                }
            }
        }
    }

    async fn internal_server_error<T: std::fmt::Display>(
        &self,
        error: T,
        mut traffic: Traffic,
        head_id: Option<usize>,
        // traffic_done_tx: TrafficDoneSender,
    ) -> Result<Response, hyper::Error> {
        let mut res = Response::default();
        match head_id {
            Some(hd_id) => {
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                // let gid = traffic.gid;
                traffic.add_error(error.to_string());
                traffic.set_transaction_state(TransactionState::Failed);
                let _ = self.state.create_traffic_head(&traffic, hd_id).await;
                // let _ = traffic_done_tx.send((gid, 0));

                Ok(res)
            }
            None => Ok(res),
        }
    }

    fn req_body_file(&self, traffic: &mut Traffic) -> Result<File> {
        let mime = extract_mime(&traffic.req_headers);
        let ext_name = to_ext_name(mime);
        let path = self
            .temp_dir
            .join(format!("{:05}-req{ext_name}", traffic.gid));
        let file = File::create(&path).with_context(|| {
            format!(
                "Failed to create file '{}' to store request body",
                path.display()
            )
        })?;
        traffic.set_req_body_file(&path);
        Ok(file)
    }

    fn res_body_file(&self, traffic: &mut Traffic, encoding: &str) -> Result<File> {
        let mime = extract_mime(&traffic.res_headers);
        let ext = to_ext_name(mime);
        let encoding_ext = match ENCODING_EXTS.iter().find(|(v, _)| *v == encoding) {
            Some((_, encoding_ext)) => encoding_ext,
            None => "",
        };
        let path = self
            .temp_dir
            .join(format!("{:05}-res{ext}{encoding_ext}", traffic.gid));
        let file = File::create(&path).with_context(|| {
            format!(
                "Failed to create file '{}' to store response body",
                path.display()
            )
        })?;
        traffic.set_res_body_file(&path);
        Ok(file)
    }
}

fn set_res_body<T: std::fmt::Display>(res: &mut Response, body: T) {
    let body = Bytes::from(body.to_string());
    if let Ok(header_value) = HeaderValue::from_str(&body.len().to_string()) {
        res.headers_mut().insert(CONTENT_LENGTH, header_value);
    }
    *res.body_mut() = Full::new(body).map_err(|err| anyhow!("{err}")).boxed();
}

pin_project! {
    pub struct BodyWrapper<B> {
        #[pin]
        inner: B,
        file: Option<File>,
        traffic_done: Option<(Option<usize>, Arc<State>)>,
        raw_size: u64,   headers: Option<HeaderMap>,
    }
     impl<B> PinnedDrop for BodyWrapper<B> {
        fn drop(this: Pin<&mut Self>) {

            if let Some((head_id, state)) = this.traffic_done.as_ref() {
                if !state.monitor_traffic.load(Ordering::SeqCst) {
                    return
                }

                match head_id{
                    Some(hd_id)=>{

                        let state = state.clone();
                        let head_id = *hd_id;
                        let raw_size = this.raw_size;

                        tokio::spawn(async move {
                            state.done_traffic(head_id, raw_size).await;
                        });
                    },None=>{}
                }

            }
        }
    }
}

fn is_protobuf_content(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            s.contains("application/protobuf")
                || s.contains("application/x-protobuffer")
                || s.contains("application/x-protobuf")
        })
        .unwrap_or(false)
}

async fn decode_protobuf(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    // 使用我们的 ProtobufUnknownParser 来解析
    let result = parse_unknown_protobuf(data).await?;

    // 将解析结果转换为 JSON
    serde_json::to_vec(&result).map_err(|e| e.into())
}

impl<B> BodyWrapper<B> {
    pub fn new(
        inner: B,
        file: Option<File>,
        traffic_done: Option<(Option<usize>, Arc<State>)>,
        headers: Option<HeaderMap>,
    ) -> Self {
        Self {
            inner,
            file,
            traffic_done,
            raw_size: 0,
            headers,
        }
    }
}

impl<B> Body for BodyWrapper<B>
where
    B: Body<Data = Bytes> + Send + Sync + 'static,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let mut this = self.project();

        match Pin::new(&mut this.inner).poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => match frame.into_data() {
                Ok(data) => {
                    if let Some(file) = this.file.as_mut() {
                        if let Some(headers) = this.headers.as_ref() {
                            if is_protobuf_content(headers) {
                                // 复制数据用于异步处理
                                let data_clone = data.clone();
                                let mut file_clone = file.try_clone().unwrap();

                                tokio::spawn(async move {
                                    match decode_protobuf(&data_clone).await {
                                        Ok(decoded) => {
                                            println!("成功解析 protobuf 数据");
                                            if let Err(e) = file_clone.write_all(&decoded) {
                                                eprintln!("写入解码数据失败: {:?}", e);
                                            }
                                        }
                                        Err(e) => {
                                            println!("protobuf 解析失败: {:?}", e);
                                            if let Err(e) = file_clone.write_all(&data_clone) {
                                                eprintln!("写入原始数据失败: {:?}", e);
                                            }
                                        }
                                    }
                                });
                            } else {
                                let _ = file.write_all(&data);
                            }
                        } else {
                            let _ = file.write_all(&data);
                        }
                    }
                    *this.raw_size += data.len() as u64;
                    Poll::Ready(Some(Ok(Frame::data(data))))
                }
                Err(e) => Poll::Ready(Some(Ok(e))),
            },
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

// fn ndjson_frame<T: Serialize>(head: &T) -> Frame<Bytes> {
//     let data = match serde_json::to_string(head) {
//         Ok(data) => format!("{data}\n"),
//         Err(_) => String::new(),
//     };
//     Frame::data(Bytes::from(data))
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintMode {
    Nothing,
    Oneline,
    #[default]
    Markdown,
}
