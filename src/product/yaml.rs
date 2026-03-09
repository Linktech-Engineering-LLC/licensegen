// ============================================================================
// Filename: licensegen/src/product/yaml.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-03-07
// Description:
// ============================================================================

// System Libraries
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Project Libraries

#[derive(Debug, Deserialize)]
pub struct ProductYaml {
    pub product: String,
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
