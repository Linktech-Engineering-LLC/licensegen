// ============================================================================
// Filename: licensegen/src/db/reader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-05
// Description: 
// ============================================================================
use mysql_async::{Row, Conn, params, prelude::*};
use anyhow::Result;

use crate::db::types::{
    DbAddress,
    DbApplication,
    DbCustomer,
    DbEdition,
    DbLicense,
    DbProduct,
    DbZipcode,
};
use crate::license::types::{LicenseBundle, ValidityInfo};
use crate::util::datetime::{to_naive_date, to_naive_datetime, opt};

pub async fn fetch_application(
    conn: &mut Conn,
    app_id: u64,
) -> mysql_async::Result<DbApplication>
{
    let row: Row = conn
        .exec_first(
            "SELECT * FROM applications WHERE id = :id",
            params!{"id" => app_id},
        )
        .await?
        .expect("application not found");

    let received = to_naive_date(row.get("received").unwrap());
    let acquired = to_naive_date(row.get("acquired").unwrap());

    let created = to_naive_datetime(row.get("created").unwrap());
    let updated = to_naive_datetime(row.get("updated").unwrap());

    let app = DbApplication {
        id: row.get("id").unwrap(),
        name: row.get("name").unwrap_or_default(),
        customer_id: row.get("customer_id").unwrap(),
        edition_id: row.get("edition_id").unwrap(),
        price: row.get("price").unwrap_or_default(),

        // Nullable fields
        valid_major: row.get("valid_major"),
        validity_unit: row.get("validity_unit").unwrap(),

        // NOT NULL in schema, but older rows may still contain NULL
        validity_value: row.get("validity_value").unwrap_or(0),
        raw_yaml: row.get("raw_yaml").unwrap_or_default(),
        status: row.get("status").unwrap_or_else(|| "pending".to_string()),

        received,
        acquired,
        created,
        updated,
    };

    Ok(app)
}

pub async fn get_product_id_by_code(
    conn: &mut Conn,
    code: &str,
) -> Result<Option<u64>, mysql_async::Error> {
    conn.exec_first(
        "SELECT id FROM products WHERE code = :code",
        params! { "code" => code },
    ).await
}

pub async fn resolve_edition_id_by_sku(
    conn: &mut Conn,
    sku: &str,
) -> Result<u64, mysql_async::Error> {
    let id: Option<u64> = conn
        .exec_first(
            r#"
            SELECT id
            FROM editions
            WHERE sku = :sku
            "#,
            params! { "sku" => sku },
        )
        .await?;

    match id {
        Some(edition_id) => Ok(edition_id),
        None => Err(mysql_async::Error::Other(
            format!("Edition with SKU '{}' not found", sku).into(),
        )),
    }
}

pub async fn load_license_bundle(conn: &mut Conn, application_id: u64) -> Result<LicenseBundle> {

    // 1. Load the application row
    let application = fetch_application(conn, application_id).await?;

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
        price: row.get("price"),
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

    let row: Row = conn.exec_first(
        "SELECT * FROM zipcodes WHERE zip = ?",
        (address.zip,)
    ).await?
     .ok_or_else(|| anyhow::anyhow!("Zipcode not found"))?;

    let zipcode = DbZipcode { 
        zip: row.get("zip").unwrap(), 
        city: row.get("city").unwrap(), 
        state: row.get("state").unwrap(), 
        county: row.get("county"), 
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
        paid: row.get("paid"),
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

    let validity = ValidityInfo {
        issued: license.issued,
        expires: license.expires,  
        valid_major: license.valid_major,
        validity_unit: None,
        validity_value: None,
    };

    Ok(LicenseBundle {
        application,
        product,
        edition,
        customer,
        address,
        license: Some(license),
        zipcode,
        validity,
    })
}
