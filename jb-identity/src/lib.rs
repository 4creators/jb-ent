// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

//! `jb-identity` provides deterministic project identity resolution.
//!
//! It implements the "Deep Root Discovery" algorithm (v0.9) to ensure stable
//! project identification across different environments, branches, and forks.

use anyhow::Result;
use std::path::Path;

pub mod git;
pub mod local;

/// The version of the identity algorithm.
pub const ALGORITHM_VERSION: &str = "0.9";

/// Represents a deterministic project identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    /// Derived from the oldest root commit of a full Git repository history.
    Git(String),
    /// A stable instance ID for shallow Git clones that lack full history.
    GitShallow(String),
    /// A persistent UUID fallback for non-Git directories.
    Uuid(String),
}

impl Identity {
    /// Formats the identity as a versioned string: `v<VERSION>:<TYPE>:<ID>`
    pub fn to_string_id(&self) -> String {
        match self {
            Identity::Git(hash) => format!("v{}:git:{}", ALGORITHM_VERSION, hash),
            Identity::GitShallow(uuid) => format!("v{}:git-shallow:{}", ALGORITHM_VERSION, uuid),
            Identity::Uuid(uuid) => format!("v{}:uuid:{}", ALGORITHM_VERSION, uuid),
        }
    }
}

/// Resolves the deterministic identity for a given path following the v0.9 specification.
pub fn resolve(path: &Path) -> Result<Identity> {
    // 1. Check local instance authority (.jbe/project.json)
    if let Some(existing) = local::load_instance_identity(path)? {
        match existing {
            Identity::Git(_) => return Ok(existing),
            Identity::GitShallow(ref uuid) => {
                // Gated Upgrade Check: If shallow ID exists, check if repo is now complete
                if !git::is_shallow(path)? {
                    // Trigger UPGRADE
                    if let Some(universal) = git::deep_root_discovery(path)? {
                        let new_id = Identity::Git(universal);
                        local::update_instance_identity(path, &new_id)?;
                        return Ok(new_id);
                    }
                }
                return Ok(Identity::GitShallow(uuid.clone()));
            }
            Identity::Uuid(_) => return Ok(existing),
        }
    }

    // 2. First Discovery
    if git::is_repo(path) {
        if !git::is_shallow(path)? {
            // Full repo: Universal Authority
            if let Some(hash) = git::deep_root_discovery(path)? {
                let id = Identity::Git(hash);
                local::create_instance_identity(path, &id)?;
                return Ok(id);
            }
        }
        // Shallow repo: Generative Authority
        let id = Identity::GitShallow(uuid::Uuid::new_v4().to_string());
        local::create_instance_identity(path, &id)?;
        Ok(id)
    } else {
        // Non-Git repo: Local Strategy
        let id = Identity::Uuid(uuid::Uuid::new_v4().to_string());
        local::create_instance_identity(path, &id)?;
        Ok(id)
    }
}
