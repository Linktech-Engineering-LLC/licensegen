// ============================================================================
// Filename: licensegen/src/signing/resolver.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-24
// Modified: 2026-03-10
// Description: Resolves the path to the RSA key pairs
// ============================================================================

// System Libraries
use std::path::PathBuf;
// Project Libraries

pub fn resolve_keypair_paths(base_name: &str, keypair_dir: &str) -> (PathBuf, PathBuf) {
    let base = base_name.to_lowercase();

    let private_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(keypair_dir)
        .join(format!("{}_prv.pem", base));

    let public_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(keypair_dir)
        .join(format!("{}_pub.pem", base));

    (private_path, public_path)
}
