// ============================================================================
// Filename: licensegen/src/product/product.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-23
// Modified: 2026-03-10
// Description:
// ============================================================================

// System Libraries
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
// Project Libraries

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

#[derive(Debug, Deserialize, Serialize)]
pub struct EditionRule {
    pub valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct LicenseSection {
    pub payload_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SigningSection {
    pub keypair: String,
}
