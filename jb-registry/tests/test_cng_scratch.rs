use windows_sys::Win32::Security::Cryptography::*;
use windows_sys::Win32::Foundation::*;
use std::ptr;

#[test]
fn test_ncrypt_create_persistent() {
    unsafe {
        let mut prov_handle: NCRYPT_PROV_HANDLE = 0;
        let mut key_handle: NCRYPT_KEY_HANDLE = 0;
        assert_eq!(0, NCryptOpenStorageProvider(&mut prov_handle, MS_KEY_STORAGE_PROVIDER, 0));
        
        let container_name = "test-cng-key-123";
        let container_u16: Vec<u16> = container_name.encode_utf16().chain(std::iter::once(0)).collect();
        let param_set: Vec<u16> = "44".encode_utf16().chain(std::iter::once(0)).collect();
        let seed_bytes = vec![0u8; 32];
        
        let blob_type: &[u16] = &[ 'P' as u16, 'Q' as u16, 'D' as u16, 'S' as u16, 'A' as u16, 'P' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'e' as u16, 'd' as u16, 'B' as u16, 'l' as u16, 'o' as u16, 'b' as u16, 0 ];
        let alg_id: &[u16] = &[ 'M' as u16, 'L' as u16, '-' as u16, 'D' as u16, 'S' as u16, 'A' as u16, 0 ];
        
        let mut blob_data = Vec::new();
        let header = [0x53535344u32, (param_set.len() * 2) as u32, seed_bytes.len() as u32];
        blob_data.extend_from_slice(std::slice::from_raw_parts(header.as_ptr() as *const u8, 12));
        blob_data.extend_from_slice(std::slice::from_raw_parts(param_set.as_ptr() as *const u8, param_set.len() * 2));
        blob_data.extend_from_slice(&seed_bytes);

        let key_material_prop = &[ 'K' as u16, 'e' as u16, 'y' as u16, ' ' as u16, 'M' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'a' as u16, 'l' as u16, 0 ];

        println!("Trying NCryptCreatePersistedKey...");
        let status1 = NCryptCreatePersistedKey(prov_handle, &mut key_handle, alg_id.as_ptr(), container_u16.as_ptr(), 0, 0x00000080); // NCRYPT_OVERWRITE_KEY_FLAG
        println!("Create Status: 0x{:x}", status1);

        if status1 == 0 {
            let status2 = NCryptSetProperty(key_handle, key_material_prop.as_ptr(), blob_data.as_ptr(), blob_data.len() as u32, 0);
            println!("SetProperty Status: 0x{:x}", status2);

            let status3 = NCryptFinalizeKey(key_handle, 0);
            println!("Finalize Status: 0x{:x}", status3);
            
            NCryptDeleteKey(key_handle, 0);
        } else {
            if key_handle != 0 { NCryptFreeObject(key_handle); }
        }
        
        NCryptFreeObject(prov_handle);
    }
}