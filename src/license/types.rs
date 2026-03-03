// ============================================================================
// Filename: licensegen/src/license/types.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-03
// Description: 
// ============================================================================

// System Libraries
use chrono::NaiveDate;
use serde::Serialize;
use serde_json;
// Project Libraries
use crate::db::{
    DbAddress,
    DbApplication,
    DbCustomer,
    DbEdition,
    DbLicense,
    DbProduct,
    DbZipcode,
};

// ---------------------------------------------------------------------------
// Pipeline decision
// ---------------------------------------------------------------------------

pub enum LicenseDecision {
    ReuseExisting(DbLicense),
    IssueNew,
}

// ---------------------------------------------------------------------------
// Pipeline bundle (not a DB table)
// ---------------------------------------------------------------------------

pub struct LicenseBundle {
    pub application: DbApplication,
    pub product: DbProduct,
    pub edition: DbEdition,
    pub customer: DbCustomer,
    pub address: DbAddress,
    pub license: Option<DbLicense>,
    pub zipcode: DbZipcode,
    pub validity: ValidityInfo,
}

// ---------------------------------------------------------------------------
// Canonical payload structs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AddressInfo {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal: String,
    pub country: String,
}

#[derive(Serialize)]
pub struct ApplicationInfo {
    pub received: NaiveDate,
    pub acquired: NaiveDate,
}

#[derive(Serialize)]
pub struct CustomerInfo {
    pub name: String,
    pub email: String,
    pub address: AddressInfo,
}

#[derive(Serialize)]
pub struct EditionInfo {
    pub code: String,
    pub name: String,
    pub features: serde_json::Value,
}

#[derive(Serialize)]
pub struct ProductInfo {
    pub name: String,
    pub code: String,
    pub version: String,
    pub payload_schema: serde_json::Value,
    pub features: serde_json::Value,
    pub editions: serde_json::Value,
}

#[derive(Serialize)]
pub struct ValidityInfo {
    pub issued: NaiveDate,
    pub expires: Option<NaiveDate>,
    pub valid_major: Option<i32>,
}

#[derive(Serialize)]
pub struct LicensePayload {
    pub product: ProductInfo,
    pub edition: EditionInfo,
    pub customer: CustomerInfo,
    pub application: ApplicationInfo,
    pub validity: ValidityInfo,
}

// ---------------------------------------------------------------------------
// Signed artifact
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct SignedLicense {
    pub payload_json: String,
    pub signature: String, // now Base64
}
