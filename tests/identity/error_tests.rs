// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use jb_identity::local::load_instance_identity;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_invalid_json_format() {
    let dir = tempdir().unwrap();
    let jbe_dir = dir.path().join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    
    // Write malformed JSON
    fs::write(jbe_dir.join("project.json"), "{ invalid JSON").unwrap();
    
    let result = load_instance_identity(dir.path());
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("key must be a string") || err_msg.contains("expected value"));
}

#[test]
fn test_load_missing_json_fields() {
    let dir = tempdir().unwrap();
    let jbe_dir = dir.path().join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    
    // Write valid JSON but missing the required 'id' field
    fs::write(jbe_dir.join("project.json"), r#"{"id_type": "git"}"#).unwrap();
    
    let result = load_instance_identity(dir.path());
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("missing field `id`"));
}

#[test]
fn test_load_unknown_identity_type() {
    let dir = tempdir().unwrap();
    let jbe_dir = dir.path().join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    
    // Write valid JSON but with an unsupported id_type
    fs::write(jbe_dir.join("project.json"), r#"{"id_type": "alien-tech", "id": "12345"}"#).unwrap();
    
    let result = load_instance_identity(dir.path());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Unknown identity type: alien-tech");
}

#[test]
fn test_git_functions_on_non_repo() {
    let dir = tempdir().unwrap();
    
    // Calling git functions on a directory without a .git folder should return an Error
    let shallow_res = jb_identity::git::is_shallow(dir.path());
    assert!(shallow_res.is_err());
    
    let root_res = jb_identity::git::deep_root_discovery(dir.path());
    assert!(root_res.is_err());
}

#[test]
fn test_load_unknown_json_properties() {
    let dir = tempdir().unwrap();
    let jbe_dir = dir.path().join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    
    // Write valid JSON but with an extra unknown property "version"
    fs::write(
        jbe_dir.join("project.json"), 
        r#"{"id_type": "git", "id": "12345", "version": 1}"#
    ).unwrap();
    
    let result = load_instance_identity(dir.path());
    
    // Serde should reject this if we enforce strict schema validation
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("unknown field `version`"));
}

#[test]
fn test_load_malformed_json_values() {
    let dir = tempdir().unwrap();
    let jbe_dir = dir.path().join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    
    // Write valid JSON syntax, but 'id' is an array instead of a string
    fs::write(
        jbe_dir.join("project.json"), 
        r#"{"id_type": "git", "id": ["12345"]}"#
    ).unwrap();
    
    let result = load_instance_identity(dir.path());
    
    // Serde should reject the type mismatch
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("invalid type: sequence, expected a string"));
}

#[test]
fn test_load_malformed_uuid() {
    let dir = tempdir().unwrap();
    let jbe_dir = dir.path().join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    
    // Write valid JSON syntax, but 'id' contains 'z' which is invalid for UUID
    fs::write(
        jbe_dir.join("project.json"), 
        r#"{"id_type": "uuid", "id": "12345678-1234-1234-1234-12345678901z"}"#
    ).unwrap();
    
    let result = load_instance_identity(dir.path());
    
    // The load should reject the malformed UUID
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid UUID format"));
}

#[test]
fn test_load_malformed_git_hash() {
    let dir = tempdir().unwrap();
    let jbe_dir = dir.path().join(".jbe");
    fs::create_dir_all(&jbe_dir).unwrap();
    
    // Write valid JSON syntax, but 'id' contains 'z' which is not valid hex
    fs::write(
        jbe_dir.join("project.json"), 
        r#"{"id_type": "git", "id": "1da177e4c3f41524e886b7f1b8a0c1fc7321cacz"}"#
    ).unwrap();
    
    let result = load_instance_identity(dir.path());
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid Git hash"));
}
