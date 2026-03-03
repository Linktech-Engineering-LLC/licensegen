// ============================================================================
// Filename: licensegen/src/license/signer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Description: Deterministic RSA signer for canonical license payloads.
// ============================================================================

use rsa::{RsaPrivateKey, PaddingScheme};
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::STANDARD as B64, Engine};

use crate::types::{LicensePayload, SignedLicense};

pub fn sign_payload(
    payload: &LicensePayload,
    private_key: &RsaPrivateKey,
) -> anyhow::Result<SignedLicense> {

    // 1. Canonical compact JSON
    let json = serde_json::to_string(payload)
        .map_err(|e| anyhow::anyhow!("Failed to serialize payload: {}", e))?;

    // 2. SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let hash = hasher.finalize();

    // 3. Deterministic RSA PKCS#1 v1.5 signature
    let padding = PaddingScheme::new_pkcs1v15_sign::<Sha256>();
    let raw_sig = private_key
        .sign(padding, &hash)
        .map_err(|e| anyhow::anyhow!("Failed to sign payload: {}", e))?;

    // 4. Base64 encode for DB storage
    let signature_b64 = B64.encode(raw_sig);

    Ok(SignedLicense {
        payload_json: json,
        signature: signature_b64,
    })
}
