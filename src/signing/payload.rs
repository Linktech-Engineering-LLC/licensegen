// ============================================================================
// Filename: licensegen/src/payload.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-02-18
// Description: Defines the LicensePayload struct used for RSA signing.
// ============================================================================

// System Libraries
use serde::Serialize;
// Project Libraries

#[derive(Debug, Serialize)]
pub struct LicensePayload {
    pub email: String,
    pub acquired: String, // ISO date: "YYYY-MM-DD"
    pub edition: String,  // "DEV", "PRO", "ENT"
    pub valid_major: u32, // e.g. 3 for 3.x versions
}
