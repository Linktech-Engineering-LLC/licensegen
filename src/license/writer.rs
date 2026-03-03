// ============================================================================
// Filename: licensegen/src/license/writer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-02
// Description: Writes signed license artifacts to disk.
// ============================================================================

// System Libraries
use std::fs;
use std::path::Path;

// Project Libraries
use crate::types::SignedLicense;

pub fn write_license_file<P: AsRef<Path>>(
    signed: &SignedLicense,
    path: P,
) -> std::io::Result<()> {
    // signed.signature is already Base64 from signer.rs

    // Construct final JSON object deterministically
    // No pretty-printing, no re-serialization of payload_json
    let final_json = format!(
        "{{\"payload\":{},\"signature\":\"{}\"}}",
        signed.payload_json,
        signed.signature,
    );

    // Write atomically
    fs::write(path, final_json)
}
