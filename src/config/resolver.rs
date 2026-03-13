// ============================================================================
// Filename: licensegen/src/config/resolver.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-13
// Modified: 2026-03-13
// Description: 
// ============================================================================


use std::path::{Path, PathBuf};

use crate::util::helpers::resolve_path;
use super::config::{Config, ConfigError};

/// Resolve all paths in the config into deterministic absolute strings.
pub fn resolve_paths(cfg: &mut Config, cfg_dir: &Path) -> Result<(), ConfigError> {
    //
    // 1. Resolve root_dir (env override or config)
    //
    let root_dir = if let Ok(env_root) = std::env::var("LICENSEGEN_ROOT_DIR") {
        PathBuf::from(shellexpand::tilde(&env_root).to_string())
    } else {
        resolve_path(cfg_dir, &cfg.paths.root_dir)
    };

    //
    // 2. Resolve keypair_dir under root_dir
    //
    let keypair_dir = resolve_path(&root_dir, &cfg.paths.keypair_dir);

    //
    // 3. Resolve products_dir under root_dir
    //
    let products_dir = resolve_path(&root_dir, &cfg.paths.products_dir);

    //
    // . Resolve vault paths (separate from cfg.paths)
    //
    let vault_file = resolve_path(&root_dir, &cfg.vault.file);
    let vault_password_file = resolve_path(&root_dir, &cfg.vault.password_file);

    //
    // 8. Write resolved paths back into cfg as Strings
    //
    cfg.paths.root_dir            = root_dir.to_string_lossy().into_owned();
    cfg.paths.keypair_dir         = keypair_dir.to_string_lossy().into_owned();
    cfg.paths.products_dir        = products_dir.to_string_lossy().into_owned();

    cfg.vault.file          = vault_file.to_string_lossy().into_owned();
    cfg.vault.password_file = vault_password_file.to_string_lossy().into_owned();

    Ok(())
}

fn product_dir(cfg: &Config, product: &str) -> PathBuf {
    PathBuf::from(&cfg.paths.products_dir).join(product)
}

fn edition_dir(cfg: &Config, product: &str, edition: &str) -> PathBuf {
    product_dir(cfg, product).join(edition)
}

fn user_dir(cfg: &Config, product: &str, edition: &str, user: &str) -> PathBuf {
    edition_dir(cfg, product, edition).join(user)
}
