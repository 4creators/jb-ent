// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

#![cfg_attr(not(debug_assertions), deny(warnings))]

//! `jb-registry` provides global project discovery, multi-environment synchronization,
//! and cryptographic signing for payload integrity across OS boundaries.

pub mod crypto;
pub mod error;
pub mod ffi_openssl;
pub mod schema;
pub mod storage;
