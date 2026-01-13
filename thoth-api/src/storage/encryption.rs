#[cfg(feature = "backend")]
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
#[cfg(feature = "backend")]
use base64::{engine::general_purpose, Engine as _};
#[cfg(feature = "backend")]
use std::env;
#[cfg(feature = "backend")]
use thoth_errors::{ThothError, ThothResult};

/// Encrypt AWS credentials using AES-256-GCM
///
/// Uses ENCRYPTION_KEY environment variable (32 bytes, base64 encoded)
/// or falls back to SECRET_KEY if ENCRYPTION_KEY is not set.
#[cfg(feature = "backend")]
pub fn encrypt_credential(plaintext: &str) -> ThothResult<String> {
    dotenv::dotenv().ok();

    // Get encryption key from environment
    let key_str = env::var("ENCRYPTION_KEY")
        .or_else(|_| env::var("SECRET_KEY"))
        .map_err(|_| {
            ThothError::InternalError(
                "ENCRYPTION_KEY or SECRET_KEY must be set for credential encryption".to_string(),
            )
        })?;

    // Decode key from base64 or use directly (padded to 32 bytes)
    let key_bytes = if key_str.len() >= 32 {
        // Use first 32 bytes if key is longer
        let mut key = [0u8; 32];
        let bytes = key_str.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Key::<Aes256Gcm>::from_slice(&key).clone()
    } else {
        // Pad key to 32 bytes
        let mut key = [0u8; 32];
        key_str.as_bytes().iter().enumerate().for_each(|(i, &b)| {
            if i < 32 {
                key[i] = b;
            }
        });
        Key::<Aes256Gcm>::from_slice(&key).clone()
    };

    let cipher = Aes256Gcm::new(&key_bytes);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| ThothError::InternalError(format!("Encryption failed: {}", e)))?;

    // Combine nonce and ciphertext, then encode as base64
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(&combined))
}

/// Decrypt AWS credentials using AES-256-GCM
#[cfg(feature = "backend")]
pub fn decrypt_credential(encrypted: &str) -> ThothResult<String> {
    dotenv::dotenv().ok();

    // Get encryption key from environment
    let key_str = env::var("ENCRYPTION_KEY")
        .or_else(|_| env::var("SECRET_KEY"))
        .map_err(|_| {
            ThothError::InternalError(
                "ENCRYPTION_KEY or SECRET_KEY must be set for credential decryption".to_string(),
            )
        })?;

    // Decode key from base64 or use directly (padded to 32 bytes)
    let key_bytes = if key_str.len() >= 32 {
        let mut key = [0u8; 32];
        let bytes = key_str.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Key::<Aes256Gcm>::from_slice(&key).clone()
    } else {
        let mut key = [0u8; 32];
        key_str.as_bytes().iter().enumerate().for_each(|(i, &b)| {
            if i < 32 {
                key[i] = b;
            }
        });
        Key::<Aes256Gcm>::from_slice(&key).clone()
    };

    // Decode base64
    let combined = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| ThothError::InternalError(format!("Invalid base64: {}", e)))?;

    if combined.len() < 12 {
        return Err(ThothError::InternalError(
            "Encrypted data too short".to_string(),
        ));
    }

    // Extract nonce (first 12 bytes) and ciphertext
    let nonce = Nonce::from_slice(&combined[..12]);
    let ciphertext = &combined[12..];

    let cipher = Aes256Gcm::new(&key_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ThothError::InternalError(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| ThothError::InternalError(format!("Invalid UTF-8: {}", e)))
}
