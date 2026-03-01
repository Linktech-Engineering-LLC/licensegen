// ============================================================================
// Filename: licensegen/src/product/sync.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-03-01
// Description: Synchronization logic for product.yml → Products table.
// ============================================================================

// System Libraries
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use mysql_async::{Pool, Row, params, prelude::*};
use mysql_common::value::Value;
// Project Libraries
use crate::product::{ApplicationRequest, DbApplication, DbProduct, EditionRoot, Product};
// Local Functions
fn to_naive_date(v: Value) -> NaiveDate {
    match v {
        Value::Date(year, month, day, ..) => {
            NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .expect("invalid date from MySQL")
        }
        _ => panic!("expected DATE"),
    }
}
fn to_naive_datetime(v: Value) -> NaiveDateTime {
    match v {
        Value::Date(year, month, day, hour, min, sec, micros) => {
            let date = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .expect("invalid date");
            let time = NaiveTime::from_hms_micro_opt(hour as u32, min as u32, sec as u32, micros)
                .expect("invalid time");
            NaiveDateTime::new(date, time)
        }
        _ => panic!("expected DATETIME"),
    }
}

/// Returns:
///   Ok((true,  id))  → inserted or updated
///   Ok((false, id))  → no change
pub async fn sync_product(
    pool: &Pool,
    product: &Product,
) -> Result<(bool, u64), mysql_async::Error> {
    // Acquire a connection explicitly
    let mut conn = pool.get_conn().await?;

    // Convert Product → DbProduct
    let dbp = DbProduct::from(product);

    log::info!("Syncing product: {}", dbp.name);
    log::debug!("  code: {}", dbp.code);
    log::debug!("  version: {}", dbp.version);
    log::debug!("  editions: {}", dbp.editions);
    log::debug!("  payload_schema: {}", dbp.payload_schema);
    log::debug!("  keypair_path: {}", dbp.keypair_path);

    // Perform upsert
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
    )
    .await?;

    // mysql_async does not return rows_affected directly from exec_drop.
    // We must query ROW_COUNT().
    let rows: Option<u64> = conn.exec_first("SELECT ROW_COUNT()", ()).await?;
    let rows = rows.unwrap_or(0);

    let changed = rows == 1 || rows == 2;

    if rows == 1 {
        log::info!("Inserted new product '{}'", dbp.name);
    } else if rows == 2 {
        log::info!("Updated existing product '{}'", dbp.name);
    } else {
        log::info!("No changes for product '{}'", dbp.name);
    }

    // Fetch product_id deterministically
    let product_id: Option<u64> = conn
        .exec_first(
            "SELECT id FROM products WHERE code = :code",
            params! { "code" => &dbp.code },
        )
        .await?;

    let product_id = match product_id {
        Some(id) => id,
        None => {
            log::error!(
                "Product '{}' was not found after sync. This should never happen.",
                dbp.code
            );
            return Err(mysql_async::Error::Other(
                "Product not found after sync".into(),
            ));
        }
    };

    Ok((changed, product_id))
}

pub async fn sync_edition(
    pool: &Pool,
    product_id: u64,
    root: &EditionRoot,
) -> Result<bool, mysql_async::Error> {
    let edition = &root.edition;

    log::info!("Syncing edition: {}", edition.sku);
    log::debug!("  code: {}", edition.code);
    log::debug!("  name: {}", edition.name);
    log::debug!("  valid: {}", edition.valid);

    // Serialize metadata explicitly
    let metadata_json = serde_json::json!({
        "features": root.features,
        "constraints": root.constraints,
        "defaults": root.defaults,
    });

    let metadata_str =
        serde_json::to_string(&metadata_json).expect("metadata JSON serialization failed");

    let mut conn = pool.get_conn().await?;

    let sql = r#"
        INSERT INTO editions
            (product_id, sku, edition_code, name, valid, metadata)
        VALUES (:product_id, :sku, :code, :name, :valid, :metadata)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            valid = VALUES(valid),
            metadata = VALUES(metadata)
    "#;

    conn.exec_drop(
        sql,
        params! {
            "product_id" => product_id,
            "sku"        => &edition.sku,
            "code"       => &edition.code,
            "name"       => &edition.name,
            "valid"      => edition.valid,
            "metadata"   => metadata_str,
        },
    )
    .await?;

    let rows = conn.affected_rows();
    let changed = rows == 1 || rows == 2;

    if rows == 1 {
        log::info!("Inserted edition '{}'", edition.sku);
    } else if rows == 2 {
        log::info!("Updated edition '{}'", edition.sku);
    } else {
        log::info!("No changes for edition '{}'", edition.sku);
    }

    Ok(changed)
}

