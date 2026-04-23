// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

#![cfg(windows)]
use windows::{core::*, Win32::Security::Cryptography::*};

unsafe fn store_pqc_key(key_name: &str) -> Result<()> {
    let mut h_prov = NCRYPT_PROV_HANDLE::default();
    let mut h_key = NCRYPT_KEY_HANDLE::default();
    let name_w = HSTRING::from(key_name);

    NCryptOpenStorageProvider(&mut h_prov, MS_KEY_STORAGE_PROVIDER, 0)?;

    // Create the persistent key handle
    NCryptCreatePersistedKey(h_prov, &mut h_key, w!("ML-DSA"), &name_w, CERT_KEY_SPEC(0), NCRYPT_FLAGS(0))?;

    // Make it exportable
    let policy: u32 = NCRYPT_ALLOW_EXPORT_FLAG;
    NCryptSetProperty(h_key.into(), w!("Export Policy"), 
        &policy.to_ne_bytes(), NCRYPT_FLAGS(0))?;

    // Save to disk
    NCryptFinalizeKey(h_key, NCRYPT_FLAGS(0))?;
    
    NCryptFreeObject(h_key.into())?;
    NCryptFreeObject(h_prov.into())?;
    Ok(())
}

#[test]
fn test_store_pqc_key() {
    let key_name = "jb-ent-pqc-test-v1";
    
    // First ensure clean state
    unsafe {
        let mut h_prov = NCRYPT_PROV_HANDLE::default();
        if NCryptOpenStorageProvider(&mut h_prov, MS_KEY_STORAGE_PROVIDER, 0).is_ok() {
            let mut h_key = NCRYPT_KEY_HANDLE::default();
            let name_w = HSTRING::from(key_name);
            if NCryptOpenKey(h_prov, &mut h_key, &name_w, CERT_KEY_SPEC(0), NCRYPT_FLAGS(0)).is_ok() {
                let _ = NCryptDeleteKey(h_key, 0);
            }
            let _ = NCryptFreeObject(h_prov.into());
        }
    }

    unsafe {
        match store_pqc_key(key_name) {
            Ok(_) => println!("Successfully created and persisted ML-DSA key!"),
            Err(e) => panic!("Failed to create ML-DSA key: {}", e),
        }
    }

    // Cleanup
    unsafe {
        let mut h_prov = NCRYPT_PROV_HANDLE::default();
        if NCryptOpenStorageProvider(&mut h_prov, MS_KEY_STORAGE_PROVIDER, 0).is_ok() {
            let mut h_key = NCRYPT_KEY_HANDLE::default();
            let name_w = HSTRING::from(key_name);
            if NCryptOpenKey(h_prov, &mut h_key, &name_w, CERT_KEY_SPEC(0), NCRYPT_FLAGS(0)).is_ok() {
                let _ = NCryptDeleteKey(h_key, 0);
            }
            let _ = NCryptFreeObject(h_prov.into());
        }
    }
}
