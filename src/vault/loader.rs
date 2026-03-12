// ============================================================================
// Filename: licensegen/src/vault/loader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-12
// Modified: 2026-03-12
// Description: 
// ============================================================================
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::util::helpers::{check_permissions, ensure_permissions, resolve_path};

pub fn load_vault(cfg: &Config, cfg_dir: &Path) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>>{
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
