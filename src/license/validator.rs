// ============================================================================
// Filename: licensegen/src/license/validator.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-04
// Description: Validates signed license artifacts.
// ============================================================================

use anyhow::Context;
use rsa::RsaPublicKey;
use std::path::Path;

use crate::license::crypto::validate_license;

/// High-level wrapper: validate a wrapped license JSON string.
/// All cryptographic work is delegated to crypto.rs.
pub fn validate(
    license_json: &str,
    public_key: &RsaPublicKey,
) -> anyhow::Result<()> {
    validate_license(license_json, public_key)
        .context("License validation failed")
}
pub fn validate_license_file(path: &Path, public_key: &RsaPublicKey) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(path)?;
    validate(&json, public_key)
}
