# Contributing to jb-ent

Contributions are welcome. This guide covers setup, testing, and PR guidelines for the **v2.0 Rust-led architecture**.

## Project Structure

This project employs a hybrid architecture combining a modern Rust application layer with a high-performance C11 parsing engine. The repository is structured as a unified Cargo Workspace:

### Rust Application Layer (`src/`)
The primary Rust codebase acts as the entry point, CLI router, and high-level protocol server.
- `src/main.rs` & `src/lib.rs`: The core `jb-ent` executable and library.
- `src/cli/`: Command-line interface definitions (powered by `clap`).
- `src/mcp/`: Model Context Protocol server implementation.
- `src/bridge/`: FFI bindings to the underlying C11 engine.
- `src/config/`: Workspace and system configuration management.

#### Internal Crates
To enforce strict architectural boundaries, distinct domains are isolated into independent Cargo crates, physically housed within the central `src/` directory:
- `src/identity/`: (`jb-identity`) Deterministic project identity resolution and branch tracking.
- `src/registry/`: (`jb-registry`) Global project registry, cross-platform OpenSSL cryptography (ML-DSA), and environment synchronization.

### Legacy C11 Core (`src-c/`)
The high-performance graph engine, built in C11. Linked to the Rust layer via FFI.
- Handles tree-sitter AST extraction, graph buffer management, and SQLite storage.
- Contains its own isolated CMake build system and vendored dependencies.
- Includes its own comprehensive C-based test suite (`src-c/tests/`).

### Testing (`tests/`)
All integration and regression tests for the Rust workspace.
- `tests/identity/`: Integration tests for the `jb-identity` crate.
- `tests/registry/`: Integration tests for the `jb-registry` crate (including deterministic cryptographic regression tests).

### Documentation (`docs/`)
- `docs/specs/`: Formal design specifications, architectural records, and coding guidelines (`coding-guidelines.md`).
- `docs/plans/`: Historical execution plans and checklists.

## Build from Source

**Prerequisites**:
- Rust toolchain (Cargo)
- **Windows:** Visual Studio 2022 or Visual Studio 18 2026 with C++ workloads. Both generators are supported.
- **Windows:** Visual Studio inbox vcpkg (for resolving external C dependencies) OR a local checkout of our custom `vcpkg` fork containing the OpenSSL 4.0.0 port.

### OpenSSL 4.0 and Vcpkg Environment

This project relies on a custom fork of `vcpkg` to provide OpenSSL 4.0.0 for the Rust cryptography abstraction layer. To successfully compile the `openssl-sys` dependency on Windows, you must configure Cargo to use this specific vcpkg root. 

Copy the provided `.cargo/config.toml.example` file to `.cargo/config.toml` and edit the `VCPKG_ROOT` variable to point to your local vcpkg fork:

```toml
[env]
# Point to your local custom vcpkg fork containing OpenSSL 4.0.0
VCPKG_ROOT = { value = "C:\\path\\to\\your\\vcpkgroot", force = true }

[target.'cfg(windows)'.env]
# Tell the vcpkg rust crate to link dynamically on Windows
VCPKGRS_DYNAMIC = { value = "1", force = true }
```

### Windows Build Quirks & Dependencies

On Windows, the C core relies on CMake and vcpkg. While some dependencies are available in the `src-c/vendored/` directory (e.g., `sqlite3`, `mimalloc`, `xxhash`), the CMake build system is configured to resolve them via vcpkg (`lz4`, `libgit2`, `sqlite3`, `xxhash`, `mimalloc`).

To build the entire project (Rust wrapper + C core via `build.rs`):
```powershell
cargo build
```

To manually configure and build the C core and its test suite using CMake, you must specify the correct generator (`-G`) and the path to your inbox vcpkg toolchain file. You can also optionally specify the architecture (`-A`).

**CMake Options Explained:**
- `-G`: Specifies the generator (e.g., `"Visual Studio 18 2026"`, `"Visual Studio 17 2022"`, or `"Ninja"`). Required.
- `-A`: Specifies the target architecture (e.g., `x64`, `ARM64`). Optional. If omitted, it defaults to the host architecture (typically `x64` on modern Windows). Note: Ignored if using the Ninja generator.
- `-DCMAKE_TOOLCHAIN_FILE`: Points to the `vcpkg.cmake` script. Required for vcpkg dependency resolution.

**For VS 2026:**
```powershell
cmake -B build -S . -G "Visual Studio 18 2026" -A x64 -DCMAKE_TOOLCHAIN_FILE="C:\Program Files\Microsoft Visual Studio\18\{Edition}\VC\vcpkg\scripts\buildsystems\vcpkg.cmake"
cmake --build build --config Debug
```

**For VS 2022:**
```powershell
cmake -B build -S . -G "Visual Studio 17 2022" -A x64 -DCMAKE_TOOLCHAIN_FILE="C:\Program Files\Microsoft Visual Studio\2022\{Edition}\VC\vcpkg\scripts\buildsystems\vcpkg.cmake"
cmake --build build --config Debug
```

**For Ninja (Must run in "x64 Native Tools Command Prompt for VS"):**
```powershell
cmake -B build-ninja -S . -G "Ninja" -DCMAKE_BUILD_TYPE=Debug -DCMAKE_TOOLCHAIN_FILE="C:\Program Files\Microsoft Visual Studio\2022\{Edition}\VC\vcpkg\scripts\buildsystems\vcpkg.cmake"
cmake --build build-ninja
```
*(Replace `{Edition}` with your installed Visual Studio edition: Enterprise, Professional, Community, or Preview, or provide the path to your custom vcpkg fork).*

## Strict Quality Gates and TDD

We follow strict **Test-Driven Development (TDD)** and enforce the following rules:

1. **Test Separation**: All Rust tests **MUST** reside in the `tests/` directory. Inline tests (`#[cfg(test)]` modules within `src/`) are strictly forbidden. If you are modifying existing code that contains inline tests, you must move them to the `tests/` directory before your PR can be accepted.
2. **Zero Warnings**: All code MUST compile with zero warnings. We do not accept PRs with compiler or linter warnings.
   ```powershell
   cargo clippy --workspace --all-targets -D warnings
   ```

## Run Tests

### Rust Tests
```powershell
cargo test
```

### C Core Tests
After building the C core with CMake, run the C test suite:
```powershell
ctest --test-dir build/src-c/tests -C Debug --output-on-failure
```

## Pull Request Guidelines

- **Open an issue first**: Discuss architectural changes, new MCP tools, or major refactors before implementing.
- **Commit Messages**: All commit messages must follow the **Linux kernel commit message standard**. Provide a short, summary line (under 50 characters), followed by a blank line, and then a detailed explanation of *why* the change was made. Include `Signed-off-by:` lines if required.
- **Licensing**: All new code must be licensed under **AGPL-3.0-only**. Include the following SPDX header at the top of all new files (replace with your own copyright details):
  ```rust
  // SPDX-License-Identifier: AGPL-3.0-only
  // Copyright © YYYY Your Name/Organization
  ```
