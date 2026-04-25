// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

#![allow(clippy::collapsible_if)]

use crate::error::{RegistryError, Result};
use crate::ffi_openssl::{OBJ_txt2nid, create_ml_dsa_pkey_from_seed};
use crate::schema::JwsEnvelope;
use crate::storage::{StorageProvider, error::StorageError};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use chrono::Utc;
use openssl::pkey::{PKey, Private};
use openssl::sign::{Signer, Verifier};
use openssl::symm::{Cipher, decrypt_aead, encrypt_aead};
use rand::{Rng, rng};
use zeroize::Zeroizing;

#[cfg(not(test))]
fn fill_random_bytes(dest: &mut [u8]) {
    rng().fill_bytes(dest);
}

#[cfg(test)]
thread_local! {
    pub static MOCK_RNG_NEXT_OUTPUT: std::cell::RefCell<Option<Vec<u8>>> = std::cell::RefCell::new(None);
    pub static LAST_SEED_PTR: std::cell::RefCell<*const u8> = std::cell::RefCell::new(std::ptr::null());
}

#[cfg(test)]
pub fn fill_random_bytes(dest: &mut [u8]) {
    LAST_SEED_PTR.with(|p| *p.borrow_mut() = dest.as_ptr());
    MOCK_RNG_NEXT_OUTPUT.with(|output| {
        if let Some(ref mock_data) = *output.borrow() {
            let len = std::cmp::min(dest.len(), mock_data.len());
            dest[..len].copy_from_slice(&mock_data[..len]);
        } else {
            rand::rng().fill_bytes(dest);
        }
    });
}

/// Abstraction for cryptographic operations, allowing different algorithms to be plugged in.
pub trait CryptoProvider {
    /// Expected size of the raw private seed in bytes.
    const SEED_SIZE: usize = 32;

    /// Generates a new keypair and returns (public_key_b64, private_key_b64).
    /// Both keys are returned as Base64 encoded standard envelopes.
    fn generate_keypair(&self) -> Result<(String, String)>;

    /// Derives the keypair from an encoded private key seed.
    fn derive_keypair(&self, private_seed_b64: &str) -> Result<(String, String)>;

    /// Signs a string payload and returns the raw signature bytes.
    fn sign(&self, payload: &[u8], private_seed_b64: &str) -> Result<Vec<u8>>;

    /// Verifies a raw signature against a payload and public key.
    fn verify(&self, payload: &[u8], signature: &[u8], public_key_b64: &str) -> Result<bool>;

    /// Encrypts data using AES-256-GCM.
    fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;

    /// Decrypts data using AES-256-GCM.
    fn decrypt(&self, encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>>;

    /// Returns the algorithm identifier for JWS 'alg' header.
    fn algorithm_name(&self) -> &'static str;
}

/// Unified cryptographic provider powered by OpenSSL 4.0.0.
///
/// This provider handles both symmetric encryption (AES-256-GCM) for local
/// storage abstraction and Post-Quantum signatures (ML-DSA) for payload integrity.
/// By delegating to a custom OpenSSL build, it avoids fragmentation across
/// Windows (CNG/DPAPI) and Linux native APIs.
pub struct OpenSslProvider {
    alg_name: &'static str,
}

impl OpenSslProvider {
    /// Constructs a new provider for a specific signature algorithm.
    ///
    /// # Arguments
    /// * `alg_name` - The exact OpenSSL string identifier for the algorithm
    ///                (e.g., "ML-DSA-44", "ML-DSA-65").
    pub fn new(alg_name: &'static str) -> Self {
        OpenSslProvider { alg_name }
    }

    /// Internal helper to create an ML-DSA PKey from a raw seed using OpenSSL 4.0.0 generic parameters.
    fn pkey_from_seed(&self, seed_bytes: &[u8]) -> Result<PKey<Private>> {
        create_ml_dsa_pkey_from_seed(self.alg_name, seed_bytes).map_err(|e| {
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::CryptoError(Box::new(e))
        })
    }
}

impl CryptoProvider for OpenSslProvider {
    fn generate_keypair(&self) -> Result<(String, String)> {
        let mut seed = Zeroizing::new([0u8; 32]);
        fill_random_bytes(seed.as_mut());

        let pkey = self.pkey_from_seed(seed.as_ref())?;

        let pub_bytes = pkey
            .raw_public_key()
            .map_err(|e| RegistryError::CryptoError(Box::new(e)))?;

        let priv_b64 = STANDARD.encode(seed.as_ref());
        let pub_b64 = STANDARD.encode(pub_bytes);

        Ok((pub_b64, priv_b64))
    }

