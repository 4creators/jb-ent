// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions, File};
use std::io::{Read, Write};
#[cfg(windows)]
use std::io::{Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use crate::Identity;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalRoot {
    pub hash: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityConfig {
    pub id_type: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub branch: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_roots: Vec<LocalRoot>,
}

// Helper for opening a file with a timeout on sharing violations.
fn open_with_timeout(options: &mut OpenOptions, path: &Path, timeout_ms: u64) -> std::io::Result<File> {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    loop {
        match options.open(path) {
            Ok(file) => return Ok(file),
            Err(e) => {
                // 32 is ERROR_SHARING_VIOLATION on Windows
                if e.raw_os_error() == Some(32) && start.elapsed() < timeout {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                return Err(e);
            }
        }
    }
}

pub fn load_instance_identity(repo_path: &Path) -> Result<Option<Identity>> {
    let jbe_dir = repo_path.join(".jbe");
    if !jbe_dir.exists() {
        return Ok(None);
    }
    
    let project_file = jbe_dir.join("project.json");
    if !project_file.exists() {
        return Ok(None);
    }
    
    let mut options = OpenOptions::new();
    options.read(true);
    
    #[cfg(windows)]
    options.share_mode(1); // FILE_SHARE_READ: block exclusive writers but allow other readers
    
    let mut file = match open_with_timeout(&mut options, &project_file, 2000) {
        Ok(f) => f,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to read project.json (locked?): {}", e));
        }
    };
    
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    let config: IdentityConfig = serde_json::from_str(&content)?;
    
    let identity = match config.id_type.as_str() {
        "git" => {
            if git2::Oid::from_str(&config.id).is_err() {
                return Err(anyhow::anyhow!("Invalid Git hash: {}", config.id));
            }
            Identity::Git { 
                hash: config.id, 
                branch: config.branch,
                local_roots: config.local_roots 
            }
        },
        "git-shallow" => {
            if uuid::Uuid::parse_str(&config.id).is_err() {
                return Err(anyhow::anyhow!("Invalid UUID format for git-shallow: {}", config.id));
            }
            Identity::GitShallow(config.id)
        },
        "uuid" => {
            if uuid::Uuid::parse_str(&config.id).is_err() {
                return Err(anyhow::anyhow!("Invalid UUID format: {}", config.id));
            }
            Identity::Uuid(config.id)
        },
        _ => return Err(anyhow::anyhow!("Unknown identity type: {}", config.id_type)),
    };
    
    Ok(Some(identity))
}

pub fn create_instance_identity(repo_path: &Path, id: &Identity) -> Result<()> {
    let jbe_dir = repo_path.join(".jbe");
    if !jbe_dir.exists() {
        fs::create_dir_all(&jbe_dir)?;
    }
    
    let (id_type, id_val, branch_val, local_roots) = match id {
        Identity::Git { hash, branch, local_roots } => ("git", hash.clone(), branch.clone(), local_roots.clone()),
        Identity::GitShallow(v) => ("git-shallow", v.clone(), String::new(), Vec::new()),
        Identity::Uuid(v) => ("uuid", v.clone(), String::new(), Vec::new()),
    };
    
    let config = IdentityConfig {
        id_type: id_type.to_string(),
        id: id_val.to_string(),
        branch: branch_val,
        local_roots,
    };
    
    let content = serde_json::to_string_pretty(&config)?;
    let project_file = jbe_dir.join("project.json");
    
    #[cfg(windows)]
    {
        let mut options = OpenOptions::new();
        options.write(true);
        options.create_new(true); // Protects against TOCTOU race
        
        options.share_mode(0); // Exclusive Lock
        
        let mut file = match open_with_timeout(&mut options, &project_file, 2000) {
            Ok(f) => f,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    // Another process safely created the file before us. Yield to their identity.
                    return Ok(());
                }
                return Err(anyhow::anyhow!("Failed to lock/create project.json: {}", e));
            }
        };
        
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }
    
    #[cfg(unix)]
    {
        let temp_name = format!("project.json.{}.tmp", uuid::Uuid::new_v4());
        let temp_file = jbe_dir.join(&temp_name);
        
        let mut temp_options = OpenOptions::new();
        temp_options.write(true).create_new(true).mode(0o600);
        
        let mut temp_fd = temp_options.open(&temp_file)?;
        temp_fd.write_all(content.as_bytes())?;
        temp_fd.sync_all()?;
        
        // Link temp to final. link(2) fails if the target already exists, avoiding 0-byte claim files.
        match std::fs::hard_link(&temp_file, &project_file) {
            Ok(_) => {
                // We won the race! The file is fully populated.
                let _ = std::fs::remove_file(&temp_file);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Someone else created it first. Yield to their identity.
                let _ = std::fs::remove_file(&temp_file);
                return Ok(());
            }
            Err(e) => {
                let _ = std::fs::remove_file(&temp_file);
                return Err(anyhow::anyhow!("Failed to link project.json: {}", e));
            }
        }
    }
    
    Ok(())
}

