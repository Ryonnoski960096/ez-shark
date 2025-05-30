use anyhow::Result;
use async_compression::tokio::bufread::{BrotliDecoder, DeflateDecoder, GzipDecoder, ZstdDecoder};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine as _;
use log::debug;
use serde::de::Error;
use serde::Serialize;
use serde::{Deserialize, Deserializer, Serializer};
use std::io::SeekFrom;
use std::sync::{Arc, LazyLock};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::io::AsyncSeekExt;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncRead, AsyncReadExt, AsyncSeek, BufReader, BufWriter},
};
use unicode_width::UnicodeWidthStr;
// 1

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use thiserror::Error;

pub const ENCODING_EXTS: [(&str, &str); 4] = [
    ("deflate", ".enc.deflate"),
    ("gzip", ".enc.gz"),
    ("br", ".enc.br"),
    ("zstd", ".enc.zst"),
];

static CLIPBOARD: LazyLock<Arc<std::sync::Mutex<Option<arboard::Clipboard>>>> =
    LazyLock::new(|| std::sync::Arc::new(std::sync::Mutex::new(arboard::Clipboard::new().ok())));

pub fn base64_encode(data: &[u8]) -> String {
    BASE64_STANDARD_NO_PAD.encode(data)
}

pub fn ellipsis_tail(text: &str, width: usize) -> String {
    let text_width = text.width();
    if text_width > width {
        format!("{}…", &text[..width - 1])
    } else {
        text.to_string()
    }
}

pub fn ellipsis_head(text: &str, width: usize) -> String {
    let text_width = text.width();
    if text_width > width {
        format!("…{}", &text[text_width - width + 1..])
    } else {
        text.to_string()
    }
}

pub fn format_size(bytes: Option<u64>) -> String {
    match bytes {
        None => String::new(),
        Some(0) => "0".to_string(),
        Some(bytes) => {
            let prefix = ["b", "kb", "mb", "gb", "tb"];
            let mut i = 0;
            while i < prefix.len() && 1024u64.pow(i as u32 + 1) <= bytes {
                i += 1;
            }
            let precision = if bytes % 1024u64.pow(i as u32) == 0 {
                0
            } else {
                1
            };
            format!(
                "{:.prec$}{}",
                bytes as f64 / 1024f64.powi(i as i32),
                prefix[i],
                prec = precision
            )
        }
    }
}

pub fn format_time_delta(delta: Option<u64>) -> String {
    let mut delta = match delta {
        Some(ms) => ms,
        None => return String::from(""),
    };

    if delta == 0 {
        return String::from("0");
    }

    if delta > 1000 && delta < 10000 {
        let seconds = delta as f64 / 1000.0;
        return format!("{:.2}s", seconds);
    }

    let prefix = ["ms", "s", "min", "h"];
    let div = [1000, 60, 60];
    let mut i = 0;

    while i < div.len() && delta >= div[i] {
        delta /= div[i];
        i += 1;
    }

    format!("{}{}", delta, prefix[i])
}

pub fn next_idx(len: usize, idx: usize) -> usize {
    if idx >= len.saturating_sub(1) {
        0
    } else {
        idx + 1
    }
}

pub fn prev_idx(len: usize, idx: usize) -> usize {
    if idx == 0 {
        len.saturating_sub(1)
    } else {
        idx - 1
    }
}

#[cfg(not(any(target_os = "android", target_os = "emscripten")))]
pub fn set_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    match clipboard.as_mut() {
        Some(clipboard) => clipboard.set_text(text)?,
        None => anyhow::bail!("No available clipboard"),
    }
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "emscripten"))]
pub fn set_text(_text: &str) -> anyhow::Result<()> {
    anyhow::bail!("No available clipboard")
}

pub async fn uncompress_data(encoding: &str, path: &str) -> Result<Vec<u8>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut decompressed = Vec::new();
    let mut decoder = uncompress_decoder(encoding, reader);
    decoder.read_to_end(&mut decompressed).await?;
    Ok(decompressed)
}

pub async fn uncompress_file(encoding: &str, source_path: &str, target_path: &str) -> Result<()> {
    let source_file = File::open(source_path).await?;
    let target_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(target_path)
        .await?;

    let reader = BufReader::new(source_file);
    let mut decoder = uncompress_decoder(encoding, reader);
    let mut writer = BufWriter::new(target_file);

    tokio::io::copy(&mut decoder, &mut writer).await?;
    fs::remove_file(source_path).await?;

    Ok(())
}

