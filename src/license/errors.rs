// ============================================================================
// Filename: licensegen/src/license/evaluator_error.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-03
// Modified: 2026-03-11
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

    #[error("signature encryption mismatch detected")]
    Crypto,

    #[error("product version is missing or invalid")]
    InvalidProductVersion,

    #[error("license is not active")]
    LicenseInactive,

    #[error("edition is not compatible with product")]
    EditionMismatch,
}
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("failed to read private key: {0}")]
    ReadError(String),

    #[error("invalid private key format: {0}")]
    ParseError(String),
}
