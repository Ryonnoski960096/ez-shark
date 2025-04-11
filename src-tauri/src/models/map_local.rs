use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreBuilder;

///  MapLocal配置的主结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct MapLocalItem {
    pub id: String,
    pub enabled: bool,
    pub url: String,
    #[serde(rename = "headerLocal")]
    pub header_local: String,
    #[serde(rename = "bodyLocal")]
    pub body_local: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapLocal {
    #[serde(rename = "toolEnabled")]
    pub tool_enabled: bool,
    #[serde(rename = "mapLocals")]
    pub map_locals: HashMap<String, MapLocalItem>,
}

/// 从settings.json中读取MapLocal配置
pub fn get_map_local_config<R: Runtime>(app: &AppHandle<R>) -> Result<MapLocal, String> {
    let path = PathBuf::from("settings.json");

    let store = StoreBuilder::new(app, path)
        .build()
        .map_err(|e| format!("创建存储失败: {}", e))?;

    match store.get("mapLocal") {
        Some(proxy_value) => {
            // println!("读取到的代理配置: {:?}", proxy_value);
            serde_json::from_value(proxy_value.clone())
                .map_err(|e| format!("代理配置解析失败: {}. 原始数据: {:?}", e, proxy_value))
        }
        None => Err("设置中未找到代理配置".to_string()),
    }
}

// let config = match get_map_local_config(app) {
//     Ok(cfg) => cfg,
//     Err(_) => return false,
// };

/// 检查是否需要使用MapLocal
pub fn check_need_map_local(map_local: MapLocal) -> Result<MapLocalItem, bool> {
    if !map_local.tool_enabled {
        return Err(false);
    }

    for (_, item) in map_local.map_locals {
        if item.enabled && item.url != "" && (item.body_local != "" || item.header_local != "") {
            return Ok(item);
        }
    }

    // 如果一切正常，可以返回 true
    return Err(false);
}
