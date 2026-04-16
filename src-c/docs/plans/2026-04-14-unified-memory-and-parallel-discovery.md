# Unified Memory Context & Parallel Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a thread-safe, deduplicating memory management system (`CBMContext`) and a high-performance parallel directory walker to optimize repository indexing on Windows.

**Architecture:**
- **Unified Heap:** Redirect all allocations (including SQLite) to audited `mimalloc` wrappers.
- **CBMContext:** A heap-based manager for thread-safe path interning and bulk freeing.
- **Parallel Discovery:** A two-phase system (Fast Walk → Background Dedup) using the worker pool (capped at 50% CPU).
- **Hierarchical Paths:** Store paths as interned relative directory prefixes + filenames to reduce RAM.

**Tech Stack:** C11, `mimalloc`, `worker_pool`, SQLite.

---

### Task 1: SQLite Mimalloc Redirection

**Files:**
- Modify: `src/foundation/mem.c`
- Modify: `src/foundation/mem.h`
- Modify: `src/main.c`

- [ ] **Step 1: Implement sqlite3_mem_methods wrappers in `mem.c`**

```c
#include <sqlite3.h>

static void *cbm_sqlite_malloc(int n) { return CBM_MALLOC((size_t)n); }
static void cbm_sqlite_free(void *p) { CBM_FREE(p); }
static void *cbm_sqlite_realloc(void *p, int n) { return CBM_REALLOC(p, (size_t)n); }
static int cbm_sqlite_size(void *p) { return (int)mi_usable_size(p); }
static int cbm_sqlite_roundup(int n) { return n; }
static int cbm_sqlite_init(void *p) { (void)p; return SQLITE_OK; }
static void cbm_sqlite_shutdown(void *p) { (void)p; }

static sqlite3_mem_methods cbm_sqlite_mem = {
    cbm_sqlite_malloc, cbm_sqlite_free, cbm_sqlite_realloc,
    cbm_sqlite_size, cbm_sqlite_roundup, cbm_sqlite_init,
    cbm_sqlite_shutdown, NULL
};

void cbm_mem_hook_sqlite(void) {
    sqlite3_config(SQLITE_CONFIG_MALLOC, &cbm_sqlite_mem);
}
```

- [ ] **Step 2: Declare `cbm_mem_hook_sqlite` in `mem.h`**
- [ ] **Step 3: Update `main.c` to call hook before `cbm_mem_init`**
- [ ] **Step 4: Verify that `MEMORY AUDIT` now includes SQLite allocations (test by opening a temp DB)**
- [ ] **Step 5: Commit changes**

---

### Task 2: CBMContext Core Implementation

**Files:**
- Create: `src/foundation/mem_context.h`
- Create: `src/foundation/mem_context.c`

- [ ] **Step 1: Define CBMContext with mi_heap_t and locking**

```c
typedef struct {
    mi_heap_t *heap;
    #ifdef _WIN32
    SRWLOCK lock;
    #else
    pthread_rwlock_t lock;
    #endif
    CBMInternPool *intern_pool;
} CBMContext;
```

- [ ] **Step 2: Implement `cbm_ctx_new`, `cbm_ctx_free`, `cbm_ctx_alloc`**
- [ ] **Step 3: Implement `cbm_ctx_intern` (thread-safe wrapper around existing interner)**
- [ ] **Step 4: Write unit test in `tests/test_mem_context.c`**
- [ ] **Step 5: Verify tests pass and memory is audited**
- [ ] **Step 6: Commit changes**

---

### Task 3: Parallel Directory Discovery

**Files:**
- Create: `src/discover/discover_optimized.c`
- Modify: `src/discover/discover.h`

- [ ] **Step 1: Implement `work_stealing_queue` for directory traversal**
- [ ] **Step 2: Implement Phase 1: Parallel walk using 50% logical cores**
- [ ] **Step 3: Implement Phase 2: Background thread for path interning and list merging**
- [ ] **Step 4: Integrate hierarchical path storage (Repo Root + Interned Rel Dir + Filename)**
- [ ] **Step 5: Add benchmark test to compare old vs new discovery**
- [ ] **Step 6: Commit changes**

---

### Task 4: Integration & Optimization Toggle

**Files:**
- Modify: `src/pipeline/pipeline.c`
- Modify: `CMakeLists.txt` / `Makefile.cbm`

- [ ] **Step 1: Wrap new logic in `#ifdef CBM_ENABLE_OPTIMIZED`**
- [ ] **Step 2: Update `cbm_pipeline_new` to use optimized discovery if flag is set**
- [ ] **Step 3: Final end-to-end verification on codebase-memory-mcp repository itself**
- [ ] **Step 4: Commit and finalize feature**

---

### Task 5: Hard Circuit Breaker (OOM Safeguard)

**Files:**
- Modify: `src/foundation/mem.c`

- [ ] **Step 1: Fix Windows Memory Tracking**
  Update `os_rss` on Windows to use `PagefileUsage` (Commit Charge) instead of `WorkingSetSize`. This prevents the OS from freezing by ensuring the graceful cancellation check correctly sees the full memory pressure even if the OS starts paging aggressively.
- [ ] **Step 2: Add Hard Circuit Breaker at 120% Budget**
  Implement `check_circuit_breaker(size_t request_size)` inside every allocation wrapper (`cbm_malloc_safe`, `cbm_strdup_safe`, etc.).
  - 100% budget triggers graceful pipeline shutdown (Layer 1).
  - 120% budget triggers an instant hard abort (Layer 2). This "buffer zone" allows the pipeline to finish processing the current file gracefully, but steps in to instantly terminate the process if a runaway allocation loop blows past the buffer, protecting the OS from total freeze.
- [ ] **Step 3: Support Exact Memory Budgets via Environment Variables**
  Add `CBM_BUDGET_MB` and `CBM_RAM_FRACTION` environment variable support to `src/main.c`. 
  - `CBM_BUDGET_MB` takes absolute precedence: if set to a valid integer > 0, it forces the budget to that exact megabyte amount (e.g., `7700` for 7.7GB).
  - `CBM_RAM_FRACTION` acts as a fallback (between 0.0 and 2.0) to calculate the budget as a fraction of total physical RAM (default is 0.5). Both strip leading quotes to handle cross-platform shell quirks.
- [ ] **Step 4: Commit and finalize safeguard**

## Future Work
There are further deduplication opportunities for handling repo filesystem data, but this is left for future work.
