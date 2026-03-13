// ============================================================================
// Filename: licensegen/src/product/ingest.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-13
// Modified: 2026-03-13
// Description: 
// ============================================================================

use log::{info, error, debug};
use mysql_async::Pool;
use tokio::io::stdin;
use std::path::PathBuf;

use crate::config::config::Config;
use super::loader::{load_all_products, load_all_editions, load_application};
use super::sync::{sync_product, sync_edition, sync_application};
use super::keypairs::{resolve_keypair_paths, load_keypair};
use super::types::{ProductError, AppError};

pub async fn ingest_all(cfg: &Config, pool: &Pool) -> Result<(), ProductError> {
    info!("Loading products from {:?}", cfg.paths.products_dir);

    let products = load_all_products(&cfg.paths.products_dir)?;
    info!("Loaded {} products", products.len());
    stdin::process::exit(0);  
    for product in &products {
        info!("Product loaded: {}", product.name);

        // ------------------------------------------------------------
        // 1. Resolve + load keypair
        // ------------------------------------------------------------
        let (private_path, public_path) =
            resolve_keypair_paths(&product.signing.keypair, &cfg.paths.keypair_dir);

        debug!("keypair: {:?} and {:?}", private_path, public_path);

        let (_private_key, _public_key) =
            load_keypair(&private_path, &public_path).unwrap_or_else(|e| {
                error!("Keypair missing or invalid: {}", e);
                error!("Run `licensegen keygen` to create the keypair.");
                std::process::exit(1);
            });

        // ------------------------------------------------------------
        // 2. Load editions
        // ------------------------------------------------------------
        let edition_roots = load_all_editions(&product.dir)?;
        info!("Loaded {} editions for {}", edition_roots.len(), product.name);

        // ------------------------------------------------------------
        // 3. Sync product
        // ------------------------------------------------------------
        let (changed, product_id) = sync_product(pool, product).await?;
        if changed {
            info!("Product '{}' updated in database", product.name);
        } else {
            info!("Product '{}' unchanged", product.name);
        }

        // ------------------------------------------------------------
        // 4. Sync editions
        // ------------------------------------------------------------
        for (_code, edition_root) in &edition_roots {
            sync_edition(pool, product_id, edition_root).await?;
        }

        // ------------------------------------------------------------
        // 5. Load application.yml
        // ------------------------------------------------------------
        let app_path = product.dir.join("application.yml");
        info!("Loading application from {:?}", app_path);

        let mut conn = pool.get_conn().await.map_err(|e| {
            ProductError::ReadError(format!("Failed to get DB connection: {}", e))
        })?;

        let application = load_application(&mut conn, app_path.to_str().unwrap())
            .await
            .map_err(|e| ProductError::ReadError(format!("Failed to load application: {}", e)))?;

        info!("Application loaded: {}", application.request.name);

        // ------------------------------------------------------------
        // 6. Sync application
        // ------------------------------------------------------------
        let application_id = sync_application(&mut conn, &application)
            .await
            .map_err(|e| ProductError::ReadError(format!("Failed to sync application: {}", e)))?;

        info!("Application synced with ID {}", application_id);
    }

    Ok(())
}

