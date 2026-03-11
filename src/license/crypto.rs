// ============================================================================
// Filename: licensegen/src/license/crypto.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-04
// Modified: 2026-03-11
// Description: 
// ============================================================================

use anyhow::Context;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::Pkcs1v15Sign;
use rsa::pkcs1::DecodeRsaPrivateKey;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;

use crate::license::types::{LicensePayload, SignedLicense, ValidationOutcome};
use crate::license::errors::CryptoError;
use crate::license::evaluator::parse_major;

/// Loads the Private key
pub fn load_private_key(path: &std::path::Path) -> Result<RsaPrivateKey, CryptoError> {
    let pem = fs::read_to_string(path)
        .map_err(|e| CryptoError::ReadError(format!("Failed to read private key: {}", e)))?;

    let key = RsaPrivateKey::from_pkcs1_pem(&pem)
        .map_err(|e| CryptoError::ParseError(format!("Invalid private key format: {}", e)))?;

    Ok(key)
}

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
#[derive(Deserialize)]
struct LicenseEnvelope {
    pub payload: LicensePayload,
    pub signature: String,
}

pub fn validate_license(
    license_json: &str,
    public_key: &RsaPublicKey,
) -> ValidationOutcome {
    // 1. Parse wrapper JSON
    let wrapper: Value = match serde_json::from_str(license_json) {
        Ok(v) => v,
        Err(_) => return ValidationOutcome::PayloadMalformed("Invalid JSON".into()),
    };

    let payload_val = match wrapper.get("payload") {
        Some(v) => v.clone(),
        None => return ValidationOutcome::PayloadMalformed("Missing payload".into()),
    };

    let signature_b64 = match wrapper.get("signature").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return ValidationOutcome::PayloadMalformed("Missing signature".into()),
    };

    // 2. Canonical JSON
    let canonical_payload = match canonicalize_value(&payload_val) {
        Ok(s) => s,
        Err(e) => return ValidationOutcome::PayloadMalformed(format!("Canonicalization failed: {}", e)),
    };

    // 3. Decode signature
    let signature_bytes = match B64.decode(signature_b64) {
        Ok(b) => b,
        Err(_) => return ValidationOutcome::SignatureInvalid,
    };

    // 4. Digest
    let digest = Sha256::digest(canonical_payload.as_bytes());

    // 5. Verify signature
    if public_key
        .verify(Pkcs1v15Sign::new::<Sha256>(), &digest, &signature_bytes)
        .is_err()
    {
        return ValidationOutcome::SignatureInvalid;
    }

    // --- SEMANTIC VALIDATION -------------------------------------------------

    let payload: LicensePayload = match serde_json::from_value(payload_val) {
        Ok(p) => p,
        Err(_) => return ValidationOutcome::PayloadMalformed("Payload schema mismatch".into()),
    };

    let edition = &payload.edition;
    let validity = &payload.validity;
    let product = &payload.product;

    let code = edition.code.as_str();
    let is_com = code == "COM";
    let is_dev = code == "DEV";
    let is_demo = code == "DEMO";

    // COM and DEV: always valid if signature is valid
    if is_com || is_dev {
        return ValidationOutcome::Valid;
    }

    // 1. Expiration
    if let Some(exp) = validity.expires {
        let today = chrono::Local::now().date_naive();
        if exp < today {
            if is_demo {
                return ValidationOutcome::DemoExpired(exp);
            }
            return ValidationOutcome::Expired(exp);
        }
    } else {
        if is_demo {
            return ValidationOutcome::DemoMissingExpiration;
        }
    }

    // 2. Major version rules
    let product_major = parse_major(&product.version);

    if let Some(prod_major) = product_major {
        if let Some(major) = validity.major {
            if is_demo {
                if major != prod_major {
                    return ValidationOutcome::DemoMajorMismatch {
                        product_major: prod_major,
                        license_major: major,
                    };
                }
            } else {
                if prod_major > major {
                    return ValidationOutcome::MajorVersionMismatch {
                        product_major: prod_major,
                        license_major: major,
                    };
                }
            }
        }
    }

    ValidationOutcome::Valid
}
