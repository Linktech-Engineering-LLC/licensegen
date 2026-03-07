// ============================================================================
// Filename: licensegen/src/util/helpers.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-07
// Modified: 2026-03-07
// Description: Helper Functions
// ============================================================================


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