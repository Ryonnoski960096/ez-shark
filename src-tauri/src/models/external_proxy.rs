use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreBuilder;

use crate::utils::{extract_domain, is_local_request};

/// 外部代理配置的主结构体  
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalProxy {
    #[serde(rename = "alwaysBypassLocalhost")]
    pub always_bypass_localhost: bool,
    #[serde(rename = "bypassDomains")]
    pub bypass_domains: BypassDomains,
    pub configurations: Configurations,
    #[serde(rename = "proxyType")]
    pub proxy_type: String,
    pub enabled: bool,
}

/// 绕过域名配置  
#[derive(Debug, Serialize, Deserialize)]
pub struct BypassDomains {
    pub string: Vec<String>,
}

/// 代理配置集合  
#[derive(Debug, Serialize, Deserialize)]
pub struct Configurations {
    pub entry: Vec<ProxyEntry>,
}

/// 单个代理条目  
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyEntry {
    #[serde(rename = "mutableExternalProxyConfiguration")]
    pub mutable_external_proxy_configuration: MutableExternalProxyConfiguration,
    pub string: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrNumber {
    String(String),
    Number(i64),
}

impl From<StringOrNumber> for String {
    fn from(value: StringOrNumber) -> Self {
        match value {
            StringOrNumber::String(s) => s,
            StringOrNumber::Number(n) => n.to_string(),
        }
    }
}

/// 代理详细配置信息  
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MutableExternalProxyConfiguration {
    #[serde(deserialize_with = "deserialize_to_string")]
    pub domain: String,
    #[serde(rename = "encryptedPassword")]
    pub encrypted_password: String,
    #[serde(deserialize_with = "deserialize_to_string")]
    pub host: String,
    pub port: u16,
    #[serde(rename = "requiresAuthentication")]
    pub requires_authentication: bool,
    #[serde(deserialize_with = "deserialize_to_string")]
    pub username: String,
}

fn deserialize_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = StringOrNumber::deserialize(deserializer)?;
    Ok(value.into())
}

impl ExternalProxy {
    pub fn new() -> Self {
        ExternalProxy {
            always_bypass_localhost: true,
            bypass_domains: BypassDomains { string: Vec::new() },
            proxy_type: "http".to_string(),
            configurations: Configurations {
                entry: vec![
                    ProxyEntry {
                        mutable_external_proxy_configuration: MutableExternalProxyConfiguration {
                            domain: String::new(),
                            encrypted_password: String::new(),
                            host: String::new(),
                            port: 8080,
                            requires_authentication: false,
                            username: String::new(),
                        },
                        string: "http".to_string(),
                    },
                    ProxyEntry {
                        mutable_external_proxy_configuration: MutableExternalProxyConfiguration {
                            domain: String::new(),
                            encrypted_password: String::new(),
                            host: String::new(),
                            port: 443,
                            requires_authentication: false,
                            username: String::new(),
                        },
                        string: "https".to_string(),
                    },
                    ProxyEntry {
                        mutable_external_proxy_configuration: MutableExternalProxyConfiguration {
                            domain: String::new(),
                            encrypted_password: String::new(),
                            host: String::new(),
                            port: 1080,
                            requires_authentication: false,
                            username: String::new(),
                        },
                        string: "socks".to_string(),
                    },
                ],
            },
            enabled: true,
        }
    }
}

// 获取代理配置
pub fn get_proxy_config<R: Runtime>(app: &AppHandle<R>) -> Result<ExternalProxy, String> {
    let path = PathBuf::from("settings.json");

    let store = StoreBuilder::new(app, path)
        .build()
        .map_err(|e| format!("创建存储失败: {}", e))?;

    match store.get("externalProxy") {
        Some(proxy_value) => {
            // println!("读取到的代理配置: {:?}", proxy_value);
            serde_json::from_value(proxy_value.clone())
                .map_err(|e| format!("代理配置解析失败: {}. 原始数据: {:?}", e, proxy_value))
        }
        None => Err("设置中未找到代理配置".to_string()),
    }
}

// 检查是否需要额外代理
pub fn check_proxy_config(proxy_config: &ExternalProxy, url: String) -> bool {
    // 检查是否启用了代理
    if !proxy_config.enabled {
        return false;
    }

    // 检查是否勾选了alwaysBypassLocalhost并且是本地请求，如127.0.0.1或者localhost
    if proxy_config.always_bypass_localhost && is_local_request(&url) {
        return false;
    }

    // 检查是否在bypassDomains中
    if !proxy_config.bypass_domains.string.is_empty() {
        let domain = extract_domain(&url);
        if proxy_config.bypass_domains.string.contains(&domain) {
            return false;
        }
    }
    // 检查是否开启了身份验证并且代理配置为空
    let proxy_type = proxy_config.proxy_type.as_str();
    if proxy_config.configurations.entry.iter().any(|entry| {
        let config = &entry.mutable_external_proxy_configuration;
        entry.string == proxy_type && (
            // 检查基本配置是否为空
            config.host.is_empty() || config.port == 0 ||
            // 检查认证配置
            (config.requires_authentication && 
             (config.username.is_empty() || config.encrypted_password.is_empty()))
        )
    }) {
        return false;
    }
    return true;
}
