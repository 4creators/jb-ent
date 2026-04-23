// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use jb_registry::crypto::{
    get_keypair, save_keypair, export_keypair, import_keypair, 
    sign_payload, verify_signature, MlDsaProvider, CryptoProvider
};
use jb_registry::schema::{GlobalRegistry, JwsEnvelope};
use tempfile::tempdir;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};

#[cfg(windows)]
use jb_registry::win_storage::{win_save_key_ncrypt, win_get_key_ncrypt, win_delete_key_ncrypt};

#[test]
fn test_generate_keypair() {
    let (public_key_b64, private_key_b64) = MlDsaProvider::generate_keypair().expect("Failed to generate keypair");
    assert!(!public_key_b64.is_empty());
    assert!(!private_key_b64.is_empty());
}

#[test]
fn test_save_and_get_keypair() {
    let dir = tempdir().unwrap();
    let env_id = "test_env_save_get";
    let fallback_dir = dir.path().join("keys");

    // 1. Generate new keypair
    let (expected_pub, expected_priv) = MlDsaProvider::generate_keypair().unwrap();

    // Check if keyring is available in this environment
    #[cfg(windows)]
    let keyring_available = true; // win_cred is always available on Windows
    #[cfg(not(windows))]
    let keyring_available = Entry::new(&format!("jb-registry-private-{}", env_id), "ml-dsa-key").is_ok();

    if keyring_available {
        // Ensure clean state
        #[cfg(windows)]
        let _ = jb_registry::win_storage::win_delete_key_ncrypt(&format!("jb-registry-env-{}", env_id));
        #[cfg(not(windows))]
        if let Ok(entry) = Entry::new(&format!("jb-registry-private-{}", env_id), "ml-dsa-key") {
            let _ = entry.delete_credential();
        }

        // 2. Save it. Should go to keyring.
        save_keypair::<MlDsaProvider>(env_id, &fallback_dir, &expected_priv).expect("Failed to save keypair");

        // If the keyring is available, the fallback directory MUST NOT exist.
        assert!(!fallback_dir.exists(), "File fallback used even though keyring is available!");

        // 3. Get it back via API
        let (retrieved_pub, retrieved_priv) = get_keypair::<MlDsaProvider>(env_id, &fallback_dir).expect("Failed get_keypair");

        assert_eq!(retrieved_priv, expected_priv);
        assert_eq!(retrieved_pub, expected_pub);

        // Cleanup
        #[cfg(windows)]
        let _ = jb_registry::win_storage::win_delete_key_ncrypt(&format!("jb-registry-env-{}", env_id));
        #[cfg(not(windows))]
        if let Ok(entry) = Entry::new(&format!("jb-registry-private-{}", env_id), "ml-dsa-key") {
            let _ = entry.delete_credential();
        }
    } else {
        // Testing file fallback path
        save_keypair::<MlDsaProvider>(env_id, &fallback_dir, &expected_priv).expect("Failed save to file fallback");
        assert!(fallback_dir.exists(), "Fallback directory not created!");
        
        let (retrieved_pub, retrieved_priv) = get_keypair::<MlDsaProvider>(env_id, &fallback_dir).expect("Failed get from file");
        assert_eq!(retrieved_priv, expected_priv);
        assert_eq!(retrieved_pub, expected_pub);
    }
}

#[cfg(windows)]
#[test]
fn test_win_storage_ncrypt_direct_access() {
    let container = "jb-ent-direct-ncrypt-test";
    let (_, secret_b64) = MlDsaProvider::generate_keypair().unwrap();
    
    let _ = win_delete_key_ncrypt(container);
    win_save_key_ncrypt(container, &secret_b64).expect("win_save_key_ncrypt failed");
    let retrieved = win_get_key_ncrypt(container).expect("win_get_key_ncrypt failed");
    assert_eq!(retrieved, secret_b64);
    win_delete_key_ncrypt(container).expect("win_delete_key_ncrypt failed");
}

