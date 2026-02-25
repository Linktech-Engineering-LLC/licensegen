// ============================================================================
// Filename: licensegen/src/vault/reader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-02-24
// Description: Ansible Vault AES256 decryptor for licensegen.
// ============================================================================

// System Libraries
use std::fs;
use std::path::Path;

// Project Libraries
use super::VaultSecrets;
use crate::config::Config;

// Crypto Libraries
use aes::Aes256;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use block_padding::Pkcs7;
use cbc::Decryptor;
use hex;
use pbkdf2::pbkdf2_hmac;
use serde_yaml;
use sha2::Sha256;

// Errors
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Failed to read vault file: {0}")]
    ReadError(String),

    #[error("Invalid vault header format")]
    HeaderError,

    #[error("Decryption failed")]
    DecryptError,

    #[error("YAML parse error: {0}")]
    YamlError(String),
}

pub fn load_vault(cfg: &Config) -> Result<VaultSecrets, VaultError> {
    // Read vault file
    let raw =
        fs::read_to_string(&cfg.vault.file).map_err(|e| VaultError::ReadError(e.to_string()))?;

    // Read password file
    let password = fs::read_to_string(&cfg.vault.password_file)
        .map_err(|e| VaultError::ReadError(e.to_string()))?;
    let password = password.trim();

    // Parse header
    let mut lines = raw.lines();
    let header = lines.next().ok_or(VaultError::HeaderError)?;

    if !header.starts_with("$ANSIBLE_VAULT;1.1;AES256") {
        return Err(VaultError::HeaderError);
    }

    // Remaining lines are hex ciphertext
    let hex_ciphertext: String = lines.collect::<Vec<_>>().join("");
    let ciphertext = hex::decode(hex_ciphertext).map_err(|_| VaultError::DecryptError)?;

    // Derive key + IV using PBKDF2-SHA256
    let salt = b"ansible"; // Ansible uses a fixed salt for vault 1.1
    let mut key_iv = [0u8; 48]; // 32-byte key + 16-byte IV

    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 10000, &mut key_iv);

    let (key, iv) = key_iv.split_at(32);

    // AES256-CBC decrypt
    let cipher =
        Decryptor::<Aes256>::new_from_slices(key, iv).map_err(|_| VaultError::DecryptError)?;

    let mut buf = ciphertext.clone();

    let decrypted = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| VaultError::DecryptError)?;

    let yaml_str = String::from_utf8_lossy(&decrypted);

    // Parse YAML
    let secrets: VaultSecrets =
        serde_yaml::from_str(&yaml_str).map_err(|e| VaultError::YamlError(e.to_string()))?;

    Ok(secrets)
}
