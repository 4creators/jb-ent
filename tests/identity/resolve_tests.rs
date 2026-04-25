// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use git2::Repository;
use jb_identity::{resolve, Identity, ALGORITHM_VERSION};
use std::fs;
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
    
    assert_eq!(id, Identity::Git { 
        hash: commit_id.to_string(), 
        branch: "refs/heads/master".to_string(), 
        local_roots: Vec::new() 
    });
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
    
    assert_eq!(id, Identity::Git { 
        hash: commit_id.to_string(), 
        branch: "refs/heads/master".to_string(), 
        local_roots: Vec::new() 
    });
    
    let content = fs::read_to_string(jbe_dir.join("project.json")).unwrap();
    assert!(content.contains(&commit_id.to_string()));
    assert!(content.contains(r#""id_type": "git""#));
}

#[test]
fn test_real_world_full_clone_resolution() {
    let dir = tempdir().unwrap();
    let source_repo_url = "https://github.com/rust-lang/hashbrown.git";

    let full_clone_path = dir.path().join("full_clone");
    let status = Command::new("git")
        .args(["clone", source_repo_url, full_clone_path.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "Failed to create full clone from {}", source_repo_url);
    
    let id_full = resolve(&full_clone_path).unwrap();
    assert!(matches!(id_full, Identity::Git { .. }));
}

#[test]
fn test_real_world_shallow_clone_resolution() {
    let dir = tempdir().unwrap();
    let source_repo_url = "https://github.com/rust-lang/hashbrown.git";

    let shallow_clone_path = dir.path().join("shallow_clone");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", source_repo_url, shallow_clone_path.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "Failed to create shallow clone from {}", source_repo_url);

    let id_shallow = resolve(&shallow_clone_path).unwrap();
    assert!(matches!(id_shallow, Identity::GitShallow(_)));

    let local_shallow_id = jb_identity::local::load_instance_identity(&shallow_clone_path).unwrap().unwrap();
    assert_eq!(id_shallow, local_shallow_id);
}

#[test]
fn test_real_world_unshallow_upgrade_resolution() {
    let dir = tempdir().unwrap();
    let source_repo_url = "https://github.com/rust-lang/hashbrown.git";

    // First create a shallow clone and resolve it
    let shallow_clone_path = dir.path().join("shallow_clone");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", source_repo_url, shallow_clone_path.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "Failed to create shallow clone from {}", source_repo_url);

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
    assert!(matches!(id_upgraded, Identity::Git { .. }));
    
    let local_upgraded_id = jb_identity::local::load_instance_identity(&shallow_clone_path).unwrap().unwrap();
    assert_eq!(id_upgraded, local_upgraded_id);
}

#[test]
fn test_continuous_convergence_multi_root_monorepo() {
    let dir = tempdir().unwrap();
    
    // We use two real, completely disconnected public repositories.
    // Repo A (hashbrown) was created around 2018.
    // Repo B (itoa) was created around 2016 (older).
    let repo_a_url = "https://github.com/rust-lang/hashbrown.git";
    let repo_b_url = "https://github.com/dtolnay/itoa.git";

    let multi_root_path = dir.path().join("multi_root");
    
    // 1. Clone Repo A (The "Newer" project)
    let status = Command::new("git")
        .args(["clone", repo_a_url, multi_root_path.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "Failed to create clone of Repo A");

    // 2. Resolve to lock in the identity of Repo A
    let id_first = resolve(&multi_root_path).unwrap();
    let (initial_hash, initial_branch) = match &id_first {
        Identity::Git { hash, branch, .. } => (hash.clone(), branch.clone()),
        _ => panic!("Expected Identity::Git"),
    };

    // 3. Add Repo B as a remote and fetch it (Injecting an older, disconnected history)
    let status = Command::new("git")
        .args(["remote", "add", "repo_b", repo_b_url])
        .current_dir(&multi_root_path)
        .status()
        .unwrap();
    assert!(status.success(), "Failed to add remote Repo B");

    let status = Command::new("git")
        .args(["fetch", "repo_b"])
        .current_dir(&multi_root_path)
        .status()
        .unwrap();
    assert!(status.success(), "Failed to fetch Repo B");

    // 4. Resolve again to trigger Continuous Convergence
    let id_second = resolve(&multi_root_path).unwrap();

    // 5. Verify the Upgrade and Accumulation
    if let Identity::Git { hash, branch, local_roots } = id_second {
        // Because Repo B (itoa) is chronologically older than Repo A (hashbrown),
        // the Universal ID must have upgraded to Repo B's root.
        assert_ne!(hash, initial_hash, "The root should have upgraded to the older Repo B root");
        assert!(branch.contains("repo_b") || branch.contains("tags") || branch.contains("FETCH_HEAD"), "Branch was: {}", branch);
        
        // Repo A's original root must have been pushed to local_roots alias array
        assert_eq!(local_roots.len(), 1);
        assert_eq!(local_roots[0].hash, initial_hash);
        assert_eq!(local_roots[0].branch, initial_branch);
    } else {
        panic!("Expected Identity::Git, got {:?}", id_second);
    }
}