fn uncompress_decoder(
    encoding: &str,
    reader: BufReader<File>,
) -> Box<dyn AsyncRead + Send + Unpin> {
    match encoding {
        "deflate" => Box::new(DeflateDecoder::new(reader)),
        "gzip" => Box::new(GzipDecoder::new(reader)),
        "br" => Box::new(BrotliDecoder::new(reader)),
        "zstd" => Box::new(ZstdDecoder::new(reader)),
        _ => Box::new(reader),
    }
}

// see https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
pub fn to_ext_name(mime: &str) -> &str {
    match mime {
        "audio/aac" => ".aac",
        "application/x-abiword" => ".abw",
        "image/apng" => ".apng",
        "application/x-freearc" => ".arc",
        "image/avif" => ".avif",
        "video/x-msvideo" => ".avi",
        "application/vnd.amazon.ebook" => ".azw",
        "application/octet-stream" => ".bin",
        "image/bmp" => ".bmp",
        "application/x-bzip" => ".bz",
        "application/x-bzip2" => ".bz2",
        "application/x-cdf" => ".cda",
        "application/x-csh" => ".csh",
        "text/css" => ".css",
        "text/csv" => ".csv",
        "application/msword" => ".doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => ".docx",
        "application/vnd.ms-fontobject" => ".eot",
        "application/epub+zip" => ".epub",
        "application/gzip" | "application/x-gzip" => ".gz",
        "image/gif" => ".gif",
        "text/html" | "text/htm" => ".html",
        "image/vnd.microsoft.icon" => ".ico",
        "text/calendar" => ".ics",
        "application/java-archive" => ".jar",
        "image/jpeg" => ".jpeg",
        "text/javascript" => ".js",
        "application/json" => ".json",
        "application/ld+json" => ".jsonld",
        "audio/midi" | "audio/x-midi" => ".mid",
        "audio/mpeg" => ".mp3",
        "video/mp4" => ".mp4",
        "video/mpeg" => ".mpeg",
        "application/vnd.apple.installer+xml" => ".mpkg",
        "application/vnd.oasis.opendocument.presentation" => ".odp",
        "application/vnd.oasis.opendocument.spreadsheet" => ".ods",
        "application/vnd.oasis.opendocument.text" => ".odt",
        "audio/ogg" => ".oga",
        "video/ogg" => ".ogv",
        "application/ogg" => ".ogx",
        "font/otf" => ".otf",
        "image/png" => ".png",
        "application/pdf" => ".pdf",
        "application/x-httpd-php" => ".php",
        "application/vnd.ms-powerpoint" => ".ppt",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => ".pptx",
        "application/vnd.rar" => ".rar",
        "application/rtf" => ".rtf",
        "application/x-sh" => ".sh",
        "image/svg+xml" => ".svg",
        "application/x-tar" => ".tar",
        "image/tiff" => ".tif",
        "video/mp2t" => ".ts",
        "font/ttf" => ".ttf",
        "text/plain" => ".txt",
        "application/vnd.visio" => ".vsd",
        "audio/wav" => ".wav",
        "audio/webm" => ".weba",
        "video/webm" => ".webm",
        "image/webp" => ".webp",
        "font/woff" => ".woff",
        "font/woff2" => ".woff2",
        "application/xhtml+xml" => ".xhtml",
        "application/vnd.ms-excel" => ".xls",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => ".xlsx",
        "application/xml" | "text/xml" => ".xml",
        "application/vnd.mozilla.xul+xml" => ".xul",
        "application/zip" | "x-zip-compressed" => ".zip",
        "video/3gpp" | "audio/3gpp" => ".3gp",
        "video/3gpp2" | "audio/3gpp2" => ".3g2",
        "application/x-7z-compressed" => ".7z",
        _ => {
            if mime.starts_with("text/") {
                ".txt"
            } else {
                ""
            }
        }
    }
}

pub fn to_md_lang(mime: &str) -> &str {
    if let Some(value) = mime
        .strip_prefix("text/")
        .or_else(|| mime.strip_prefix("application/"))
    {
        if let Some(value) = value.strip_prefix("x-") {
            value
        } else {
            value
        }
    } else {
        ""
    }
}

pub fn serialize_datetime<S>(date: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = date.format(&Rfc3339).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&formatted)
}

