// ============================================================================
// Filename: licensegen/src/vault/loader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-12
// Modified: 2026-03-13
// Description: 
// ============================================================================
use log::info;
use std::path::{Path, PathBuf};

use crate::config::config::Config;
use crate::util::helpers::{ensure_permissions, resolve_path};
use super::types::{VaultError, VaultSecrets};
use super::ansible::decrypt_with_ansible;

pub fn load_secrets(cfg: &Config) -> Result<VaultSecrets, VaultError> {
    // Vault paths are already resolved by config::resolver
    let vault_file = PathBuf::from(&cfg.vault.file);
    let password_file = PathBuf::from(&cfg.vault.password_file);

    // 1. Decrypt vault
    let yaml_str = decrypt_with_ansible(&vault_file, &password_file)?;
    info!("Vault decrypted successfully");

    // 2. Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&yaml_str)
        .map_err(|e| VaultError::YamlError(e.to_string()))?;

    // 3. Extract subtree for this application
    let app_key = env!("CARGO_PKG_NAME").to_lowercase();
    let subtree = yaml
        .get(&app_key)
        .ok_or_else(|| VaultError::YamlError(format!("Missing vault key '{}'", app_key)))?;

    // 4. Deserialize into VaultSecrets
    let secrets: VaultSecrets = serde_yaml::from_value(subtree.clone())
        .map_err(|e| VaultError::YamlError(e.to_string()))?;

    info!("Vault secrets loaded for {}", app_key);
    Ok(secrets)
}

pub fn load_vault(cfg: &Config, cfg_dir: &Path) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let (vault_file, password_file) = resolve_vault_paths(cfg, cfg_dir);

    // Check directory permissions
    let vault_dir = vault_file.parent().unwrap();
    ensure_permissions(vault_dir, 0o700)?;

    // Check file permissions
    ensure_permissions(&vault_file, 0o600)?;
    ensure_permissions(&password_file, 0o600)?;

    Ok((vault_file, password_file))
}

pub fn resolve_vault_paths(cfg: &Config, cfg_dir: &Path) -> (PathBuf, PathBuf) {
    let raw_vault_file = std::env::var("LICENSEGEN_VAULT_FILE")
        .unwrap_or_else(|_| cfg.vault.file.clone());

    let raw_password_file = std::env::var("LICENSEGEN_VAULT_PASSWORD_FILE")
        .unwrap_or_else(|_| cfg.vault.password_file.clone());

    let vault_file = resolve_path(cfg_dir, &raw_vault_file);
    let password_file = resolve_path(cfg_dir, &raw_password_file);

    (vault_file, password_file)
}
