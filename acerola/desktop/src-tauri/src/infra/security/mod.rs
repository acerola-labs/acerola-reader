//! Encryption at rest for everything `acerola-p2p` persists (node identity,
//! peer cache, TOFU trust list) — see [`p2p_storage`] and [`trusted_store`].
//!
//! Design: the OS keystore (Credential Manager on Windows, Secret Service on Linux,
//! via the `keyring` crate) only protects a small AES-256 master key, generated once.
//! That master key encrypts (AES-256-GCM) everything else before it touches the disk —
//! growing lists cannot be stored directly in the OS keyring (it is not meant for
//! large blobs), but a 32-byte key fits just fine.
//!
//! **Fallback without keyring**: on Linux without a running Secret Service provider (e.g.
//! Hyprland/sway without gnome-keyring/kwallet configured), the master key falls back to a
//! local file without the OS protection — it works, but with reduced security. This
//! NEVER fails silently: [`get_or_create_master_key`] returns which source the
//! key came from, and the caller (`bios::network::setup_network`) emits an event for the
//! frontend to warn the user — same principle as VS Code when it cannot find a keyring.

pub mod p2p_storage;
pub mod trusted_store;

use std::path::Path;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Key,
};
use rand::RngCore;

use crate::infra::error::ComicError;

const KEYRING_SERVICE: &str = "Acerola Reader";
const LEGACY_KEYRING_SERVICES: &[&str] = &["br.acerola.reader", "br.acerola.comic"];
const KEYRING_USERNAME: &str = "p2p-master-key";
const MASTER_KEY_LEN: usize = 32;
/// Deliberately explicit name — if someone sees this file on disk, the name already
/// warns that it is not protected by the system keyring.
const FALLBACK_KEY_FILE_NAME: &str = "master.key.insecure";

/// Where the master key actually came from — the caller uses this to decide whether
/// it needs to warn the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterKeySource {
    /// Protected by the OS keystore (Credential Manager / Secret Service).
    Keyring,
    /// The OS has no usable keyring backend right now — the key fell back to a
    /// local file without that extra layer of protection.
    FallbackFile,
}

/// Gets the master key, trying the OS keyring first and falling back to a local
/// file (`master.key.insecure` in `fallback_dir`) only if the keyring is really not
/// available — it does not confuse "no entry exists yet" (normal first use)
/// with "there is no backend at all" (that is when the user must be warned).
pub fn get_or_create_master_key(
    fallback_dir: &Path,
) -> Result<([u8; MASTER_KEY_LEN], MasterKeySource), ComicError> {
    match try_keyring_master_key() {
        Ok(key) => Ok((key, MasterKeySource::Keyring)),
        Err(err) => {
            tracing::warn!(
                error = %err,
                "OS keyring unavailable, falling back to unencrypted local key file \
                 (identity/peers/trust will still be AES-encrypted with it, just without \
                 OS-level protection on the key itself)"
            );
            let key = get_or_create_fallback_master_key(fallback_dir)?;
            Ok((key, MasterKeySource::FallbackFile))
        },
    }
}

/// Only returns `Ok` if the key really came from the OS keyring. Any real backend
/// failure (not simply "entry does not exist yet") becomes `Err`, signaling the
/// caller to fall back.
fn try_keyring_master_key() -> keyring::Result<[u8; MASTER_KEY_LEN]> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)?;

    match entry.get_secret() {
        Ok(bytes) if bytes.len() == MASTER_KEY_LEN => {
            let mut key = [0u8; MASTER_KEY_LEN];
            key.copy_from_slice(&bytes);
            Ok(key)
        },
        // Missing (first use) or corrupted/wrong size — try migrating from old services
        // before generating a new one.
        Ok(_) | Err(keyring::Error::NoEntry) => {
            for &legacy_service in LEGACY_KEYRING_SERVICES {
                if let Ok(old_entry) = keyring::Entry::new(legacy_service, KEYRING_USERNAME) {
                    if let Ok(old_bytes) = old_entry.get_secret() {
                        if old_bytes.len() == MASTER_KEY_LEN {
                            let mut key = [0u8; MASTER_KEY_LEN];
                            key.copy_from_slice(&old_bytes);
                            if let Err(save_err) = entry.set_secret(&key) {
                                tracing::warn!(
                                    error = %save_err,
                                    "Failed to persist migrated master key to new keyring service"
                                );
                            } else {
                                tracing::info!(
                                    "Migrated master key from '{legacy_service}' to '{KEYRING_SERVICE}'"
                                );
                            }
                            return Ok(key);
                        }
                    }
                }
            }

            let mut key = [0u8; MASTER_KEY_LEN];
            rand::thread_rng().fill_bytes(&mut key);
            entry.set_secret(&key)?;
            Ok(key)
        },
        Err(other) => Err(other),
    }
}

