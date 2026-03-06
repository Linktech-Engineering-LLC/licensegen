// ============================================================================
// Filename: licensegen/src/main.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-03-06
// Description: Entry point for licensegen.
// ============================================================================

use std::path::{self, PathBuf, Path};

use licensegen::db::pool::init_pool;
use licensegen::db::reader::{fetch_address, fetch_application};
use licensegen::db::types::DbAddress;
use licensegen::config::Config;
use licensegen::license::generator::generate_license;
use licensegen::product::loader::{load_all_editions, load_all_products, load_application};
use licensegen::product::sync::{sync_application, sync_edition, sync_product};
use licensegen::signing::loaders::load_keypair;
use licensegen::signing::resolver::resolve_keypair_paths;
use licensegen::vault::types::{VaultError, VaultSecrets};
use licensegen::vault::ansible::decrypt_with_ansible;

use log::{debug, error, info};
use logger::{end_banner, init, start_banner};

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
    let pool = init_pool(&secrets).await?;

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
    let mut conn = pool.get_conn().await?;
    let app = fetch_application(&mut conn, application_id).await?;
    println!("Fetched Application {}", app);
    
    let adr_id: u64 = 1;
    let adr = fetch_address(&mut conn, adr_id).await?;
    println!("Address Loaded {:?}", adr);

    let private_key_path = Path::new(&cfg.paths.keypair_dir);
    let output_dir_path = Path::new(&cfg.paths.output_dir);

    let (license_id, license_path) = generate_license(
        &mut conn,
        application_id,
        &private_key_path,
        &output_dir_path,
    ).await?;
    info!("License generated and synced with ID {}", license_id);
    // End banner
    end_banner();

    Ok(())
}
