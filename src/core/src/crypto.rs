//! AES-256-GCM encryption/decryption for sensitive data at rest.
//!
//! The master key is read from the `OPSPILOT_MASTER_KEY` environment variable
//! (base64-encoded 32-byte key). If the variable is unset, a random key is
//! generated for the session (data will NOT survive restarts — dev mode only).

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use tracing::warn;

/// Errors from crypto operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("cipher error: {0}")]
    Cipher(String),

    #[error("invalid key length: expected 32 bytes, got {0}")]
    InvalidKeyLength(usize),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
}

/// 256-bit (32 byte) key for AES-256-GCM.
pub struct MasterKey([u8; 32]);

impl MasterKey {
    /// Load the master key from `OPSPILOT_MASTER_KEY` env var, or generate a
    /// random ephemeral key for development.
    pub fn load() -> Self {
        if let Ok(encoded) = std::env::var("OPSPILOT_MASTER_KEY") {
            match B64.decode(&encoded) {
                Ok(bytes) if bytes.len() == 32 => {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes);
                    Self(key)
                }
                Ok(bytes) => {
                    warn!(
                        expected = 32,
                        got = bytes.len(),
                        "OPSPILOT_MASTER_KEY has wrong length; generating ephemeral key"
                    );
                    Self::ephemeral()
                }
                Err(e) => {
                    warn!(error = %e, "failed to decode OPSPILOT_MASTER_KEY; generating ephemeral key");
                    Self::ephemeral()
                }
            }
        } else {
            warn!("OPSPILOT_MASTER_KEY not set; using ephemeral key (data won't survive restart)");
            Self::ephemeral()
        }
    }

    fn ephemeral() -> Self {
        use aes_gcm::aead::rand_core::RngCore;
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Encrypt `plaintext` using AES-256-GCM.
///
/// Returns `(ciphertext, nonce_iv)` where both are owned byte vecs.
pub fn encrypt(plaintext: &[u8], key: &MasterKey) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    let cipher =
        Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|e| CryptoError::Cipher(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    use aes_gcm::aead::rand_core::RngCore;
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::Cipher(e.to_string()))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypt `ciphertext` using AES-256-GCM.
pub fn decrypt(
    ciphertext: &[u8],
    key: &MasterKey,
    iv: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher =
        Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|e| CryptoError::Cipher(e.to_string()))?;

    let nonce = Nonce::from_slice(iv);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = MasterKey::load();
        let plaintext = b"hello, world! this is a secret message.";

        let (ciphertext, iv) = encrypt(plaintext, &key).unwrap();
        assert_ne!(&ciphertext, plaintext);
        assert_eq!(iv.len(), 12);

        let decrypted = decrypt(&ciphertext, &key, &iv).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext_each_time() {
        let key = MasterKey::load();
        let plaintext = b"same input";

        let (ct1, iv1) = encrypt(plaintext, &key).unwrap();
        let (ct2, iv2) = encrypt(plaintext, &key).unwrap();

        // Nonces should differ (random), so ciphertext should differ too
        assert_ne!(iv1, iv2);
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = MasterKey::load();
        let key2 = MasterKey::load();

        let (ciphertext, iv) = encrypt(b"secret", &key1).unwrap();
        let result = decrypt(&ciphertext, &key2, &iv);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_wrong_iv_fails() {
        let key = MasterKey::load();
        let (ciphertext, _iv) = encrypt(b"secret", &key).unwrap();
        let wrong_iv = [0u8; 12];
        let result = decrypt(&ciphertext, &key, &wrong_iv);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let key = MasterKey::load();
        let (mut ciphertext, iv) = encrypt(b"secret", &key).unwrap();
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xff;
        }
        let result = decrypt(&ciphertext, &key, &iv);
        assert!(result.is_err());
    }

    #[test]
    fn test_master_key_load_ephemeral() {
        // With no env var set, should get an ephemeral key
        let key = MasterKey::load();
        assert_eq!(key.as_bytes().len(), 32);
    }
}
