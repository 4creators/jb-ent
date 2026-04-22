// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use jb_identity::{Identity, ALGORITHM_VERSION};
use jb_identity::local::{load_instance_identity, create_instance_identity, update_instance_identity};
use tempfile::tempdir;

#[test]
fn test_identity_persistence_uuid() {
    let dir = tempdir().unwrap();
    let id = Identity::Uuid("11111111-1111-1111-1111-111111111111".to_string());
    
    // Save
    create_instance_identity(dir.path(), &id).unwrap();
    
    // Check file existence
    let project_json = dir.path().join(".jbe").join("project.json");
    assert!(project_json.exists());
    
    // Load and verify
    let loaded = load_instance_identity(dir.path()).unwrap().unwrap();
    assert_eq!(loaded, id);
    assert_eq!(loaded.to_string_id(), format!("v{}:uuid:11111111-1111-1111-1111-111111111111", ALGORITHM_VERSION));
}

#[test]
fn test_identity_persistence_git_shallow() {
    let dir = tempdir().unwrap();
    let id = Identity::GitShallow("22222222-2222-2222-2222-222222222222".to_string());
    
    create_instance_identity(dir.path(), &id).unwrap();
    
    let loaded = load_instance_identity(dir.path()).unwrap().unwrap();
    assert_eq!(loaded, id);
    assert_eq!(loaded.to_string_id(), format!("v{}:git-shallow:22222222-2222-2222-2222-222222222222", ALGORITHM_VERSION));
}

#[test]
fn test_identity_upgrade_flow_persistence() {
    let dir = tempdir().unwrap();
    
    // 1. Initially shallow
    let id_shallow = Identity::GitShallow("33333333-3333-3333-3333-333333333333".to_string());
    create_instance_identity(dir.path(), &id_shallow).unwrap();
    
    // 2. Upgrade to universal (Full Git history becomes available)
    let id_universal = Identity::Git { 
        hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        branch: "main".to_string(),
        local_roots: Vec::new()
    };
    update_instance_identity(dir.path(), &id_universal).unwrap();
    
    let loaded = load_instance_identity(dir.path()).unwrap().unwrap();
    assert_eq!(loaded, id_universal);
}

#[test]
fn test_load_non_existent() {
    let dir = tempdir().unwrap();
    let loaded = load_instance_identity(dir.path()).unwrap();
    assert!(loaded.is_none());
}
