use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

#[derive(Debug, Error)]
pub enum DecryptError {
    #[error("Failed to decode base64: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("UTF-8 conversion failed: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub struct CryptoService {
    key: [u8; 32],
    iv: [u8; 16],
}

impl CryptoService {
    pub fn new(secret_key: &str, iv: &str) -> Result<Self, DecryptError> {
        // 验证密钥长度
        if secret_key.len() != 32 {
            return Err(DecryptError::InvalidInput(
                "SECRET_KEY must be 32 characters long".to_string(),
            ));
        }

        // 验证IV长度
        if iv.len() != 16 {
            return Err(DecryptError::InvalidInput(
                "IV must be 16 characters long".to_string(),
            ));
        }

        let key = secret_key.as_bytes().try_into().map_err(|_| {
            DecryptError::InvalidInput("Failed to convert key to bytes".to_string())
        })?;

        let iv = iv
            .as_bytes()
            .try_into()
            .map_err(|_| DecryptError::InvalidInput("Failed to convert IV to bytes".to_string()))?;

        Ok(Self { key, iv })
    }

    pub fn decrypt(&self, encrypted_text: &str) -> Result<String, DecryptError> {
        // 验证输入
        if encrypted_text.is_empty() {
            return Err(DecryptError::InvalidInput(
                "Encrypted text cannot be empty".to_string(),
            ));
        }

        // Base64解码
        let encrypted_data = BASE64.decode(encrypted_text)?;

        // 创建解密器
        let cipher = Aes256CbcDec::new(&self.key.into(), &self.iv.into());

        // 解密
        let mut buf = encrypted_data.to_vec();
        let decrypted_data = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buf)
            .map_err(|e| DecryptError::DecryptionError(e.to_string()))?;

        // 转换为UTF-8字符串
        let decrypted_text = String::from_utf8(decrypted_data.to_vec())?;

        Ok(decrypted_text)
    }
}

// 为了方便在命令中使用，可以创建一个包含配置的结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoConfig {
    secret_key: String,
    iv: String,
}

// 创建全局实例
lazy_static::lazy_static! {
    pub static ref CRYPTO_SERVICE: CryptoService = CryptoService::new(
        "mK9bP2vN8xL5tR7hJ4fD1cA3gE6iQ0wS",  // 32字符密钥
        "uY5nM2kX7pJ9vB4c"   // 16字符IV
    ).expect("Failed to initialize crypto service");
}