#[test]
fn test_export_keypair_format_verification() {
    let dir = tempdir().unwrap();
    let env_id = "test_env_export_format";
    let fallback_dir = dir.path().join("keys");

    let (_, expected_priv) = MlDsaProvider::generate_keypair().unwrap();
    save_keypair::<MlDsaProvider>(env_id, &fallback_dir, &expected_priv).unwrap();

    let exported_pem = export_keypair::<MlDsaProvider>(env_id, &fallback_dir).expect("Failed export");
    assert!(exported_pem.starts_with("-----BEGIN ML-DSA-44 SEED-----\n"));
    assert!(exported_pem.ends_with("-----END ML-DSA-44 SEED-----\n"));
    
    let inner_b64 = exported_pem
        .replace("-----BEGIN ML-DSA-44 SEED-----\n", "")
        .replace("\n-----END ML-DSA-44 SEED-----\n", "");
    assert_eq!(inner_b64, expected_priv);
}

#[test]
fn test_import_keypair_cross_environment() {
    let dir = tempdir().unwrap();
    let env_id_target = "test_env_import_cross";
    let fallback_dir = dir.path().join("keys");

    let (_, raw_priv) = MlDsaProvider::generate_keypair().unwrap();
    let simulated_wire_pem = format!("-----BEGIN ML-DSA-44 SEED-----\n{}\n-----END ML-DSA-44 SEED-----\n", raw_priv);

    import_keypair::<MlDsaProvider>(env_id_target, &fallback_dir, &simulated_wire_pem).expect("Failed import");
    let (_, imported_priv) = get_keypair::<MlDsaProvider>(env_id_target, &fallback_dir).expect("Failed retrieve imported");
    assert_eq!(imported_priv, raw_priv);
}

#[test]
fn test_sign_compress_transmit_decompress_verify() {
    let (public_key_b64, private_key_b64) = MlDsaProvider::generate_keypair().unwrap();
    let registry = GlobalRegistry { local_environment_id: "test_env_network".to_string(), ..Default::default() };
    
    let signed_payload = sign_payload::<MlDsaProvider, _>("test_env_network", &registry, &private_key_b64).expect("Failed sign");
    let serialized_envelope = serde_json::to_string(&signed_payload).unwrap();
    
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(serialized_envelope.as_bytes()).unwrap();
    let compressed_bytes = encoder.finish().unwrap();
    
    let mut decoder = GzDecoder::new(&compressed_bytes[..]);
    let mut decompressed_string = String::new();
    decoder.read_to_string(&mut decompressed_string).unwrap();
    
    let received_envelope: JwsEnvelope = serde_json::from_str(&decompressed_string).unwrap();
    let is_valid = verify_signature::<MlDsaProvider>(&received_envelope, &public_key_b64).expect("Failed verify");
    assert!(is_valid);
}

#[test]
fn test_sign_and_verify_payload() {
    let (public_key_b64, private_key_b64) = MlDsaProvider::generate_keypair().unwrap();
    let registry = GlobalRegistry { local_environment_id: "test_env".to_string(), ..Default::default() };
    let signed_payload = sign_payload::<MlDsaProvider, _>("test_env", &registry, &private_key_b64).expect("Failed sign");
    let is_valid = verify_signature::<MlDsaProvider>(&signed_payload, &public_key_b64).expect("Failed verify");
    assert!(is_valid);
}

#[test]
fn test_verify_invalid_signature_tampered_payload() {
    let (public_key_b64, private_key_b64) = MlDsaProvider::generate_keypair().unwrap();
    let registry = GlobalRegistry::default();
    let mut signed_payload = sign_payload::<MlDsaProvider, _>("test_env", &registry, &private_key_b64).unwrap();
    signed_payload.payload = "tampered_data".to_string();
    let is_valid = verify_signature::<MlDsaProvider>(&signed_payload, &public_key_b64).unwrap();
    assert!(!is_valid);
}

#[test]
fn test_verify_invalid_signature_wrong_key() {
    let (_, private_key_b64) = MlDsaProvider::generate_keypair().unwrap();
    let (wrong_public_key_b64, _) = MlDsaProvider::generate_keypair().unwrap();
    let registry = GlobalRegistry::default();
    let signed_payload = sign_payload::<MlDsaProvider, _>("test_env", &registry, &private_key_b64).unwrap();
    let is_valid = verify_signature::<MlDsaProvider>(&signed_payload, &wrong_public_key_b64).unwrap();
    assert!(!is_valid);
}
