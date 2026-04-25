// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

//! This module contains isolated tests for the FFI OpenSSL interactions.

use jb_registry::ffi_openssl::{FfiOpensslError, create_ml_dsa_pkey_from_seed};

// --- Happy Paths (Single Conditions) ---

#[test]
fn test_ffi_ml_dsa_44_creates_private_key() {
    let seed = [0x42; 32];
    let pkey = create_ml_dsa_pkey_from_seed("ML-DSA-44", &seed)
        .expect("Failed ML-DSA-44 private key creation");

    assert!(pkey.raw_private_key().is_ok());
}

#[test]
fn test_ffi_ml_dsa_44_creates_public_key() {
    let seed = [0x42; 32];
    let pkey = create_ml_dsa_pkey_from_seed("ML-DSA-44", &seed)
        .expect("Failed ML-DSA-44 public key creation");

    assert!(pkey.raw_public_key().is_ok());
}

#[test]
fn test_ffi_ml_dsa_65_creates_valid_key() {
    let seed = [0x65; 32];
    let pkey =
        create_ml_dsa_pkey_from_seed("ML-DSA-65", &seed).expect("Failed ML-DSA-65 key creation");

    assert!(pkey.raw_private_key().is_ok());
}

#[test]
fn test_ffi_ml_dsa_87_creates_valid_key() {
    let seed = [0x87; 32];
    let pkey =
        create_ml_dsa_pkey_from_seed("ML-DSA-87", &seed).expect("Failed ML-DSA-87 key creation");

    assert!(pkey.raw_private_key().is_ok());
}

#[test]
fn test_ffi_deterministic_derivation() {
    let seed = [0x77; 32];
    let pkey1 = create_ml_dsa_pkey_from_seed("ML-DSA-44", &seed)
        .expect("Failed first deterministic derivation");

    let pkey2 = create_ml_dsa_pkey_from_seed("ML-DSA-44", &seed)
        .expect("Failed second deterministic derivation");

    assert_eq!(
        pkey1.raw_public_key().expect("Failed to get public key 1"),
        pkey2.raw_public_key().expect("Failed to get public key 2")
    );
}

// --- Error Conditions (Strictly Isolated) ---

#[test]
fn test_error_invalid_seed_length_returns_exact_length() {
    let seed = [0x42; 31];
    let err = create_ml_dsa_pkey_from_seed("ML-DSA-44", &seed)
        .expect_err("Expected seed length error but creation succeeded");

    assert!(matches!(err, FfiOpensslError::InvalidSeedLength(31)));
}

#[test]
fn test_error_invalid_seed_length_too_long() {
    let seed = [0x42; 33];
    let err = create_ml_dsa_pkey_from_seed("ML-DSA-44", &seed)
        .expect_err("Expected seed length error but creation succeeded");

    assert!(matches!(err, FfiOpensslError::InvalidSeedLength(33)));
}

#[test]
fn test_error_empty_seed() {
    let seed: [u8; 0] = [];
    let err = create_ml_dsa_pkey_from_seed("ML-DSA-44", &seed)
        .expect_err("Expected seed length error but creation succeeded");

    assert!(matches!(err, FfiOpensslError::InvalidSeedLength(0)));
}

#[test]
fn test_error_invalid_algorithm_name_null_byte() {
    let seed = [0x42; 32];
    let err = create_ml_dsa_pkey_from_seed("ML-DSA\0-44", &seed)
        .expect_err("Expected null byte algorithm error but creation succeeded");

    assert!(matches!(err, FfiOpensslError::InvalidAlgorithmName));
}

#[test]
fn test_error_ctx_init_failed_unsupported_algorithm() {
    let seed = [0x42; 32];
    // OpenSSL 4.0.0 won't recognize this name, causing EVP_PKEY_CTX_new_from_name to fail
    let err = create_ml_dsa_pkey_from_seed("UNSUPPORTED-ALG", &seed)
        .expect_err("Expected algorithm error but creation succeeded");

    assert!(matches!(err, FfiOpensslError::CtxInitFailed(_)));
}

#[test]
fn test_error_from_data_failed_unsupported_parameters() {
    let seed = [0x42; 32];
    // RSA exists, so CtxInit passes. However, RSA does not accept a raw 32-byte
    // "seed" parameter for deterministic generation via fromdata.
    // This correctly isolates the failure to the EVP_PKEY_fromdata step.
    let err = create_ml_dsa_pkey_from_seed("RSA", &seed)
        .expect_err("Expected parameter generation error but creation succeeded");

    assert!(matches!(err, FfiOpensslError::FromDataFailed(_)));
}
