// ============================================================================
// Filename: licensegen/src/product/loader.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-23
// Modified: 2026-03-01
// Description:
// ============================================================================

// System Libraries
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
// Project Libraries
use super::edition::EditionRoot;
use super::product::Product;
use super::request::ApplicationRequest;

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
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(msg) => write!(f, "IO error: {}", msg),
            AppError::Yaml(msg) => write!(f, "YAML error: {}", msg),
            AppError::Invalid(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}
impl std::error::Error for AppError {}
#[derive(Debug)]
pub enum AppError {
    Io(String),
    Yaml(String),
    Invalid(String),
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

                let mut product: Product = serde_yaml::from_str(&contents)
                    .map_err(|e| ProductError::YamlError(format!("YAML parse failed: {}", e)))?;

                // Attach the directory path
                product.dir = path.clone();

                products.push(product);
            }
        }
    }

    Ok(products)
}

pub fn load_all_editions(
    product_path: &Path,
) -> Result<HashMap<String, EditionRoot>, ProductError> {
    let editions_dir = product_path.join("editions");
    let mut editions = HashMap::new();

    for entry in fs::read_dir(&editions_dir)
        .map_err(|e| ProductError::ReadError(format!("read_dir failed: {}", e)))?
    {
        let entry = entry.map_err(|e| ProductError::ReadError(format!("entry failed: {}", e)))?;
        let path = entry.path();

        if path.is_dir() {
            let sku = entry.file_name().to_string_lossy().to_string();
            let edition_file = path.join("edition.yml");

            let contents = fs::read_to_string(&edition_file)
                .map_err(|e| ProductError::ReadError(format!("read_to_string failed: {}", e)))?;

            let edition: EditionRoot = serde_yaml::from_str(&contents)
                .map_err(|e| ProductError::YamlError(format!("YAML parse failed: {}", e)))?;

            editions.insert(sku, edition);
        }
    }

    Ok(editions)
}

pub fn load_application(path: &str) -> Result<ApplicationRequest, AppError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| AppError::Io(format!("Failed to read {}: {}", path, e)))?;

    let mut req: ApplicationRequest = serde_yaml::from_str(&contents)
        .map_err(|e| AppError::Yaml(format!("Invalid application.yml: {}", e)))?;

    // Required field checks
    if req.request.product.is_empty() {
        return Err(AppError::Invalid("Missing request.product".into()));
    }
    if req.contact.name.first.is_empty() || req.contact.name.last.is_empty() {
        return Err(AppError::Invalid("Missing first/last name".into()));
    }
    if req.request.edition.is_empty() {
        return Err(AppError::Invalid("Missing edition".into()));
    }
    if req.request.name.is_empty() {
        return Err(AppError::Invalid("Missing app name".into()));
    }

    // Attach raw YAML for DB sync
    req.raw_yaml = contents;

    Ok(req)
}
