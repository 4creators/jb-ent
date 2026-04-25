// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

#![allow(non_camel_case_types)]

use foreign_types_shared::ForeignType;
use openssl::error::ErrorStack;
use openssl::pkey::{PKey, Private};
use std::ffi::{CString, c_char, c_int, c_void};
use std::ptr;
use std::result::Result;
use thiserror::Error;

/// Represents granular failure states when interacting with the OpenSSL C API.
/// This enum ensures that we do not lose critical diagnostic information (like the OpenSSL ErrorStack)
/// when an FFI call fails, allowing the calling Rust code to either recover or log exactly which step failed.
#[derive(Debug, Error)]
pub enum FfiOpensslError {
    /// According to NIST FIPS 204, the core entropy seed (xi) for
    /// deterministic key generation is exactly 32 bytes for ALL
    /// parameter sets (ML-DSA-44, ML-DSA-65, and ML-DSA-87).
    #[error("Invalid seed length: expected 32, got {0}")]
    InvalidSeedLength(usize),

    /// Thrown if the requested algorithm name (e.g., "ML-DSA-44") cannot be safely converted
    /// into a C-compatible null-terminated string.
    #[error("Algorithm name contains null bytes")]
    InvalidAlgorithmName,

    /// Failed to allocate or initialize the core EVP_PKEY context.
    /// This usually happens if the algorithm name is not recognized by the linked OpenSSL version.
    #[error("EVP_PKEY_CTX_new_from_name failed: {0}")]
    CtxInitFailed(ErrorStack),

    /// Failed to allocate the parameter builder. Indicates severe memory or OpenSSL state issues.
    #[error("OSSL_PARAM_BLD_new failed: {0}")]
    ParamBldNewFailed(ErrorStack),

    /// Failed to push the raw seed bytes into the parameter builder.
    #[error("OSSL_PARAM_BLD_push_octet_string failed: {0}")]
    ParamBldPushFailed(ErrorStack),

    /// Failed to finalize the parameter array.
    #[error("OSSL_PARAM_BLD_to_param failed: {0}")]
    ParamBldToParamFailed(ErrorStack),

    /// The algorithm does not support fromdata key generation.
    #[error("EVP_PKEY_fromdata_init failed: {0}")]
    FromDataInitFailed(ErrorStack),

    /// The final key derivation failed. This can happen if the provided parameters
    /// (like the seed) are rejected by the specific algorithm implementation.
    #[error("EVP_PKEY_fromdata failed: {0}")]
    FromDataFailed(ErrorStack),
}

/// Opaque C-struct representing the OpenSSL 3.0+ parameter builder.
pub enum OSSL_PARAM_BLD {}

/// Opaque C-struct representing the finalized OpenSSL parameter array.
pub enum OSSL_PARAM {}

// We manually declare these FFI bindings because the `openssl` crate (as of 0.10.x)
// does not yet provide safe, high-level Rust wrappers for OpenSSL 3.0+ parameter building
// (OSSL_PARAM_BLD), which is strictly required for ML-DSA seed injection.
unsafe extern "C" {
    pub fn EVP_PKEY_CTX_new_from_name(
        libctx: *mut c_void,
        name: *const c_char,
        propquery: *const c_char,
    ) -> *mut openssl_sys::EVP_PKEY_CTX;

    pub fn OSSL_PARAM_BLD_new() -> *mut OSSL_PARAM_BLD;

    pub fn OSSL_PARAM_BLD_free(bld: *mut OSSL_PARAM_BLD);

    pub fn OSSL_PARAM_BLD_push_octet_string(
        bld: *mut OSSL_PARAM_BLD,
        key: *const c_char,
        buf: *const c_void,
        bsize: usize,
    ) -> c_int;

    pub fn OSSL_PARAM_BLD_to_param(bld: *mut OSSL_PARAM_BLD) -> *mut OSSL_PARAM;

    pub fn OSSL_PARAM_free(params: *mut OSSL_PARAM);

    pub fn EVP_PKEY_fromdata_init(ctx: *mut openssl_sys::EVP_PKEY_CTX) -> c_int;

    pub fn EVP_PKEY_fromdata(
        ctx: *mut openssl_sys::EVP_PKEY_CTX,
        ppkey: *mut *mut openssl_sys::EVP_PKEY,
        selection: c_int,
        params: *mut OSSL_PARAM,
    ) -> c_int;

    pub fn OBJ_txt2nid(s: *const c_char) -> c_int;
}

