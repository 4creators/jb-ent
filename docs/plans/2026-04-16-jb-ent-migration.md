# Project jb-ent Migration Implementation Plan (Consolidated Commit)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transition the codebase to a Rust-led architecture named **jb-ent** with a stable identity and standardized configuration, resulting in a single "v2.0" commit on the master branch.

**Architecture:** Rust-first application layer with a legacy C11 structural analysis core linked via FFI.

**Tech Stack:** Rust (Cargo), C11, libgit2, serde_json, SPDX-headers.

---

### Phase 1: Workspace & Directory Reorganization (on current branch)

**Goal:** Prepare the physical structure of the repository while preserving git history.

- [x] **Step 1: Create the consolidation directory**
  Action: `mkdir src-c`

- [x] **Step 2: Consolidate legacy codebase**
  Action: Moved `src/`, `graph-ui/`, `internal/`, `tests/`, `scripts/`, `docs/`, `vendored/`, `test-infrastructure/`, and `tools/` into `src-c/`.

- [x] **Step 3: Flatten structure**
  Action: Moved `src-c/src/*` to `src-c/` and corrected legacy filenames (`src-main.c` -> `main.c`).

- [x] **Step 4: Update Build System**
  Action: Modified root `CMakeLists.txt` and `src-c/tests/CMakeLists.txt` to point to the new `src-c/` locations.

- [x] **Step 5: Preservation**
  Action: Moved original `README.md` and `LICENSE` to `src-c/`.

- [x] **Step 6: Commit Reorganization**
  Action: `git commit -m "chore: complete legacy reorganization and update CMake for src-c"` (on branch `fix-msvc-windows-build`).

---

### Phase 2: Master Branch & Standards Scaffolding (No intermediate commits)

**Goal:** Set up the new branch and project standards for **jb-ent**.

- [x] **Step 1: Create master branch**
  Action: `git checkout -b master`

- [x] **Step 2: Create Standards Files**
  Action: Created `.editorconfig`, `.gitattributes`, and `.gitconfig` (UTF-8, LF).

- [x] **Step 3: Initialize Licenses**
  Action: Created new root `LICENSE` (AGPL-3.0-only) and `OTHER_LICENSES.md`.

---

### Phase 3: Rust Project Scaffolding (No intermediate commits)

**Goal:** Initialize the new application layer.

- [x] **Step 1: Initialize Cargo.toml**
  Action: Created `Cargo.toml` (v0.1.0, Edition 2024).

- [x] **Step 2: Scaffold src/ directory**
  Action: Created `src/main.rs`, `src/config/`, `src/identity/`, `src/mcp/`, `src/cli/`, and `src/bridge/`.

---

### Phase 4: Standardization & License Headers

**Goal:** Ensure every file meets the new project standards.

- [x] **Step 1: Apply SPDX headers**
  Action: Prepend SPDX AGPL headers to all new Rust files.

- [x] **Step 2: Final Audit**
  Action: Verified LF line endings and UTF-8 encoding across all root files.

---

### Phase 5: The "Big Bang" Master Commit

**Goal:** Finalize the transition with a single comprehensive commit.

- [ ] **Step 1: Stage all changes**
  Run: `git add .`

- [ ] **Step 2: Create the first master commit**
  Run: `git commit -m "feat: initialize jb-ent master branch with Rust application layer, unified identity, and AGPLv3 standards"`

- [ ] **Step 3: Verification**
  Run: `cargo build` and verify the directory structure is exactly as designed.
