// ============================================================================
// Filename: licensegen/src/license/evaluator.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-11
// Description: 
// ============================================================================

use anyhow::Result;
use chrono::Duration;

use crate::license::types::{LicenseBundle, LicenseDecision, ValidityInfo, ValidityUnit};
use crate::license::errors::EvaluationError;
use crate::db::types::DbApplication;

pub fn determine_validity(app: &DbApplication) -> ValidityInfo {
    // Case 1: No expiration
    if app.validity_value == 0 {
        return ValidityInfo {
            issued: app.received,
            expires: None,
            major: app.valid_major,
            validity_value: Some(app.validity_value.into()),
            validity_unit: app.validity_unit.as_deref().and_then(ValidityUnit::from_str),
        };
    }

    // Case 2: Duration-based expiration
    let issued = app.received;
    let expires = match app.validity_unit.as_deref() {
        Some("days") => Some(issued + Duration::days(app.validity_value.into())),
        Some("weeks") => Some(issued + Duration::weeks(app.validity_value.into())),
        Some("months") => Some(issued + Duration::days(30 * app.validity_value as i64)),
        Some("years") => Some(issued + Duration::days(365 * app.validity_value as i64)),
        _ => None,
    };

    ValidityInfo {
        issued,
        expires,
        major: app.valid_major,
        validity_value: app.validity_value.into(),
        validity_unit: app.validity_unit.as_deref().and_then(ValidityUnit::from_str),
    }
}

pub fn parse_major(version: &str) -> Option<u8> {
    version.split('.')
        .next()
        .and_then(|s| s.parse::<u8>().ok())
}

pub fn evaluate_license(bundle: &LicenseBundle) -> Result<LicenseDecision, EvaluationError>{
    // 1. No license exists → issue new
    let Some(license) = &bundle.license else {
        return Ok(LicenseDecision::IssueNew);
    };

    // 2. Revoked → must issue new
    if license.revoked {
        return Ok(LicenseDecision::IssueNew);
    }

    // 3. Edition mismatch → issue new
    if license.edition_id != bundle.edition.id {
        return Ok(LicenseDecision::IssueNew);
    }

    // 4. Product mismatch → issue new
    if bundle.product.id != bundle.edition.product_id {
        return Ok(LicenseDecision::IssueNew);
    }

    // 5. Expired → issue new
    if let Some(exp) = license.expires {
        let today = chrono::Local::now().date_naive();
        if exp < today {
            return Ok(LicenseDecision::IssueNew);
        }
    }

    // 6. Valid major mismatch → issue new
    let product_major = match bundle.product.version.as_deref() {
        Some(v) => parse_major(v),
        None => None,
    };

    let validity = &bundle.validity;
    let valid_major = validity.as_ref().and_then(|v| v.major);

    let product_major = bundle.product.version
        .as_deref()
        .and_then(parse_major);

    let valid_major = validity.as_ref().and_then(|v| v.major);

    if valid_major != product_major {
        return Err(EvaluationError::MajorVersionMismatch { 
            expected: product_major,
            found: valid_major,
        });
    }

    // 7. Otherwise → reuse existing license
    Ok(LicenseDecision::ReuseExisting(license.clone()))
}
