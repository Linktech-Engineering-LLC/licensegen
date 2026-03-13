// ============================================================================
// Filename: licensegen/src/product/keypairs.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-13
// Modified: 20026-03-13
// Description: Resolves and loads RSA keypairs for products.
// ============================================================================

use log::{debug, error};
use std::fs;
use std::path::{Path, PathBuf};

use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::product::types::ProductError;

/// Resolve the private and public key paths for a product.
///
/// Example:
/// signing:
///     keypair: "botscanner"
///
/// Produces:
///     <keypair_dir>/botscanner_prv.pem
///     <keypair_dir>/botscanner_pub.pem
pub fn resolve_keypair_paths(keypair_base: &str, keypair_dir: &str) -> (PathBuf, PathBuf) {
    let private_name = format!("{}_prv.pem", keypair_base.to_lowercase());
    let public_name  = format!("{}_pub.pem", keypair_base.to_lowercase());

    let private_path = Path::new(keypair_dir).join(private_name);
    let public_path  = Path::new(keypair_dir).join(public_name);

    (private_path, public_path)
}

/// Load RSA keypair from PEM files.
///
/// Returns `(RsaPrivateKey, RsaPublicKey)` or an error.
pub fn load_keypair(
    private_path: &Path,
    public_path: &Path,
) -> Result<(RsaPrivateKey, RsaPublicKey), ProductError> {
    debug!("Loading private key from {:?}", private_path);
    debug!("Loading public key from {:?}", public_path);

    // Read PEM files
    let private_pem = fs::read_to_string(private_path).map_err(|e| {
        ProductError::ReadError(format!("Failed to read private key {:?}: {}", private_path, e))
    })?;

    let public_pem = fs::read_to_string(public_path).map_err(|e| {
        ProductError::ReadError(format!("Failed to read public key {:?}: {}", public_path, e))
    })?;

    // Decode RSA keys
    let private_key = RsaPrivateKey::from_pkcs1_pem(&private_pem).map_err(|e| {
        ProductError::ReadError(format!("Invalid private key {:?}: {}", private_path, e))
    })?;

    let public_key = RsaPublicKey::from_pkcs1_pem(&public_pem).map_err(|e| {
        ProductError::ReadError(format!("Invalid public key {:?}: {}", public_path, e))
    })?;

    Ok((private_key, public_key))
}
