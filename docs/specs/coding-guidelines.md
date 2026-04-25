# Design Specification: jb-ent Coding Guidelines

**Date:** 2026-04-23
**Author:** Jacek Błaszczyński
**Assistance:** Gemini CLI
**Copyright:** © 2026 Jacek Błaszczyński
**License:** AGPL-3.0-only

## 1. Philosophy: Pragmatism over Tooling Dictates
While automated tools like `rustfmt` and `clippy` are valuable for catching common mistakes, they do not dictate the project's ultimate architectural or stylistic truth. We follow a pragmatic approach (inspired by Linux kernel conventions) where **human readability, explicit logic, and rigorous safety** always supersede automated formatting rules. If a tool enforces a less readable or less safe pattern, the tool must be configured to allow the exception, or the exception must be manually maintained.

## 2. Architecture and Separation of Concerns
The project adheres to strict boundaries and encapsulation to ensure long-term maintainability. We blend pragmatic object-oriented design patterns with the safety and purity of functional programming.

*   **Crates (Universal Domains):** Distinct, largely independent domains of functionality must be encapsulated in their own crates (e.g., `jb-identity` vs. `jb-registry`). A crate should represent a universal concept that could theoretically stand alone.
*   **Modules (Sub-Domains):** Within a crate, closely related but distinct sub-domains must be isolated into modules. A module should hide its internal implementation details and only expose a clean, minimal public API.
*   **Traits and Structs (Behaviors and Data):** Use traits to define shared behaviors (interfaces) and structs to encapsulate state. Prefer composition over complex inheritance-like structures. Logic should operate on data immutably where possible, favoring pure functions for core transformations to ease testing and maintenance.

## 3. Naming and Self-Documentation
Code must be self-documenting through its structure and naming conventions.
*   **Descriptive Types:** Structs, Enums, and Traits must accurately reflect their domain and purpose. Avoid generic prefixes/suffixes unless necessary. (e.g., `FfiOpensslError` instead of a generic `FfiError`).
*   **Clear Imports:** Do not use fully qualified inline paths (e.g., `crate::module::submodule::Function()`). Always use `use` statements at the top of the file to bring types and functions into scope, utilizing the shortest unambiguous name.

## 4. Formatting and Readability
Density does not equal quality. We explicitly reject "clever" one-liners that obscure control flow or error handling.
*   **No "One-Line Snakes":** Complex expressions, especially those involving `unsafe` blocks, nested closures, or multiple dereferences, must be broken across multiple lines.
    *   *Bad:* `if !ptr.is_null() { unsafe { free_resource(ptr) } }`
    *   *Good:*
        ```rust
        if !ptr.is_null() { 
            unsafe { 
                free_resource(ptr) 
            } 
        }
        ```
*   **Explicit Scoping:** Use whitespace and block scoping liberally to group related logical operations.

## 5. Documentation and Code Comments
Clear communication is essential for the long-term viability of the project.
*   **Public API Documentation:** Every publicly exported item (functions, structs, enums, traits, macros) *must* be documented using standard Rust documentation comments (`///`). This documentation should explain *what* the item does, its arguments, return values, and potential error conditions.
*   **Implementation Comments:** Any internal logic that is complex, non-obvious, or involves workarounds (e.g., specific FFI behaviors, unusual algorithms, or state mutations that might confuse fellow developers) *must* be accompanied by inline comments (`//`) explaining *why* the implementation was chosen and *how* the logic operates. Code should explain the "what," but comments must explain the "why" when it's not immediately apparent.

## 6. Error Handling and FFI Abstraction
Errors are not just strings; they are critical data structures for application logic and debugging.
*   **No Blanket Errors:** Internal library logic must never return generic catch-all errors (like a blanket `CryptoError`) when a more specific failure state is known.
*   **Exhaustive Variant Mapping:** When interacting with FFI (like OpenSSL), every distinct C-function failure must map to a specific, uniquely identifiable variant in a local Error Enum.
*   **Error Context:** Where possible, capture the underlying system/C error stack (e.g., `openssl::error::ErrorStack`) and embed it within the specific error variant for precise diagnostic reporting.
*   **Boundary Abstraction:** FFI-specific errors must be contained within their module. They should be logged at the module boundary and mapped to a generic domain error (e.g., `RegistryError::CryptoError`) before being returned to the wider application, preventing implementation details from leaking.

## 7. Testing Rigor
Testing must prove both the presence of desired behavior and the explicit handling of all defined failure states.
*   **Strict Isolation:** Single tests must never evaluate multiple distinct conditions or logical branches. Each distinct scenario requires its own dedicated `#[test]` function.
*   **Exhaustive Error Coverage:** For every error variant defined in the codebase (e.g., every enum variant representing a failed FFI call), there MUST be a corresponding, isolated test that reliably triggers that exact failure state and asserts the correct variant is returned. "Happy path" testing alone is insufficient.
*   **Regression Tests First:** Before fixing any reported bug or security vulnerability, you MUST write a deterministic regression test that explicitly reproduces the failure. If the bug is an invisible memory leak or a zeroization failure, you are authorized and expected to use `unsafe` Rust (e.g., raw pointer probing or allocator tracking) to empirically prove the vulnerability. Hackers do not limit themselves to safe Rust, and our security tests must not either. The test must fail before the fix, and pass after the fix.

## 8. Baseline File Formatting (EditorConfig / Git)
To ensure consistency across the repository, the project enforces the following baseline formatting rules for all text files (configured via `.editorconfig` and `.gitattributes`):
*   **Encoding:** All files must be encoded in UTF-8.
*   **Line Endings:** All committed files in the repository must use Unix-style Line Feed (`LF` / `\n`) line endings. What line endings a developer uses locally in their editor is their choice, provided the Git configuration correctly converts them to `LF` upon commit.
*   **Indentation:** Use spaces exclusively. Tabs are strictly forbidden. Standard indentation width is 4 spaces (unless dictated otherwise by specific formats like JSON or YAML, which may use 2).
*   **Trailing Whitespace:** All trailing whitespace at the end of lines must be removed.
*   **Final Newline:** All files must end with a single, empty newline character.
