// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use super::StorageProvider;
use crate::crypto::CryptoProvider;
use crate::storage::StorageError;
use std::fs;
use std::io::ErrorKind as IoErrorKind;
use std::path::{Path, PathBuf};

/// A storage provider that persists encrypted data as binary files on the local filesystem.
pub struct FileStorageProvider<C: CryptoProvider> {
    base_dir: PathBuf,
    crypto: C,
}

impl<C: CryptoProvider> FileStorageProvider<C> {
    /// Constructs a new file storage provider.
    ///
    /// # Arguments
    /// * `base_dir` - The root directory where environment files will be stored.
    /// * `crypto` - The cryptographic provider used to perform the AES-256-GCM encryption.
    pub fn new<P: AsRef<Path>>(base_dir: P, crypto: C) -> Self {
        FileStorageProvider {
            base_dir: base_dir.as_ref().to_path_buf(),
            crypto,
        }
    }

    /// Resolves the absolute path for a specific environment file.
    fn get_file_path(&self, id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.enc", id))
    }
}

impl<C: CryptoProvider> StorageProvider for FileStorageProvider<C> {
    fn save_encrypted(
        &self,
        environment_id: &str,
        data: &[u8],
        encryption_key: &[u8],
    ) -> std::result::Result<(), StorageError> {
        let encrypted_payload = self.crypto.encrypt(data, encryption_key).map_err(|e| {
            // TODO: Implement advanced structured logging for storage I/O failures
            StorageError::CryptoError(Box::new(e))
        })?;

        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir).map_err(|e| {
                // TODO: Implement advanced structured logging for storage I/O failures
                StorageError::BackendError(Box::new(e))
            })?;
        }

        let file_path = self.get_file_path(environment_id);

        fs::write(file_path, encrypted_payload).map_err(|e| {
            // TODO: Implement advanced structured logging for storage I/O failures
            StorageError::BackendError(Box::new(e))
        })
    }

    fn load_encrypted(
        &self,
        environment_id: &str,
        encryption_key: &[u8],
    ) -> std::result::Result<Vec<u8>, StorageError> {
        let file_path = self.get_file_path(environment_id);

        let encrypted_payload = fs::read(file_path).map_err(|e| {
            if e.kind() == IoErrorKind::NotFound {
                StorageError::KeyNotFound
            } else {
                // TODO: Implement advanced structured logging for storage I/O failures
                StorageError::BackendError(Box::new(e))
            }
        })?;

        self.crypto
            .decrypt(&encrypted_payload, encryption_key)
            .map_err(|e| {
                // TODO: Implement advanced structured logging for storage I/O failures
                StorageError::CryptoError(Box::new(e))
            })
    }
}
