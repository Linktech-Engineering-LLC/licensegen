// ============================================================================
// Filename: licensegen/src/main.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-02-25
// Description: Entry point for licensegen.
// ============================================================================
// Declare modules
mod config;
mod db;
mod logger_init;
mod product;
mod signing;
mod vault;
// System Libraries
use std::path::PathBuf;
// User Libraries
use crate::product::load_all_products;
use crate::product::sync_product;
use config::Config;

use log::{debug, error, info};
use logger::{end_banner, init, start_banner};
use signing::{load_keypair, resolve_keypair_paths};
use vault::VaultError;
use vault::VaultSecrets;
use vault::decrypt_with_ansible;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let app_name = env!("CARGO_PKG_NAME");
    init(app_name);
    start_banner();

    // Load configuration

    let cfg_path = format!("{}/licensegen.yml", env!("CARGO_MANIFEST_DIR"));
    let cfg = Config::load(&cfg_path)?;
    info!("Loaded configuration from {:?}", cfg_path);

    // Decrypt vault
    let yaml_str = decrypt_with_ansible(&cfg.vault.file, &cfg.vault.password_file)?;
    info!("Vault decrypted successfully");

    let yaml: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;
    let app_key = env!("CARGO_PKG_NAME").to_lowercase();

    let subtree = yaml
        .get(&app_key)
        .ok_or_else(|| VaultError::YamlError(format!("Missing vault key '{}'", app_key)))?;

    let secrets: VaultSecrets = serde_yaml::from_value(subtree.clone())
        .map_err(|e| VaultError::YamlError(e.to_string()))?;
    info!("Vault secrets loaded for {}", app_key);
    let pool = db::pool::init_pool(&secrets).await?;

    // Load all products
    let prod_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&cfg.paths.products_dir);
    info!("Loading products from {:?}", prod_path);

    let products = load_all_products(prod_path.to_str().unwrap())?;
    info!("Loaded {} products", products.len());

    for p in &products {
        info!("Product loaded: {}", p.name);
        let (private_path, public_path) =
            resolve_keypair_paths(&p.signing.keypair, &cfg.paths.keypair_dir);
        debug!("keypair: {:?} and {:?}", private_path, public_path);
        let (private_key, public_key) =
            load_keypair(&private_path, &public_path).unwrap_or_else(|e| {
                error!("Keypair missing or invalid: {}", e);
                error!("Run `licensegen keygen` to create the keypair.");
                std::process::exit(1);
            });
        sync_product(&pool, p).await?;
    }

    // NOTE:
    // We cannot call sync_product yet because it requires a DB pool.
    info!("Product sync skipped (DB pool not implemented yet)");

    // End banner
    end_banner();

    Ok(())
}
