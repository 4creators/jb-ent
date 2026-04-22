// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use git2::Repository;
use jb_identity::git::{deep_root_discovery, is_repo};
use tempfile::tempdir;

#[test]
fn test_is_repo() {
    let dir = tempdir().unwrap();
    assert!(!is_repo(dir.path()));
    
    Repository::init(dir.path()).unwrap();
    assert!(is_repo(dir.path()));
}

#[test]
fn test_deep_root_discovery_single_history() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    
    // Create a first commit
    let sig = repo.signature().unwrap();
    let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let first_oid = repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
    
    // Discovery should find the first OID
    let resolved = deep_root_discovery(dir.path()).unwrap().unwrap();
    assert_eq!(resolved.0, first_oid.to_string());
}

#[test]
fn test_deep_root_discovery_multiple_roots() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = repo.signature().unwrap();
    let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    
    // Root 1 (Oldest)
    let root1_oid = repo.commit(None, &sig, &sig, "root 1", &tree, &[]).unwrap();
    
    // Root 2 (Newer - simulate by shifting time forward)
    let sig2 = git2::Signature::new("Test", "test@example.com", &git2::Time::new(sig.when().seconds() + 100, 0)).unwrap();
    let root2_oid = repo.commit(None, &sig2, &sig2, "root 2", &tree, &[]).unwrap();
    
    // Create branches pointing to these roots
    repo.branch("branch-1", &repo.find_commit(root1_oid).unwrap(), false).unwrap();
    repo.branch("branch-2", &repo.find_commit(root2_oid).unwrap(), false).unwrap();
    
    // Discovery should pick root1_oid (older)
    let resolved = deep_root_discovery(dir.path()).unwrap();
    assert_eq!(resolved, Some((root1_oid.to_string(), "refs/heads/branch-1".to_string())));
}
