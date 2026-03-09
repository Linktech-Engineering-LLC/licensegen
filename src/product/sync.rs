// ============================================================================
// Filename: licensegen/src/product/sync.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-03-09
// Description: Synchronization logic for product.yml → Products table.
// ============================================================================

use mysql_async::Pool;

use crate::db::types::{
    DbApplication, 
    DbProduct,
};
use crate::db::writer::{
    resolve_or_insert_address, 
    resolve_or_upsert_application,
    resolve_or_insert_customer,
    upsert_product, 
    upsert_edition
};
use crate::db::reader::{
    resolve_edition_id_by_sku,
    get_product_id_by_code
};
use super::types::{ApplicationRequest, EditionRoot, Product};
use crate::util::datetime::{to_naive_date, to_naive_datetime};

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
    log::debug!("  version: {:?}", dbp.version.as_deref());
    log::debug!("  editions: {:?}", dbp.editions.as_deref());
    log::debug!("  payload_schema: {}", dbp.payload_schema);
    log::debug!("  keypair_path: {}", dbp.keypair_path);

    // Perform upsert
    let (changed, product_id) = upsert_product(&mut conn, &dbp).await?;

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

    let changed = upsert_edition(
        &mut conn,
        product_id,
        edition,
        &metadata_str,
    ).await?;

    Ok(changed)
}

pub async fn sync_application(
    conn: &mut mysql_async::Conn,
    req: &ApplicationRequest,
) -> Result<u64, mysql_async::Error> {
    // 1. Resolve existing address_id (Option<u64> → u64)
     let address_id = resolve_or_insert_address(conn, &req.contact.address).await?;
    //
    // 3. Resolve or insert customer
    //
    let customer_id = resolve_or_insert_customer(conn, &req.contact, address_id).await?;
    let edition_id = resolve_edition_id_by_sku(conn, &req.request.sku).await?;
    let app_id = resolve_or_upsert_application(conn, customer_id, edition_id, req).await?;

    Ok(app_id)
}
