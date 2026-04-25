// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use jb_registry::crypto::OpenSslProvider;
use jb_registry::crypto::import_keypair;
use jb_registry::error::RegistryError;
use jb_registry::storage::file_provider::FileStorageProvider;
use tempfile::tempdir;

#[test]
fn test_import_keypair_rejects_malformed_pem_base64() {
    let alg_name = "ML-DSA-44";
    let provider = OpenSslProvider::new(alg_name);

    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let crypto_backend = OpenSslProvider::new(alg_name);

    let storage = FileStorageProvider::new(dir_path, crypto_backend);

    let enc_key = b"0123456789abcdef0123456789abcdef";
    let malformed_pem =
        "-----BEGIN ML DSA 44 SEED-----\n!!!INVALID BASE64!!!\n-----END ML DSA 44 SEED-----";

    let result = import_keypair("test-env", &storage, enc_key, malformed_pem, &provider);

    match result {
        Err(RegistryError::InvalidKeyFormat(msg)) => {
            let msg_lower = msg.to_lowercase();
            let has_base64 = msg_lower.contains("base64");
            let has_decode = msg_lower.contains("decode");

            assert!(
                has_base64 || has_decode,
                "Expected decode error message, got: {}",
                msg
            );
        }
        Err(e) => {
            panic!("Expected InvalidKeyFormat, got {:?}", e);
        }
        Ok(_) => {
            panic!("Expected import_keypair to fail on malformed base64");
        }
    }
}

#[test]
fn test_import_keypair_rejects_wrong_pem_label() {
    let alg_name = "ML-DSA-44";
    let provider = OpenSslProvider::new(alg_name);

    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let crypto_backend = OpenSslProvider::new(alg_name);

    let storage = FileStorageProvider::new(dir_path, crypto_backend);

    let enc_key = b"0123456789abcdef0123456789abcdef";
    let wrong_label_pem = "-----BEGIN WRONG SEED-----\nQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQT0=\n-----END WRONG SEED-----";

    let result = import_keypair("test-env", &storage, enc_key, wrong_label_pem, &provider);

    match result {
        Err(RegistryError::InvalidKeyFormat(msg)) => {
            assert!(
                msg.contains("Expected label"),
                "Expected specific wrong label error message, got: {}",
                msg
            );
        }
        Err(e) => {
            panic!("Expected InvalidKeyFormat for wrong label, got {:?}", e);
        }
        Ok(_) => {
            panic!("Expected import_keypair to fail on wrong label");
        }
    }
}

#[test]
fn test_import_keypair_rejects_missing_end_tag() {
    let alg_name = "ML-DSA-44";
    let provider = OpenSslProvider::new(alg_name);

    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let crypto_backend = OpenSslProvider::new(alg_name);

    let storage = FileStorageProvider::new(dir_path, crypto_backend);

    let enc_key = b"0123456789abcdef0123456789abcdef";
    let no_end_pem = "-----BEGIN ML DSA 44 SEED-----\nQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQT0=";

    let result = import_keypair("test-env", &storage, enc_key, no_end_pem, &provider);

    match result {
        Err(RegistryError::InvalidKeyFormat(msg)) => {
            assert!(
                !msg.is_empty(),
                "Expected an error message from pem-rfc7468"
            );
        }
        Err(e) => {
            panic!("Expected InvalidKeyFormat for missing end tag, got {:?}", e);
        }
        Ok(_) => {
            panic!("Expected import_keypair to fail on missing end tag");
        }
    }
}

#[test]
fn test_import_keypair_rejects_invalid_raw_base64() {
    let alg_name = "ML-DSA-44";
    let provider = OpenSslProvider::new(alg_name);

    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let crypto_backend = OpenSslProvider::new(alg_name);

    let storage = FileStorageProvider::new(dir_path, crypto_backend);

    let enc_key = b"0123456789abcdef0123456789abcdef";
    let raw_bad_base64 = "THIS_IS_NOT_VALID_BASE64_###";

    let result = import_keypair("test-env", &storage, enc_key, raw_bad_base64, &provider);

    match result {
        Err(RegistryError::InvalidKeyFormat(msg)) => {
            let msg_lower = msg.to_lowercase();
            assert!(
                msg_lower.contains("decode") || msg_lower.contains("invalid symbol"),
                "Expected base64 decode error from derive_keypair, got: {}",
                msg
            );
        }
        Err(e) => {
            panic!("Expected InvalidKeyFormat for bad raw base64, got {:?}", e);
        }
        Ok(_) => {
            panic!("Expected import_keypair to fail on invalid raw base64");
        }
    }
}

#[test]
fn test_import_keypair_rejects_wrong_seed_length() {
    let alg_name = "ML-DSA-44";
    let provider = OpenSslProvider::new(alg_name);

    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let crypto_backend = OpenSslProvider::new(alg_name);

    let storage = FileStorageProvider::new(dir_path, crypto_backend);

    let enc_key = b"0123456789abcdef0123456789abcdef";
    let short_seed_pem =
        "-----BEGIN ML DSA 44 SEED-----\nQUFBQUFBQUFBQUFBQUFBQUFB\n-----END ML DSA 44 SEED-----";

    let result = import_keypair("test-env", &storage, enc_key, short_seed_pem, &provider);

    match result {
        Err(RegistryError::CryptoError(inner)) => {
            let msg = inner.to_string();
            assert!(
                msg.contains("Invalid seed length"),
                "Expected FFI InvalidSeedLength error, got: {}",
                msg
            );
        }
        Err(e) => {
            panic!("Expected CryptoError for wrong seed length, got {:?}", e);
        }
        Ok(_) => {
            panic!("Expected import_keypair to fail on wrong seed length");
        }
    }
}
