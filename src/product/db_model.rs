// ============================================================================
// Filename: licensegen/src/product/db_model.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-19
// Modified: 2026-02-19
// Description:
// ============================================================================

// System Libraries
use crate::product::yaml::ProductYaml;
use serde_json;
// Project Libraries

#[derive(Debug)]
pub struct DbProduct {
    pub name: String,
    pub code: String,
    pub version: String,
    pub payload_schema: String,
    pub features: String,
    pub editions: String,
    pub keypair_path: String,
    pub active: String,
}
impl From<ProductYaml> for DbProduct {
    fn from(y: ProductYaml) -> Self {
        Self {
            name: y.product,
            code: "BS".into(),   // temporary until YAML includes this
            version: "3".into(), // temporary until YAML includes this

            payload_schema: serde_json::to_string(&y.license.payload_fields)
                .expect("payload_schema JSON"),

            features: serde_json::to_string(&y.editions).expect("features JSON"),

            editions: serde_json::to_string(&y.editions).expect("editions JSON"),

            keypair_path: y.signing.keypair,
            active: "Y".into(),
        }
    }
}
impl DbProduct {
    pub fn as_params(&self) -> (&str, &str, &str, &str, &str, &str, &str, &str) {
        (
            &self.name,
            &self.code,
            &self.version,
            &self.payload_schema,
            &self.features,
            &self.editions,
            &self.keypair_path,
            &self.active,
        )
    }
}
