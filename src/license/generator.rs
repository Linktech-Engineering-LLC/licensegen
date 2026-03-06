// ============================================================================
// Filename: licensegen/src/license/generator.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-06
// Description: 
// ============================================================================
use mysql_async::Conn;
use std::path::{Path, PathBuf};
use rsa::RsaPublicKey;

use crate::db::writer::{insert_new_license_row, update_license_row};
use crate::db::reader::load_license_bundle;
use crate::license::evaluator::evaluate_license;
use crate::license::payload::build_payload;
use crate::license::crypto::{sign_payload, load_private_key, validate_license};
use crate::license::writer::write_license_file;
use crate::license::types::{LicenseDecision, ValidationOutcome};
use crate::util::datetime::compute_expiration;

pub async fn generate_license(
    conn: &mut Conn,
    application_id: u64,
    private_key_path: &Path,
    output_dir: &Path,
) -> anyhow::Result<(u64, PathBuf)> {
    // 1. Load all DB rows needed for this license
    let bundle = load_license_bundle(conn, application_id).await?;

    // 2. Decide whether to reuse or issue new
    let decision = evaluate_license(&bundle)?;
    let expires = compute_expiration(&bundle.validity);

    let id = match decision {
        LicenseDecision::ReuseExisting(ref lic) => lic.id,
        LicenseDecision::IssueNew => {
            insert_new_license_row(conn, &bundle).await?
        }
    };

    // 3. Build the canonical payload
    let payload = build_payload(
        &bundle.application,
        &bundle.product,
        &bundle.edition,
        &bundle.customer,
        &bundle.address,
        &bundle.zipcode,
        bundle.validity,   // <-- required
   )?;

    // 4. Sign the payload
    let private_key = load_private_key(private_key_path)?;
    let public_key = RsaPublicKey::from(&private_key);
    let signed = sign_payload(&payload, &private_key)?;

    // 4b. Build wrapped JSON for validation
    let signed_json = serde_json::json!({
        "payload": serde_json::from_str::<serde_json::Value>(&signed.payload_json)
            .expect("payload_json must be valid JSON"),
        "signature": signed.signature,
    }).to_string();

    // 4c. Validate our own output
    validate_license(&signed_json, &public_key).into_anyhow()?;
    update_license_row(conn, id, &signed).await?;

    // 5. Determine output path
    let filename = match decision {
        LicenseDecision::ReuseExisting(ref lic) => {
            format!("license_{}.lic", lic.id)
        }
        LicenseDecision::IssueNew => {
            // You need a strategy here: new licenses don’t have an ID yet.
            "license_new.lic".to_string()
        }
    };
    let path = PathBuf::from(output_dir).join(filename);

    // 6. Write the final license file
    write_license_file(&signed, &path)?;

    Ok((id, path))
}
