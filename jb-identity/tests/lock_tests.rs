// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use jb_identity::local::{load_instance_identity, create_instance_identity};
#[cfg(windows)]
use jb_identity::local::update_instance_identity;
use jb_identity::Identity;
use std::fs;
#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::thread;
use tempfile::tempdir;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[test]
fn test_save_exclusive_lock_prevents_overwrite() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();
    
    // Simulate Process A writing the file first
    let id_a = Identity::Uuid("11111111-1111-1111-1111-111111111111".to_string());
    create_instance_identity(repo_path, &id_a).unwrap();
    
    // Simulate Process B trying to write its own generated UUID exactly after Process A.
    // Because we use create_new(true), Process B should gracefully return Ok(()) 
    // without overwriting Process A's file.
    let id_b = Identity::Uuid("22222222-2222-2222-2222-222222222222".to_string());
    create_instance_identity(repo_path, &id_b).unwrap();
    
    // Verify Process A's ID is still the one on disk
    let loaded = load_instance_identity(repo_path).unwrap().unwrap();
    assert_eq!(loaded, id_a);
    assert_ne!(loaded, id_b);
}

#[test]
#[cfg(windows)]
fn test_load_waits_for_exclusive_write_lock_to_release() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path().to_path_buf();
    
    let jbe_dir = repo_path.join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    let project_file = jbe_dir.join("project.json");
    
    // Write valid initial config
    fs::write(&project_file, r#"{"id_type": "uuid", "id": "11111111-1111-1111-1111-111111111111"}"#).unwrap();
    
    // Simulate Process A holding an EXCLUSIVE write lock on the file
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    
    options.share_mode(0); // 0 = No sharing allowed (Exclusive)
    
    let locked_file = options.open(&project_file).unwrap();
    
    // Spawn a thread to release the lock after a short delay
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(500));
        // Lock is released when `locked_file` is dropped here
        drop(locked_file);
    });
    
    let start_time = std::time::Instant::now();
    
    // Process B tries to read the file. It should block and wait for Process A to release the lock.
    let result = load_instance_identity(&repo_path);
    
    let elapsed = start_time.elapsed();
    
    // It should succeed because the lock was released within the 2000ms timeout
    assert!(result.is_ok());
    let id = result.unwrap().unwrap();
    assert_eq!(id, Identity::Uuid("11111111-1111-1111-1111-111111111111".to_string()));
    
    // The load should have taken at least 500ms (the delay before the lock was released)
    assert!(elapsed.as_millis() >= 450, "Did not wait for lock release, elapsed: {}ms", elapsed.as_millis());
}

#[test]
#[cfg(windows)]
fn test_load_blocked_by_exclusive_write_lock_timeout() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path().to_path_buf();
    
    let jbe_dir = repo_path.join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    let project_file = jbe_dir.join("project.json");
    
    // Write valid initial config
    fs::write(&project_file, r#"{"id_type": "uuid", "id": "11111111-1111-1111-1111-111111111111"}"#).unwrap();
    
    // Simulate Process A holding an EXCLUSIVE write lock on the file
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    
    options.share_mode(0); // 0 = No sharing allowed (Exclusive)
    
    let _locked_file = options.open(&project_file).unwrap();
    
    let start_time = std::time::Instant::now();
    
    // Process B tries to read the file. It should block and timeout.
    let result = load_instance_identity(&repo_path);
    
    let elapsed = start_time.elapsed();
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Failed to read project.json (locked?)"));
    
    // The lock timeout is 2000ms. We should have waited at least ~1900ms.
    assert!(elapsed.as_millis() >= 1900, "Did not wait for timeout, elapsed: {}ms", elapsed.as_millis());
}

#[test]
#[cfg(windows)]
fn test_update_blocked_by_exclusive_write_lock_timeout() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path().to_path_buf();
    
    let jbe_dir = repo_path.join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    let project_file = jbe_dir.join("project.json");
    
    // Write valid initial config
    fs::write(&project_file, r#"{"id_type": "git-shallow", "id": "11111111-1111-1111-1111-111111111111"}"#).unwrap();
    
    // Simulate Process A holding an EXCLUSIVE write lock on the file
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    
    options.share_mode(0); // 0 = No sharing allowed (Exclusive)
    
    let _locked_file = options.open(&project_file).unwrap();
    
    let start_time = std::time::Instant::now();
    
    // Simulate Process B trying to update the file while Process A holds the exclusive lock.
    let id_update = Identity::Git { 
        hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        branch: "main".to_string(),
        local_roots: Vec::new()
    };
    let result = update_instance_identity(&repo_path, &id_update);
    
    let elapsed = start_time.elapsed();
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Failed to lock/open project.json for update"));
    
    // The lock timeout is 2000ms. We should have waited at least ~1900ms.
    assert!(elapsed.as_millis() >= 1900, "Did not wait for timeout, elapsed: {}ms", elapsed.as_millis());
}
