// ============================================================================
// Filename: licensegen/src/product/loader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-23
// Modified: 2026-02-25
// Description:
// ============================================================================

// System Libraries
use std::fmt;
use std::fs;
use std::path::Path;
// Project Libraries

use super::product::Product;

impl fmt::Display for ProductError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ProductError {}

#[derive(Debug)]
pub enum ProductError {
    ReadError(String),
    YamlError(String),
}

pub fn load_all_products(products_dir: &str) -> Result<Vec<Product>, ProductError> {
    let mut products = Vec::new();
    let root = Path::new(products_dir);

    // Ensure the directory exists
    if !root.exists() {
        return Err(ProductError::ReadError(format!(
            "Products directory does not exist: {}",
            products_dir
        )));
    }

    // Walk the first-level entries
    for entry in fs::read_dir(root)
        .map_err(|e| ProductError::ReadError(format!("read_dir failed: {}", e)))?
    {
        let entry = entry.map_err(|e| ProductError::ReadError(format!("entry failed: {}", e)))?;
        let path = entry.path();

        // Only process subdirectories
        if path.is_dir() {
            let product_file = path.join("product.yml");

            if product_file.exists() {
                let contents = fs::read_to_string(&product_file).map_err(|e| {
                    ProductError::ReadError(format!("read_to_string failed: {}", e))
                })?;

                let product: Product = serde_yaml::from_str(&contents)
                    .map_err(|e| ProductError::YamlError(format!("YAML parse failed: {}", e)))?;

                products.push(product);
            }
        }
    }

    Ok(products)
}
