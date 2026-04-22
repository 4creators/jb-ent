// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

#![allow(clippy::collapsible_if)]

//! `jb-identity` provides deterministic project identity resolution.
//!
//! It implements the "Deep Root Discovery" algorithm (v0.9.1) to ensure stable
//! project identification across different environments, branches, and forks.

use anyhow::Result;
use std::path::Path;

pub mod git;
pub mod local;

/// The version of the identity algorithm.
pub const ALGORITHM_VERSION: &str = "0.9.1";

/// Represents a deterministic project identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    /// Derived from the oldest root commit of a full Git repository history.
    /// Tracks historical local roots if the repository fetched deeper history.
    Git { 
        hash: String, 
        branch: String,
        local_roots: Vec<local::LocalRoot> 
    },
    /// A stable instance ID for shallow Git clones that lack full history.
    GitShallow(String),
    /// A persistent UUID fallback for non-Git directories.
    Uuid(String),
}

impl Identity {
    /// Formats the identity as a versioned string: `v<VERSION>:<TYPE>:<ID>`
    pub fn to_string_id(&self) -> String {
        match self {
            Identity::Git { hash, .. } => format!("v{}:git:{}", ALGORITHM_VERSION, hash),
            Identity::GitShallow(uuid) => format!("v{}:git-shallow:{}", ALGORITHM_VERSION, uuid),
            Identity::Uuid(uuid) => format!("v{}:uuid:{}", ALGORITHM_VERSION, uuid),
        }
    }
}

pub fn resolve(path: &Path) -> Result<Identity> {
    // 1. Check local instance authority (.jbe/project.json)
    if let Some(existing) = local::load_instance_identity(path)? {
        match existing {
            Identity::Git { hash, branch, mut local_roots } => {
                // Continuous Convergence Check
                if let Some((universal_hash, universal_branch)) = git::deep_root_discovery(path)? {
                    if universal_hash != hash {
                        // A deeper/different root was found. Accumulate the old root.
                        if !local_roots.iter().any(|r| r.hash == hash) {
                            local_roots.push(local::LocalRoot {
                                hash: hash.clone(),
                                branch: branch.clone(),
                            });
                        }
                        let new_id = Identity::Git { 
                            hash: universal_hash, 
                            branch: universal_branch,
                            local_roots 
                        };
                        local::update_instance_identity(path, &new_id)?;
                        return Ok(new_id);
                    }
                }
                return Ok(Identity::Git { hash, branch, local_roots });
            },
            Identity::GitShallow(ref uuid) => {
                // Gated Upgrade Check
                if !git::is_shallow(path)? {
                    if let Some((universal_hash, universal_branch)) = git::deep_root_discovery(path)? {
                        let new_id = Identity::Git { 
                            hash: universal_hash, 
                            branch: universal_branch,
                            local_roots: Vec::new() 
                        };
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
            if let Some((hash, branch)) = git::deep_root_discovery(path)? {
                let id = Identity::Git { hash, branch, local_roots: Vec::new() };
                local::create_instance_identity(path, &id)?;
                return Ok(id);
            }
        }
        let id = Identity::GitShallow(uuid::Uuid::new_v4().to_string());
        local::create_instance_identity(path, &id)?;
        Ok(id)
    } else {
        let id = Identity::Uuid(uuid::Uuid::new_v4().to_string());
        local::create_instance_identity(path, &id)?;
        Ok(id)
    }
}
