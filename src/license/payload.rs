// ============================================================================
// Filename: licensegen/src/payload.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Description: Canonical payload builder for licensegen.
// ============================================================================

use chrono::NaiveDate;
use serde_json::Value;

use crate::db::{
    DbAddress, DbApplication, DbCustomer, DbEdition, DbProduct, DbZipcode,
};
use crate::license::types::{
    AddressInfo, ApplicationInfo, CustomerInfo, EditionInfo, LicensePayload,
    ProductInfo, ValidityInfo,
};

// ----------------------------------------------------------------------------
// Public entry point
// ----------------------------------------------------------------------------

pub fn build_payload(
    app: &DbApplication,
    product: &DbProduct,
    edition: &DbEdition,
    customer: &DbCustomer,
    address: &DbAddress,
    zipcode: &DbZipcode,
    validity: ValidityInfo,
) -> anyhow::Result<LicensePayload> {
    let address_info = build_address_info(address, zipcode)?;
    let customer_info = build_customer_info(customer, address_info);
    let edition_info = build_edition_info(edition)?;
    let product_info = build_product_info(product)?;
    let application_info = build_application_info(app);

    Ok(LicensePayload {
        product: product_info,
        edition: edition_info,
        customer: customer_info,
        application: application_info,
        validity,
    })
}

// ----------------------------------------------------------------------------
// Address builder
// ----------------------------------------------------------------------------

fn build_address_info(addr: &DbAddress, zip: &DbZipcode) -> anyhow::Result<AddressInfo> {
    // line1: PO Box or street
    let line1 = match (&addr.maildrop, &addr.street) {
        (Some(po), _) => format!("PO Box {}", po),
        (None, Some(street)) => street.clone(),
        _ => anyhow::bail!("Address missing both maildrop and street"),
    };

    // line2: suite
    let line2 = addr.suite.clone();

    // postal: ZIP or ZIP+4
    let postal = match addr.zip4 {
        Some(z4) => format!("{:05}-{:04}", addr.zip, z4),
        None => format!("{:05}", addr.zip),
    };

    // city/state: prefer customer-provided, fall back to zipcodes
    let city = addr.city.clone().unwrap_or_else(|| zip.city.clone());
    let state = addr.state.clone().unwrap_or_else(|| zip.state.clone());

    Ok(AddressInfo {
        line1,
        line2,
        city,
        state,
        postal,
        country: addr.country.clone(),
    })
}

// ----------------------------------------------------------------------------
// Customer builder
// ----------------------------------------------------------------------------

fn build_customer_info(cust: &DbCustomer, address: AddressInfo) -> CustomerInfo {
    CustomerInfo {
        name: format!("{} {}", cust.first, cust.last),
        email: cust.email.clone(),
        address,
    }
}

// ----------------------------------------------------------------------------
// Edition builder
// ----------------------------------------------------------------------------

fn build_edition_info(ed: &DbEdition) -> anyhow::Result<EditionInfo> {
    Ok(EditionInfo {
        code: ed.edition_code.clone(),
        name: ed.name.clone(),
        features: serde_json::from_str(&ed.metadata)?,
    })
}

// ----------------------------------------------------------------------------
// Product builder
// ----------------------------------------------------------------------------

fn build_product_info(p: &DbProduct) -> anyhow::Result<ProductInfo> {
    Ok(ProductInfo {
        name: p.name.clone(),
        code: p.code.clone(),
        version: p.version.clone().unwrap_or_else(|| "0".to_string()),
        payload_schema: serde_json::from_str(&p.payload_schema)?,
        features: serde_json::from_str(&p.features)?,
        editions: match &p.editions {
            Some(json) => serde_json::from_str(json)?,                 // Value
            None => serde_json::Value::Array(Vec::new()),              // Value
        }
    })
}

// ----------------------------------------------------------------------------
// Application builder
// ----------------------------------------------------------------------------

fn build_application_info(app: &DbApplication) -> ApplicationInfo {
    ApplicationInfo {
        received: app.received,
        acquired: app.acquired,
    }
}