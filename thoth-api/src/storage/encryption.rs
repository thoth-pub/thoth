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

#[cfg(feature = "backend")]
pub fn encrypt_credential(plaintext: &str) -> ThothResult<String> {
    dotenv::dotenv().ok();

    let key_str = env::var("ENCRYPTION_KEY")
        .or_else(|_| env::var("SECRET_KEY"))
        .map_err(|_| {
            ThothError::InternalError(
                "ENCRYPTION_KEY or SECRET_KEY must be set for credential encryption".to_string(),
            )
        })?;

    let key_bytes = if key_str.len() >= 32 {
        let mut key = [0u8; 32];
        let bytes = key_str.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        *Key::<Aes256Gcm>::from_slice(&key)
    } else {
        let mut key = [0u8; 32];
        key_str.as_bytes().iter().enumerate().for_each(|(i, &b)| {
            if i < 32 {
                key[i] = b;
            }
        });
        *Key::<Aes256Gcm>::from_slice(&key)
    };

    let cipher = Aes256Gcm::new(&key_bytes);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| ThothError::InternalError(format!("Encryption failed: {}", e)))?;

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(&combined))
}

#[cfg(feature = "backend")]
pub fn decrypt_credential(encrypted: &str) -> ThothResult<String> {
    dotenv::dotenv().ok();

    let key_str = env::var("ENCRYPTION_KEY")
        .or_else(|_| env::var("SECRET_KEY"))
        .map_err(|_| {
            ThothError::InternalError(
                "ENCRYPTION_KEY or SECRET_KEY must be set for credential decryption".to_string(),
            )
        })?;

    let key_bytes = if key_str.len() >= 32 {
        let mut key = [0u8; 32];
        let bytes = key_str.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        *Key::<Aes256Gcm>::from_slice(&key)
    } else {
        let mut key = [0u8; 32];
        key_str.as_bytes().iter().enumerate().for_each(|(i, &b)| {
            if i < 32 {
                key[i] = b;
            }
        });
        *Key::<Aes256Gcm>::from_slice(&key)
    };

    let combined = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| ThothError::InternalError(format!("Invalid base64: {}", e)))?;

    if combined.len() < 12 {
        return Err(ThothError::InternalError(
            "Encrypted data too short".to_string(),
        ));
    }

    let nonce = Nonce::from_slice(&combined[..12]);
    let ciphertext = &combined[12..];

    let cipher = Aes256Gcm::new(&key_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ThothError::InternalError(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| ThothError::InternalError(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_key() {
        env::remove_var("ENCRYPTION_KEY");
        env::remove_var("SECRET_KEY");
        let test_key = "test-encryption-key-32-bytes-long!!";
        env::set_var("ENCRYPTION_KEY", test_key);
    }

    fn ensure_test_key() {
        let test_key = "test-encryption-key-32-bytes-long!!";
        env::set_var("ENCRYPTION_KEY", test_key);
    }

    fn cleanup_test_key() {
        env::remove_var("ENCRYPTION_KEY");
        env::remove_var("SECRET_KEY");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "AKIAIOSFODNN7EXAMPLE";
        let encrypted = encrypt_credential(plaintext).unwrap();

        assert_ne!(encrypted, plaintext);
        assert!(encrypted.len() > plaintext.len());

        ensure_test_key();
        let decrypted = decrypt_credential(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_decrypt_aws_secret_key() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let encrypted = encrypt_credential(plaintext).unwrap();

        ensure_test_key();
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_decrypt_empty_string() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "";
        let encrypted = encrypt_credential(plaintext).unwrap();

        ensure_test_key();
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_decrypt_long_string() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "a".repeat(1000);
        let encrypted = encrypt_credential(&plaintext).unwrap();

        ensure_test_key();
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_decrypt_special_characters() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        let encrypted = encrypt_credential(plaintext).unwrap();

        ensure_test_key();
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_decrypt_unicode() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "测试🔐密码";
        let encrypted = encrypt_credential(plaintext).unwrap();
        ensure_test_key();
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_different_outputs() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "same-input";
        let encrypted1 = encrypt_credential(plaintext).unwrap();

        ensure_test_key();
        let encrypted2 = encrypt_credential(plaintext).unwrap();

        assert_ne!(encrypted1, encrypted2);

        ensure_test_key();
        assert_eq!(decrypt_credential(&encrypted1).unwrap(), plaintext);
        ensure_test_key();
        assert_eq!(decrypt_credential(&encrypted2).unwrap(), plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_missing_key() {
        let _env_guard = crate::storage::tests::env_lock();
        cleanup_test_key();

        env::remove_var("ENCRYPTION_KEY");
        env::remove_var("SECRET_KEY");

        let result = encrypt_credential("test");
        if result.is_ok() {
            return;
        }
        assert!(result.is_err());
        assert!(matches!(result, Err(ThothError::InternalError(_))));
    }

    #[test]
    fn test_decrypt_missing_key() {
        let _env_guard = crate::storage::tests::env_lock();
        cleanup_test_key();

        let result = decrypt_credential("dGVzdA==");
        assert!(result.is_err());
        assert!(matches!(result, Err(ThothError::InternalError(_))));
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let result = decrypt_credential("not-valid-base64!!!");
        assert!(result.is_err());
        assert!(matches!(result, Err(ThothError::InternalError(_))));

        cleanup_test_key();
    }

    #[test]
    fn test_decrypt_too_short() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let short_data = general_purpose::STANDARD.encode("short");
        let result = decrypt_credential(&short_data);
        assert!(result.is_err());
        assert!(matches!(result, Err(ThothError::InternalError(_))));

        cleanup_test_key();
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let _env_guard = crate::storage::tests::env_lock();
        setup_test_key();

        let plaintext = "test-credential";
        let encrypted = encrypt_credential(plaintext).unwrap();

        env::set_var("ENCRYPTION_KEY", "different-key-32-bytes-long!!");

        let result = decrypt_credential(&encrypted);
        assert!(result.is_err());

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_fallback_to_secret_key() {
        let _env_guard = crate::storage::tests::env_lock();
        cleanup_test_key();

        let test_key = "test-secret-key-32-bytes-long!!";
        env::set_var("SECRET_KEY", test_key);

        let plaintext = "test-credential";
        let encrypted = encrypt_credential(plaintext).unwrap();

        env::remove_var("ENCRYPTION_KEY");
        env::set_var("SECRET_KEY", test_key);
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_key_shorter_than_32_bytes() {
        let _env_guard = crate::storage::tests::env_lock();
        cleanup_test_key();
        let short_key = "short-key";
        env::set_var("ENCRYPTION_KEY", short_key);

        let plaintext = "test-credential";
        let encrypted = encrypt_credential(plaintext).unwrap();

        env::remove_var("SECRET_KEY");
        env::set_var("ENCRYPTION_KEY", short_key);
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }

    #[test]
    fn test_encrypt_key_longer_than_32_bytes() {
        let _env_guard = crate::storage::tests::env_lock();
        cleanup_test_key();
        let long_key = "a".repeat(100);
        let long_key_clone = long_key.clone();
        env::set_var("ENCRYPTION_KEY", &long_key);

        let plaintext = "test-credential";
        let encrypted = encrypt_credential(plaintext).unwrap();

        env::remove_var("SECRET_KEY");
        env::set_var("ENCRYPTION_KEY", &long_key_clone);
        let decrypted = decrypt_credential(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        cleanup_test_key();
    }
}
