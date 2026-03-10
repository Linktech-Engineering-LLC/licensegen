// ============================================================================
// Filename: licensegen/src/license/validator.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-09
// Description: Validates signed license artifacts.
// ============================================================================

use anyhow::Context;
use rsa::RsaPublicKey;
use std::path::Path;

use crate::license::crypto::validate_license;
use crate::license::types::ValidationOutcome;

/// High-level wrapper: validate a wrapped license JSON string.
/// All cryptographic + semantic work is delegated to crypto.rs.
pub fn validate(
    license_json: &str,
    public_key: &RsaPublicKey,
) -> ValidationOutcome {
    // validate_license() now returns ValidationOutcome
    validate_license(license_json, public_key)
}

/// Validate a license file on disk.
/// This function *must* return anyhow::Result<()> because the generator
/// uses it in a fallible pipeline.
pub fn validate_license_file(
    path: &Path,
    public_key: &RsaPublicKey,
) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(path)
        .context("Failed to read license file")?;

    // Convert structured outcome into anyhow::Result
    validate(&json, public_key).into_anyhow()
}
