// ============================================================================
// Filename: licensegen/src/product/editions.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-26
// Modified: 2026-03-11
// Description:
// ============================================================================

//use mysql_common::binlog::decimal::Decimal;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

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