pub fn serialize_option_datetime<S>(
    date: &Option<OffsetDateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => serialize_datetime(date, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_option_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<&str>::deserialize(deserializer)?;
    match opt {
        Some(s) => OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
            .map(Some)
            .map_err(D::Error::custom),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md_lang() {
        assert_eq!(to_md_lang("application/json"), "json");
        assert_eq!(to_md_lang("application/xml"), "xml");
        assert_eq!(to_md_lang("application/octet-stream"), "octet-stream");
        assert_eq!(to_md_lang("application/javascript"), "javascript");
        assert_eq!(to_md_lang("text/x-rust"), "rust");
        assert_eq!(to_md_lang("text/css"), "css");
    }
}

#[derive(Error, Debug)]
pub enum ProtobufError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Unknown wire type: {0}")]
    UnknownWireType(u64),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Too many consecutive errors")]
    TooManyErrors,

    #[error("Field size too large: {0} bytes")]
    FieldTooLarge(u64),

    #[error("End of stream")]
    EndOfStream,

    #[error("Failed to recover from parsing errors")]
    RecoveryFailed,

    #[error("Parse timeout")]
    ParseTimeout,
}
// Protobuf 值类型
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Value {
    Varint(i64),
    Fixed64(f64),
    #[serde(serialize_with = "serialize_bytes")]
    LengthDelimited(Vec<u8>),
    String(String),
    Fixed32(f32),
    Boolean(bool),
    Array(Vec<Value>),
}

// 序列化二进制数据为 base64
fn serialize_bytes<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let base64_str = BASE64.encode(bytes);
    serializer.serialize_str(&base64_str)
}

// Protobuf 解析器
pub struct ProtobufUnknownParser<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    data: R,
    result: HashMap<u32, Value>,
}

