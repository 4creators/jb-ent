// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

pub mod error;
pub mod file_provider;

use crate::storage::error::StorageError;

/// Defines the contract for securely persisting and retrieving environment data.
/// By abstracting storage, the application can easily swap between file-based storage,
/// in-memory testing mocks, or potential future cloud/OS-native keyrings.
pub trait StorageProvider {
    /// Encrypts and saves data for a specific environment identifier.
    ///
    /// # Arguments
    /// * `environment_id` - The logical environment identifier used as the key.
    /// * `data` - The raw data payload to be encrypted and stored.
    /// * `encryption_key` - The symmetric key used to encrypt the data at rest.
    fn save_encrypted(
        &self,
        environment_id: &str,
        data: &[u8],
        encryption_key: &[u8],
    ) -> std::result::Result<(), StorageError>;

    /// Loads and decrypts data for a specific environment identifier.
    ///
    /// # Arguments
    /// * `environment_id` - The logical environment identifier used as the key.
    /// * `encryption_key` - The symmetric key used to decrypt the data.
    fn load_encrypted(
        &self,
        environment_id: &str,
        encryption_key: &[u8],
    ) -> std::result::Result<Vec<u8>, StorageError>;
}
