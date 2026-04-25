// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use jb_registry::crypto::{CryptoProvider, OpenSslProvider};

#[test]
fn test_openssl_pqc_signature_validates_correct_payload() {
    let provider = OpenSslProvider::new("ML-DSA-44");

    let (pub_b64, priv_b64) = provider
        .generate_keypair()
        .expect("Failed to generate keypair");

    let payload = b"critical project metadata payload";
    let signature = provider.sign(payload, &priv_b64).expect("Failed to sign");

    let is_valid = provider
        .verify(payload, &signature, &pub_b64)
        .expect("Failed to verify");

    assert!(is_valid, "Signature should be valid");
}

#[test]
fn test_openssl_pqc_signature_rejects_tampered_payload() {
    let provider = OpenSslProvider::new("ML-DSA-44");

    let (pub_b64, priv_b64) = provider
        .generate_keypair()
        .expect("Failed to generate keypair");

    let payload = b"critical project metadata payload";
    let signature = provider.sign(payload, &priv_b64).expect("Failed to sign");

    let is_valid = provider
        .verify(b"tampered payload", &signature, &pub_b64)
        .expect("Verification failed");

    assert!(!is_valid, "Signature should be invalid for tampered data");
}

#[test]
fn test_openssl_seed_derivation_deterministic_private_key() {
    let provider = OpenSslProvider::new("ML-DSA-44");
    let seed = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; // 32 bytes of '0' in base64

    let (_, priv1) = provider
        .derive_keypair(seed)
        .expect("First derivation failed");

    let (_, priv2) = provider
        .derive_keypair(seed)
        .expect("Second derivation failed");

    assert_eq!(priv1, priv2, "Private keys from same seed must match");
}

#[test]
fn test_openssl_seed_derivation_deterministic_public_key() {
    let provider = OpenSslProvider::new("ML-DSA-44");
    let seed = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; // 32 bytes of '0' in base64

    let (pub1, _) = provider
        .derive_keypair(seed)
        .expect("First derivation failed");

    let (pub2, _) = provider
        .derive_keypair(seed)
        .expect("Second derivation failed");

    assert_eq!(pub1, pub2, "Public keys from same seed must match");
}

#[test]
fn test_openssl_aead_encryption_roundtrip() {
    let provider = OpenSslProvider::new("ML-DSA-44");
    let key = b"0123456789abcdef0123456789abcdef"; // 32 bytes
    let data = b"sensitive cross-platform registry data";

    let encrypted = provider.encrypt(data, key).expect("Encryption failed");

    assert!(encrypted.len() > data.len()); // Should include IV and Tag

    let decrypted = provider
        .decrypt(&encrypted, key)
        .expect("Decryption failed");

    assert_eq!(decrypted, data, "Decrypted data must match original");
}

#[test]
fn test_openssl_provider_respects_algorithm_polymorphism() {
    let provider = OpenSslProvider::new("ML-DSA-65");
    assert_eq!(
        provider.algorithm_name(),
        "ML-DSA-65",
        "Algorithm name must match initialized value"
    );
}

#[test]
fn test_openssl_aead_encryption_various_bad_key_lengths() {
    let provider = OpenSslProvider::new("ML-DSA-44");
    let data = b"sensitive data";

    // Exhaustively try invalid AES-256-GCM lengths (0 through 31, and 33)
    for len in (0..32).chain(std::iter::once(33)) {
        let bad_key = vec![0u8; len];
        let result = provider.encrypt(data, &bad_key);
        assert!(
            result.is_err(),
            "Encryption with key length {} should return Err, not panic",
            len
        );
    }
}

#[test]
fn test_openssl_aead_decryption_various_bad_key_lengths() {
    let provider = OpenSslProvider::new("ML-DSA-44");
    let encrypted_data = vec![0u8; 40]; // Dummy data length > 28

    // Exhaustively try invalid AES-256-GCM lengths (0 through 31, and 33)
    for len in (0..32).chain(std::iter::once(33)) {
        let bad_key = vec![0u8; len];
        let result = provider.decrypt(&encrypted_data, &bad_key);
        assert!(
            result.is_err(),
            "Decryption with key length {} should return Err, not panic",
            len
        );
    }
}
use jb_registry::error::RegistryError;
use jb_registry::storage::StorageProvider;
use jb_registry::storage::error::StorageError;

#[test]
fn test_openssl_seed_derivation_cleans_up_on_error() {
    let provider = OpenSslProvider::new("ML-DSA-44");

    // We use a highly specific pattern to avoid false positives when scanning uninitialized memory.
    // Length is 31 bytes to trigger a failure in ML-DSA-44 seed creation.
    let secret = b"VERY_SECRET_SEED_DO_NOT_LEAK_IT";
    let bad_seed = STANDARD.encode(secret);

    let mut leak_detected = false;

    // Try multiple times to increase the chance that the allocator reuses the exact same block
    for _ in 0..100 {
        let result = provider.derive_keypair(&bad_seed);
        assert!(
            result.is_err(),
            "Derivation should fail for invalid seed length"
        );

        // Immediately allocate to grab the freed memory block from the same thread's cache
        let mut probe = Vec::<u8>::with_capacity(31);
        let ptr = probe.as_mut_ptr();

        unsafe {
            // Read the uninitialized memory to check if our secret was left behind
            let mut matches = 0;
            for i in 0..31 {
                if ptr.add(i).read_volatile() == secret[i] {
                    matches += 1;
                }
            }
            if matches == 31 {
                leak_detected = true;
                break;
            }
        }
    }

    assert!(
        !leak_detected,
        "SECURITY REGRESSION: Sensitive seed material was found un-zeroized in memory after an error!"
    );
}

struct MockStorage;

impl StorageProvider for MockStorage {
    fn load_encrypted(
        &self,
        _env_id: &str,
        _encryption_key: &[u8],
    ) -> std::result::Result<Vec<u8>, StorageError> {
        Err(StorageError::KeyNotFound)
    }

    fn save_encrypted(
        &self,
        _env_id: &str,
        _data: &[u8],
        _encryption_key: &[u8],
    ) -> std::result::Result<(), StorageError> {
        Ok(())
    }
}

#[test]
fn test_get_keypair_propagates_key_not_found_error() {
    let storage = MockStorage;
    let dummy_key = [0u8; 32];

    let result = jb_registry::crypto::get_keypair("test_env", &storage, &dummy_key);

    match result {
        Err(RegistryError::KeyNotFound) => {} // Expected success
        Err(e) => panic!("Expected KeyNotFound error, got: {:?}", e),
        Ok(_) => panic!("Expected error, but got Ok"),
    }
}

#[test]
fn test_openssl_sign_cleans_up_on_error() {
    let provider = OpenSslProvider::new("ML-DSA-44");

    let secret = b"VERY_SECRET_SEED_DO_NOT_LEAK_IT";
    let bad_seed = STANDARD.encode(secret);
    let payload = b"dummy payload";

    let mut leak_detected = false;

    for _ in 0..100 {
        let result = provider.sign(payload, &bad_seed);
        assert!(result.is_err(), "Sign should fail for invalid seed length");

        let mut probe = Vec::<u8>::with_capacity(31);
        let ptr = probe.as_mut_ptr();

        unsafe {
            let mut matches = 0;
            for i in 0..31 {
                if ptr.add(i).read_volatile() == secret[i] {
                    matches += 1;
                }
            }
            if matches == 31 {
                leak_detected = true;
                break;
            }
        }
    }

    assert!(
        !leak_detected,
        "SECURITY REGRESSION: Sensitive seed material was found un-zeroized in memory after a sign error!"
    );
}
