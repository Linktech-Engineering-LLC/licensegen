// ============================================================================
// Filename: licensegen/src/signers.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-03-11
// Description:
// ============================================================================

// System Libraries
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::Pkcs1v15Sign;
//use rsa::pkcs1v15::SigningKey; // <-- REQUIRED
use sha2::{Digest, Sha256};

// Project Libraries
// (none yet)

pub fn sign_message(
    private_key_pem: &str,
    message: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    // Load private key
    let private_key = RsaPrivateKey::from_pkcs1_pem(private_key_pem)?;

    // Hash the message manually
    let digest = Sha256::digest(message);

    // Create PKCS#1 v1.5 padding (no OID required)
    let padding = Pkcs1v15Sign::new_unprefixed();

    // Sign the raw digest
    let signature = private_key.sign(padding, &digest)?;

    // Base64 encode
    Ok(STANDARD.encode(signature))
}
