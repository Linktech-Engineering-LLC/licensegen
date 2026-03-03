// ============================================================================
// Filename: licensegen/src/license/evaluator_error.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-03
// Description: 
// ============================================================================


use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("major version mismatch: expected {expected:?}, found {found:?}")]
    MajorVersionMismatch {
        expected: Option<i32>,
        found: Option<i32>,
    },

    #[error("product version is missing or invalid")]
    InvalidProductVersion,

    #[error("license is not active")]
    LicenseInactive,

    #[error("edition is not compatible with product")]
    EditionMismatch,
}