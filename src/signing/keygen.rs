// ============================================================================
// Filename: licensegen/src/keygen.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-18
// Modified: 2026-03-11
// Description:
// ============================================================================

// System Libraries
use rand::thread_rng;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPrivateKey, pkcs1::EncodeRsaPublicKey};
use std::fs;
use std::path::Path;

// Project Libraries

pub fn generate_rsa_keypair(
    private_path: &str,
    public_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // If both keys already exist, skip generation
    if Path::new(private_path).exists() && Path::new(public_path).exists() {
        println!("Keypair already exists, skipping generation.");
        return Ok(());
    }

    let mut rng = thread_rng();

    // Generate a 4096-bit RSA private key
    let private_key = RsaPrivateKey::new(&mut rng, 4096)?;
    let public_key = RsaPublicKey::from(&private_key);

    // Encode keys to PKCS#1 PEM
    let private_pem = private_key.to_pkcs1_pem(Default::default())?;
    let public_pem = public_key.to_pkcs1_pem(Default::default())?;

    // Write to disk
    fs::write(private_path, private_pem)?;
    fs::write(public_path, public_pem)?;

    println!("Generated:");
    println!("  Private key: {}", private_path);
    println!("  Public key:  {}", public_path);

    Ok(())
}
