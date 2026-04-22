# jb-identity v0.9.1 Algorithm Specification

**Status:** Draft / Dogfooding
**Algorithm Version:** 0.9.1
**Date:** 2026-04-21
**Identifier Format:** `v<ALGO_VERSION>:<TYPE>:<ID>`

## 1. Goal
To provide a physically deterministic project identifier that is a **Property of Code**, independent of local paths or OS environments (Windows/Linux/WSL).
With v0.9.1, it introduces **Local Root Accumulation**, remembering historic roots to support transition from shorter to older/universal histories without orphaning graph databases, while preserving local root branch name pairs for human-readable data discovery.

## 2. Resolution Logic (Order of Operations)

### 2.1 The "Gated Upgrade" Rule (Stability & Continuous Convergence)
To ensure identity stability, performance, and convergence, the resolution MUST follow this priority:
1. **Instance Authority:** If `<path>/.jbe/project.json` exists:
    - If the ID is **Local/Shallow** (`v0.9.1:git-shallow:<UUID>`):
        - Ask Git: `repo.is_shallow()`?
        - **If TRUE (Still Shallow):** Use the existing UUID ID.
        - **If FALSE (Complete):** Trigger **Deep Root Discovery** and **UPGRADE** `.jbe/project.json` to the new Universal Hash.
    - If the ID is **Universal** (`v0.9.1:git:<HASH>`):
        - Perform a lightweight **Deep Root Discovery** (Continuous Convergence).
        - If a chronologically older/deeper root is found (e.g., after fetching an older disconnected branch), **UPGRADE** the primary ID to the new Universal Hash.
        - Move the previous `<HASH>` and its associated branch into a `local_roots` array in `.jbe/project.json` to preserve local root branch name pairs. This prevents orphaning existing databases tied to the old root and supports human-readable tracking.

2. **First Discovery:** If no local ID exists:
    - Ask Git: `repo.is_shallow()`?
    - **If NOT Shallow (Full Repo):** Perform **Git Strategy (Deep Root Discovery)** and save to `.jbe/project.json`.
    - **If TRUE (Shallow) or NOT a Git Repo:** Generate a new UUID and save to `.jbe/project.json` with the appropriate prefix.

### 2.2 Git Strategy ("Deep Root Discovery")
1. **Reference Scan:** Iterate through all repository references (`refs/heads/*`, `refs/remotes/*`, `refs/tags/*`).
2. **Root Pathfinding:** For every unique reference, walk back to the absolute root commit(s) (any commit with 0 parents), tracking the branch name it was discovered on.
3. **Chronological Ancestry:** Collect all unique root commit OIDs and their branches found across the entire repository.
4. **Deterministic Tie-Breaker:** Sort roots by **Author Timestamp** (ascending), then by **OID** to guarantee absolute determinism in case of identical timestamps.
5. **Output:** `v0.9.1:git:<OLDEST_OID>` alongside its `branch`.

### 2.3 Local Strategy ("Persistent UUID")
1. **Generation:** Generate a new UUID v4.
2. **Persistence:** Save the UUID to `.jbe/project.json`.
3. **Output:** `v0.9.1:uuid:<UUID>` (or `v0.9.1:git-shallow:<UUID>`)

## 3. Stability & Integrity Guarantees
- **Convergence:** Shallow clones that are later un-shallowed automatically graduate to the Universal ID. Short branches that fetch older histories dynamically accumulate local roots while graduating to the Universal ID.
- **Environment Invariance:** Identical folders mounted differently yield the same ID.
- **Atomic Creation:** On Unix, `.jbe/project.json` creation utilizes a temporary file and `hard_link` to prevent TOCTOU races, while Windows uses `share_mode(0)` for an exclusive kernel lock.
- **Database Integrity:** The storage layer tracks the ID and its `local_roots` (aliases) within the database to prevent orphaned data during deep root upgrades.
