// ============================================================================
// Filename: licensegen/src/vault/ansible.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-23
// Modified: 2026-03-10
// Description:
// ============================================================================

// System Libraries
use std::process::Command;
use std::path::Path;
// Project Libraries
use crate::vault::types::VaultError;

pub fn decrypt_with_ansible(vault_path: &Path, password_file: &Path) -> Result<String, VaultError> {
    let vault_str = vault_path.to_string_lossy();
    let password_str = password_file.to_string_lossy();

    let output = Command::new("ansible-vault")
        .arg("view")
        .arg(vault_str.as_ref())
        .arg("--vault-password-file")
        .arg(password_str.as_ref())
        .output()
        .map_err(|e| VaultError::ReadError(format!("Failed to run ansible-vault: {}", e)))?;

    if !output.status.success() {
        return Err(VaultError::ReadError(format!(
            "ansible-vault failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
