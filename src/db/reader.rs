// ============================================================================
// Filename: licensegen/src/db/reader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-10
// Description: 
// ============================================================================
use mysql_async::{Row, Conn, params, prelude::*, Transaction};
use anyhow::{anyhow, Result};

use crate::db::types::{
    DbAddress,
    DbAddressView,
    DbApplication,
    DbCustomer,
    DbEdition,
    DbLicense,
    DbProduct,
    DbZipcode,
};
use crate::license::types::{LicenseBundle, ValidityInfo};
use crate::util::datetime::{to_naive_date, to_naive_datetime, opt};

pub async fn fetch_by_id<E>(
    exec: &mut E,
    source: &str,   // table or view name
    id: u64,
) -> mysql_async::Result<Option<Row>>
where
    E: mysql_async::prelude::Queryable,
{
    let query = format!("SELECT * FROM {} WHERE id = ?", source);
    exec.exec_first(query, (id,)).await
}

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

pub async fn fetch_address(
    conn: &mut Conn,
    adr_id: u64,
) -> anyhow::Result<DbAddressView>
{
    let row: Row = conn.exec_first(
        "SELECT * FROM v_address WHERE id = ?",
        (adr_id,)
    ).await?
     .ok_or_else(|| anyhow::anyhow!("Address not found"))?;

    let address = DbAddressView {
        id: row.get("id").unwrap(),
        maildrop: row.get("maildrop").unwrap(),
        street: row.get("street").unwrap(),
        suite: row.get("suite").unwrap(),
        zip: row.get("zip").unwrap(),
        city: row.get("city").unwrap(),
        state: row.get("state").unwrap(),
        county: row.get("county").unwrap(),
        country: row.get("country").unwrap(),
    };

    Ok(address)
}

pub async fn fetch_zip_data(
    conn: &mut Conn,
    zip: &String,
) -> anyhow::Result<DbZipcode>
{
    let row: Row = conn.exec_first(
        r#"
        SELECT *
        FROM zipcodes
        WHERE zip = CAST(LEFT(:zip, 5) AS UNSIGNED)
        "#,
        params! { "zip" => zip },
    ).await?
     .ok_or_else(|| anyhow::anyhow!("Zipcode not found"))?;

    let zipcode = DbZipcode { 
        zip: row.get("zip").unwrap(), 
        city: row.get("city").unwrap(), 
        state: row.get("state").unwrap(), 
        county: row.get("county"), 
    };

    Ok(zipcode)
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

pub async fn load_license_bundle(conn: &mut Conn, application_id: u64)
    -> Result<LicenseBundle>
{
    // 1. Load everything except license
    let row: Row = conn.exec_first(
        "SELECT * FROM applications WHERE id = ?",
        (application_id,)
    ).await?
     .ok_or_else(|| anyhow!("Application not found"))?;
    let application = DbApplication::from_row(&row);
    println!("Fetched application {}", application);
   
    let row: Row = conn.exec_first(
        "SELECT * FROM editions WHERE id = ?",
        (application.edition_id,)
    ).await?
     .ok_or_else(|| anyhow!("Edition not found"))?;
    let edition     = DbEdition::from_row(&row);
    println!("Fetched edition {:?}", edition);

    let row: Row = conn.exec_first(
        "SELECT * FROM products WHERE id = ?",
        (edition.product_id,)
    ).await?
     .ok_or_else(|| anyhow!("Product not found"))?;
    let product     = DbProduct::from_row(&row);
    println!("Fetched product {:?}", product);

    let row: Row = conn.exec_first(
        "SELECT * FROM customers WHERE id = ?",
        (application.customer_id,)
    ).await?
     .ok_or_else(|| anyhow!("Customer not found"))?;
    let customer    = DbCustomer::from_row(&row);
    println!("Fetched customer {:?}", customer);

    let row: Row = conn.exec_first(
        "SELECT * FROM address WHERE id = ?",
        (customer.address_id,)
    ).await?
     .ok_or_else(|| anyhow!("Address not found"))?;
    let address     = DbAddress::from_row(&row);
    println!("Fetched address {:?}", address);

    let zipcode = fetch_zip_data(conn, &address.zip).await?;
    println!("Fetched zipcode {:?}", zipcode);

    // 2. Load license (optional)
    let license: Option<DbLicense> = conn
        .exec_first("SELECT ...", (application_id,))
        .await?
        .map(|row: Row| DbLicense::from_row(&row));

    let validity = license.as_ref().map(|l| ValidityInfo {
        issued: l.issued,
        expires: l.expires,
        major: l.valid_major,
        validity_unit: None,
        validity_value: None,
    });

    Ok(LicenseBundle {
        application,
        product,
        edition,
        customer,
        address,
        zipcode,
        license,
        validity,
    })
}