pub async fn sync_application(
    conn: &mut mysql_async::Conn,
    req: &ApplicationRequest,
) -> Result<u64, mysql_async::Error> {
    // 1. Resolve existing address_id (Option<u64> → u64)
    let mut address_id: u64 = conn
        .exec_first::<u64, _, _>(
            r#"
        SELECT id FROM address
        WHERE maildrop = :maildrop
          AND street   = :street
          AND city     = :city
          AND state    = :state
          AND zip      = :zip
        "#,
            params! {
                "maildrop" => &req.contact.address.maildrop,
                "street"   => &req.contact.address.street,
                "city"     => &req.contact.address.city,
                "state"    => &req.contact.address.state,
                "zip"      => req.contact.address.zip,
            },
        )
        .await?
        .unwrap_or(0);

    if address_id == 0 {
        conn.exec_drop(
            r#"
        INSERT INTO address (maildrop, street, suite, city, state, county, country, zip, zip4)
        VALUES (:maildrop, :street, :suite, :city, :state, :county, :country, :zip, :zip4)
        "#,
            params! {
                "maildrop" => &req.contact.address.maildrop,
                "street"   => &req.contact.address.street,
                "suite"    => &req.contact.address.suite,
                "city"     => &req.contact.address.city,
                "state"    => &req.contact.address.state,
                "county"   => &req.contact.address.county,
                "country"  => &req.contact.address.country,
                "zip"      => req.contact.address.zip,
                "zip4"     => req.contact.address.zip4,
            },
        )
        .await?;

        address_id = conn
            .last_insert_id()
            .expect("INSERT into address did not produce a last_insert_id");
    }
    //
    // 3. Resolve or insert customer
    //
    let mut customer_id: u64 = conn
        .exec_first::<u64, _, _>(
            r#"
            SELECT id FROM customers WHERE email = :email
            "#,
            params! {
                "email" => &req.contact.email,
            },
        )
        .await?
        .unwrap_or(0);

    if customer_id == 0 {
        conn.exec_drop(
            r#"
        INSERT INTO customers (first, last, email, company, phone, address_id, notes)
        VALUES (:first, :last, :email, :company, :phone, :address_id, :notes)
        "#,
            params! {
                "first" => &req.contact.name.first,
                "last"  => &req.contact.name.last,
                "email"      => &req.contact.email,
                "company"    => &req.contact.company,
                "phone"      => &req.contact.phone,
                "address_id" => address_id,
                "notes"      => &req.contact.notes,
            },
        )
        .await?;

        customer_id = conn
            .last_insert_id()
            .expect("INSERT into customers did not produce a last_insert_id");
    }

    //
    // 4. Resolve edition_id
    //
    let product_id: u64 = conn
        .exec_first(
            r#"
        SELECT id FROM products
        WHERE name = :product
        "#,
            params! {
                "product" => &req.request.product,
            },
        )
        .await?
        .expect("product must exist before syncing application");

    let edition_id: u64 = conn
        .exec_first(
            r#"
        SELECT id FROM editions
        WHERE product_id = :product_id
          AND edition_code = :edition
        "#,
            params! {
                "product_id" => product_id,
                "edition" => &req.request.edition,
            },
        )
        .await?
        .expect("edition must exist before syncing application");

    //
    // 5. Resolve or insert application
    //
    let mut app_id: u64 = conn
        .exec_first::<u64, _, _>(
            r#"
        SELECT id FROM applications
        WHERE customer_id      = :customer_id
          AND edition_id       = :edition_id
          AND name = :app_name
        "#,
            params! {
                "customer_id" => customer_id,
                "edition_id"  => edition_id,
                "app_name"    => &req.request.name,
            },
        )
        .await?
        .unwrap_or(0);

    if app_id == 0 {
        conn.exec_drop(
            r#"
        INSERT INTO applications (customer_id, edition_id, name, raw_yaml, received, acquired)
        VALUES (:customer_id, :edition_id, :app_name, :raw_yaml, :received, :acquired)
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

        app_id = conn
            .last_insert_id()
            .expect("INSERT into applications did not produce a last_insert_id");
    }

    Ok(app_id)
}

pub async fn fetch_application(
    pool: &mysql_async::Pool,
    app_id: u64,
) -> mysql_async::Result<DbApplication> {
    let mut conn = pool.get_conn().await?;

    let row: Row = conn
        .exec_first(
            r#"
        SELECT
            id,
            customer_id,
            edition_id,
            name,
            raw_yaml,
            received,
            acquired,
            status
        FROM applications
        WHERE id = :id
        "#,
            params! { "id" => app_id },
        )
        .await?
        .expect("application not found");

    let received = to_naive_date(row.get("received").unwrap());
    let acquired = to_naive_date(row.get("acquired").unwrap());

    let app = DbApplication {
        id: row.get("id").unwrap(),
        customer_id: row.get("customer_id").unwrap(),
        edition_id: row.get("edition_id").unwrap(),
        name: row.get("name").unwrap(),
        raw_yaml: row.get("raw_yaml").unwrap(),
        received,
        acquired,
        status: row.get("status").unwrap(),
    };

    Ok(app)
}
