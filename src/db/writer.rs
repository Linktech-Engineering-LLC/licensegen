// ============================================================================
// Filename: licensegen/src/db/writer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-03
// Description: 
// ============================================================================

use mysql_async::{Conn, Error, params, prelude::*};

use crate::db::{
    DbApplication,
    DbEdition,
    DbProduct,
};
use crate::license::types::{
    SignedLicense,
    ValidityInfo,
};
use crate::product::{Address, ContactSection, EditionInfo, ApplicationRequest};
use crate::util::{from_naive_date, opt_i32};

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
            "valid_major" => opt_i32(validity.valid_major),
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
            "zip"      => addr.zip,
        },
    ).await?;

    if let Some(id) = id {
        return Ok(id);
    }

    conn.exec_drop(
        r#"
        INSERT INTO address (maildrop, street, suite, city, state, county, country, zip, zip4)
        VALUES (:maildrop, :street, :suite, :city, :state, :county, :country, :zip, :zip4)
        "#,
        params! {
            "maildrop" => &addr.maildrop,
            "street"   => &addr.street,
            "suite"    => &addr.suite,
            "city"     => &addr.city,
            "state"    => &addr.state,
            "county"   => &addr.county,
            "country"  => &addr.country,
            "zip"      => addr.zip,
            "zip4"     => addr.zip4,
        },
    ).await?;

    Ok(conn.last_insert_id().expect("address insert missing last_insert_id"))
}

pub async fn resolve_or_insert_application(
    conn: &mut Conn,
    customer_id: u64,
    edition_id: u64,
    req: &ApplicationRequest,
) -> Result<u64, mysql_async::Error> {
    // 1. Try to resolve existing application
    let existing: Option<u64> = conn
        .exec_first(
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
        )
        .await?;

    if let Some(id) = existing {
        return Ok(id);
    }

    // 2. Insert new application
    conn.exec_drop(
        r#"
        INSERT INTO applications
            (customer_id, edition_id, name, raw_yaml, received, acquired)
        VALUES
            (:customer_id, :edition_id, :app_name, :raw_yaml, :received, :acquired)
        "#,
        params! {
            "customer_id" => customer_id,
            "edition_id"  => edition_id,
            "app_name"    => &req.request.name,
            "raw_yaml"    => &req.raw_yaml,
            "received"    => &req.vendor.received_on,
            "acquired"    => &req.vendor.acquired,
        },
    )
    .await?;

    // 3. Return new ID
    Ok(conn.last_insert_id().expect("INSERT into applications did not produce a last_insert_id"))
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
            (product_id, sku, edition_code, name, valid, metadata)
        VALUES
            (:product_id, :sku, :code, :name, :valid, :metadata)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            valid = VALUES(valid),
            metadata = VALUES(metadata)
        "#,
        params! {
            "product_id" => product_id,
            "sku"        => &edition.sku,
            "code"       => &edition.code,
            "name"       => &edition.name,
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
) -> Result<bool, mysql_async::Error> {
    conn.exec_drop(
        r#"
        INSERT INTO products
            (name, code, version, payload_schema, features, editions, keypair_path, active)
        VALUES
            (:name, :code, :version, :payload_schema, :features, :editions, :keypair_path, :active)
        ON DUPLICATE KEY UPDATE
            payload_schema = VALUES(payload_schema),
            features = VALUES(features),
            editions = VALUES(editions),
            keypair_path = VALUES(keypair_path),
            active = VALUES(active)
        "#,
        params! {
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

    let rows: Option<u64> = conn.exec_first("SELECT ROW_COUNT()", ()).await?;
    let rows = rows.unwrap_or(0);

    Ok(rows == 1 || rows == 2)
}