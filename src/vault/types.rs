// ============================================================================
// Filename: licensegen/src/vault/types.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-04
// Modified: 2026-03-12
// Description: 
// ============================================================================
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct VaultSecrets {
    pub host: String,
    pub user: String,
    pub pass: String,
    pub port: u16, // 3306
    pub database: String,
    pub rdbms: String,
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for VaultError {}
#[derive(Debug)]
pub enum VaultError {
    Io(std::io::Error),
    ReadError(String),
    HeaderError,
    HexDecodeError(&'static str),
    KeyDeriveError,
    CipherError(&'static str),
    PaddingError(&'static str),
    Utf8Error,
    YamlError(String),
    HmacError,
    PermissionError(String),
}
impl From<std::io::Error> for VaultError {
    fn from(err: std::io::Error) -> Self {
        VaultError::Io(err)
    }
}


