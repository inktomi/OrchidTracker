use aes_gcm::{Aes256Gcm, AeadCore, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use sha2::{Sha256, Digest};

use crate::config::config;
use crate::error::AppError;

const DOMAIN_PREFIX: &str = "orchid-tracker-encryption:";
const NONCE_LEN: usize = 12;

fn derive_key() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN_PREFIX.as_bytes());
    hasher.update(config().session_secret.as_bytes());
    hasher.finalize().into()
}

pub fn encrypt(plaintext: &str) -> Result<String, AppError> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Serialization(format!("cipher init: {e}")))?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Serialization(format!("encrypt: {e}")))?;

    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt(ciphertext_b64: &str) -> Result<String, AppError> {
    let combined = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| AppError::Serialization(format!("base64 decode: {e}")))?;

    if combined.len() < NONCE_LEN + 1 {
        return Err(AppError::Serialization("ciphertext too short".into()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Serialization(format!("cipher init: {e}")))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Serialization(format!("decrypt: {e}")))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Serialization(format!("utf8: {e}")))
}

/// Try to decrypt; fall back to returning the raw string (handles legacy plaintext).
pub fn decrypt_or_raw(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    decrypt(value).unwrap_or_else(|_| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_config() {
        INIT.call_once(|| {
            // SAFETY: only called once via Once, before any threads read these vars
            unsafe {
                std::env::set_var("SESSION_SECRET", "test-secret-for-unit-tests-must-be-long-enough");
            }
            let _ = std::panic::catch_unwind(|| crate::config::init_config());
        });
    }

    #[test]
    fn round_trip() {
        ensure_config();

        let original = r#"{"token":"abc123","station_id":"42"}"#;
        let encrypted = encrypt(original).expect("encrypt failed");
        assert_ne!(encrypted, original);
        let decrypted = decrypt(&encrypted).expect("decrypt failed");
        assert_eq!(decrypted, original);
    }

    #[test]
    fn decrypt_or_raw_plaintext_fallback() {
        ensure_config();

        let plaintext = r#"{"email":"a@b.com","password":"secret"}"#;
        let result = decrypt_or_raw(plaintext);
        assert_eq!(result, plaintext);
    }

    #[test]
    fn decrypt_or_raw_empty() {
        let result = decrypt_or_raw("");
        assert_eq!(result, "");
    }
}
