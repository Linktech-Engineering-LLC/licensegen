// ============================================================================
// Filename: licensegen/src/license/signer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Description: Deterministic RSA signer for canonical license payloads.
// ============================================================================
use crate::license::types::{LicensePayload, SignedLicense};

use base64::Engine;
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::signature::{Signer, SignatureEncoding}; // 👈 add SignatureEncoding
//use rsa::signature::Signer;
use sha2::{Sha256, Digest};
use base64::engine::general_purpose::STANDARD as B64;
// 👇 bring the trait into scope
//use signature::Signature as _;

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
    let signing_key = SigningKey::<Sha256>::new(private_key.clone());
    let raw_sig = signing_key.sign(&hash);

    // 4. Base64 encode for DB storage
    let signature_b64 = B64.encode(raw_sig.to_bytes());

    Ok(SignedLicense {
        payload_json: json,
        signature: signature_b64,
    })
}