    fn derive_keypair(&self, private_seed_b64: &str) -> Result<(String, String)> {
        let seed_bytes = Zeroizing::new(STANDARD.decode(private_seed_b64).map_err(|e| {
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::InvalidKeyFormat(e.to_string())
        })?);

        let pkey = self.pkey_from_seed(seed_bytes.as_ref())?;

        let pub_bytes = pkey
            .raw_public_key()
            .map_err(|e| RegistryError::CryptoError(Box::new(e)))?;

        let pub_b64 = STANDARD.encode(pub_bytes);
        Ok((pub_b64, private_seed_b64.to_string()))
    }

    fn sign(&self, payload: &[u8], private_seed_b64: &str) -> Result<Vec<u8>> {
        let seed_bytes = Zeroizing::new(
            STANDARD
                .decode(private_seed_b64)
                .map_err(|e| RegistryError::InvalidKeyFormat(e.to_string()))?,
        );

        let pkey = self.pkey_from_seed(seed_bytes.as_ref())?;

        let mut signer = Signer::new_without_digest(&pkey).map_err(|e| {
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::CryptoError(Box::new(e))
        })?;

        signer.sign_oneshot_to_vec(payload).map_err(|e| {
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::CryptoError(Box::new(e))
        })
    }

    fn verify(&self, payload: &[u8], signature: &[u8], public_key_b64: &str) -> Result<bool> {
        let pub_bytes = STANDARD.decode(public_key_b64).map_err(|e|
                // TODO: Implement advanced structured logging for cryptographic failures
                RegistryError::InvalidKeyFormat(e.to_string()))?;

        let alg_cstring = std::ffi::CString::new(self.alg_name).map_err(|e|
                // TODO: Implement advanced structured logging for cryptographic failures
                RegistryError::InvalidKeyFormat(e.to_string()))?;

        let nid = unsafe { OBJ_txt2nid(alg_cstring.as_ptr()) };
        if nid == openssl_sys::NID_undef {
            // TODO: Implement advanced structured logging for cryptographic failures
            return Err(RegistryError::InvalidKeyFormat(format!(
                "Unknown OpenSSL NID for algorithm {}",
                self.alg_name
            )));
        }

        let pkey = PKey::public_key_from_raw_bytes(&pub_bytes, openssl::pkey::Id::from_raw(nid))
            .map_err(|e| {
                // TODO: Implement advanced structured logging for cryptographic failures
                RegistryError::CryptoError(Box::new(e))
            })?;

        let mut verifier = Verifier::new_without_digest(&pkey).map_err(|e| {
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::CryptoError(Box::new(e))
        })?;

        verifier.verify_oneshot(signature, payload).map_err(|e| {
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::CryptoError(Box::new(e))
        })
    }

    fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        let cipher = Cipher::aes_256_gcm();
        let mut iv = [0u8; 12];
        rng().fill_bytes(&mut iv);

        let mut tag = [0u8; 16];

        let mut ciphertext =
            encrypt_aead(cipher, key, Some(&iv), &[], data, &mut tag).map_err(|e| {
                // TODO: Implement advanced structured logging for cryptographic failures
                RegistryError::CryptoError(Box::new(e))
            })?;

        let mut out = iv.to_vec();
        out.extend_from_slice(&tag);
        out.append(&mut ciphertext);

        Ok(out)
    }

    fn decrypt(&self, encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 28 {
            return Err(RegistryError::CryptoError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Encrypted data too short",
            ))));
        }

        let cipher = Cipher::aes_256_gcm();
        let (iv, rest) = encrypted_data.split_at(12);
        let (tag, ciphertext) = rest.split_at(16);

        decrypt_aead(cipher, key, Some(iv), &[], ciphertext, tag).map_err(|e| {
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::CryptoError(Box::new(e))
        })
    }

    fn algorithm_name(&self) -> &'static str {
        self.alg_name
    }
}

