// ============================================================================
// Filename: licensegen/src/vault/ansible.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-02-23
// Modified: 2026-02-23
// Description:
// ============================================================================

// System Libraries
use std::process::Command;
// Project Libraries
use crate::vault::VaultError;

pub fn decrypt_with_ansible(vault_path: &str, password_file: &str) -> Result<String, VaultError> {
    let output = Command::new("ansible-vault")
        .arg("view")
        .arg(vault_path)
        .arg("--vault-password-file")
        .arg(password_file)
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
