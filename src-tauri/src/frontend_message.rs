use serde::Serialize;
use tauri::Emitter;

use crate::traffic::TrafficHead;

#[derive(Debug, Serialize, Clone)] // 添加 Serialize
pub enum Status {
    Success,
    Fail,
}

#[derive(Debug, Serialize, Clone)]
pub struct Payload<T> {
    pub status: Status,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SendData<T> {
    pub event_name: String,
    pub payload: Payload<T>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewTrafficHeadData {
    pub send_data: SendData<TrafficHead>,
}

impl NewTrafficHeadData {
    pub fn new(head: &TrafficHead) -> Self {
        // 先创建 Payload
        let payload = Payload {
            status: Status::Success,
            message: "请求状态更新".to_string(),
            data: Some(head.clone()),
        };

        // 创建完整的结构体
        Self {
            send_data: SendData {
                event_name: "new-traffic".to_string(),
                payload,
            },
        }
    }
}

pub fn send_to_frontend<T: Clone + Serialize>(
    send_data: SendData<T>,
    app_handle: &tauri::AppHandle,
) {
    let _ = app_handle.emit(&send_data.event_name, &send_data.payload);
}
