// ============================================================================
// Filename: licensegen/src/db/types.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-11
// Description: Database Structures
// ============================================================================

use chrono::{NaiveDate, NaiveDateTime};
//use mysql_common::binlog::decimal::Decimal;
use mysql_async::Row;
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json;
use std::f32::consts::E;
use std::fmt;

use crate::product::types::Product;
use crate::util::datetime::{to_naive_date, to_naive_datetime};

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
    pub validity_value: u16,
    pub validity_unit: Option<String>,
    pub raw_yaml: String,

    pub received: NaiveDate,
    pub acquired: NaiveDate,

    pub status: Option<String>,

    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,

}
impl DbApplication {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id").expect("id missing"),
            name: row.get("name").expect("name missing"),
            customer_id: row.get("customer_id").expect("customer_id missing"),
            edition_id: row.get("edition_id").expect("edition_id missing"),
            price: row.get("price").expect("price missing"),
            valid_major: row.get("valid_major").expect(("valid_major missing")),
            validity_value: row.get("validity_value").expect("validity_value missing"),
            validity_unit: row.get("validity_unit").expect("validity_unit missing"),
            raw_yaml: row.get("raw_yaml").expect("raw_yaml missing"),
            received: to_naive_date(row.get("received").expect("received missing")),
            acquired: to_naive_date(row.get("acquired").expect("acquired missing")),
            status: row.get("status").expect("status missing"),
            created: to_naive_datetime(row.get("created").expect("created missing")),
            updated: to_naive_datetime(row.get("updated").expect("updated missing")),
        }
    }
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
    pub payload: Option<String>,           // LONGTEXT JSON
    pub features: Option<String>,          // LONGTEXT JSON
    pub signature: Option<String>,         // TEXT (base64 or hex)

    pub issued: NaiveDate,
    pub expires: Option<NaiveDate>,
    pub valid_major: Option<u8>,
    pub revoked: bool,

    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}
impl DbLicense{
    pub fn from_row(row: &Row) -> Self{
        Self { id: row.get("id").unwrap(), 
            application_id: row.get("application_id").unwrap(), 
            edition_id: row.get("edition_id").unwrap(), 
            paid: row.get("paid"), 
            version: row.get("version"), 
            payload: row.get("payload"), 
            features: row.get("features"), 
            signature: row.get("signature"), 
            issued: to_naive_date(row.get("issued").unwrap()), 
            expires: row.get("expires").map(to_naive_date),
            valid_major: row.get("valid_major"), 
            revoked: row.get("revoked").unwrap(), 
            created: to_naive_datetime(row.get("created").unwrap()), 
            updated: to_naive_datetime(row.get("updated").unwrap()), 
        }
    }
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

impl DbProduct {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap(),
            name: row.get("name").unwrap(),
            code: row.get("code").unwrap(),
            version: row.get("version"),
            editions: row.get("editions"),
            payload_schema: row.get("payload_schema").unwrap(),
            features: row.get("features").unwrap(),
            keypair_path: row.get("keypair_path").unwrap(),
            active: row.get("active").unwrap(),
            created: to_naive_datetime(row.get("created").unwrap()),
            updated: to_naive_datetime(row.get("updated").unwrap()),
        }
    }
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
impl DbEdition {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id").expect("id missing"),
            name: row.get("name").expect("name missing"),
            product_id: row.get("product_id").expect("product_id missing"),
            sku: row.get("sku").expect("sku missing"),
            edition_code: row.get("edition_code").expect("edition_code missing"),
            price: row.get("price"),
            metadata: row.get("metadata").expect("metadata missing"),
            valid: row.get("valid").unwrap(),
            created: to_naive_datetime(row.get("created").expect("created missing")),
            updated: to_naive_datetime(row.get("updated").expect("updated missing")),
        }
    }
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
impl DbCustomer {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap(),
            company: row.get("company"),
            first: row.get("first").unwrap(),
            last: row.get("last").unwrap(),
            email: row.get("email").unwrap(),
            phone: row.get("phone").unwrap(),
            address_id: row.get("address_id").unwrap(),
            notes: row.get("notes"),
            created: to_naive_datetime(row.get("created").unwrap()),
            updated: to_naive_datetime(row.get("updated").unwrap()),
        }
    }
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

    pub zip: String,                 // INT(5) UNSIGNED ZEROFILL

    pub city: Option<String>,
    pub state: Option<String>,
    pub county: Option<String>,
    pub country: Option<String>,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}
