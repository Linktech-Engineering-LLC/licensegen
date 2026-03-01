// ============================================================================
// Filename: licensegen/src/db/pool.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-25
// Modified: 2026-03-01
// Description:
// ============================================================================

// System Libraries
use mysql_async::{Error, Opts, Pool};
// Project Libraries
use crate::vault::VaultSecrets;

pub async fn init_pool(cfg: &VaultSecrets) -> Result<Pool, mysql_async::Error> {
    let conn_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        cfg.user, cfg.pass, cfg.host, cfg.port, cfg.database
    );

    let opts = Opts::from_url(&conn_string).expect("invalid MySQL connection string");

    let pool = Pool::new(opts);
    Ok(pool)
}
