# Contributing to jb-ent

Contributions are welcome. This guide covers setup, testing, and PR guidelines for the **v2.0 Rust-led architecture**.

## Project Structure

This project employs a hybrid architecture:
- **Rust Orchestration Layer** (`src/`): Manages CLI (`clap`), configuration (`serde`), identity, and the MCP protocol (`rmcp`).
- **Legacy C11 Core** (`src-c/`): High-performance tree-sitter AST extraction, graph buffer, and SQLite storage. Linked via FFI.

## Build from Source

**Prerequisites**:
- Rust toolchain (Cargo)
- **Windows:** Visual Studio 2022 or Visual Studio 18 2026 with C++ workloads. Both generators are supported.
- **Windows:** Visual Studio inbox vcpkg (for resolving external C dependencies).

### Windows Build Quirks & Dependencies

On Windows, the C core relies on CMake and vcpkg. While some dependencies are available in the `src-c/vendored/` directory (e.g., `sqlite3`, `mimalloc`, `xxhash`), the CMake build system is configured to resolve them via vcpkg (`lz4`, `libgit2`, `sqlite3`, `xxhash`, `mimalloc`).

To build the entire project (Rust wrapper + C core via `build.rs`):
```powershell
cargo build
```

To manually configure and build the C core and its test suite using CMake, you must specify the correct generator (`-G`) and the path to your inbox vcpkg toolchain file. You can also optionally specify the architecture (`-A`).

**CMake Options Explained:**
- `-G`: Specifies the generator (e.g., `"Visual Studio 18 2026"` or `"Visual Studio 17 2022"`). Required.
- `-A`: Specifies the target architecture (e.g., `x64`, `ARM64`). Optional. If omitted, it defaults to the host architecture (typically `x64` on modern Windows).
- `-DCMAKE_TOOLCHAIN_FILE`: Points to the `vcpkg.cmake` script. Required for vcpkg dependency resolution.

**For VS 2026:**
```powershell
cmake -B build -S . -G "Visual Studio 18 2026" -A x64 -DCMAKE_TOOLCHAIN_FILE="C:\Program Files\Microsoft Visual Studio\18\Preview\VC\vcpkg\scripts\buildsystems\vcpkg.cmake"
cmake --build build --config Debug
```

**For VS 2022:**
```powershell
cmake -B build -S . -G "Visual Studio 17 2022" -A x64 -DCMAKE_TOOLCHAIN_FILE="C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\vcpkg\scripts\buildsystems\vcpkg.cmake"
cmake --build build --config Debug
```
*(Adjust the toolchain path depending on your VS installation edition: Enterprise, Professional, Community, or Preview).*

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
