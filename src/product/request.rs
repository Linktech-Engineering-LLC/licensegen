// ============================================================================
// Filename: licensegen/src/product/request.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-28
// Modified: 2026-03-10
// Description:
// ============================================================================

use chrono::NaiveDate;
//use mysql_common::binlog::decimal::Decimal;
use rust_decimal::Decimal;
// System Libraries
use serde::{Deserialize, Serialize};
// Project Libraries

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationRequest {
    pub request: RequestSection,
    pub contact: ContactSection,
    pub vendor: VendorSection,
    #[serde(skip)]
    pub raw_yaml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSection {
    pub product: String,
    pub edition: String,
    pub sku: String,
    #[serde(rename = "app")]
    pub name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub first: String,
    pub last: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub maildrop: Option<String>,
    pub street: Option<String>,
    pub suite: Option<String>, // fixed spelling
    pub city: Option<String>,
    pub state: Option<String>,
    pub county: Option<String>,
    pub country: Option<String>,
    pub zip: u32,          // correct MySQL INT UNSIGNED mapping
    pub zip4: Option<u32>, // correct MySQL INT UNSIGNED mapping
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