pub fn update_instance_identity(repo_path: &Path, id: &Identity) -> Result<()> {
    let jbe_dir = repo_path.join(".jbe");
    let project_file = jbe_dir.join("project.json");
    
    let (id_type, id_val, branch_val, new_local_roots) = match id {
        Identity::Git { hash, branch, local_roots } => ("git", hash.clone(), branch.clone(), local_roots.clone()),
        Identity::GitShallow(v) => ("git-shallow", v.clone(), String::new(), Vec::new()),
        Identity::Uuid(v) => ("uuid", v.clone(), String::new(), Vec::new()),
    };
    
    #[cfg(windows)]
    {
        let mut options = OpenOptions::new();
        options.read(true).write(true);
        
        options.share_mode(0); // Exclusive Lock
        
        let mut file = match open_with_timeout(&mut options, &project_file, 2000) {
            Ok(f) => f,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to lock/open project.json for update: {}", e));
            }
        };
        
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let mut existing_config: IdentityConfig = serde_json::from_str(&content)?;
        
        // Merge local roots: Keep existing ones, append new ones if they don't already exist by hash
        for new_root in new_local_roots {
            if !existing_config.local_roots.iter().any(|r| r.hash == new_root.hash) {
                existing_config.local_roots.push(new_root);
            }
        }
        
        // Update primary identity fields
        existing_config.id_type = id_type.to_string();
        existing_config.id = id_val;
        existing_config.branch = branch_val;
        
        let new_content = serde_json::to_string_pretty(&existing_config)?;
        
        file.seek(SeekFrom::Start(0))?;
        file.write_all(new_content.as_bytes())?;
        
        let current_pos = file.stream_position()?;
        file.set_len(current_pos)?;
        
        file.sync_all()?;
    }
    
    #[cfg(unix)]
    {
        let lock_path = jbe_dir.join("project.lock");
        let start = Instant::now();
        let timeout = Duration::from_millis(2000);
        
        let _lock_file = loop {
            let mut opts = OpenOptions::new();
            opts.write(true).create_new(true).mode(0o600);
            match opts.open(&lock_path) {
                Ok(f) => break f,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if let Ok(metadata) = fs::metadata(&lock_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified.elapsed().unwrap_or_default() > Duration::from_secs(10) {
                                let _ = fs::remove_file(&lock_path);
                                continue;
                            }
                        }
                    }
                    if start.elapsed() >= timeout {
                        return Err(anyhow::anyhow!("Failed to acquire lock for project.json update (timeout)"));
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to create lock file: {}", e)),
            }
        };

        let mut content = String::new();
        let mut file = OpenOptions::new().read(true).open(&project_file)?;
        file.read_to_string(&mut content)?;
        
        let mut existing_config: IdentityConfig = serde_json::from_str(&content)?;
        
        // Merge local roots: Keep existing ones, append new ones if they don't already exist by hash
        for new_root in new_local_roots {
            if !existing_config.local_roots.iter().any(|r| r.hash == new_root.hash) {
                existing_config.local_roots.push(new_root);
            }
        }
        
        // Update primary identity fields
        existing_config.id_type = id_type.to_string();
        existing_config.id = id_val;
        existing_config.branch = branch_val;
        
        let new_content = serde_json::to_string_pretty(&existing_config)?;
        
        let temp_name = format!("project.json.{}.tmp", uuid::Uuid::new_v4());
        let temp_file = jbe_dir.join(&temp_name);
        
        let mut temp_options = OpenOptions::new();
        temp_options.write(true).create_new(true).mode(0o600);
        
        let mut temp_fd = temp_options.open(&temp_file)?;
        
        temp_fd.write_all(new_content.as_bytes())?;
        temp_fd.sync_all()?;
        
        fs::rename(&temp_file, &project_file)?;
        
        let _ = fs::remove_file(&lock_path);
    }
    
    Ok(())
}
