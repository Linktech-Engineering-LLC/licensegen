// ============================================================================
// Filename: licensegen/src/license/generator.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-03
// Description: 
// ============================================================================

// System Libraries
use mysql_async::Conn;
use rsa::RsaPrivateKey;
use std::path::PathBuf;
// Project Libraries
use crate::db::{insert_new_license_row, load_license_bundle, update_license_row};
use crate::license::evaluator::evaluate_license;
use crate::license::payload::build_payload;
use crate::license::signer::sign_payload;
use crate::license::writer::write_license_file;
use crate::license::types::LicenseDecision;
use crate::util::compute_expiration;

pub async  fn generate_license(
    conn: &mut Conn,
    license_id: i64,
    private_key: &RsaPrivateKey,
    output_dir: &str,
) -> anyhow::Result<PathBuf> {
    // 1. Load all DB rows needed for this license
    let bundle = load_license_bundle(conn,license_id).await?;

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
    let signed = sign_payload(&payload, private_key)?;
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

    Ok(path)
}
