// ============================================================================
// Filename: licensegen/src/logger_init.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-24
// Modified: 2026-03-11
// Description:
// ============================================================================

// System Libraries
// Project Libraries
use log::{error, info};
use logger::{Summary, end_banner, init, start_banner};

pub fn setup_logging() {
    let app_name = env!("CARGO_PKG_NAME");
    init(app_name);
    start_banner();
}

pub fn shutdown_logging() {
    end_banner();
}
