// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Failed to serialize or deserialize JSON payload: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to access or mutate storage: {0}")]
    StorageError(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Invalid key format or length: {0}")]
    InvalidKeyFormat(String),

    #[error("Invalid signature format or length: {0}")]
    InvalidSignatureFormat(String),

    #[error("Cryptographic key not found in storage")]
    KeyNotFound,

    #[error("Signature verification failed: payload may have been tampered with")]
    SignatureVerificationFailed,

    #[error("Cryptographic error occurred: {0}")]
    CryptoError(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type Result<T> = std::result::Result<T, RegistryError>;