/// Retrieves a cryptographic keypair from the underlying storage mechanism.
///
/// # Arguments
/// * `environment_id` - The logical environment identifier used as the storage key.
/// * `storage` - The storage provider implementation.
/// * `storage_encryption_key` - Key used to decrypt data from storage.
pub fn get_keypair<S: StorageProvider>(
    environment_id: &str,
    storage: &S,
    storage_encryption_key: &[u8],
) -> Result<(String, String)> {
    let encrypted_data = storage
        .load_encrypted(environment_id, storage_encryption_key)
        .map_err(|e| match e {
            StorageError::KeyNotFound => RegistryError::KeyNotFound,
            _ => {
                // TODO: Implement advanced structured logging for storage I/O failures
                RegistryError::StorageError(e.to_string())
            }
        })?;

    let json_str = String::from_utf8(encrypted_data)
        .map_err(|e| RegistryError::InvalidKeyFormat(e.to_string()))?;

    let keypair: (String, String) = serde_json::from_str(&json_str)?;

    Ok(keypair)
}

/// Persists a cryptographic keypair to the underlying storage mechanism.
///
/// # Arguments
/// * `environment_id` - The logical environment identifier used as the storage key.
/// * `storage` - The storage provider implementation.
/// * `storage_encryption_key` - Key used to encrypt data for storage.
/// * `public_key_b64` - The base64 encoded public key to store.
/// * `private_key_b64` - The base64 encoded private key (seed) to store.
pub fn save_keypair<S: StorageProvider>(
    environment_id: &str,
    storage: &S,
    storage_encryption_key: &[u8],
    public_key_b64: &str,
    private_key_b64: &str,
) -> Result<()> {
    let keypair = (public_key_b64.to_string(), private_key_b64.to_string());

    let json_str = serde_json::to_string(&keypair)?;

    storage
        .save_encrypted(environment_id, json_str.as_bytes(), storage_encryption_key)
        .map_err(|e| {
            // TODO: Implement advanced structured logging for storage I/O failures
            RegistryError::StorageError(e.to_string())
        })
}

/// Exports a keypair from storage into a portable PEM-like string format.
///
/// # Arguments
/// * `environment_id` - The logical environment identifier used as the storage key.
/// * `storage` - The storage provider implementation.
/// * `storage_encryption_key` - Key used to decrypt data from storage.
/// * `provider` - The cryptographic provider used to resolve the algorithm name.
pub fn export_keypair<P: CryptoProvider, S: StorageProvider>(
    environment_id: &str,
    storage: &S,
    storage_encryption_key: &[u8],
    provider: &P,
) -> Result<String> {
    let (_, priv_b64) = get_keypair(environment_id, storage, storage_encryption_key)?;

    let algo = provider.algorithm_name();
    let label = format!("{} SEED", algo.replace("-", " "));

    let priv_bytes = STANDARD.decode(&priv_b64).map_err(|e| {
        // TODO: Implement advanced structured logging for cryptographic failures
        RegistryError::InvalidKeyFormat(e.to_string())
    })?;

    let pem_string = pem_rfc7468::encode_string(&label, pem_rfc7468::LineEnding::LF, &priv_bytes)
        .map_err(|e| {
        // TODO: Implement advanced structured logging for cryptographic failures
        RegistryError::InvalidKeyFormat(e.to_string())
    })?;

    Ok(pem_string)
}

/// Imports a keypair from a portable PEM-like string format into storage.
///
/// # Arguments
/// * `environment_id` - The logical environment identifier used as the storage key.
/// * `storage` - The storage provider implementation.
/// * `storage_encryption_key` - Key used to encrypt data for storage.
/// * `exported_data` - The PEM-like string containing the key data.
/// * `provider` - The cryptographic provider used to derive the public key.
pub fn import_keypair<P: CryptoProvider, S: StorageProvider>(
    environment_id: &str,
    storage: &S,
    storage_encryption_key: &[u8],
    exported_data: &str,
    provider: &P,
) -> Result<()> {
    let algo = provider.algorithm_name();
    let expected_label = format!("{} SEED", algo.replace("-", " "));

    let priv_b64 = if exported_data.contains("-----BEGIN") {
        let (label, decoded_bytes) =
            pem_rfc7468::decode_vec(exported_data.as_bytes()).map_err(|e| {
                // TODO: Implement advanced structured logging for cryptographic failures
                RegistryError::InvalidKeyFormat(e.to_string())
            })?;

        if label != expected_label {
            // TODO: Implement advanced structured logging for cryptographic failures
            return Err(RegistryError::InvalidKeyFormat(format!(
                "Expected label '{}', got '{}'",
                expected_label, label
            )));
        }

        STANDARD.encode(&decoded_bytes)
    } else {
        exported_data.trim().to_string()
    };

    let (pub_b64, _) = provider.derive_keypair(&priv_b64)?;

    save_keypair(
        environment_id,
        storage,
        storage_encryption_key,
        &pub_b64,
        &priv_b64,
    )
}