impl<R: AsyncRead + AsyncSeek + Unpin> ProtobufUnknownParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            data: reader,
            result: HashMap::new(),
        }
    }

    async fn read_varint(&mut self) -> Result<u64, ProtobufError> {
        let mut result = 0u64;
        let mut shift = 0;

        loop {
            let byte = self.data.read_u8().await?;
            result |= ((byte & 0x7f) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        Ok(result)
    }

    async fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, ProtobufError> {
        let mut buffer = vec![0u8; length];
        self.data.read_exact(&mut buffer).await?;
        Ok(buffer)
    }

    async fn read_double(&mut self) -> Result<f64, ProtobufError> {
        let mut buffer = [0u8; 8];
        self.data.read_exact(&mut buffer).await?;
        Ok(f64::from_le_bytes(buffer))
    }

    async fn read_float(&mut self) -> Result<f32, ProtobufError> {
        let mut buffer = [0u8; 4];
        self.data.read_exact(&mut buffer).await?;
        Ok(f32::from_le_bytes(buffer))
    }

    fn process_varint(&self, value: u64) -> Value {
        if value <= 1 {
            Value::Boolean(value == 1)
        } else {
            Value::Varint(value as i64)
        }
    }

    fn is_valid_utf8(data: &[u8]) -> bool {
        String::from_utf8(data.to_vec()).is_ok()
    }
    pub async fn parse(&mut self) -> &HashMap<u32, Value> {
        let mut error_count = 0;
        const MAX_ERRORS: usize = 10; // 设置一个最大连续错误数
        const MAX_FIELDS: usize = 100000; // 设置一个最大字段数限制

        let mut field_count = 0;

        loop {
            if field_count > MAX_FIELDS {
                debug!("Reached maximum field count: {}", MAX_FIELDS);
                break;
            }
            field_count += 1;

            match self.parse_field().await {
                Ok(true) => {
                    error_count = 0; // 重置错误计数
                    continue;
                }
                Ok(false) => break, // 正常结束
                Err(e) => {
                    debug!("Error parsing field: {}", e);
                    error_count += 1;

                    // 如果连续错误过多，可能是数据彻底损坏，直接退出
                    if error_count > MAX_ERRORS {
                        debug!(
                            "Too many consecutive errors ({}), stopping parsing",
                            MAX_ERRORS
                        );
                        break; // 不返回错误，只是停止解析
                    }

                    // 尝试查找下一个可能的字段开始位置
                    match self.skip_to_next_field().await {
                        Ok(_) => continue, // 找到了可能的下一个字段，继续解析
                        Err(_) => break,   // 无法找到下一个字段，停止解析但不返回错误
                    }
                }
            }
        }

        debug!(
            "Parsed {} fields, result contains {} entries",
            field_count,
            self.result.len()
        );
        &self.result
    }

    // 新增方法，尝试跳过当前损坏的字段
    async fn skip_to_next_field(&mut self) -> Result<(), ProtobufError> {
        // 最多跳过1KB以找到下一个可能的有效字段
        const MAX_SKIP_BYTES: usize = 1024;

        let mut skipped = 0;
        while skipped < MAX_SKIP_BYTES {
            // 尝试读取一个字节
            let byte = match self.data.read_u8().await {
                Ok(b) => b,
                Err(_) => return Err(ProtobufError::EndOfStream),
            };

            skipped += 1;

            // 检查是否可能是一个新字段的开始
            let wire_type = byte & 0x7;
            if wire_type <= 5 {
                // 有效的wire type范围是0-5
                // 计算字段编号 (对于单字节的标记)
                let field_number = byte >> 3;

                // 对于单字节标记，字段编号范围检查
                // 对于u8类型，只需确保field_number > 0即可
                if field_number > 0 {
                    // 找到了可能有效的字段标记
                    self.data.seek(SeekFrom::Current(-1)).await?;
                    debug!("Skipped {} bytes to find next valid field", skipped);
                    return Ok(());
                }
            }
        }

        // 如果跳过了太多字节仍未找到有效字段，返回错误
        debug!(
            "Failed to find valid field after skipping {} bytes",
            skipped
        );
        Err(ProtobufError::RecoveryFailed)
    }

    async fn parse_field(&mut self) -> Result<bool, ProtobufError> {
        let tag = match self.read_varint().await {
            Ok(t) => t,
            Err(e) => {
                if let ProtobufError::Io(ref io_err) = e {
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                        return Ok(false); // 正常结束
                    }
                }
                return Err(e); // 其他错误
            }
        };

        let wire_type = tag & 0x07;
        let field_number = (tag >> 3) as u32;

        // 验证字段编号的合理性
        if field_number == 0 || field_number >= 536870912 {
            // 2^29
            debug!(
                "Invalid field number: {}, attempting to recover",
                field_number
            );
            return self.skip_to_next_field().await.map(|_| true);
        }

        match wire_type {
            0 => {
                // varint
                match self.read_varint().await {
                    Ok(varint_value) => {
                        let value = self.process_varint(varint_value);
                        self.store_value(field_number, value);
                    }
                    Err(e) => {
                        debug!("Error reading varint: {}, attempting to recover", e);
                        return self.skip_to_next_field().await.map(|_| true);
                    }
                }
            }
            1 => {
                // 64-bit
                match self.read_double().await {
                    Ok(value) => {
                        self.store_value(field_number, Value::Fixed64(value));
                    }
                    Err(e) => {
                        debug!("Error reading fixed64: {}, attempting to recover", e);
                        return self.skip_to_next_field().await.map(|_| true);
                    }
                }
            }
            2 => {
                // Length-delimited
                match self.read_varint().await {
                    Ok(length) => {
                        // 安全检查: 限制长度
                        const MAX_FIELD_SIZE: usize = 10 * 1024 * 1024; // 10MB
                        let length = length as usize;

                        if length > MAX_FIELD_SIZE {
                            debug!("Field too large: {}bytes, attempting to recover", length);
                            return self.skip_to_next_field().await.map(|_| true);
                        }

                        match self.read_bytes(length).await {
                            Ok(bytes) => {
                                let value = if Self::is_valid_utf8(&bytes) {
                                    Value::String(String::from_utf8(bytes)?)
                                } else {
                                    Value::LengthDelimited(bytes)
                                };
                                self.store_value(field_number, value);
                            }
                            Err(e) => {
                                debug!("Error reading bytes: {}, attempting to recover", e);
                                return self.skip_to_next_field().await.map(|_| true);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Error reading length: {}, attempting to recover", e);
                        return self.skip_to_next_field().await.map(|_| true);
                    }
                }
            }
            5 => {
                // 32-bit
                match self.read_float().await {
                    Ok(value) => {
                        self.store_value(field_number, Value::Fixed32(value));
                    }
                    Err(e) => {
                        debug!("Error reading fixed32: {}, attempting to recover", e);
                        return self.skip_to_next_field().await.map(|_| true);
                    }
                }
            }
            _ => {
                // Unknown wire type
                debug!("Unknown wire type: {}, attempting to recover", wire_type);
                return self.skip_to_next_field().await.map(|_| true);
            }
        }

        Ok(true)
    }

    // 辅助方法，用于存储值
    fn store_value(&mut self, field_number: u32, value: Value) {
        match self.result.get_mut(&field_number) {
            Some(existing_value) => match existing_value {
                Value::Array(vec) => vec.push(value),
                _ => {
                    let old_value = std::mem::replace(existing_value, Value::Array(vec![value]));
                    if let Value::Array(vec) = existing_value {
                        vec.insert(0, old_value);
                    }
                }
            },
            None => {
                self.result.insert(field_number, value);
            }
        }
    }

    pub fn analyze_structure(&self) {
        let mut structure: HashMap<u32, HashSet<String>> = HashMap::new();

        for (field, value) in &self.result {
            let type_set = structure.entry(*field).or_insert_with(HashSet::new);
            match value {
                Value::Array(arr) => {
                    if let Some(first) = arr.first() {
                        type_set.insert(format!("Array<{}>", Self::get_type_name(first)));
                    } else {
                        type_set.insert("Array<unknown>".to_string());
                    }
                }
                _ => {
                    type_set.insert(Self::get_type_name(value));
                }
            }
        }

        debug!("推测的消息结构:");
        for (field, types) in &structure {
            let type_strings: Vec<String> = types.iter().cloned().collect();
            debug!("{}: {}", field, type_strings.join(" | "));
        }
    }

    fn get_type_name(value: &Value) -> String {
        match value {
            Value::Varint(_) => "Varint".to_string(),
            Value::Fixed64(_) => "Fixed64".to_string(),
            Value::LengthDelimited(_) => "Bytes".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Fixed32(_) => "Fixed32".to_string(),
            Value::Boolean(_) => "Boolean".to_string(),
            Value::Array(_) => "Array".to_string(),
        }
    }
}

pub async fn parse_unknown_protobuf(buffer: &[u8]) -> HashMap<u32, Value> {
    let cursor = Cursor::new(buffer);
    let mut parser = ProtobufUnknownParser::new(cursor);

    // 尝试解析，即使出错也返回已解析的部分
    match parser.parse().await {
        result => result.clone(),
    }
}

/// 检查是否为本地请求  
pub fn is_local_request(url: &str) -> bool {
    // 尝试解析URL
    if let Ok(parsed_url) = url::Url::parse(url) {
        let host = parsed_url.host_str().unwrap_or("");

        // 检查是否是localhost
        if host.eq_ignore_ascii_case("localhost") {
            return true;
        }

        // 检查是否是127.0.0.1
        if host == "127.0.0.1" {
            return true;
        }

        // 检查是否是::1 (IPv6的本地地址)
        if host == "[::1]" || host == "::1" {
            return true;
        }

        // 可选：检查是否是本地网络地址
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            return ip.is_loopback();
        }
    }

    false
}

/// 从URL中提取域名  
pub fn extract_domain(url: &str) -> String {
    if let Ok(parsed_url) = url::Url::parse(url) {
        // 获取主机名
        if let Some(host) = parsed_url.host_str() {
            return host.to_string();
        }
    }

    // 如果URL解析失败，尝试直接从字符串中提取域名
    // 移除协议部分
    let without_protocol = url.split("://").nth(1).unwrap_or(url);

    // 获取第一个斜杠或问号之前的部分（域名部分）
    let domain = without_protocol
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("");

    domain.to_string()
}

pub fn add_data_url_prefix(base64_data: &str) -> String {
    // 移除可能存在的空白字符
    let clean_data = base64_data.trim();

    // 判断是否已经是Data URL
    if clean_data.starts_with("data:") {
        // 已经是Data URL，直接返回
        return clean_data.to_string();
    }

    // 文件类型识别规则（基于Base64编码的文件头）
    let mime_type = if clean_data.starts_with("iVBOR") {
        Some("image/png")
    } else if clean_data.starts_with("/9j/") {
        Some("image/jpeg")
    } else if clean_data.starts_with("R0lG") {
        Some("image/gif")
    } else if clean_data.starts_with("UklGR") {
        Some("image/webp")
    } else if clean_data.starts_with("Qk") {
        Some("image/bmp")
    } else if clean_data.starts_with("PD94") {
        Some("image/svg+xml") // 可能是SVG（XML头部）
    } else if clean_data.starts_with("JVBERi0") {
        Some("application/pdf") // PDF
    } else {
        // 无法识别类型，返回None
        None
    };

    // 如果能识别类型，构建完整的Data URL；否则返回原始Base64
    match mime_type {
        Some(mime) => format!("data:{};base64,{}", mime, clean_data),
        None => clean_data.to_string(), // 无法识别时返回原始Base64
    }
}
