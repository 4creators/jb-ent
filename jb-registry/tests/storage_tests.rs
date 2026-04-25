// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use jb_registry::storage::error::StorageError;

#[test]
fn test_storage_error_key_not_found_formatting() {
    let err = StorageError::KeyNotFound;
    assert_eq!(err.to_string(), "Key not found in storage");
}

#[test]
fn test_storage_error_crypto_error_formatting() {
    let inner_err = std::io::Error::new(std::io::ErrorKind::InvalidData, "corrupted data");
    let err = StorageError::CryptoError(Box::new(inner_err));
    assert_eq!(
        err.to_string(),
        "Cryptographic error during storage access: corrupted data"
    );
}

#[test]
fn test_storage_error_backend_error_formatting() {
    let inner_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err = StorageError::BackendError(Box::new(inner_err));
    assert_eq!(err.to_string(), "Storage backend failure: access denied");
}

#[test]
fn test_get_keypair_propagates_key_not_found() {
    use jb_registry::crypto::{OpenSslProvider, get_keypair};
    use jb_registry::error::RegistryError;
    use jb_registry::storage::file_provider::FileStorageProvider;
    use tempfile::tempdir;

    let alg_name = "ML-DSA-44";
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let crypto_backend = OpenSslProvider::new(alg_name);

    let storage = FileStorageProvider::new(dir_path, crypto_backend);

    let enc_key = b"0123456789abcdef0123456789abcdef";

    // The key "missing-env" does not exist
    let result = get_keypair("missing-env", &storage, enc_key);

    match result {
        Err(RegistryError::KeyNotFound) => {
            // This is the expected, structured error
        }
        Err(e) => {
            panic!("Expected RegistryError::KeyNotFound, got {:?}", e);
        }
        Ok(_) => {
            panic!("Expected get_keypair to fail on missing key");
        }
    }
}
