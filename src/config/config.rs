// ============================================================================
// Filename: licensegen/src/config.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-03-13
// Description: Configuration loader for licensegen.yml.
// ============================================================================

// System Libraries
use serde::Deserialize;
use std::fs;
use std::path::Path;

// Project Libraries
use thiserror::Error;

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),

    #[error("Failed to parse config file: {0}")]
    ParseError(String),
}

// ============================================================================
// Config Structures
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct Config {
    pub vault: VaultConfig,
    pub paths: PathsConfig,
    pub db: Option<DbConfig>,
}

#[derive(Debug, Deserialize)]
pub struct VaultConfig {
    pub file: String,
    pub password_file: String,
}

#[derive(Debug, Deserialize)]
pub struct PathsConfig {
    pub root_dir: String,
    pub keypair_dir: String,
    pub products_dir: String,

    pub product_file: String,
    pub editions_subdir: String,

    pub edition_file: String,
    pub applications_subdir: String,

    pub application_file: String,
    pub license_file: String,

    pub output_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub rdbms: String,    // "mysql"
    pub host: String,     // "localhost"
    pub port: u16,        // 3306
    pub user: String,     // "root"
    pub password: String, // "secret"
    pub database: String, // "licensegen"
}

// ============================================================================
// Loader
// ============================================================================

impl Config {
    /// Load and parse licensegen.yml
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(&path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        serde_yaml::from_str::<Config>(&raw).map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}
