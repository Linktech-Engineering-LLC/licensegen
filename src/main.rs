// ============================================================================
// Filename: licensegen/src/main.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-03-13
// Description: Entry point for licensegen.
// ============================================================================

use std::path::{self, PathBuf, Path};

use licensegen::config::loader::load_and_resolve;
use licensegen::db::pool::init_pool;
use licensegen::db::reader::{fetch_address, fetch_application};
use licensegen::license::generator::generate_license;
use licensegen::product::ingest::ingest_all;
use licensegen::product::loader::{load_all_editions, load_all_products, load_application};
use licensegen::product::sync::{sync_application, sync_edition, sync_product};
use licensegen::signing::loaders::load_keypair;
use licensegen::signing::resolver::resolve_keypair_paths;
use licensegen::util::helpers::resolve_path;
use licensegen::vault::loader::load_secrets;

use log::{debug, error, info};
use logger::{end_banner, init, start_banner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let app_name = env!("CARGO_PKG_NAME");
    init(app_name);
    start_banner();

    // Load configuration
    let cfg = load_and_resolve()?;
    let cfg_path = PathBuf::from(format!("{}/licensegen.yml", env!("CARGO_MANIFEST_DIR")));
    let cfg_dir = cfg_path.parent().unwrap().to_path_buf();
    info!("Loaded configuration from {:?}", cfg_path);
    println!("Configuration contains: {:?}", cfg);

    // Decrypt vault
    let secrets = load_secrets(&cfg)?;
    let app_key = env!("CARGO_PKG_NAME").to_lowercase();
    info!("Vault secrets loaded from {}", app_key);

    let pool = init_pool(&secrets).await?;

    ingest_all(&cfg, &pool).await?;    

    // ------------------------------------------------------------
    // Generate and sync license
    // ------------------------------------------------------------
    //let mut conn = pool.get_conn().await?;
    //let app = fetch_application(&mut conn, application_id).await?;
    //println!("Fetched Application {:?}", app);
    
    //let keypair_dir      = resolve_path(&cfg_dir, &cfg.paths.keypair_dir);
    //let output_dir       = resolve_path(&cfg_dir, &cfg.paths.applications_subdir);

    //let (license_id, license_path) = generate_license(
    //    &mut conn,
    //    application_id,
    //    &keypair_dir,
    //    &output_dir,
    //).await?;
    //info!("License generated and synced with ID {}", license_id);

    // End banner
    end_banner();

    Ok(())
}
