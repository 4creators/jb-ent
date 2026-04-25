// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use thiserror::Error;

/// Universal errors that can be returned by any `StorageProvider` implementation.
#[derive(Debug, Error)]
pub enum StorageError {
    /// The requested cryptographic key or environment ID was not found in the storage backend.
    #[error("Key not found in storage")]
    KeyNotFound,

    /// A cryptographic operation (e.g., decryption/encryption of the stored payload) failed.
    #[error("Cryptographic error during storage access: {0}")]
    CryptoError(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// An implementation-specific storage backend error occurred (e.g., file I/O, network failure).
    #[error("Storage backend failure: {0}")]
    BackendError(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}
