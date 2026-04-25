# Testing Guidelines for jb-ent

This document consolidates all testing procedures, guidelines, and frameworks used within the **jb-ent** project to improve developer onboarding. It covers both the modern Rust application layer and the high-performance C11 parsing engine.

## 1. Core Testing Philosophy

*   **Test-Driven Development (TDD):** We strictly adhere to TDD principles. You must write a failing test that reproduces a bug or defines a new feature *before* implementing the fix or feature.
*   **Verification Before Completion:** Evidence before claims, always. You must run the tests and verify they pass before making any claims of completion or creating a Pull Request.
*   **Exhaustive Error Coverage:** "Happy path" testing alone is insufficient. Every error variant (especially FFI failures) must have a corresponding, isolated test that reliably triggers the failure state.
*   **No Blanket Errors:** Internal logic must not return generic catch-all errors when specific failure states are known.

## 2. Rust Application Layer Testing (`tests/`)

The primary Rust codebase (orchestration, CLI, MCP server, FFI bridge) uses standard Rust testing tools, but with strict architectural boundaries.

### Rules & Guidelines
1.  **Test Separation:** All Rust tests **MUST** reside in the `tests/` directory (e.g., `tests/identity/`, `tests/registry/`). Inline tests (`#[cfg(test)]` modules within `src/`) are strictly forbidden to ensure clean architectural boundaries.
2.  **Zero Warnings:** All code, including tests, must compile with zero warnings (`cargo clippy --workspace --all-targets -D warnings`).
3.  **Isolation:** Single tests must never evaluate multiple distinct conditions or logical branches. Each distinct scenario requires its own dedicated `#[test]` function.

### Running Rust Tests
To run the entire Rust test suite:
```powershell
cargo test
```

## 3. C11 Engine Testing (`src-c/tests/`)

The legacy C11 core (graph engine, tree-sitter AST extraction, SQLite storage) has its own comprehensive C-based test suite.

### Build Systems (CMake vs. Makefile)
*   **CMake (New, Primary on Windows):** The modern CMake build system (`CMakeLists.txt`) integrates with vcpkg and compiles standalone test executables that link against the core `codebase-memory-mcp-lib`. It is currently the primary testing method on Windows. Adaptation for Unix platforms is planned for the future.
*   **Makefile (Legacy, Primary on Unix/WSL):** The legacy `Makefile.cbm` compiles a unified `test-runner` binary out of `tests/test_main.c`. It remains as-is until the CMake system fully replaces it across all platforms.

### Running C Tests (CMake on Windows)
We use `ctest` to discover and run the standalone test executables defined in CMake.

**1. Clean Build & Configure:**
```powershell
Remove-Item -Recurse -Force build -ErrorAction SilentlyContinue
cmake -B build -S . -G "Visual Studio 18 2026" -A x64 -DCMAKE_TOOLCHAIN_FILE="C:\path\to\your\vcpkgroot\scripts\buildsystems\vcpkg.cmake"
```

**2. Compile & Run:**
```powershell
cmake --build build --config Debug
ctest --test-dir build/src-c/tests -C Debug --output-on-failure
```

### Running C Tests (Makefile on Unix/WSL)
```bash
make -f src-c/Makefile.cbm test
```

## 4. Writing C Tests (Custom Framework)

The C tests utilize a custom, minimal testing framework defined in `src-c/tests/test_framework.h`.

### Framework Structure
*   **Assertions:** Use macros like `ASSERT()`, `ASSERT_EQ()`, `ASSERT_STR_EQ()`, `ASSERT_NULL()`, etc. They provide file and line number context on failure.
*   **Tests:** Define a test using the `TEST(name)` macro. Return `PASS()` on success or `SKIP(reason)` to bypass.
*   **Suites:** Group tests using the `SUITE(name)` macro, calling individual tests with `RUN_TEST(test_name)`.

### Memory Auditing & Initialization
Memory tracking is strictly enforced.
1.  **Initialization:** Every standalone test executable `main()` function *must* call `cbm_mem_init(0, 0.5);` to initialize the project memory tracking (50% RAM budget).
2.  **No Explicit Audits:** Do **not** call `cbm_mem_print_audit()` manually at the end of your test. 
3.  **Automatic Cleanup:** Standalone tests link against `codebase-memory-mcp-lib`, which includes `preprocessor.cpp`. The C++ static destructor (`~CbmMemTrackerReporter`) automatically handles the final memory audit *after* all other static states (like `simplecpp` allocations) have been cleaned up, preventing false-positive leak reports.
4.  **Obligatory Macro Usage:** To enable memory tracking, `CBM_HARDEN_MEMORY` and `MI_OVERRIDE` must be defined at build time. All memory management in tests has to use `CBM_MALLOC`, `CBM_CALLOC`, `CBM_REALLOC`, `CBM_FREE`, and other safe variants allowing for memory tracking. Note that we track only `mimalloc` allocations. Additionally, any C++ code must override the `new` and `delete` operators using these safe memory management functions.
5.  **Memory Safeguard Tests:** Memory safeguard tests follow separate rules dictated by their specific test logic and may bypass standard tracker checks when intentionally testing OOM limits or raw allocators.

### Example C Test Structure
```c
#include "tests/test_framework.h"
#include "foundation/mem.h"

TEST(my_feature_basic) {
    void* ptr = CBM_MALLOC(128);
    ASSERT_NOT_NULL(ptr);
    // ... test logic ...
    CBM_FREE(ptr);
    PASS();
}

SUITE(my_feature) {
    RUN_TEST(my_feature_basic);
}

int main(void) {
#if MI_OVERRIDE
    mi_version();
#endif
    // Initialize memory tracking
    cbm_mem_init(0, 0.5);
    
    RUN_SUITE(my_feature);
    TEST_SUMMARY();
}
```

## 5. Security & Regression Tests

Before fixing any reported bug or security vulnerability, you **MUST** write a deterministic regression test that explicitly reproduces the failure.
*   If the bug is an invisible memory leak or a zeroization failure, you are expected to use `unsafe` Rust (e.g., raw pointer probing or allocator tracking) or direct C memory auditing to empirically prove the vulnerability. 
*   The test must fail *before* the fix, and pass *after* the fix.
 any reported bug or security vulnerability, you **MUST** write a deterministic regression test that explicitly reproduces the failure.
*   If the bug is an invisible memory leak or a zeroization failure, you are expected to use `unsafe` Rust (e.g., raw pointer probing or allocator tracking) or direct C memory auditing to empirically prove the vulnerability. 
*   The test must fail *before* the fix, and pass *after* the fix.
