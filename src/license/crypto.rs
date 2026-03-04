// ============================================================================
// Filename: licensegen/src/license/crypto.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-04
// Modified: 2026-03-04
// Description: 
// ============================================================================

use anyhow::Context;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::Pkcs1v15Sign;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::license::types::{LicensePayload, SignedLicense};

/// Canonicalize a LicensePayload to compact, deterministic JSON.
fn canonicalize_payload(payload: &LicensePayload) -> anyhow::Result<String> {
    let value: Value = serde_json::to_value(payload)
        .context("Failed to convert payload to JSON value")?;

    let json = serde_json::to_string(&value)
        .context("Failed to serialize payload to canonical JSON")?;

    Ok(json)
}

/// Canonicalize an arbitrary JSON `Value` (e.g. parsed payload from wrapper).
fn canonicalize_value(value: &Value) -> anyhow::Result<String> {
    let json = serde_json::to_string(value)
        .context("Failed to serialize JSON value to canonical JSON")?;

    Ok(json)
}

/// Sign a LicensePayload: canonical JSON → SHA-256 digest → PKCS#1 v1.5 signature → Base64.
pub fn sign_payload(
    payload: &LicensePayload,
    private_key: &RsaPrivateKey,
) -> anyhow::Result<SignedLicense> {
    // 1. Canonical JSON
    let json = canonicalize_payload(payload)?;

    // 2. SHA-256 digest of canonical JSON
    let digest = Sha256::digest(json.as_bytes());

    // 3. Low-level PKCS#1 v1.5 signing of the digest
    let raw_sig = private_key
        .sign(Pkcs1v15Sign::new::<Sha256>(), &digest)
        .context("Failed to sign license payload")?;

    // 4. Base64 encode for storage/transport
    let signature_b64 = B64.encode(raw_sig);

    Ok(SignedLicense {
        payload_json: json,
        signature: signature_b64,
    })
}

/// Validate a wrapped license JSON:
/// {
///   "payload": { ...canonicalizable... },
///   "signature": "<base64 PKCS#1 v1.5 over SHA-256(payload_json)>"
/// }
pub fn validate_license(
    license_json: &str,
    public_key: &RsaPublicKey,
) -> anyhow::Result<()> {
    // 1. Parse wrapper JSON
    let wrapper: Value = serde_json::from_str(license_json)
        .context("Invalid license JSON")?;

    let payload = wrapper
        .get("payload")
        .context("Missing 'payload' field")?;

    let signature_b64 = wrapper
        .get("signature")
        .and_then(|v| v.as_str())
        .context("Missing or invalid 'signature' field")?;

    // 2. Canonical JSON for payload
    let canonical_payload = canonicalize_value(payload)?;
    println!("VALIDATOR canonical payload: {}", canonical_payload);

    // 3. Base64 decode signature → raw bytes
    let signature_bytes = B64
        .decode(signature_b64)
        .context("Invalid Base64 signature")?;

    // 4. SHA-256 digest of canonical JSON
    let digest = Sha256::digest(canonical_payload.as_bytes());

    // 5. Verify digest-level PKCS#1 v1.5 signature
    public_key
        .verify(Pkcs1v15Sign::new::<Sha256>(), &digest, &signature_bytes)
        .map_err(|_| anyhow::anyhow!("Signature verification failed"))?;

    Ok(())
}