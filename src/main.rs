// ============================================================================
// Filename: licensegen/src/main.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-03-01
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
use crate::product::{fetch_application, load_all_editions, load_all_products, load_application};
use crate::product::{sync_application, sync_edition, sync_product};
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

        // Resolve keypair paths
        let (private_path, public_path) =
            resolve_keypair_paths(&p.signing.keypair, &cfg.paths.keypair_dir);

        debug!("keypair: {:?} and {:?}", private_path, public_path);

        // Load keypair
        let (private_key, public_key) =
            load_keypair(&private_path, &public_path).unwrap_or_else(|e| {
                error!("Keypair missing or invalid: {}", e);
                error!("Run `licensegen keygen` to create the keypair.");
                std::process::exit(1);
            });

        // Load editions from filesystem
        let edition_roots = match load_all_editions(&p.dir) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to load editions for {}: {}", p.name, e);
                std::process::exit(1);
            }
        };

        // Sync product and get product_id
        let (changed, product_id) = match sync_product(&pool, p).await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to sync product {}: {}", p.name, e);
                std::process::exit(1);
            }
        };

        if changed {
            info!("Product '{}' updated in database", p.name);
        } else {
            info!("Product '{}' unchanged", p.name);
        }

        // Sync editions
        for (_code, root) in &edition_roots {
            sync_edition(&pool, product_id, root).await?;
        }
    }
    // ------------------------------------------------------------
    // Load and sync application
    // ------------------------------------------------------------

    // Determine the application file path for this product
    let app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&cfg.paths.products_dir)
        .join("BotScanner")
        .join("application.yml");

    info!("Loading application from {:?}", app_path);

    let application = load_application(app_path.to_str().unwrap())?;
    info!("Application loaded: {}", application.request.name);

    // Sync application
    let mut conn = pool.get_conn().await?;

    let application_id = match sync_application(&mut conn, &application).await {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to sync application: {}", e);
            return Err(e.into());
        }
    };

    info!("Application synced with ID {}", application_id);

    // ------------------------------------------------------------
    // Generate and sync license
    // ------------------------------------------------------------
    let app = fetch_application(&pool, application_id).await?;
    println!("Fetched Application {}", app);
    //let license = generate_license(&application)?;
    //let license_id = sync_license(&pool, application_id, &license).await?;

    //info!("License generated and synced with ID {}", license_id);
    // End banner
    end_banner();

    Ok(())
}
