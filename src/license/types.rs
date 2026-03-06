// ============================================================================
// Filename: licensegen/src/license/types.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-06
// Description: 
// ============================================================================

// System Libraries
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde_json;
// Project Libraries
use crate::db::types::{
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

#[derive(Serialize, Deserialize)]
pub struct AddressInfo {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal: String,
    pub country: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub received: NaiveDate,
    pub acquired: NaiveDate,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerInfo {
    pub name: String,
    pub email: String,
    pub address: AddressInfo,
}

#[derive(Serialize, Deserialize)]
pub struct EditionInfo {
    pub code: String,
    pub name: String,
    pub features: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct ProductInfo {
    pub name: String,
    pub code: String,
    pub version: String,
    pub payload_schema: serde_json::Value,
    pub features: serde_json::Value,
    pub editions: serde_json::Value,
}
#[derive(Serialize, Deserialize)]
pub enum ValidityUnit{
    Days,
    Months,
    Years,
}

#[derive(Serialize, Deserialize)]
pub struct ValidityInfo {
    pub issued: NaiveDate,
    pub expires: Option<NaiveDate>,
    pub valid_major: Option<u8>,
    pub validity_value: Option<u16>,
    pub validity_unit: Option<ValidityUnit>,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Debug, Serialize)]
pub struct SignedLicense {
    pub payload_json: String,
    pub signature: String, // now Base64
}
#[derive(Debug)]
pub enum ValidationOutcome {
    Valid,                          // Fully valid license
    SignatureInvalid,               // Cryptographic failure
    PayloadMalformed(String),       // JSON/schema mismatch
    Expired(NaiveDate),             // License expired on this date
    DemoExpired(NaiveDate),         // DEMO-specific expiration
    DemoMissingExpiration,          // DEMO must have expires
    MajorVersionMismatch {
        product_major: u8,
        license_major: u8,
    },
    DemoMajorMismatch {
        product_major: u8,
        license_major: u8,
    },
    EditionNotAllowed(String),      // Unknown or unsupported edition
}

impl ValidationOutcome {
    pub fn into_anyhow(self) -> anyhow::Result<()> {
        match self {
            ValidationOutcome::Valid => Ok(()),

            ValidationOutcome::SignatureInvalid => {
                anyhow::bail!("Signature verification failed")
            }

            ValidationOutcome::PayloadMalformed(msg) => {
                anyhow::bail!("Malformed payload: {}", msg)
            }

            ValidationOutcome::Expired(date) => {
                anyhow::bail!("License expired on {}", date)
            }

            ValidationOutcome::DemoExpired(date) => {
                anyhow::bail!("DEMO license expired on {}", date)
            }

            ValidationOutcome::DemoMissingExpiration => {
                anyhow::bail!("DEMO license missing expiration date")
            }

            ValidationOutcome::MajorVersionMismatch { product_major, license_major } => {
                anyhow::bail!(
                    "Major version mismatch: product={}, license={}",
                    product_major,
                    license_major
                )
            }

            ValidationOutcome::DemoMajorMismatch { product_major, license_major } => {
                anyhow::bail!(
                    "DEMO major version mismatch: product={}, license={}",
                    product_major,
                    license_major
                )
            }

            ValidationOutcome::EditionNotAllowed(code) => {
                anyhow::bail!("Edition not allowed: {}", code)
            }
        }
    }
}
impl ValidationOutcome {
    pub fn is_ok(&self) -> bool {
        matches!(self, ValidationOutcome::Valid)
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
