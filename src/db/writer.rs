// ============================================================================
// Filename: licensegen/src/db/writer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-07
// Description: 
// ============================================================================

use mysql_async::{Conn, Error, params, prelude::*, TxOpts};
use rust_decimal::Decimal;

use crate::db::types::{
    DbApplication,
    DbEdition,
    DbProduct,
};
use crate::license::types::{
    LicenseBundle,
    SignedLicense,
    ValidityInfo,
};
use crate::product::types::{Address, ContactSection, ApplicationRequest, EditionInfo};
use crate::util::datetime::{from_naive_date, opt};

pub async fn write_license_to_db(
    conn: &mut Conn,
    app: &DbApplication,
    edition: &DbEdition,
    signed: &SignedLicense,
    validity: &ValidityInfo,
) -> Result<u64, mysql_async::Error> {
    // Insert license row
    conn.exec_drop(
        r#"
        INSERT INTO licenses
            (application_id, edition_id, payload, signature, issued, expires, valid_major)
        VALUES
            (:application_id, :edition_id, :payload, :signature, :issued, :expires, :valid_major)
        "#,
        params! {
            "application_id" => app.id,
            "edition_id"     => edition.id,
            "payload"        => &signed.payload_json,
            "signature"      => &signed.signature,
            "issued"         => from_naive_date(validity.issued),
            "expires"        => validity.expires.map(from_naive_date),
            "valid_major" => opt(validity.valid_major),
        },
    )
    .await?;

    let license_id = conn
        .last_insert_id()
        .expect("INSERT into licenses did not produce a last_insert_id");

    // Update application status
    conn.exec_drop(
        r#"
        UPDATE applications
        SET status = 'approved'
        WHERE id = :id
        "#,
        params! {
            "id" => app.id,
        },
    )
    .await?;

    Ok(license_id)
}

pub async fn resolve_or_insert_address(
    conn: &mut Conn,
    addr: &Address,
) -> Result<u64, mysql_async::Error> {
    let id: Option<u64> = conn.exec_first(
        r#"
        SELECT id FROM address
        WHERE maildrop = :maildrop
          AND street   = :street
          AND city     = :city
          AND state    = :state
          AND zip      = :zip
        "#,
        params! {
            "maildrop" => &addr.maildrop,
            "street"   => &addr.street,
            "city"     => &addr.city,
            "state"    => &addr.state,
            "zip"      => &addr.zip,
        },
    ).await?;

    if let Some(id) = id {
        return Ok(id);
    }

    conn.exec_drop(
        r#"
        INSERT INTO address (id, maildrop, street, suite, city, state, county, country, zip)
        VALUES (:maildrop, :street, :suite, :city, :state, :county, :country, :zip)
        "#,
        params! {
            "maildrop" => &addr.maildrop,
            "street"   => &addr.street,
            "suite"    => &addr.suite,
            "city"     => &addr.city,
            "state"    => &addr.state,
            "county"   => &addr.county,
            "country"  => &addr.country,
            "zip"      => &addr.zip,
        },
    ).await?;

    Ok(conn.last_insert_id().expect("address insert missing last_insert_id"))
}

pub async fn resolve_or_upsert_application(
    conn: &mut Conn,
    customer_id: u64,
    edition_id: u64,
    req: &ApplicationRequest,
) -> Result<u64, mysql_async::Error> {

    // 1. Perform UPSERT using the unique key (name, customer_id, edition_id)
    conn.exec_drop(
        r#"
        INSERT INTO applications
            (name, customer_id, edition_id, price, valid_major,
             validity_value, validity_unit, raw_yaml, received, acquired)
        VALUES
            (:app_name, :customer_id, :edition_id, :price, :valid_major,
             :validity_value, :validity_unit, :raw_yaml, :received, :acquired)
        ON DUPLICATE KEY UPDATE
            price          = VALUES(price),
            valid_major    = VALUES(valid_major),
            validity_value = VALUES(validity_value),
            validity_unit  = VALUES(validity_unit),
            raw_yaml       = VALUES(raw_yaml),
            received       = VALUES(received),
            acquired       = VALUES(acquired),
            updated        = CURRENT_TIMESTAMP
        "#,
        params! {
            "app_name"       => &req.request.name,
            "customer_id"    => customer_id,
            "edition_id"     => edition_id,
            "price"          => req.vendor.price.unwrap_or(Decimal::ZERO),
            "valid_major"    => req.vendor.valid_major,
            "validity_value" => req.vendor.validity_value.unwrap_or(0),
            "validity_unit"  => &req.vendor.validity_unit.as_deref(),
            "raw_yaml"       => &req.raw_yaml,
            "received"       => from_naive_date(req.vendor.received_on),
            "acquired"       => from_naive_date(req.vendor.acquired),
        },
    ).await?;

    // 2. Retrieve the application ID (works for both INSERT and UPDATE)
    let id: Option<u64> = conn.exec_first(
        r#"
        SELECT id FROM applications
        WHERE customer_id = :customer_id
          AND edition_id  = :edition_id
          AND name        = :app_name
        "#,
        params! {
            "customer_id" => customer_id,
            "edition_id"  => edition_id,
            "app_name"    => &req.request.name,
        },
    ).await?;

    Ok(id.expect("UPSERT succeeded but SELECT id returned None"))
}