/// Signs a JSON-serializable payload, returning a standard JWS envelope.
///
/// # Arguments
/// * `environment_id` - The logical environment identifier used as the `kid` in the JWS header.
/// * `payload` - The strongly-typed payload to serialize and sign.
/// * `private_key_b64` - The base64 encoded private key (seed) to use for signing.
/// * `provider` - The cryptographic provider handling the signature logic.
pub fn sign_payload<P: CryptoProvider, T: serde::Serialize>(
    environment_id: &str,
    payload: &T,
    private_key_b64: &str,
    provider: &P,
) -> Result<JwsEnvelope> {
    let payload_json = serde_json::to_string(payload)?;
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    let header = crate::schema::JwsHeader {
        alg: provider.algorithm_name().to_string(),
        kid: environment_id.to_string(),
        iat: Utc::now().timestamp(),
    };
    let header_json = serde_json::to_string(&header)?;
    let protected_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());

    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature_bytes = provider.sign(signing_input.as_bytes(), private_key_b64)?;
    let signature_b64 = URL_SAFE_NO_PAD.encode(signature_bytes);

    Ok(JwsEnvelope {
        protected: protected_b64,
        payload: payload_b64,
        signature: signature_b64,
    })
}

/// Verifies a JWS envelope against a provided public key and enforces replay protection.
///
/// # Arguments
/// * `envelope` - The JWS envelope containing the payload and signature.
/// * `public_key_b64` - The base64 encoded public key to use for verification.
/// * `provider` - The cryptographic provider handling the verification logic.
pub fn verify_signature<P: CryptoProvider>(
    envelope: &JwsEnvelope,
    public_key_b64: &str,
    provider: &P,
) -> Result<bool> {
    let protected_bytes = URL_SAFE_NO_PAD.decode(&envelope.protected).map_err(|e|
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::InvalidKeyFormat(e.to_string()))?;
    let header: crate::schema::JwsHeader = serde_json::from_slice(&protected_bytes)?;

    if header.alg != provider.algorithm_name() {
        return Ok(false);
    }

    // Replay protection: Reject payloads older than 1 hour
    let now = Utc::now().timestamp();
    if (now - header.iat).abs() > 3600 {
        return Ok(false);
    }

    let signing_input = format!("{}.{}", envelope.protected, envelope.payload);
    let signature_bytes = URL_SAFE_NO_PAD.decode(&envelope.signature).map_err(|e|
            // TODO: Implement advanced structured logging for cryptographic failures
            RegistryError::InvalidSignatureFormat(e.to_string()))?;

    provider.verify(signing_input.as_bytes(), &signature_bytes, public_key_b64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_generate_keypair_cleans_up_on_error() {
        let provider = OpenSslProvider::new("INVALID-ALG-TO-FORCE-ERROR");
        let secret = b"MOCK_SEED_DO_NOT_LEAK_IN_GEN_KEY";

        MOCK_RNG_NEXT_OUTPUT.with(|output| {
            *output.borrow_mut() = Some(secret.to_vec());
        });

        let result = provider.generate_keypair();
        assert!(result.is_err());

        let ptr = super::LAST_SEED_PTR.with(|p| *p.borrow());
        assert!(!ptr.is_null(), "Pointer was not captured");

        let mut leak_detected = true;
        unsafe {
            for i in 0..32 {
                // Directly read from the exact memory address where the stack variable lived
                if ptr.add(i).read_volatile() != secret[i] {
                    leak_detected = false;
                    break;
                }
            }
        }

        MOCK_RNG_NEXT_OUTPUT.with(|output| {
            *output.borrow_mut() = None;
        });

        assert!(
            !leak_detected,
            "SECURITY REGRESSION: Stack memory leak detected!"
        );
    }
}
