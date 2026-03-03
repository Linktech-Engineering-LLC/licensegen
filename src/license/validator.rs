// ============================================================================
// Filename: licensegen/src/license/validator.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-02
// Description: Validates signed license artifacts.
// ============================================================================

use rsa::RsaPublicKey;
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::Value;

pub fn validate_license(
    license_json: &str,
    public_key: &RsaPublicKey,
) -> anyhow::Result<()> {

    // 1. Parse wrapper JSON: {"payload": {...}, "signature": "<b64>"}
    let wrapper: Value = serde_json::from_str(license_json)
        .map_err(|e| anyhow::anyhow!("Invalid license JSON: {}", e))?;

    let payload = wrapper.get("payload")
        .ok_or_else(|| anyhow::anyhow!("Missing 'payload' field"))?;

    let signature_b64 = wrapper.get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'signature' field"))?;

    // 2. Canonical compact JSON serialization of payload
    let canonical_payload = serde_json::to_string(payload)
        .map_err(|e| anyhow::anyhow!("Failed to canonicalize payload: {}", e))?;

    // 3. Base64 decode signature
    let signature = B64.decode(signature_b64)
        .map_err(|e| anyhow::anyhow!("Invalid Base64 signature: {}", e))?;

    // 4. Compute SHA-256 hash of canonical payload
    let mut hasher = Sha256::new();
    hasher.update(canonical_payload.as_bytes());
    let hash = hasher.finalize();

    // 5. Verify RSA PKCS#1 v1.5 signature

    let mut hasher = Sha256::new();
    hasher.update(message_bytes);
    let hashed_message = hasher.finalize();

    public_key.verify(
        Pkcs1v15Sign::new_unprefixed(),
        &hashed_message,
        &signature_bytes,
    )
        .map_err(|_| anyhow::anyhow!("Signature verification failed"))?;

    Ok(())
}