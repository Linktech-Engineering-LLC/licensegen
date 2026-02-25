// ============================================================================
// Filename: licensegen/src/db/pool.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-25
// Modified: 2026-02-25
// Description:
// ============================================================================

// System Libraries
use sqlx::MySqlPool;
// Project Libraries
use crate::vault::VaultSecrets;

pub async fn init_pool(cfg: &VaultSecrets) -> Result<MySqlPool, sqlx::Error> {
    let conn_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        cfg.user, cfg.pass, cfg.host, cfg.port, cfg.database
    );

    MySqlPool::connect(&conn_string).await
}
