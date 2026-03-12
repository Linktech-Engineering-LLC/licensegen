// ============================================================================
// Filename: licensegen/src/license/signer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-12
// Description: Deterministic RSA signer for canonical license payloads.
// ============================================================================
use crate::license::types::{LicensePayload, SignedLicense};
use crate::license::crypto::sign_payload;
use anyhow::Context;
use rsa::RsaPrivateKey;

/// High-level wrapper: build a SignedLicense from a LicensePayload.
/// All cryptographic work is delegated to crypto.rs.
pub fn sign(
    payload: &LicensePayload,
    private_key: &RsaPrivateKey,
) -> anyhow::Result<SignedLicense> {
    sign_payload(payload, private_key)
        .context("Failed to sign license payload")
}
