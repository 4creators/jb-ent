# jb-identity v0.9 Algorithm Specification

**Status:** Draft / Dogfooding
**Algorithm Version:** 0.9
**Date:** 2026-04-21
**Identifier Format:** `v<ALGO_VERSION>:<TYPE>:<ID>`

## 1. Goal
To provide a physically deterministic project identifier that is a **Property of Code**, independent of local paths or OS environments (Windows/WSL).

## 2. Resolution Logic (Order of Operations)

### 2.1 The "Gated Upgrade" Rule (Stability)
To ensure identity stability and performance, the resolution MUST follow this strict priority:
1. **Instance Authority:** If `<path>/.jbe/project.json` exists:
    - If the ID is **Universal** (`v0.9:git:<HASH>`), use it immediately.
    - If the ID is **Local/Shallow** (`v0.9:git-shallow:<UUID>`):
        - Ask Git: `repo.is_shallow()`?
        - **If TRUE (Still Shallow):** Use the existing UUID ID.
        - **If FALSE (Complete):** Trigger **Deep Root Discovery** and **UPGRADE** `.jbe/project.json` to the new Universal Hash.
2. **First Discovery:** If no local ID exists:
    - Ask Git: `repo.is_shallow()`?
    - **If NOT Shallow (Full Repo):** Perform **Git Strategy (Deep Root Discovery)** and save to `.jbe/project.json`.
    - **If TRUE (Shallow) or NOT a Git Repo:** Generate a new UUID and save to `.jbe/project.json` with the appropriate prefix.

### 2.2 Git Strategy ("Deep Root Discovery")
1. **Reference Scan:** Iterate through all repository references (`refs/heads/*`, `refs/remotes/*`, `refs/tags/*`).
2. **Root Pathfinding:** For every unique reference, walk back to the absolute root commit(s) (any commit with 0 parents).
3. **Chronological Ancestry:** Collect all unique root commit OIDs found across the entire repository.
4. **Deterministic Tie-Breaker:** Sort roots by **Author Timestamp** (ascending).
5. **Output:** `v0.9:git:<OLDEST_OID>`

### 2.3 Local Strategy ("Persistent UUID")
1. **Generation:** Generate a new UUID v4.
2. **Persistence:** Save the UUID to `.jbe/project.json` as `{"id": "<UUID>"}`.
3. **Output:** `v0.9:uuid:<UUID>` (or `v0.9:git-shallow:<UUID>`)

## 3. Stability & Integrity Guarantees
- **Convergence:** Shallow clones that are later un-shallowed automatically graduate to the Universal ID.
- **Environment Invariance:** Identical folders mounted differently yield the same ID.
- **Database Integrity:** The storage layer MUST track the `indexed_commit_hash` within the database (e.g., `master.db`) to detect stale graph data.
