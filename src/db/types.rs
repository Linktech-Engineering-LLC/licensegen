// ============================================================================
// Filename: licensegen/src/db/types.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-05
// Description: Database Structures
// ============================================================================

use chrono::{NaiveDate, NaiveDateTime};
//use mysql_common::binlog::decimal::Decimal;
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json;
use std::fmt;

use crate::product::product::Product;

// ---------------------------------------------------------------------------
// Display for DbApplication
// ---------------------------------------------------------------------------

impl fmt::Display for DbApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DbApplication {{ id: {}, name: {}, customer_id: {}, edition_id: {} }}",
            self.id, self.name, self.customer_id, self.edition_id
        )
    }
}

// ---------------------------------------------------------------------------
// Applications table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct DbApplication {
    pub id: u64,
    pub name: String,
    pub customer_id: u64,
    pub edition_id: u64,
    pub price: Decimal,
    pub valid_major: Option<u8>,
    pub validity_value: u8,
    pub validity_unit: Option<String>,
    pub raw_yaml: String,

    pub received: NaiveDate,
    pub acquired: NaiveDate,

    pub status: String,

    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,

}

// ---------------------------------------------------------------------------
// Licenses table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DbLicense {
    pub id: u64,
    pub application_id: u64,
    pub edition_id: u64,
    pub paid: Option<Decimal>,

    pub version: Option<String>,   // VARCHAR(5)
    pub payload: String,           // LONGTEXT JSON
    pub features: String,          // LONGTEXT JSON
    pub signature: String,         // TEXT (base64 or hex)

    pub issued: NaiveDate,
    pub expires: Option<NaiveDate>,
    pub valid_major: Option<u8>,
    pub revoked: bool,

    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Products table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DbProduct {
    pub id: u64,
    pub name: String,
    pub code: String,
    pub version: Option<String>,

    pub editions: Option<String>,        // JSON string
    pub payload_schema: String,  // JSON string
    pub features: String,        // JSON string

    pub keypair_path: String,
    pub active: bool,

    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

// Convert YAML Product → DbProduct row
impl From<&Product> for DbProduct {
    fn from(p: &Product) -> Self {
        Self {
            id: 0, // auto-increment

            name: p.name.clone(),
            code: p.code.clone(),
            version: p.version.clone(),

            payload_schema: serde_json::to_string(&p.license.payload_fields)
                .expect("payload_schema JSON"),

            editions: Some(serde_json::to_string(&p.editions)
                .expect("editions JSON")),

            features: "{}".into(), // placeholder until feature model is finalized

            keypair_path: p.signing.keypair.clone(),
            active: true,

            // MySQL will overwrite these on insert
        created: NaiveDateTime::from_timestamp(0, 0),
        updated: NaiveDateTime::from_timestamp(0, 0),        }
    }
}

impl DbProduct {
    pub fn as_params(&self) -> (
        &str,
        &str,
        Option<&str>,
        &str,
        &str,
        Option<&str>,
        &str,
        &bool,
    ) {
        (
            &self.name,
            &self.code,
            self.version.as_deref(),     // Option<String> → Option<&str>
            &self.payload_schema,
            &self.features,
            self.editions.as_deref(),    // Option<String> → Option<&str>
            &self.keypair_path,
            &self.active,
        )
    }
}
// ---------------------------------------------------------------------------
// Editions table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DbEdition {
    pub id: u64,
    pub name: String,
    pub product_id: u64,
    pub sku: String,
    pub edition_code: String,
    pub price: Option<Decimal>,
    pub metadata: String,               // JSON string from DB
    pub valid: bool,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Customers table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DbCustomer {
    pub id: u64,
    pub company: Option<String>,
    pub first: String,
    pub last: String,
    pub email: String,
    pub phone: String,
    pub address_id: u64,
    pub notes: Option<String>,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}
// ---------------------------------------------------------------------------
// Addresses table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DbAddress {
    pub id: u64,

    pub maildrop: Option<String>,
    pub street: Option<String>,
    pub suite: Option<String>,

    pub zip: u32,                 // INT(5) UNSIGNED ZEROFILL
    pub zip4: Option<u32>,        // INT(4) UNSIGNED ZEROFILL

    pub city: Option<String>,
    pub state: Option<String>,
    pub county: Option<String>,
    pub country: String,

    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}
#[derive(Debug, Clone)]
pub struct DbZipcode {
    pub zip: u32,                 // INT(5) UNSIGNED ZEROFILL
    pub city: String,
    pub state: String,
    pub county: Option<String>,
}
