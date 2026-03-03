// ============================================================================
// Filename: licensegen/src/license/db_writer.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-03
// Description: Writes signed licenses into the database.
// ============================================================================

use mysql_async::{params, prelude::Queryable, Pool};

use crate::license::types::{SignedLicense, ValidityInfo};
use crate::product::db_model::{DbApplication, DbEdition};
use crate::util::from_naive_date;

pub async fn write_license_to_db(
    pool: &Pool,
    app: &DbApplication,
    edition: &DbEdition,
    signed: &SignedLicense,
    validity: &ValidityInfo,
) -> anyhow::Result<u64> {

    let mut conn = pool.get_conn().await?;

    // ------------------------------------------------------------------------
    // 1. Insert license row
    // ------------------------------------------------------------------------

    let license_id: u64 = conn.exec_first(
        r#"
        INSERT INTO licenses
            (application_id, edition_id, version,
             payload, features, signature,
             issued, expires, valid_major, revoked)
        VALUES
            (:app_id, :edition_id, :version,
             :payload, :features, :signature,
             :issued, :expires, :valid_major, :revoked)
        "#,
        params! {
            "app_id"      => app.id,
            "edition_id"  => edition.id,
            "version"     => validity.valid_major,
            "payload"     => &signed.payload_json,   // compact JSON
            "features"    => "{}",                   // placeholder
            "signature"   => &signed.signature,      // Base64
            "issued"      => from_naive_date(validity.issued),
            "expires"     => validity.expires.map(from_naive_date),
            "valid_major" => validity.valid_major,
            "revoked"     => false,
        }
    ).await?
    .ok_or_else(|| anyhow::anyhow!("INSERT returned no ID"))?;

    // ------------------------------------------------------------------------
    // 2. Update application status → approved
    // ------------------------------------------------------------------------

    conn.exec_drop(
        r#"
        UPDATE applications
        SET status = 'approved'
        WHERE id = :id
        "#,
        params! { "id" => app.id }
    ).await?;

    Ok(license_id)
}
