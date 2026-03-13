// ============================================================================
// Filename: licensegen/src/product/types.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-07
// Modified: 2026-03-13
// Description: Structures for Products, Editions, Requests, etc
// ============================================================================

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::NaiveDate;
use std::path::PathBuf;
use std::fmt;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub maildrop: Option<String>,
    pub street: Option<String>,
    pub suite: Option<String>, 
    pub city: String,
    pub state: String,
    pub county: String,
    pub country: String,
    pub zip: String,          
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Application {
    pub id: Option<u64>,
    pub customer_id: u64, // resolved from customer sync
    pub product_id: u64,  // resolved from product name
    pub edition_id: u64,  // resolved from edition code
    pub app_name: String, // "Monitor Dad's Firewall"
    pub received: NaiveDate,
    pub valid_major: u8,
    pub acquired: NaiveDate,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationRequest {
    pub request: RequestSection,
    pub contact: ContactSection,
    pub vendor: VendorSection,
    #[serde(skip)]
    pub raw_yaml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactSection {
    pub company: Option<String>,
    pub name: Name,
    pub email: String,
    pub phone: String,
    pub address: Address,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Edition {
    pub id: u64,
    pub application_id: u64,
    pub name: String,
    pub code: String,
    pub price: Option<Decimal>,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct EditionInfo {
    pub sku: String,
    pub code: String, // COM, PRO, ENT, DEV
    pub name: String, // "Community Edition"
    pub price: Option<Decimal>,
    pub valid: bool,  // true/false
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditionRule {
    pub valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct EditionRoot {
    pub edition: EditionInfo,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub constraints: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub defaults: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
pub struct LicenseSection {
    pub payload_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub first: String,
    pub last: String,
}

#[derive(Debug, Deserialize)]
pub struct Product {
    pub name: String,
    pub code: String,
    pub version: Option<String>,

    pub license: LicenseSection,
    pub signing: SigningSection,
    pub editions: Vec<String>,

    #[serde(skip)]
    pub dir: PathBuf, // <-- add this
}

#[derive(Debug, Deserialize)]
pub struct ProductYaml {
    pub product: String,
    pub editions: HashMap<String, EditionRule>,
    pub license: LicenseSection,
    pub signing: SigningSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSection {
    pub product: String,
    pub edition: String,
    pub sku: String,
    #[serde(rename = "app")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SigningSection {
    pub keypair: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSection {
    pub received_on: NaiveDate,
    pub acquired: NaiveDate,
    pub price: Option<Decimal>,
    pub valid_major: Option<u8>,
    pub validity_value: Option<u8>,
    pub validity_unit: Option<String>,
    pub notes: Option<String>,
}

impl fmt::Display for ProductError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ProductError {}

#[derive(Debug)]
pub enum ProductError {
    ReadError(String),
    YamlError(String),
}
impl From<mysql_async::Error> for ProductError {
    fn from(e: mysql_async::Error) -> Self {
        ProductError::ReadError(format!("Database error: {}", e))
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(msg) => write!(f, "IO error: {}", msg),
            AppError::Yaml(msg) => write!(f, "YAML error: {}", msg),
            AppError::Invalid(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}
impl std::error::Error for AppError {}
#[derive(Debug)]
pub enum AppError {
    Io(String),
    Yaml(String),
    Invalid(String),
}
