// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

#![allow(clippy::collapsible_if)]

use anyhow::Result;
use git2::{Repository, Oid, Time};
use std::collections::HashSet;
use std::path::Path;

pub fn is_repo(path: &Path) -> bool {
    Repository::discover(path).is_ok()
}

pub fn is_shallow(path: &Path) -> Result<bool> {
    let repo = Repository::discover(path)?;
    Ok(repo.is_shallow())
}

pub fn deep_root_discovery(path: &Path) -> Result<Option<String>> {
    let repo = Repository::discover(path)?;
    
    let mut all_roots: Vec<(Oid, Time)> = Vec::new();
    let mut seen_roots = HashSet::new();
    let mut global_seen_commits = HashSet::new();

    if let Ok(refs) = repo.references() {
        for reference in refs.flatten() {
            if let Ok(commit) = reference.peel_to_commit() {
                for root_info in find_root_for_commit(&repo, commit.id(), &mut global_seen_commits) {
                    if seen_roots.insert(root_info.0) {
                        all_roots.push(root_info);
                    }
                }
            }
        }
    }

    // Sort by author timestamp (ascending)
    all_roots.sort_by_key(|a| a.1.seconds());
    
    Ok(all_roots.first().map(|(oid, _)| oid.to_string()))
}

fn find_root_for_commit(repo: &Repository, start_oid: Oid, global_seen_commits: &mut HashSet<Oid>) -> Vec<(Oid, Time)> {
    let mut roots = Vec::new();
    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return roots,
    };
    if revwalk.push(start_oid).is_err() {
        return roots;
    }
    let _ = revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE);

    while let Some(oid_result) = revwalk.next() {
        if let Ok(oid) = oid_result {
            if !global_seen_commits.insert(oid) {
                // Already visited this commit in another branch walk.
                // Hide it from this revwalk so we don't process its ancestors again.
                let _ = revwalk.hide(oid);
                continue;
            }
            if let Ok(commit) = repo.find_commit(oid) {
                if commit.parent_count() == 0 {
                    roots.push((oid, commit.author().when()));
                }
            }
        }
    }
    roots
}