fn get_or_create_fallback_master_key(dir: &Path) -> Result<[u8; MASTER_KEY_LEN], ComicError> {
    std::fs::create_dir_all(dir)
        .map_err(|err| ComicError::SystemFailure(format!("failed to create data dir: {err}")))?;
    let path = dir.join(FALLBACK_KEY_FILE_NAME);

    match std::fs::read(&path) {
        Ok(bytes) if bytes.len() == MASTER_KEY_LEN => {
            let mut key = [0u8; MASTER_KEY_LEN];
            key.copy_from_slice(&bytes);
            Ok(key)
        },
        _ => {
            let mut key = [0u8; MASTER_KEY_LEN];
            rand::thread_rng().fill_bytes(&mut key);
            std::fs::write(&path, key).map_err(|err| {
                ComicError::SystemFailure(format!("failed to save fallback master key: {err}"))
            })?;
            Ok(key)
        },
    }
}

/// Encrypts `plaintext` with AES-256-GCM. The nonce (96 bits, random per call) is
/// prefixed to the returned blob — it does not need to be stored separately to decrypt later.
pub fn encrypt(key: &[u8; MASTER_KEY_LEN], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    // Only fails if the buffer is absurdly large (>2^36 bytes) — not a real case here.
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("AES-GCM encryption should not fail for small blobs");

    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    out
}

/// Decrypts a blob produced by [`encrypt`] (prefixed nonce + ciphertext).
pub fn decrypt(key: &[u8; MASTER_KEY_LEN], blob: &[u8]) -> Result<Vec<u8>, ComicError> {
    const NONCE_LEN: usize = 12;

    if blob.len() < NONCE_LEN {
        return Err(ComicError::SystemFailure(
            "encrypted blob too short to contain a nonce".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    cipher.decrypt(nonce_bytes.into(), ciphertext).map_err(|_| {
        ComicError::SystemFailure("failed to decrypt blob: wrong key or corrupted data".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip_returns_original_plaintext() {
        let key = [7u8; MASTER_KEY_LEN];
        let plaintext = b"acerola p2p identity seed material";

        let ciphertext = encrypt(&key, plaintext);
        assert_ne!(ciphertext, plaintext, "ciphertext should not equal plaintext");

        let decrypted = decrypt(&key, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_is_non_deterministic_due_to_random_nonce() {
        let key = [3u8; MASTER_KEY_LEN];
        let plaintext = b"same input, different ciphertext each time";

        let first = encrypt(&key, plaintext);
        let second = encrypt(&key, plaintext);

        assert_ne!(
            first, second,
            "two encryptions of the same plaintext must differ (random nonce)"
        );
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key_a = [1u8; MASTER_KEY_LEN];
        let key_b = [2u8; MASTER_KEY_LEN];
        let ciphertext = encrypt(&key_a, b"secret");

        assert!(decrypt(&key_b, &ciphertext).is_err());
    }

    #[test]
    fn decrypt_rejects_truncated_blob() {
        let key = [9u8; MASTER_KEY_LEN];
        assert!(decrypt(&key, b"short").is_err());
    }

    #[test]
    fn fallback_master_key_is_stable_across_calls() {
        let dir = tempfile::tempdir().unwrap();
        let first = get_or_create_fallback_master_key(dir.path()).unwrap();
        let second = get_or_create_fallback_master_key(dir.path()).unwrap();
        assert_eq!(first, second, "re-reading the fallback key file must return the same key");
    }
}
