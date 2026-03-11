// ============================================================================
// Filename: licensegen/src/db/reader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-11
// Description: 
// ============================================================================
use mysql_async::{Row, Conn, params, Transaction, prelude::Queryable};
use anyhow::{anyhow, Result};

use crate::db::types::{
    DbAddress, DbAddressView, 
    DbApplication, DbApplicationView, 
    DbCustomer, DbCustomerView,
    DbEdition, DbEditionView,  
    DbLicense, DbProduct, DbZipcode
};
use crate::license::types::{LicenseBundle, ValidityInfo};
use crate::signing::types::ZipCode;
use crate::util::datetime::{to_naive_date, to_naive_datetime};

pub async fn fetch_by_id_conn(
    conn: &mut mysql_async::Conn,
    source: &str,
    id: u64,
) -> mysql_async::Result<Option<mysql_async::Row>> {
    let query = format!("SELECT * FROM {} WHERE id = ?", source);
    conn.exec_first(query, (id,)).await
}
pub async fn fetch_by_id_tx<'a>(
    tx: &mut mysql_async::Transaction<'a>,
    source: &str,
    id: u64,
) -> mysql_async::Result<Option<mysql_async::Row>> {
    let query = format!("SELECT * FROM {} WHERE id = ?", source);
    tx.exec_first(query, (id,)).await
}
pub async fn fetch_application(
    conn: &mut Conn,
    app_id: u64,
) -> mysql_async::Result<DbApplication>
{
    let row = fetch_by_id_conn(conn, "applications", app_id)
    .await?
    .expect("Application not found");
 
    let app = DbApplication::from_row(&row);

    Ok(app)
}

pub async fn fetch_address(
    conn: &mut Conn,
    adr_id: u64,
) -> anyhow::Result<DbAddressView>
{
    let row = fetch_by_id_conn(conn, "v_address", adr_id)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Address not found"))?;
    let address = DbAddressView::from_row(&row);

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

    let zipcode =DbZipcode::from_row(&row);

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
    let row = fetch_by_id_conn(
        conn, "applications", application_id
    ).await?
     .ok_or_else(|| anyhow!("Application not found"))?;
    let application = DbApplication::from_row(&row);
   
    let row: Row = fetch_by_id_conn(
        conn, "editions",application.edition_id
    ).await?
     .ok_or_else(|| anyhow!("Edition not found"))?;
    let edition     = DbEdition::from_row(&row);

    let row: Row = fetch_by_id_conn(
        conn, "products",edition.product_id
    ).await?
     .ok_or_else(|| anyhow!("Product not found"))?;
    let product     = DbProduct::from_row(&row);

    let row: Row = fetch_by_id_conn(
        conn, "customers",application.customer_id
    ).await?
     .ok_or_else(|| anyhow!("Customer not found"))?;
    let customer    = DbCustomer::from_row(&row);

    let row: Row = fetch_by_id_conn(
        conn, "address",customer.address_id
    ).await?
     .ok_or_else(|| anyhow!("Address not found"))?;
    let address     = DbAddress::from_row(&row);

    let zipcode = fetch_zip_data(conn, &address.zip).await?;

    // 2. Load license (optional)
    println!("searching licenses for {}", application_id);
    let license: Option<DbLicense> = conn
        .exec_first("SELECT * from licenses where application_id = ?", (application_id,))
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
