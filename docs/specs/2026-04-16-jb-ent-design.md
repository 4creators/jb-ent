# Design Specification: Project jb-ent (v2.0)

**Date:** 2026-04-16
**Author:** Jacek Błaszczyński
**Assistance:** Gemini CLI
**Copyright:** © 2026 Jacek Błaszczyński
**License:** AGPL-3.0-only

## 1. Goal
Transition the "codebase-memory-mcp" project into a Rust-led, universally identifiable intelligence engine named **jb-ent**. This transition resolves the "Identity Crisis" between Windows/WSL environments, modernizes the configuration system, and establishes a memory-safe foundation for future application layer code.

## 2. Core Identity Strategy: "Universal Path Resolution"
To ensure a stable Project ID across OS boundaries:
1. **Primary ID:** The hash of the first Git commit (`git rev-list --max-parents=0 HEAD`).
2. **Fallback ID:** A UUID generated and stored in `.cbm/project.json` for non-Git projects.
3. **Registry:** A global mapping in `%USERPROFILE%\.cbm\projects.json` that tracks all observed access paths (WSL, Windows, Network) for a given Project ID.

## 3. Architecture Overview

### 3.1 Directory Structure
The project is organized with Rust as the build owner and C11 as the structural analysis core.

```text
jb-ent/
├── .editorconfig         # LF, UTF-8, and indent standards
├── .gitattributes        # eol=lf enforcement
├── .gitconfig            # Local repo overrides
├── Cargo.toml            # Rust build owner
├── build.rs              # FFI bridge configuration
├── LICENSE               # SPDX: AGPL-3.0-only
├── OTHER_LICENSES.md     # SPDX registry for legacy/vendored code
├── README.md             # Project overview
├── .cbm/                 # Local project data (side-by-side with .git)
│   ├── project.json      # Project UUID and Identity
│   └── branch/           # <branch-name>.db stores
├── src/                  # Rust Application Layer (AGPL-3.0-only)
│   ├── main.rs           # Entry point
│   ├── config/           # 3-level JSON settings
│   ├── identity/         # First-commit / UUID resolver
│   ├── mcp/              # MCP server implementation
│   ├── cli/              # CLI command handlers
│   └── bridge/           # C FFI bindings
├── src-c/                # Legacy C, UI, and Extraction
│   ├── pipeline/         # C analysis pipeline
│   ├── store/            # SQLite graph storage
│   ├── ...               # Other C modules (foundation, etc.)
│   ├── graph-ui/         # Visualizer frontend (moved from root)
│   └── internal/         # AST engine and tree-sitter grammars (moved from root)
├── tests/                # Unified C and Rust test suites
├── scripts/              # Build and utility scripts
├── docs/                 # Documentation and specs
└── vendored/             # Third-party dependencies
```

### 3.2 Technical Components
* **Settings System:** Native Rust JSON merging of System, User, and Project levels.
* **Storage Logic:** Deterministic storage in the repository-local `.cbm` directory.
* **Database Naming:** Branch-isolated databases: `<repo>/.cbm/branch/<branch_name>.db`.
* **FFI Bridge:** Rust `build.rs` compiles the `src-c` code (including `internal/`) into a static library linked to the main Rust binary.

## 4. Licensing & Standards
* **New Code:** AGPL-3.0-only (Copyright © 2026 Jacek Błaszczyński).
* **Legacy Code:** MIT (documented in `OTHER_LICENSES.md`).
* **Headers:** SPDX-compliant headers in all copyrightable files using UTF-8 copyright sign.
* **Encodings:** Strict UTF-8 and LF (Unix) line endings.

## 5. Migration Workflow (Actual Execution)
The migration was performed in two distinct phases to maintain a clean history:

### Phase 1: Legacy Reorganization (Committed to fix-msvc-windows-build)
1. **Directory Consolidation:** All legacy C source code, documentation, scripts, and build artifacts were moved into the `src-c/` directory.
2. **Flattening:** The `src/` directory was flattened directly into `src-c/` to avoid redundant nesting.
3. **Build System Update:** The root `CMakeLists.txt` and `src-c/tests/CMakeLists.txt` were updated to correctly reference the new consolidated paths.
4. **Git Metadata Update:** `.gitattributes` was updated to track moved machine-generated files in `src-c/internal/`.
5. **Preservation:** The original `README.md` and `LICENSE` files were moved to `src-c/` for record-keeping.

### Phase 2: Project Initialization (Committed to master)
1. **Clean Slate:** Switched to the new `master` branch.
2. **Scaffolding:** Initialized the Rust project (`Cargo.toml`, `build.rs`, `src/`).
3. **Standards:** Applied `.editorconfig`, `.gitconfig`, and the new AGPL-3.0 `LICENSE`.
4. **Documentation:** Created the new project `README.md` and definitive design/migration documents in `docs/`.
5. **Headers:** Applied SPDX compliant AGPL headers to the new Rust application layer.
6. **Finalization:** Performed a single comprehensive "v0.1.0" commit on the `master` branch.

