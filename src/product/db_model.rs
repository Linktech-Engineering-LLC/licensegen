// ============================================================================
// Filename: licensegen/src/product/db_model.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-03-01
// Description:
// ============================================================================

// System Libraries
use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;
use serde_json;
use std::fmt;

// Project Libraries
use crate::product::Product;

#[derive(Debug)]
pub struct DbProduct {
    pub name: String,
    pub code: String,
    pub version: String,
    pub payload_schema: String,
    pub features: String,
    pub editions: String,
    pub keypair_path: String,
    pub active: bool,
}
impl From<&Product> for DbProduct {
    fn from(p: &Product) -> Self {
        Self {
            name: p.name.clone(),
            code: p.code.clone(),
            version: p.version.clone(),

            payload_schema: serde_json::to_string(&p.license.payload_fields)
                .expect("payload_schema JSON"),

            editions: serde_json::to_string(&p.editions).expect("editions JSON"),

            features: "{}".into(), // or whatever you decide later

            keypair_path: p.signing.keypair.clone(),
            active: true,
        }
    }
}
impl DbProduct {
    pub fn as_params(&self) -> (&str, &str, &str, &str, &str, &str, &str, &bool) {
        (
            &self.name,
            &self.code,
            &self.version,
            &self.payload_schema,
            &self.features,
            &self.editions,
            &self.keypair_path,
            &self.active,
        )
    }
}
impl fmt::Display for DbApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DbApplication {{ id: {}, name: {}, customer_id: {}, edition_id: {} }}",
            self.id, self.name, self.customer_id, self.edition_id
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DbApplication {
    pub id: u64,          // PK assigned by DB
    pub customer_id: u64, // FK -> customers.id
    pub edition_id: u64,  // FK -> editions.id

    pub name: String,     // application name
    pub raw_yaml: String, // exact YAML used to create/update this record

    pub received: NaiveDate, // vendor.received_on (DATE)
    pub acquired: NaiveDate, // vendor.acquired (DATE)

    pub status: String, // 'pending', 'approved', 'rejected'
}
