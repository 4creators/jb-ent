// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

//! `jb-registry` provides global project discovery, multi-environment synchronization,
//! and cryptographic signing for payload integrity across OS boundaries.

pub mod schema;
pub mod crypto;
pub mod error;
