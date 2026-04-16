# Project jb-ent Scaffold Completeness Checklist

## Root Standards & Configuration
- [x] `.editorconfig` (LF, UTF-8, space-based indentation)
- [x] `.gitattributes` (LF enforcement, legacy path fixes)
- [x] `.gitconfig` (local overrides: diff.ignore-all-space, apply.whitespace)
- [x] `.gitignore` (Rust-focused, tracking Cargo.lock and src-c/.gitignore)
- [x] `LICENSE` (Full AGPL-3.0-only text + attribution)
- [x] `OTHER_LICENSES.md` (SPDX registry + full MIT/BSD texts)
- [x] `README.md` (Project overview, license in footer)
- [x] `Cargo.toml` (v0.1.0, Edition 2024, modernized dependencies, MCP SDK)
- [x] `build.rs` (FFI bridge placeholder)
- [x] `gemini.md` (Updated project mandates v2.0)
- [x] `vcpkg.json` (New project name: jb-ent)

## Documentation (`docs/`)
- [x] `docs/specs/2026-04-16-jb-ent-design.md` (Actual process history)
- [x] `docs/plans/2026-04-16-jb-ent-migration.md` (Actual workflow)
- [x] `docs/plans/jb-ent-completeness-checklist.md` (Current status)

## Rust Orchestration (`src/`)
- [x] `src/main.rs` (Entry point with AGPL headers)
- [x] `src/bridge/mod.rs` (FFI bindings with AGPL headers)
- [x] `src/cli/mod.rs` (CLI handlers with AGPL headers)
- [x] `src/config/mod.rs` (Settings logic with AGPL headers)
- [x] `src/identity/mod.rs` (Project ID logic with AGPL headers)
- [x] `src/mcp/mod.rs` (MCP server with AGPL headers and SDK scaffold)

## Legacy Consolidation (`src-c/`)
- [x] `src-c/` (All legacy directories moved and flattened)
- [x] `src-c/CMakeLists.txt` (Updated internal paths)
- [x] `src-c/tests/CMakeLists.txt` (Fixed relative paths)
- [x] `CMakeLists.txt` (Root file redirected to `src-c/`)
