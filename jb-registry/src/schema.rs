// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use chrono::{DateTime, Utc};
use jb_identity::local::LocalRoot;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EnvStatus {
    Online,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    pub status: EnvStatus,
    pub last_seen: DateTime<Utc>,
    /// Base64 encoded ML-DSA-44 public key for verifying payloads from this environment
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectEntry {
    pub canonical_name: String,
    pub universal_id: String,
    /// Tracks the historical local roots reported by jb-identity
    pub local_roots: Vec<LocalRoot>,
    pub environment_id: String,
    pub instance_paths: BTreeSet<String>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct GlobalRegistry {
    /// ID of the local environment (e.g. "windows_host")
    pub local_environment_id: String,
    pub environments: BTreeMap<String, Environment>,
    pub projects: Vec<ProjectEntry>,
}

/// Standard JWS Flattened JSON Serialization (RFC 7515)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwsEnvelope {
    /// Base64URL encoded JWS Protected Header
    pub protected: String,
    /// Base64URL encoded JWS Payload
    pub payload: String,
    /// Base64URL encoded JWS Signature
    pub signature: String,
}

/// JWS Protected Header containing authenticated metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwsHeader {
    /// Algorithm (e.g. "ML-DSA-44")
    pub alg: String,
    /// Environment ID of the signer
    pub kid: String,
    /// Issued At (Unix timestamp) for replay protection
    pub iat: i64,
}
