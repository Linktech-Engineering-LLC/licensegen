// ============================================================================
// Filename: licensegen/src/db/writer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-10
// Description: 
// ============================================================================

use mysql_async::{Conn, Error, params, prelude::*, TxOpts, Transaction};
use rust_decimal::Decimal;
use anyhow::Result;

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

pub async fn next_deterministic_id(
    tx: &mut Transaction<'_>,
    table: &str,
) -> Result<u64, Error> {
    let query = format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table);
    let next: Option<u64> = tx.exec_first(query, ()).await?;
    Ok(next.unwrap_or(1))
}

pub async fn upsert_product(
    conn: &mut Conn,
    dbp: &DbProduct,
) -> Result<(bool, u64), Error> {
    // Start transaction
    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    // Determine next deterministic ID
    let next_id = next_deterministic_id(&mut tx, "products").await?;

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

pub async fn upsert_edition(
    conn: &mut Conn,
    product_id: u64,
    edition: &EditionInfo,
    metadata: &str,
) -> Result<bool, mysql_async::Error> {
    // Start transaction
    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    // Determine next deterministic ID
    let next_id = next_deterministic_id(&mut tx, "editions").await?;

    // Perform upsert
    tx.exec_drop(
        r#"
        INSERT INTO editions
            (id, product_id, sku, edition_code, name, price, valid, metadata)
        VALUES
            (:id, :product_id, :sku, :code, :name, :price, :valid, :metadata)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            price = VALUES(price),
            valid = VALUES(valid),
            metadata = VALUES(metadata)
        "#,
        params! {
            "id"         => next_id,
            "product_id" => product_id,
            "sku"        => &edition.sku,
            "code"       => &edition.code,
            "name"       => &edition.name,
            "price"      => edition.price,
            "valid"      => edition.valid,
            "metadata"   => metadata,
        },
    ).await?;

    // Check affected rows
    let rows = tx.affected_rows();
    let changed = rows == 1 || rows == 2;

    // Commit
    tx.commit().await?;

    Ok(changed)
}

pub async fn resolve_or_insert_address(
    conn: &mut Conn,
    addr: &Address,
) -> Result<u64, mysql_async::Error> {
    // Start transaction
    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    // Try to resolve existing address
    let existing: Option<u64> = tx
        .exec_first(
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
        )
        .await?;

    if let Some(id) = existing {
        tx.commit().await?;
        return Ok(id);
    }

    // Allocate deterministic ID
    let next_id = next_deterministic_id(&mut tx, "address").await?;

    // Insert new address
    tx.exec_drop(
        r#"
        INSERT INTO address
            (id, maildrop, street, suite, city, state, county, country, zip)
        VALUES
            (:id, :maildrop, :street, :suite, :city, :state, :county, :country, :zip)
        "#,
        params! {
            "id"       => next_id,
            "maildrop" => &addr.maildrop,
            "street"   => &addr.street,
            "suite"    => &addr.suite,
            "city"     => &addr.city,
            "state"    => &addr.state,
            "county"   => &addr.county,
            "country"  => &addr.country,
            "zip"      => &addr.zip,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(next_id)
}

pub async fn resolve_or_insert_customer(
    conn: &mut Conn,
    addr: &ContactSection,
    address_id: u64,
) -> Result<u64, mysql_async::Error> {
    // Start transaction
    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    // 1. Try to resolve existing customer by email
    let existing: Option<u64> = tx
        .exec_first(
            r#"
            SELECT id FROM customers
            WHERE email = :email
            "#,
            params! {
                "email" => &addr.email,
            },
        )
        .await?;

    if let Some(id) = existing {
        tx.commit().await?;
        return Ok(id);
    }

    // 2. Allocate deterministic ID
    let next_id = next_deterministic_id(&mut tx, "customers").await?;

    // 3. Insert new customer
    tx.exec_drop(
        r#"
        INSERT INTO customers
            (id, first, last, email, company, phone, address_id, notes)
        VALUES
            (:id, :first, :last, :email, :company, :phone, :address_id, :notes)
        "#,
        params! {
            "id"         => next_id,
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

    // 4. Commit and return deterministic ID
    tx.commit().await?;
    Ok(next_id)
}

pub async fn resolve_or_upsert_application(
    conn: &mut Conn,
    customer_id: u64,
    edition_id: u64,
    req: &ApplicationRequest,
) -> Result<u64, mysql_async::Error> {
    // Start transaction
    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    // 1. Try to resolve existing application by unique key
    let existing: Option<u64> = tx
        .exec_first(
            r#"
            SELECT id FROM applications
            WHERE customer_id = :customer_id
              AND edition_id  = :edition_id
              AND name        = :name
            "#,
            params! {
                "customer_id" => customer_id,
                "edition_id"  => edition_id,
                "name"        => &req.request.name,
            },
        )
        .await?;

    let id = if let Some(id) = existing {
        // 2a. Update existing application
        tx.exec_drop(
            r#"
            UPDATE applications
            SET price          = :price,
                valid_major    = :valid_major,
                validity_value = :validity_value,
                validity_unit  = :validity_unit,
                raw_yaml       = :raw_yaml,
                received       = :received,
                acquired       = :acquired,
                updated        = CURRENT_TIMESTAMP
            WHERE id = :id
            "#,
            params! {
                "id"            => id,
                "price"         => req.vendor.price.unwrap_or(Decimal::ZERO),
                "valid_major"   => req.vendor.valid_major,
                "validity_value"=> req.vendor.validity_value.unwrap_or(0),
                "validity_unit" => &req.vendor.validity_unit.as_deref(),
                "raw_yaml"      => &req.raw_yaml,
                "received"      => from_naive_date(req.vendor.received_on),
                "acquired"      => from_naive_date(req.vendor.acquired),
            },
        )
        .await?;

        id
    } else {
        // 2b. Allocate deterministic ID
        let next_id = next_deterministic_id(&mut tx, "applications").await?;

        // Insert new application
        tx.exec_drop(
            r#"
            INSERT INTO applications
                (id, name, customer_id, edition_id, price, valid_major,
                 validity_value, validity_unit, raw_yaml, received, acquired)
            VALUES
                (:id, :name, :customer_id, :edition_id, :price, :valid_major,
                 :validity_value, :validity_unit, :raw_yaml, :received, :acquired)
            "#,
            params! {
                "id"            => next_id,
                "name"          => &req.request.name,
                "customer_id"   => customer_id,
                "edition_id"    => edition_id,
                "price"         => req.vendor.price.unwrap_or(Decimal::ZERO),
                "valid_major"   => req.vendor.valid_major,
                "validity_value"=> req.vendor.validity_value.unwrap_or(0),
                "validity_unit" => &req.vendor.validity_unit.as_deref(),
                "raw_yaml"      => &req.raw_yaml,
                "received"      => from_naive_date(req.vendor.received_on),
                "acquired"      => from_naive_date(req.vendor.acquired),
            },
        )
        .await?;

        next_id
    };

    // Commit
    tx.commit().await?;

    Ok(id)
}

pub async fn insert_new_license_row(
    conn: &mut Conn,
    bundle: &LicenseBundle,
) ->Result<u64> {

    // 1. Begin transaction
    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    // 2. Compute deterministic ID using your helper
    let new_id = next_deterministic_id(&mut tx, "licenses").await?;

    // 3. Insert the new row using the deterministic ID
    tx.exec_drop(
        r#"
        INSERT INTO licenses (
            id,
            application_id,
            edition_id,
            issued
        )
        VALUES (:id, :app, :edition, :issued)
        "#,
        params! {
            "id"      => new_id,
            "app"     => bundle.application.id,
            "edition" => bundle.edition.id,
            "issued"  => from_naive_date(bundle.validity.as_ref().unwrap().issued),
        },
    ).await?;

    // 4. Commit the transaction
    tx.commit().await?;

    // 5. Return deterministic ID
    Ok(new_id)
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

