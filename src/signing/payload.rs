// ============================================================================
// Filename: licensegen/src/payload.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-03-04
// Description: Defines the LicensePayload struct used for RSA signing.
// ============================================================================

// System Libraries
use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;
// Project Libraries
use crate::product::product::Application;
use crate::product::edition::Edition;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Address {
    pub id: i64,
    pub maildrop: String,
    pub street: String,
    pub suite: Option<String>,
    pub zip: i64,
    pub zip4: u8,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub updated: chrono::NaiveDateTime,
}
pub struct CustomerBundle {
    pub customer: Customer,
    pub address: Address,
    pub zipcode: ZipCode,
}
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedLicense {
    pub application_id: i64,
    pub customer_id: i64,
    pub edition_id: i64,

    pub application_name: String,
    pub received: NaiveDate,
    pub acquired: NaiveDate,
    pub status: String,

    pub issued: NaiveDateTime,
    pub expires: Option<NaiveDateTime>,

    pub payload: serde_json::Value,
    pub signature: String,
    pub public_key_pem: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct License {
    pub id: i64,
    pub application_id: i64,
    pub edition_id: i64,
    pub version: Option<String>,
    pub payload: serde_json::Value,
    pub features: serde_json::Value,
    pub signature: String,
    pub issued: NaiveDateTime,
    pub expires: Option<NaiveDateTime>,
    pub revoked: bool,
}
pub struct LicenseBundle {
    pub license: License,
    pub application: Application,
    pub edition: Edition,
}
#[derive(Debug, Serialize)]
pub struct LicensePayload {
    pub email: String,
    pub acquired: String, // ISO date: "YYYY-MM-DD"
    pub edition: String,  // "DEV", "PRO", "ENT"
    pub valid_major: u32, // e.g. 3 for 3.x versions
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZipCode {
    pub id: i64,
    pub zip: String,
    pub city: String,
    pub state: String,
}