// RAII wrappers to ensure OpenSSL resources are reliably freed even during early returns or panics
struct CtxGuard(*mut openssl_sys::EVP_PKEY_CTX);
impl Drop for CtxGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { openssl_sys::EVP_PKEY_CTX_free(self.0) }
        }
    }
}

struct BldGuard(*mut OSSL_PARAM_BLD);
impl Drop for BldGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { OSSL_PARAM_BLD_free(self.0) }
        }
    }
}

struct ParamGuard(*mut OSSL_PARAM);
impl Drop for ParamGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { OSSL_PARAM_free(self.0) }
        }
    }
}

/// Safely creates a Post-Quantum PKey from a 32-byte seed by calling OpenSSL 4.0.0 FFI.
/// Supports varying algorithm names (e.g. "ML-DSA-44", "ML-DSA-65", "ML-DSA-87").
pub fn create_ml_dsa_pkey_from_seed(
    alg_name: &str,
    seed_bytes: &[u8],
) -> Result<PKey<Private>, FfiOpensslError> {
    // FIPS 204 mandates the ML-DSA seed (xi) is exactly 32 bytes for all parameter sets
    if seed_bytes.len() != 32 {
        return Err(FfiOpensslError::InvalidSeedLength(seed_bytes.len()));
    }

    let alg_cstring = match CString::new(alg_name) {
        Ok(s) => s,
        Err(_) => return Err(FfiOpensslError::InvalidAlgorithmName),
    };

    unsafe {
        // Clear previous thread-local OpenSSL errors
        ErrorStack::get();

        // 1. Context initialization
        let ctx_ptr =
            EVP_PKEY_CTX_new_from_name(ptr::null_mut(), alg_cstring.as_ptr(), ptr::null());

        if ctx_ptr.is_null() {
            return Err(FfiOpensslError::CtxInitFailed(ErrorStack::get()));
        }
        let _ctx = CtxGuard(ctx_ptr);

        // 2. Parameter Builder
        let pbld_ptr = OSSL_PARAM_BLD_new();
        if pbld_ptr.is_null() {
            return Err(FfiOpensslError::ParamBldNewFailed(ErrorStack::get()));
        }
        let _pbld = BldGuard(pbld_ptr);

        // 3. Inject Seed
        let seed_name = CString::new("seed").unwrap();
        let push_result = OSSL_PARAM_BLD_push_octet_string(
            pbld_ptr,
            seed_name.as_ptr(),
            seed_bytes.as_ptr() as *const _,
            32,
        );

        if push_result != 1 {
            return Err(FfiOpensslError::ParamBldPushFailed(ErrorStack::get()));
        }

        // 4. Generate Params Array
        let param_ptr = OSSL_PARAM_BLD_to_param(pbld_ptr);
        if param_ptr.is_null() {
            return Err(FfiOpensslError::ParamBldToParamFailed(ErrorStack::get()));
        }
        let _param = ParamGuard(param_ptr);

        // 5. Initialize from_data mechanism
        if EVP_PKEY_fromdata_init(ctx_ptr) <= 0 {
            return Err(FfiOpensslError::FromDataInitFailed(ErrorStack::get()));
        }

        // 6. Build the PKey
        const EVP_PKEY_KEYPAIR: c_int = 135; // OSSL_KEYMGMT_SELECT_ALL_PARAMETERS | OSSL_KEYMGMT_SELECT_KEYPAIR
        let mut pkey_ptr: *mut openssl_sys::EVP_PKEY = ptr::null_mut();

        let fromdata_result =
            EVP_PKEY_fromdata(ctx_ptr, &mut pkey_ptr, EVP_PKEY_KEYPAIR, param_ptr);

        if fromdata_result <= 0 {
            if !pkey_ptr.is_null() {
                openssl_sys::EVP_PKEY_free(pkey_ptr);
            }
            return Err(FfiOpensslError::FromDataFailed(ErrorStack::get()));
        }

        // Hand ownership to the safe openssl crate PKey wrapper
        Ok(PKey::from_ptr(pkey_ptr))
    }
}
