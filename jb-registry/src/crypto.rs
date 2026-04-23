// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

#![allow(clippy::collapsible_if)]

use crate::error::{RegistryError, Result};
use std::path::Path;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use std::io::Write;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

/// Abstraction for cryptographic operations, allowing different algorithms to be plugged in.
pub trait CryptoProvider {
    /// Expected size of the raw private seed in bytes.
    const SEED_SIZE: usize;

    /// Generates a new keypair and returns (public_key_b64, private_key_b64).
    /// Both keys are returned as Base64 encoded standard envelopes (e.g. PKCS#8).
    fn generate_keypair() -> Result<(String, String)>;

    /// Derives the keypair from an encoded private key envelope.
    fn derive_keypair(private_key_b64: &str) -> Result<(String, String)>;

    /// Signs a string payload and returns the raw signature bytes.
    fn sign(payload: &[u8], private_key_b64: &str) -> Result<Vec<u8>>;

    /// Verifies a raw signature against a payload and public key.
    fn verify(payload: &[u8], signature: &[u8], public_key_b64: &str) -> Result<bool>;

    /// Returns the algorithm identifier for JWS 'alg' header.
    fn algorithm_name() -> &'static str;
}

pub struct MlDsaProvider;

impl CryptoProvider for MlDsaProvider {
    const SEED_SIZE: usize = 32;

    fn generate_keypair() -> Result<(String, String)> {
        use rand::prelude::*;
        use ml_dsa::{MlDsa44, KeyGen};
        use ml_dsa::signature::Keypair;

        let mut raw_seed = [0u8; Self::SEED_SIZE];
        rand::rng().fill_bytes(&mut raw_seed);

        let mut seed = ml_dsa::Seed::default();
        seed.copy_from_slice(&raw_seed);

        let signing_key = MlDsa44::from_seed(&seed);
        let verifying_key = signing_key.verifying_key();

        let priv_b64 = base64::engine::general_purpose::STANDARD.encode(raw_seed);
        let pub_b64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.encode());

        Ok((pub_b64, priv_b64))
    }

    fn derive_keypair(private_seed_b64: &str) -> Result<(String, String)> {
        use ml_dsa::{MlDsa44, KeyGen};
        use ml_dsa::signature::Keypair;

        let priv_bytes = base64::engine::general_purpose::STANDARD.decode(private_seed_b64)
            .map_err(|_| RegistryError::InvalidKeyFormat)?;

        if priv_bytes.len() != Self::SEED_SIZE {
            return Err(RegistryError::InvalidKeyFormat);
        }

        let mut seed = ml_dsa::Seed::default();
        seed.copy_from_slice(&priv_bytes);

        let signing_key = MlDsa44::from_seed(&seed);
        let verifying_key = signing_key.verifying_key();

        let pub_b64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.encode());

        Ok((pub_b64, private_seed_b64.to_string()))
    }

    fn sign(payload: &[u8], private_key_b64: &str) -> Result<Vec<u8>> {
        use ml_dsa::{MlDsa44, KeyGen};
        use ml_dsa::signature::{Signer, SignatureEncoding};

        let priv_bytes = base64::engine::general_purpose::STANDARD.decode(private_key_b64)
            .map_err(|_| RegistryError::InvalidKeyFormat)?;

        if priv_bytes.len() != Self::SEED_SIZE {
            return Err(RegistryError::InvalidKeyFormat);
        }

        let mut seed = ml_dsa::Seed::default();
        seed.copy_from_slice(&priv_bytes);

        let signing_key = MlDsa44::from_seed(&seed);
        let signature = signing_key.sign(payload);
        Ok(signature.to_bytes().to_vec())
    }

    fn verify(payload: &[u8], signature_bytes: &[u8], public_key_b64: &str) -> Result<bool> {
        use ml_dsa::MlDsa44;
        use ml_dsa::VerifyingKey;
        use ml_dsa::signature::Verifier;

        let pub_bytes = base64::engine::general_purpose::STANDARD.decode(public_key_b64)
            .map_err(|_| RegistryError::InvalidKeyFormat)?;

        let enc_pub_key: &ml_dsa::EncodedVerifyingKey<MlDsa44> = pub_bytes.as_slice().try_into()
            .map_err(|_| RegistryError::InvalidKeyFormat)?;
        let verifying_key = VerifyingKey::<MlDsa44>::decode(enc_pub_key);

        let enc_sig: &ml_dsa::EncodedSignature<MlDsa44> = signature_bytes.try_into()
            .map_err(|_| RegistryError::InvalidSignatureFormat)?;
        let signature = ml_dsa::Signature::<MlDsa44>::decode(enc_sig)
            .ok_or(RegistryError::InvalidSignatureFormat)?;

        match verifying_key.verify(payload, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn algorithm_name() -> &'static str {
        "ML-DSA-44"
    }
}

