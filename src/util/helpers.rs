// ============================================================================
// Filename: licensegen/src/util/helpers.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-07
// Modified: 2026-03-10
// Description: Helper Functions
// ============================================================================


use std::path::{Path, PathBuf};

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();

    // Only expand if the path starts with "~/" exactly
    if let Ok(stripped) = p.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }

    // Otherwise return unchanged
    p.to_path_buf()
}
pub fn fill_if_empty(target: &mut String, source: &str) {
    if target.is_empty() {
        *target = source.to_string();
    }
}
pub fn fill_if_empty_opt(target: &mut String, source: &Option<String>) {
    if target.is_empty() {
        if let Some(val) = source {
            *target = val.clone();
        }
    }
}
