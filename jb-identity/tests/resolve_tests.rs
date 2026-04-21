// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use git2::Repository;
use jb_identity::{resolve, Identity, ALGORITHM_VERSION};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_resolve_non_git_generates_uuid() {
    let dir = tempdir().unwrap();
    let id = resolve(dir.path()).unwrap();
    assert!(matches!(id, Identity::Uuid(_)));
}

#[test]
fn test_resolve_non_git_loads_existing_uuid() {
    let dir = tempdir().unwrap();
    let id1 = resolve(dir.path()).unwrap();
    let id2 = resolve(dir.path()).unwrap();
    assert_eq!(id1, id2);
}

#[test]
fn test_resolve_full_git_repo_generates_git_hash() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    
    let sig = repo.signature().unwrap();
    let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let commit_id = repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
    
    let id = resolve(dir.path()).unwrap();
    
    assert_eq!(id, Identity::Git(commit_id.to_string()));
    assert_eq!(id.to_string_id(), format!("v{}:git:{}", ALGORITHM_VERSION, commit_id));
}

#[test]
fn test_resolve_shallow_repo_generates_git_shallow() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();
    
    let _repo = Repository::init(repo_path).unwrap();
    let git_dir = repo_path.join(".git");
    fs::write(git_dir.join("shallow"), "0000000000000000000000000000000000000000\n").unwrap();
    
    let id = resolve(repo_path).unwrap();
    assert!(matches!(id, Identity::GitShallow(_)));
}

#[test]
fn test_resolve_shallow_repo_loads_existing_git_shallow() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();
    
    let _repo = Repository::init(repo_path).unwrap();
    let git_dir = repo_path.join(".git");
    fs::write(git_dir.join("shallow"), "0000000000000000000000000000000000000000\n").unwrap();
    
    let id1 = resolve(repo_path).unwrap();
    let id2 = resolve(repo_path).unwrap();
    assert_eq!(id1, id2);
}

#[test]
fn test_resolve_upgrade_from_shallow_to_full() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();
    
    let _repo = Repository::init(repo_path).unwrap();
    let jbe_dir = repo_path.join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    fs::write(
        jbe_dir.join("project.json"), 
        r#"{"id_type": "git-shallow", "id": "44444444-4444-4444-4444-444444444444"}"#
    ).unwrap();

    let repo = Repository::open(repo_path).unwrap();
    let sig = repo.signature().unwrap();
    let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let commit_id = repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
    
    let id = resolve(repo_path).unwrap();
    
    assert_eq!(id, Identity::Git(commit_id.to_string()));
    
    let content = fs::read_to_string(jbe_dir.join("project.json")).unwrap();
    assert!(content.contains(&commit_id.to_string()));
    assert!(content.contains(r#""id_type": "git""#));
}

#[test]
fn test_real_world_full_clone_resolution() {
    let dir = tempdir().unwrap();
    let source_repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    assert!(source_repo.join(".git").exists(), "Source must be a git repository");
    let source_repo_str = source_repo.to_str().unwrap();

    let full_clone_path = dir.path().join("full_clone");
    let status = Command::new("git")
        .args(["clone", source_repo_str, full_clone_path.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "Failed to create full clone");
    
    let id_full = resolve(&full_clone_path).unwrap();
    assert!(matches!(id_full, Identity::Git(_)));
}

#[test]
fn test_real_world_shallow_clone_resolution() {
    let dir = tempdir().unwrap();
    let source_repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    assert!(source_repo.join(".git").exists(), "Source must be a git repository");
    let source_repo_str = source_repo.to_str().unwrap();

    let shallow_clone_path = dir.path().join("shallow_clone");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--no-local", source_repo_str, shallow_clone_path.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "Failed to create shallow clone");

    let id_shallow = resolve(&shallow_clone_path).unwrap();
    assert!(matches!(id_shallow, Identity::GitShallow(_)));

    let local_shallow_id = jb_identity::local::load_instance_identity(&shallow_clone_path).unwrap().unwrap();
    assert_eq!(id_shallow, local_shallow_id);
}

#[test]
fn test_real_world_unshallow_upgrade_resolution() {
    let dir = tempdir().unwrap();
    let source_repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    assert!(source_repo.join(".git").exists(), "Source must be a git repository");
    let source_repo_str = source_repo.to_str().unwrap();

    // First create a shallow clone and resolve it
    let shallow_clone_path = dir.path().join("shallow_clone");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--no-local", source_repo_str, shallow_clone_path.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "Failed to create shallow clone");

    let _id_shallow = resolve(&shallow_clone_path).unwrap();

    // Now unshallow it
    let status = Command::new("git")
        .args(["fetch", "--unshallow"])
        .current_dir(&shallow_clone_path)
        .status()
        .unwrap();
    assert!(status.success(), "Failed to unshallow the clone");

    // Resolve again to trigger the upgrade
    let id_upgraded = resolve(&shallow_clone_path).unwrap();
    assert!(matches!(id_upgraded, Identity::Git(_)));
    
    let local_upgraded_id = jb_identity::local::load_instance_identity(&shallow_clone_path).unwrap().unwrap();
    assert_eq!(id_upgraded, local_upgraded_id);
}