pub fn get_keypair<P: CryptoProvider>(environment_id: &str, fallback_dir: &Path) -> Result<(String, String)> {
    unimplemented!()
}

pub fn save_keypair<P: CryptoProvider>(environment_id: &str, fallback_dir: &Path, private_key_b64: &str) -> Result<()> {
    unimplemented!()
}

pub fn export_keypair<P: CryptoProvider>(environment_id: &str, fallback_dir: &Path) -> Result<String> {
    let (_, priv_b64) = get_keypair::<P>(environment_id, fallback_dir)?;
    let algo = P::algorithm_name();
    Ok(format!("-----BEGIN {} SEED-----\n{}\n-----END {} SEED-----\n", algo, priv_b64, algo))
}

pub fn import_keypair<P: CryptoProvider>(environment_id: &str, fallback_dir: &Path, exported_data: &str) -> Result<()> {
    let algo = P::algorithm_name();
    let header = format!("-----BEGIN {} SEED-----", algo);

    let b64_data = if exported_data.contains(&header) {
        exported_data.lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<String>()
    } else {
        exported_data.trim().to_string()
    };

    save_keypair::<P>(environment_id, fallback_dir, &b64_data)
}

pub fn sign_payload<P: CryptoProvider, T: serde::Serialize>(
    environment_id: &str,
    payload: &T,
    private_key_b64: &str,
) -> Result<crate::schema::JwsEnvelope> {
    let payload_json = serde_json::to_string(payload)?;
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    let header = crate::schema::JwsHeader {
        alg: P::algorithm_name().to_string(),
        kid: environment_id.to_string(),
        iat: Utc::now().timestamp(),
    };
    let header_json = serde_json::to_string(&header)?;
    let protected_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());

    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature_bytes = P::sign(signing_input.as_bytes(), private_key_b64)?;
    let signature_b64 = URL_SAFE_NO_PAD.encode(signature_bytes);

    Ok(crate::schema::JwsEnvelope {
        protected: protected_b64,
        payload: payload_b64,
        signature: signature_b64,
    })
}

pub fn verify_signature<P: CryptoProvider>(
    envelope: &crate::schema::JwsEnvelope,
    public_key_b64: &str,
) -> Result<bool> {
    let protected_bytes = URL_SAFE_NO_PAD.decode(&envelope.protected)?;
    let header: crate::schema::JwsHeader = serde_json::from_slice(&protected_bytes)?;

    if header.alg != P::algorithm_name() {
        return Ok(false);
    }

    // Replay protection: Reject payloads older than 1 hour
    let now = Utc::now().timestamp();
    if (now - header.iat).abs() > 3600 {
        return Ok(false);
    }

    let signing_input = format!("{}.{}", envelope.protected, envelope.payload);
    let signature_bytes = URL_SAFE_NO_PAD.decode(&envelope.signature)?;

    P::verify(signing_input.as_bytes(), &signature_bytes, public_key_b64)
}