impl  DbAddress {
    pub fn from_row(row: &Row) -> Self {
        Self { 
            id: row.get("id").unwrap(), 
            maildrop: row.get("maildrop").expect("maildrop missing"), 
            street: row.get("street").expect("street missing"), 
            suite: row.get("suite").expect(("suite missing")), 
            zip: row.get("zip").expect("zip missing"), 
            city: row.get("city").expect("city missing"), 
            state: row.get("state").expect("state missing"), 
            county: row.get("county").expect("county missing"), 
            country: row.get("country").expect("country missing"), 
            created: to_naive_datetime(row.get("created").unwrap()), 
            updated: to_naive_datetime(row.get("updated").unwrap()), 
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbZipcode {
    pub zip: u32,                 // INT(5) UNSIGNED ZEROFILL
    pub city: String,
    pub state: String,
    pub county: Option<String>,
}
impl DbZipcode{
    pub fn from_row(row: &Row) -> Self{
        Self { 
            zip: row.get("zip").unwrap(), 
            city: row.get("city").unwrap(), 
            state: row.get("state").unwrap(), 
            county: row.get("county"), 
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbAddressView {
    pub id: u64,
    pub maildrop: String,
    pub street: String,
    pub suite: String,
    pub zip: String,
    pub city: String,      // now non-null
    pub state: String,     // now non-null
    pub county: String,    // now non-null
    pub country: String,   // now non-null
}
impl  DbAddressView {
    pub fn from_row(row: &Row) -> Self {
        Self { 
            id: row.get("id").unwrap(), 
            maildrop: row.get("maildrop").unwrap(), 
            street: row.get("street").unwrap(), 
            suite: row.get("suite").unwrap(), 
            zip: row.get("zip").unwrap(), 
            city: row.get("city").unwrap(), 
            state: row.get("state").unwrap(), 
            county: row.get("county").unwrap(), 
            country: row.get("country").unwrap(), 
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbCustomerView {
    pub id: u64,
    pub company: String,
    pub first: String,
    pub last: String,
    pub email: String,
    pub phone: String,
    pub notes: String,
    pub address_id: u64,
}
impl DbCustomerView {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap(),
            company: row.get("company").unwrap(),
            first: row.get("first").unwrap(),
            last: row.get("last").unwrap(),
            email: row.get("email").unwrap(),
            phone: row.get("phone").unwrap(),
            address_id: row.get("address_id").unwrap(),
            notes: row.get("notes").unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbEditionView {
    pub id: u64,

    // Product-level fields
    pub product_name: String,
    pub product_id: u64,
    pub version: String,
    pub editions: String,
    pub payload_schema: String,
    pub features: String,
    pub keypair_path: String,
    pub active: bool,

    // Edition-level fields
    pub edition_name: String,
    pub sku: String,
    pub edition_code: String,
    pub metadata: String,
    pub price: Decimal,
    pub valid: bool,
}
impl DbEditionView {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap(),
            product_name: row.get("product_name").unwrap(),
            product_id: row.get("product_id").unwrap(),
            version: row.get("version").unwrap(),
            editions: row.get("editions").unwrap(),
            payload_schema: row.get("payload_schema").unwrap(),
            features: row.get("features").unwrap(),
            keypair_path: row.get("keypair_path").unwrap(),
            active: row.get("active").unwrap(),
            edition_name: row.get("edition_name").unwrap(),
            sku: row.get("sku").unwrap(),
            edition_code: row.get("edition_code").unwrap(),
            metadata: row.get("metadata").expect("metadata missing"),
            price: row.get("price").unwrap(),
            valid: row.get("valid").unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbApplicationView {
    pub id: u64,
    pub application_name: String,
    pub edition_id: u64,

    // Customer fields
    pub customer_id: u64,
    pub company: String,
    pub first: String,
    pub last: String,

    // Edition fields
    pub product_name: String,
    pub edition_name: String,
    pub sku: String,
    pub edition_valid: bool,

    // Commercial terms
    pub application_price: Decimal,
    pub major: u8,
    pub validity_value: u16,
    pub validity_unit: String,

    // Metadata
    pub raw_yaml: String,
    pub received: NaiveDate,
    pub acquired: NaiveDate,
    pub status: String,
}
impl DbApplicationView {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap(),
            application_name: row.get("application_name").unwrap(),
            edition_id: row.get("edition_id").unwrap(),
            customer_id: row.get("customer_id").unwrap(),
            company: row.get("company").unwrap(),
            first: row.get("first").unwrap(),
            last: row.get("last").unwrap(),
            product_name: row.get("product_name").unwrap(),
            edition_name: row.get("edition_name").unwrap(),
            sku: row.get("sku").unwrap(),
            edition_valid: row.get("edition_valid").unwrap(),
            application_price: row.get("application_price").unwrap(),
            major: row.get("major").unwrap(),
            validity_value: row.get("validity_value").unwrap(),
            validity_unit: row.get("validity_unit").unwrap(),
            raw_yaml: row.get("raw_yaml").unwrap(),
            received: to_naive_date(row.get("received").unwrap()),
            acquired: to_naive_date(row.get("acquired").unwrap()),
            status: row.get("status").unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbLicenseView {
    pub id: u64,
    pub version: String,
    pub payload: String,
    pub features: String,
    pub signature: String,
    pub issued: NaiveDate,
    pub expires: NaiveDate,
    pub revoked: bool,

    // Application fields
    pub application_name: String,
    pub application_price: Decimal,
    pub major: u8,
    pub validity_value: u8,
    pub validity_unit: String,

    // Customer fields
    pub company: String,
    pub first: String,
    pub last: String,

    // Edition fields
    pub product_name: String,
    pub edition_name: String,
    pub sku: String,
}
