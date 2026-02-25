// ============================================================================
// Filename: licensegen/src/product/sync.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-02-25
// Description: Synchronization logic for product.yml → Products table.
// ============================================================================

use crate::product::{DbProduct, Product};
use sqlx::MySqlPool;

/// Load product.yml, convert to DbProduct, and upsert into the Products table.
impl From<&Product> for DbProduct {
    fn from(p: &Product) -> Self {
        Self {
            name: p.name.clone(),
            code: p.code.clone(),
            version: p.version.clone(),

            payload_schema: serde_json::to_string(&p.license.payload_fields)
                .expect("payload_schema JSON"),

            editions: serde_json::to_string(&p.editions).expect("editions JSON"),

            features: "{}".into(), // or whatever you decide later

            keypair_path: p.signing.keypair.clone(),
            active: "Y".into(),
        }
    }
}
/// Returns:
///   Ok(true)  → inserted or updated
///   Ok(false) → no change
pub async fn sync_product(pool: &MySqlPool, yaml: &Product) -> Result<bool, sqlx::Error> {
    let dbp = DbProduct::from(yaml);

    log::info!("Syncing product: {}", dbp.name);
    log::debug!("  code: {}", dbp.code);
    log::debug!("  version: {}", dbp.version);
    log::debug!("  editions: {}", dbp.editions);
    log::debug!("  payload_schema: {}", dbp.payload_schema);
    log::debug!("  keypair_path: {}", dbp.keypair_path);

    // Runtime SQL — no macros, no compile-time validation
    let result = sqlx::query(
        r#"
        INSERT INTO products
            (name, code, version, payload_schema, features, editions, keypair_path, active)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            payload_schema = VALUES(payload_schema),
            features = VALUES(features),
            editions = VALUES(editions),
            keypair_path = VALUES(keypair_path)
        "#,
    )
    .bind(&dbp.name)
    .bind(&dbp.code)
    .bind(&dbp.version)
    .bind(&dbp.payload_schema)
    .bind(&dbp.features)
    .bind(&dbp.editions)
    .bind(&dbp.keypair_path)
    .bind(&dbp.active)
    .execute(pool)
    .await?;

    let rows = result.rows_affected();

    if rows == 1 {
        log::info!("Inserted new product '{}'", dbp.name);
        Ok(true)
    } else if rows == 2 {
        log::info!("Updated existing product '{}'", dbp.name);
        Ok(true)
    } else {
        log::info!("No changes for product '{}'", dbp.name);
        Ok(false)
    }
}
