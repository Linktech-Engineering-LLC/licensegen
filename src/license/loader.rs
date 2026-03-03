// ============================================================================
// Filename: licensegen/src/license/loader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-03
// Description: 
// ============================================================================

// System Libraries
use mysql_async::{Pool, Row, prelude::*};
use anyhow::Result;
// Project Libraries
use crate::product::db_model::{
    DbLicense, DbProduct, DbEdition, DbCustomer, DbAddress, DbApplication,
};
use crate::license::types::LicenseBundle;
use crate::util::{to_naive_date, to_naive_datetime};

pub async fn load_license_bundle(pool: &Pool, application_id: i64) -> Result<LicenseBundle> {
    let mut conn = pool.get_conn().await?;

    // 1. Load the application row
    let row: Row = conn.exec_first(
        "SELECT * FROM applications WHERE id = ?",
        (application_id,)
    ).await?
    .ok_or_else(|| anyhow::anyhow!("Application not found"))?;

    let application = DbApplication {
        id: row.get("id").unwrap(),
        edition_id: row.get("edition_id").unwrap(),
        customer_id: row.get("customer_id").unwrap(),
        name: row.get("name").unwrap(),
        raw_yaml: row.get("raw_yaml").unwrap(),
        received: to_naive_date(row.get("received").unwrap()),
        acquired: to_naive_date(row.get("acquired").unwrap()),
        status: row.get("status").unwrap(),
        created: to_naive_datetime(row.get("created").unwrap()),
        updated: to_naive_datetime(row.get("updated").unwrap()),
    };

    // 2. Load edition (this contains product_id)
    let row: Row = conn.exec_first(
        "SELECT * FROM editions WHERE id = ?",
        (application.edition_id,)
    ).await?
    .ok_or_else(|| anyhow::anyhow!("Edition not found"))?;

    let edition = DbEdition {
        id: row.get("id").unwrap(),
        name: row.get("name").unwrap(),
        product_id: row.get("product_id").unwrap(),
        sku: row.get("sku").unwrap(),
        edition_code: row.get("edition_code").unwrap(),
        metadata: row.get("metadata").unwrap(),
        valid: row.get("valid").unwrap(),
        created: to_naive_datetime(row.get("created").unwrap()),
        updated: to_naive_datetime(row.get("updated").unwrap()),
    };

    // 3. Load product using edition.product_id  ← FIXED
    let row: Row = conn.exec_first(
        "SELECT * FROM products WHERE id = ?",
        (edition.product_id,)
    ).await?
    .ok_or_else(|| anyhow::anyhow!("Product not found"))?;

    let product = DbProduct {
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
    };
    // 4. Load customer
    let row: Row = conn.exec_first(
        "SELECT * WHERE id = ?",
        (application.customer_id,)
    ).await?
     .ok_or_else(|| anyhow::anyhow!("Customer not found"))?;

     let customer =DbCustomer { 
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
    };

    // 5. Load address
    let row: Row = conn.exec_first(
        "SELECT * FROM addresses WHERE id = ?",
        (customer.address_id,)
    ).await?
     .ok_or_else(|| anyhow::anyhow!("Address not found"))?;

    let address = DbAddress { 
        id: row.get("id").unwrap(), 
        maildrop: row.get("maildrop"), 
        street: row.get("street"), 
        suite: row.get("suite"), 
        zip: row.get("zip").unwrap(), 
        zip4: row.get("zip4"), 
        city: row.get("city"), 
        state: row.get("state"), 
        county: row.get("county"), 
        country: row.get("country").unwrap(), 
        created: to_naive_datetime(row.get("created").unwrap()), 
        updated: to_naive_datetime(row.get("updated").unwrap()), 
    };

    // 6. Try to load an existing license for this application
    let row: Row = conn.exec_first(
        "SELECT * FROM licenses WHERE application_id = ?",
        (application_id,)
    ).await?
     .ok_or_else(|| anyhow::anyhow!("License not found"))?;

    let license = DbLicense {
        id: row.get("id").unwrap(),
        application_id: row.get("application_id").unwrap(),
        edition_id: row.get("edition_id").unwrap(),
        version: row.get("version"),
        payload: row.get("payload").unwrap(),
        features: row.get("features").unwrap(),
        signature: row.get("signature").unwrap(),
        issued: to_naive_date(row.get("issued").unwrap()),
        expires: row.get("expires").map(|v| to_naive_date(v)),
        valid_major: row.get("valid_major"),
        revoked: row.get("revoked").unwrap(),
        created: to_naive_datetime(row.get("created").unwrap()),
        updated: to_naive_datetime(row.get("updated").unwrap()),
    };

    Ok(LicenseBundle {
        application,
        product,
        edition,
        customer,
        address,
        license: Some(license),
    })
}
