// ============================================================================
// Filename: licensegen/src/util/helpers.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-07
// Modified: 2026-03-12
// Description: Helper Functions
// ============================================================================


use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

use crate::vault::types::VaultError;

pub fn check_permissions(path: &Path, required: u32) -> Result<(), String> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("Failed to stat {:?}: {}", path, e))?;

    let mode = meta.permissions().mode() & 0o777;

    if mode != required {
        return Err(format!(
            "{:?} must have permissions {:o}, found {:o}",
            path, required, mode
        ));
    }

    Ok(())
}

pub fn ensure_permissions(path: &Path, required: u32) -> Result<(), VaultError> {
    let meta = std::fs::metadata(path)?;
    let mode = meta.permissions().mode() & 0o777;

    if mode != required {
        // Attempt to fix
        let mut perms = meta.permissions();
        perms.set_mode(required);
        std::fs::set_permissions(path, perms);

        // Re-check
        let new_mode = std::fs::metadata(path)?.permissions().mode() & 0o777;
        if new_mode != required {
            return Err(VaultError::PermissionError(format!(
                "Failed to set permissions on {:?}: expected {:o}, got {:o}",
                path, required, new_mode
            )));
        }
    }

    Ok(())
}

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
pub fn resolve_path(base: &Path, raw: &str) -> PathBuf {
    // Expand ~
    let expanded = shellexpand::tilde(raw).to_string();
    let p = PathBuf::from(expanded);

    if p.is_absolute() {
        p
    } else {
        base.join(p)
    }
}
