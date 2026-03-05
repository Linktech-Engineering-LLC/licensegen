// ============================================================================
// Filename: licensegen/src/license/evaluator_error.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-05
// Description: 
// ============================================================================


use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("major version mismatch: expected {expected:?}, found {found:?}")]
    MajorVersionMismatch {
        expected: Option<u8>,
        found: Option<u8>,
    },

    #[error("product version is missing or invalid")]
    InvalidProductVersion,

    #[error("license is not active")]
    LicenseInactive,

    #[error("edition is not compatible with product")]
    EditionMismatch,
}
