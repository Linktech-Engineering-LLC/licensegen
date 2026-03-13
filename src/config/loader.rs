// ============================================================================
// Filename: licensegen/src/config/loader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-13
// Modified: 2026-03-13
// Description: 
// ============================================================================

use log::info;
use std::path::PathBuf;

use super::config::{Config, ConfigError};
use super::resolver::resolve_paths;

pub fn load_and_resolve() -> Result<Config, ConfigError> {
    // 1. Determine cfg_path
    let cfg_path = PathBuf::from(format!("{}/licensegen.yml", env!("CARGO_MANIFEST_DIR")));
    let cfg_dir = cfg_path.parent().unwrap().to_path_buf();

    // 2. Load raw config
    let mut cfg = Config::load(&cfg_path)?;
    info!("Loaded configuration from {:?}", cfg_path);

    // 3. Resolve paths (root_dir, products_dir, applications_subdir, etc.)
    resolve_paths(&mut cfg, &cfg_dir)?;

    Ok(cfg)
}
