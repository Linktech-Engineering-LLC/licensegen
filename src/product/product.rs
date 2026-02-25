// ============================================================================
// Filename: licensegen/src/product/product.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-23
// Modified: 2026-02-25
// Description:
// ============================================================================

// System Libraries
use serde::{Deserialize, Serialize};
// Project Libraries

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Product {
    pub name: String,
    pub code: String,
    pub version: String,

    pub editions: HashMap<String, EditionRule>,
    pub license: LicenseSection,
    pub signing: SigningSection,
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
