// ============================================================================
// Filename: licensegen/src/signing/loaders.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-24
// Modified: 2026-03-11
// Description: Loads the RSA Keypairs if the exist, and returns to main
// ============================================================================

// System Libraries
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::DecodeRsaPrivateKey, pkcs1::DecodeRsaPublicKey};
use std::path::{Path, PathBuf};
// Project Libraries

pub fn load_keypair(
    private_path: &Path,
    public_path: &Path,
) -> Result<(RsaPrivateKey, RsaPublicKey), String> {
    if !private_path.exists() {
        return Err(format!("Missing private key: {}", private_path.display()));
    }

    if !public_path.exists() {
        return Err(format!("Missing public key: {}", public_path.display()));
    }

    let private_pem = std::fs::read_to_string(private_path)
        .map_err(|e| format!("Failed to read private key: {}", e))?;

    let public_pem = std::fs::read_to_string(public_path)
        .map_err(|e| format!("Failed to read public key: {}", e))?;

    let private_key = RsaPrivateKey::from_pkcs1_pem(&private_pem)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let public_key = RsaPublicKey::from_pkcs1_pem(&public_pem)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    Ok((private_key, public_key))
}
