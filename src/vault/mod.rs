// ============================================================================
// Filename: licensegen/src/vault/mod.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-02-25
// Description:
// ============================================================================

// System Libraries
use serde::Deserialize;
use std::fmt;
// Submodules
pub mod ansible;

// Re-export reader functions/types so main.rs can use them directly
pub use ansible::*;

#[derive(Debug, Deserialize)]
pub struct VaultSecrets {
    pub host: String,
    pub user: String,
    pub pass: String,
    pub port: u16, // 3306
    pub database: String,
    pub rdbms: String,
}
#[derive(Debug)]
pub enum VaultError {
    ReadError(String),
    HeaderError,
    HexDecodeError(&'static str),
    KeyDeriveError,
    CipherError(&'static str),
    PaddingError(&'static str),
    Utf8Error,
    YamlError(String),
    HmacError,
}
impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for VaultError {}
