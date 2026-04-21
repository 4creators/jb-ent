// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use crate::Identity;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityConfig {
    pub id_type: String,
    pub id: String,
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
            Identity::Git(config.id)
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
    
    let (id_type, id_val) = match id {
        Identity::Git(v) => ("git", v),
        Identity::GitShallow(v) => ("git-shallow", v),
        Identity::Uuid(v) => ("uuid", v),
    };
    
    let config = IdentityConfig {
        id_type: id_type.to_string(),
        id: id_val.to_string(),
    };
    
    let content = serde_json::to_string_pretty(&config)?;
    let project_file = jbe_dir.join("project.json");
    
    let mut options = OpenOptions::new();
    options.write(true);
    options.create_new(true); // Protects against TOCTOU race
    
    #[cfg(windows)]
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
    
    Ok(())
}

pub fn update_instance_identity(repo_path: &Path, id: &Identity) -> Result<()> {
    let jbe_dir = repo_path.join(".jbe");
    let project_file = jbe_dir.join("project.json");
    
    let (id_type, id_val) = match id {
        Identity::Git(v) => ("git", v),
        Identity::GitShallow(v) => ("git-shallow", v),
        Identity::Uuid(v) => ("uuid", v),
    };
    
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    
    #[cfg(windows)]
    options.share_mode(0); // Exclusive Lock
    
    let mut file = match open_with_timeout(&mut options, &project_file, 2000) {
        Ok(f) => f,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to lock/open project.json for update: {}", e));
        }
    };
    
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    let mut config: serde_json::Value = serde_json::from_str(&content)?;
    
    if let Some(obj) = config.as_object_mut() {
        obj.insert("id_type".to_string(), serde_json::Value::String(id_type.to_string()));
        obj.insert("id".to_string(), serde_json::Value::String(id_val.to_string()));
    } else {
        return Err(anyhow::anyhow!("project.json is not a valid JSON object"));
    }
    
    let new_content = serde_json::to_string_pretty(&config)?;
    
    file.seek(SeekFrom::Start(0))?;
    file.write_all(new_content.as_bytes())?;
    
    let current_pos = file.stream_position()?;
    file.set_len(current_pos)?;
    
    file.sync_all()?;
    
    Ok(())
}
