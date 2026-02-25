// ============================================================================
// Filename: licensegen/src/vault/decryptor
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-23
// Modified: 2026-02-23
// Description: Ansible Vault Decryptor Modeled from Python Calls
// ============================================================================
#![allow(dead_code)]
#![allow(unused_imports)]
// System Libraries
use aes::Aes256;
//use cipher::block_padding::Padding;
use cbc::Decryptor;
use cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
//use pkcs7::pad::unpad;
use sha2::Sha256;

type Aes256CbcDec = Decryptor<Aes256>;
type Aes256Ctr = Ctr128BE<Aes256>;
type HmacSha256 = Hmac<Sha256>;

// Project Libraries
use crate::Config;
use crate::vault::VaultSecrets;

#[derive(Debug)]
pub enum VaultError {
    ReadError(String),
    HeaderError,
    HexDecodeError(&'static str),
    KeyDeriveError,
    CipherError(&'static str),
    PaddingError(&'static str),
    Utf8Error,
    YamlError(String),
    HmacError,
}

fn pkcs7_unpad(data: &[u8]) -> Result<&[u8], VaultError> {
    if data.is_empty() {
        return Err(VaultError::PaddingError("empty buffer"));
    }

    let pad_len = *data.last().unwrap() as usize;

    if pad_len == 0 || pad_len > data.len() {
        return Err(VaultError::PaddingError("invalid pad length"));
    }

    if !data[data.len() - pad_len..]
        .iter()
        .all(|&b| b as usize == pad_len)
    {
        return Err(VaultError::PaddingError("padding bytes mismatch"));
    }

    Ok(&data[..data.len() - pad_len])
}

pub fn load_vault(cfg: &Config) -> Result<VaultSecrets, VaultError> {
    // 1. Read vault file + password
    let raw = std::fs::read_to_string(&cfg.vault.file)
        .map_err(|e| VaultError::ReadError(e.to_string()))?;
    let password = std::fs::read_to_string(&cfg.vault.password_file)
        .map_err(|e| VaultError::ReadError(e.to_string()))?;
    let password = password.trim();

    // 2. Parse header
    let mut lines = raw.lines();
    let header = lines.next().ok_or(VaultError::HeaderError)?;
    if !header.starts_with("$ANSIBLE_VAULT;1.1;AES256") {
        return Err(VaultError::HeaderError);
    }

    // 3. Join remaining lines into hex vaulttext
    let hex_body: String = lines.collect::<Vec<_>>().join("").trim().to_string();

    // 4. Convert hex → bytes (this is b_vaulttext)
    let vaulttext = hex::decode(hex_body).map_err(|_| VaultError::HexDecodeError("vault body"))?;

    // 5. Split into salt / hmac / ciphertext (newline-separated)
    let parts: Vec<&[u8]> = vaulttext.split(|b| *b == b'\n').collect();
    if parts.len() < 3 {
        return Err(VaultError::CipherError(
            "vault body missing salt/hmac/ciphertext parts",
        ));
    }

    let salt_hex = parts[0];
    let hmac_hex = parts[1];
    let ciphertext_hex = parts[2];

    eprintln!("DEBUG: password bytes = {:?}", password.as_bytes());
    eprintln!("DEBUG: salt_hex = {:?}", std::str::from_utf8(salt_hex));
    eprintln!("DEBUG: hmac_hex = {:?}", std::str::from_utf8(hmac_hex));
    eprintln!(
        "DEBUG: ciphertext_hex = {:?}",
        std::str::from_utf8(ciphertext_hex)
    );

    let salt = hex::decode(salt_hex).map_err(|_| VaultError::HexDecodeError("salt"))?;
    let ciphertext =
        hex::decode(ciphertext_hex).map_err(|_| VaultError::HexDecodeError("ciphertext"))?;

    eprintln!("DEBUG: ciphertext len = {}", ciphertext.len());
    eprintln!("DEBUG: ciphertext len % 16 = {}", ciphertext.len() % 16);

    let crypted_hmac_hex = std::str::from_utf8(hmac_hex)
        .map_err(|_| VaultError::CipherError("hmac line not valid UTF-8"))?;

    // 6. Derive keys (PBKDF2-HMAC-SHA256, 10k iterations, 64 bytes)
    let mut derived = [0u8; 64];

    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 10_000, &mut derived);

    let enc_key = &derived[0..32];
    let hmac_key = &derived[32..64]; // kept for future HMAC, even if unused now

    // 7. Verify HMAC
    // Ansible Vault 1.1 does not include an HMAC.
    // The second line is an HMAC salt, not an HMAC.
    // Skip HMAC verification for this format.
    if !header.starts_with("$ANSIBLE_VAULT;1.1;AES256") {
        let expected_hmac =
            hex::decode(crypted_hmac_hex).map_err(|_| VaultError::HexDecodeError("hmac"))?;

        let mut mac = HmacSha256::new_from_slice(&hmac_key).map_err(|_| VaultError::HmacError)?;
        mac.update(&ciphertext);
        mac.verify_slice(&expected_hmac)
            .map_err(|_| VaultError::HmacError)?;
    }

    // 8. AES-256-CTR decrypt
    if ciphertext.len() < 16 {
        return Err(VaultError::CipherError("ciphertext shorter than IV"));
    }

    let (iv, enc_payload) = ciphertext.split_at(16);

    let mut buf = enc_payload.to_vec();

    let mut cipher = Aes256Ctr::new_from_slices(enc_key, iv)
        .map_err(|_| VaultError::CipherError("invalid key/iv length"))?;

    cipher.apply_keystream(&mut buf);
    eprintln!(
        "DEBUG: raw decrypted bytes = {:02x?}",
        &buf[..64.min(buf.len())]
    );

    // No PKCS7 in CTR mode
    let yaml_str = String::from_utf8_lossy(&buf);

    // 10. YAML parse + subtree
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&yaml_str).map_err(|e| VaultError::YamlError(e.to_string()))?;

    let app_key = env!("CARGO_PKG_NAME").to_lowercase();
    let subtree = yaml
        .get(&app_key)
        .ok_or_else(|| VaultError::YamlError(format!("Missing vault key '{}'", app_key)))?;

    let secrets: VaultSecrets = serde_yaml::from_value(subtree.clone())
        .map_err(|e| VaultError::YamlError(e.to_string()))?;

    Ok(secrets)
}
