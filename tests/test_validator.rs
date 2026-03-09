// ============================================================================
// Filename: licensegen/tests/license/test_validator.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-04
// Modified: 2026-03-07
// Description: Tests the license generation from end to end
// ============================================================================
use licensegen::license::types::{
    LicensePayload, ProductInfo, EditionInfo, CustomerInfo, ApplicationInfo, ValidityInfo,
    AddressInfo, ValidityUnit
};
use licensegen::license::crypto::validate_license;
use licensegen::license::signer::sign;
use licensegen::license::types::ValidationOutcome;
use chrono::NaiveDate;
use serde_json::json;
use rsa::{RsaPrivateKey, RsaPublicKey};
use rand::thread_rng;

#[test]
fn test_signer_minimal_payload() {
    // Minimal ProductInfo
    let product = ProductInfo {
        name: "BotScanner".into(),
        code: "BSCAN".into(),
        version: "0.3.0".into(),
        payload_schema: json!({}),
        features: json!({}),
        editions: json!([]),
    };

    // Minimal EditionInfo
    let edition = EditionInfo {
        code: "BSPRO".into(),
        name: "Professional Edition".into(),
        features: json!([]),
    };

    // Minimal CustomerInfo
    let customer = CustomerInfo {
        name: "Test Customer".into(),
        email: "test@example.com".into(),
        address: AddressInfo {
            line1: "123 Main St".into(),
            line2: None,
            city: "St John".into(),
            state: "KS".into(),
            postal: "67576".into(),
            country: "USA".into(),
        },
    };

    // Minimal ApplicationInfo
    let application = ApplicationInfo {
        received: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
        acquired: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
    };

    // Minimal ValidityInfo
    let validity = ValidityInfo {
        issued: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
        expires: None,
        valid_major: Some(3),
        validity_value: Some(1),
        validity_unit: Some(ValidityUnit::Years),
    };

    // Full payload
    let payload = LicensePayload {
        product,
        edition,
        customer,
        application,
        validity,
    };

    // Load or generate a test key
    let private_key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048)
        .expect("failed to generate test key");

    // Sign
    let signature = sign(&payload, &private_key)
        .expect("signing failed");

    assert!(!signature.payload_json.is_empty());
    assert!(!signature.signature.is_empty());
}

#[test]
fn test_canonical_json_serialization() {
    let payload = LicensePayload {
        product: ProductInfo {
            name: "BotScanner".into(),
            code: "BSCAN".into(),
            version: "0.3.0".into(),
            payload_schema: json!({}),
            features: json!({}),
            editions: json!([]),
        },
        edition: EditionInfo {
            code: "BSPRO".into(),
            name: "Professional Edition".into(),
            features: json!([]),
        },
        customer: CustomerInfo {
            name: "Test Customer".into(),
            email: "test@example.com".into(),
            address: AddressInfo {
                line1: "123 Main St".into(),
                line2: None,
                city: "St John".into(),
                state: "KS".into(),
                postal: "67576".into(),
                country: "USA".into(),
            },
        },
        application: ApplicationInfo {
            received: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
            acquired: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
        },
        validity: ValidityInfo {
            issued: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
            expires: None,
            valid_major: Some(3),
            validity_value: Some(1),
            validity_unit: Some(ValidityUnit::Years),
        },
    };

    let json = serde_json::to_string(&payload).expect("serialization failed");

    // Basic invariants
    assert!(json.contains("\"product\""));
    assert!(json.contains("\"edition\""));
    assert!(json.contains("\"customer\""));
    assert!(json.contains("\"application\""));
    assert!(json.contains("\"validity\""));

    // Deterministic ordering check: product must appear before edition
    let product_pos = json.find("\"product\"").unwrap();
    let edition_pos = json.find("\"edition\"").unwrap();
    assert!(product_pos < edition_pos, "product must serialize before edition");

    // No pretty-print whitespace
    assert!(!json.contains('\n'));
    assert!(!json.contains("  "));

    // JSON must be stable across runs
    let json_again = serde_json::to_string(&payload).unwrap();
    assert_eq!(json, json_again, "canonical JSON must be stable");
}
#[test]
fn test_signature_validation() {
    // Generate a test keypair
    let mut rng = thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("key generation failed");
    let public_key = RsaPublicKey::from(&private_key);

    // Build a valid payload
    let payload = LicensePayload {
        product: ProductInfo {
            name: "BotScanner".into(),
            code: "BSCAN".into(),
            version: "0.3.0".into(),
            payload_schema: json!({}),
            features: json!({}),
            editions: json!([]),
        },
        edition: EditionInfo {
            code: "BSPRO".into(),
            name: "Professional Edition".into(),
            features: json!([]),
        },
        customer: CustomerInfo {
            name: "Test Customer".into(),
            email: "test@example.com".into(),
            address: AddressInfo {
                line1: "123 Main St".into(),
                line2: None,
                city: "St John".into(),
                state: "KS".into(),
                postal: "67576".into(),
                country: "USA".into(),
            },
        },
        application: ApplicationInfo {
            received: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
            acquired: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
        },
        validity: ValidityInfo {
            issued: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
            expires: None,
            valid_major: Some(3),
            validity_value: Some(1),
            validity_unit: Some(ValidityUnit::Years),
        },
    };
    // Sign the payload
    let signed = sign(&payload, &private_key)
        .expect("signing failed");
println!("SIGNED canonical payload: {}", signed.payload_json);
    // Wrap into the JSON structure expected by validate_license
    let mut license_json = String::new();
    license_json.push_str("{\"payload\":");
    license_json.push_str(&signed.payload_json);
    license_json.push_str(",\"signature\":\"");
    license_json.push_str(&signed.signature);
    license_json.push_str("\"}");

    // Positive test: signature must validate
    let ok = validate_license(&license_json, &public_key);
    assert!(ok.is_ok(), "signature should validate");

    // Negative test: corrupt the signature
    let mut bad_sig = signed.signature.clone().into_bytes();
    bad_sig[0] ^= 0xFF;

    let bad_license_json = json!({
        "payload": serde_json::from_str::<serde_json::Value>(&signed.payload_json).unwrap(),
        "signature": String::from_utf8_lossy(&bad_sig),
    })
    .to_string();

    let bad = validate_license(&bad_license_json, &public_key);
    assert!(bad.is_err(), "corrupted signature must fail validation");
}
