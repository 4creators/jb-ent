# Design Specification: jb-registry Cryptography Architecture

**Date:** 2026-04-23
**Author:** Jacek Błaszczyński
**Assistance:** Gemini CLI
**Copyright:** © 2026 Jacek Błaszczyński
**License:** AGPL-3.0-only

## 1. Goal
Establish a unified, cross-platform cryptographic architecture for the `jb-registry` crate. The registry manages the global mapping of Universal Identities (from `jb-identity`) to their physical OS paths and handles secure synchronization across boundaries (e.g., Windows/WSL, Linux, Unix). 

The primary requirement is robust, cross-platform support for Post-Quantum Cryptography (PQC), specifically **ML-DSA (FIPS 204)**, for digital signatures, alongside standard symmetric encryption (AES-256) for secure local storage.

## 2. The Problem: OS-Native Cryptography Fragmentation
Initial experimental development attempted to rely on native OS cryptographic APIs:
*   **Windows:** Cryptography Next Generation (CNG) and DPAPI.
*   **Linux/Unix:** Various native credential stores and OS-level OpenSSL versions.

This approach proved excessively fragile. Support for cutting-edge PQC algorithms like ML-DSA is inconsistent or absent across different operating systems. Writing divergent, conditionally compiled Rust code (`#[cfg(windows)]`, `#[cfg(unix)]`) for core cryptographic primitives undermined the cross-platform mandate and introduced significant maintenance and security risks.

## 3. The Solution: Unified OpenSSL 4.0.0 via vcpkg
To guarantee consistent, cross-platform availability of advanced cryptographic primitives, we have pivoted to a unified abstraction layer powered by **OpenSSL 4.0.0**.

By compiling OpenSSL 4.0.0 directly into our project via a custom `vcpkg` fork, we bypass the fragmented OS-level implementations entirely. All platforms (Windows, Linux, WSL) will link against the exact same, statically or dynamically provided OpenSSL 4.0.0 binary containing native ML-DSA support.

## 4. Implementation Rationale: The `rust-openssl` Crate

We selected the `openssl` Rust crate to interface with our custom OpenSSL 4.0.0 build.

### 4.1 Provenance and Maintenance
*   **Maintainer:** The crate is an official project under the `rust-lang` umbrella, primarily maintained by Steven Fackler (`sfackler`), a prominent member of the Rust community.
*   **Activity:** It is highly active, with releases frequently tracking upstream OpenSSL changes (3.4, 3.5, 4.0) and responding immediately to security vulnerabilities (e.g., CVE-2025-24898).
*   **Ecosystem Usage:** With hundreds of millions of downloads, it is the most battle-tested crypto bridge in the Rust ecosystem. While pure-Rust implementations like `rustls` are gaining popularity for standard TLS, `rust-openssl` remains the industry standard for specialized, FIPS-compliant, or bleeding-edge cryptographic operations (like PQC) where a C-bridge is required.

### 4.2 Handling ML-DSA Seeds (The FFI Escape Hatch)
A critical requirement for `jb-registry` is deep access to the ML-DSA private key data, specifically the 32-byte **seed ($\xi$)**, to enable on-the-fly recreation of the massive public/private key structures for storage and transmission efficiency.

OpenSSL 4.0 natively treats the 32-byte seed as a first-class parameter (`OSSL_PKEY_PARAM_ML_DSA_SEED` or simply `"seed"`). However, the safe, high-level API of the `openssl` Rust crate (as of v0.10.x) does not yet provide dedicated, strongly-typed wrappers for ML-DSA seed injection (e.g., no `PKey::ml_dsa_seed()`).

**The Strategy:**
To bridge this gap without forking the crate, `jb-registry` will utilize the generic `EVP` (Envelope) parameter methods and direct FFI (`openssl-sys`):
1.  **Extraction:** We will extract the 32-byte seed using the generic safe API: `pkey.raw_private_key()` or `pkey.get_octet_string_param("seed")`.
2.  **Injection (FFI):** To recreate a keypair from a seed, we will drop into `openssl-sys` for a single function call (e.g., `EVP_PKEY_fromdata` with the `"seed"` parameter).
3.  **Safe Signing/Verification:** Once the raw pointer is wrapped back into a safe `openssl::pkey::PKey` struct, all subsequent signing and verification operations will use the standard, safe `openssl::sign::Signer` and `Verifier` APIs, delegating the heavy PQC math safely to OpenSSL.

## 5. Storage Abstraction
By centralizing all encryption logic within the `CryptoProvider` trait (powered by OpenSSL), we eliminate the need for OS-specific storage wrappers (like `win_storage.rs`). The registry will utilize a cross-platform `FileStorageProvider` that encrypts data at rest using AES-256, ensuring the same storage code works seamlessly across Windows, WSL, and Linux.