pub async fn resolve_or_insert_customer(
    conn: &mut Conn,
    addr: &ContactSection,
    address_id: u64,
) -> Result<u64, mysql_async::Error> {
    // 1. Try to resolve existing customer
    let existing: Option<u64> = conn
        .exec_first(
            r#"
            SELECT id FROM customers WHERE email = :email
            "#,
            params! {
                "email" => &addr.email,
            },
        )
        .await?;

    if let Some(id) = existing {
        return Ok(id);
    }

    // 2. Insert new customer
    conn.exec_drop(
        r#"
        INSERT INTO customers (first, last, email, company, phone, address_id, notes)
        VALUES (:first, :last, :email, :company, :phone, :address_id, :notes)
        "#,
        params! {
            "first"      => &addr.name.first,
            "last"       => &addr.name.last,
            "email"      => &addr.email,
            "company"    => &addr.company,
            "phone"      => &addr.phone,
            "address_id" => address_id,
            "notes"      => &addr.notes,
        },
    )
    .await?;

    // 3. Return new ID
    Ok(conn.last_insert_id().expect("INSERT into customers did not produce a last_insert_id"))
}

pub async fn upsert_edition(
    conn: &mut Conn,
    product_id: u64,
    edition: &EditionInfo,
    metadata: &str,
) -> Result<bool, mysql_async::Error> {
    conn.exec_drop(
        r#"
        INSERT INTO editions
            (product_id, sku, edition_code, name, price, valid, metadata)
        VALUES
            (:product_id, :sku, :code, :name, :price, :valid, :metadata)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            price = VALUES(price),
            valid = VALUES(valid),
            metadata = VALUES(metadata)
        "#,
        params! {
            "product_id" => product_id,
            "sku"        => &edition.sku,
            "code"       => &edition.code,
            "name"       => &edition.name,
            "price"      => edition.price,
            "valid"      => edition.valid,
            "metadata"   => metadata,
        },
    ).await?;

    let rows: Option<u64> = conn.exec_first("SELECT ROW_COUNT()", ()).await?;
    let rows = rows.unwrap_or(0);

    Ok(rows == 1 || rows == 2)
}

pub async fn upsert_product(
    conn: &mut Conn,
    dbp: &DbProduct,
) -> Result<(bool, u64), Error> {
    // Start transaction
    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    // Determine next deterministic ID
    let next_id: Option<u64> = tx
        .exec_first("SELECT COALESCE(MAX(id), 0) + 1 FROM products", ())
        .await?;

    let next_id = next_id.unwrap_or(1);

    // Perform deterministic upsert
    tx.exec_drop(
        r#"
        INSERT INTO products
            (id, name, code, version, payload_schema, features, editions, keypair_path, active)
        VALUES
            (:id, :name, :code, :version, :payload_schema, :features, :editions, :keypair_path, :active)
        ON DUPLICATE KEY UPDATE
            payload_schema = VALUES(payload_schema),
            features = VALUES(features),
            editions = VALUES(editions),
            keypair_path = VALUES(keypair_path),
            active = VALUES(active)
        "#,
        params! {
            "id" => next_id,
            "name" => &dbp.name,
            "code" => &dbp.code,
            "version" => &dbp.version,
            "payload_schema" => &dbp.payload_schema,
            "features" => &dbp.features,
            "editions" => &dbp.editions,
            "keypair_path" => &dbp.keypair_path,
            "active" => dbp.active,
        },
    ).await?;

    // Determine if row changed
    let rows: Option<u64> = tx.exec_first("SELECT ROW_COUNT()", ()).await?;
    let rows = rows.unwrap_or(0);
    let changed = rows == 1 || rows == 2;

    // Fetch product ID deterministically
    let product_id: Option<u64> = tx.exec_first(
        "SELECT id FROM products WHERE code = :code",
        params! { "code" => &dbp.code },
    ).await?;

    let product_id = product_id.ok_or_else(|| {
        Error::Other("Product not found after upsert".into())
    })?;

    // Commit
    tx.commit().await?;

    Ok((changed, product_id))
}

pub async fn insert_new_license_row(
    conn: &mut Conn,
    bundle: &LicenseBundle,
) -> anyhow::Result<u64> {

    conn.exec_drop(
        r#"
        INSERT INTO licenses (
            application_id,
            edition_id,
            issued,
        )
        VALUES (:app, :edition, :issued)
        "#,
        params! {
            "app" => bundle.application.id,
            "edition" => bundle.edition.id,
            "issued" => from_naive_date(bundle.validity.issued),
        },
    ).await?;

    let id = conn.last_insert_id().unwrap() as u64;
    Ok(id)
}

pub async fn update_license_row(
    conn: &mut Conn,
    license_id: u64,
    signed: &SignedLicense,
) -> anyhow::Result<()> {
    conn.exec_drop(
        r#"
        UPDATE license
        SET payload_json = ?, signature = ?
        WHERE id = ?
        "#,
        (
            &signed.payload_json,
            &signed.signature,
            license_id,
        ),
    ).await?;

    Ok(())
}

