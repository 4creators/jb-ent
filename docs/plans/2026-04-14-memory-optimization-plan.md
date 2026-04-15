# Memory Optimization Implementation Plan

**Goal:** Dramatically reduce the graph buffer memory footprint by replacing string-based indexing with integer hash tables, packed keys, and string interning.

**Architecture:** We will implement an open-addressing integer hash table (`cbm_int_ht_t`) and edge set (`cbm_edge_set_t`), inject a `CBMInternPool` into `cbm_gbuf_t`, and transition all node/edge indexes to use raw `uint64_t` keys (or packed `src << 16 | type_id` keys). Interned string pointers will also be cast to `uint64_t` for O(1) label/name indexing.

**Tech Stack:** C11, Robin Hood Hashing, xxHash (`XXH3_64bits`), CMake, Unity-style test framework.

---

### Task 1: Implement Integer Hash Table (`cbm_int_ht_t`)

We need a fast, open-addressing hash table mapping `uint64_t` to `void*`.

**Files:**
- Create: `src/foundation/int_hash_table.h`
- Create: `src/foundation/int_hash_table.c`
- Create: `tests/test_int_hash_table.c`
- Modify: `src/CMakeLists.txt`
- Modify: `tests/CMakeLists.txt`
- Modify: `tests/test_main.c`

- [ ] **Step 1: Write the interface and failing test**

`src/foundation/int_hash_table.h`:
```c
#ifndef CBM_INT_HASH_TABLE_H
#define CBM_INT_HASH_TABLE_H

#include <stdint.h>
#include <stdbool.h>

typedef struct {
    uint64_t key;
    void *value;
    uint32_t psl; /* probe sequence length */
} cbm_int_ht_entry_t;

typedef struct {
    cbm_int_ht_entry_t *entries;
    uint32_t capacity;
    uint32_t count;
    uint32_t mask;
} cbm_int_ht_t;

cbm_int_ht_t *cbm_int_ht_create(uint32_t initial_capacity);
void cbm_int_ht_free(cbm_int_ht_t *ht);
void *cbm_int_ht_set(cbm_int_ht_t *ht, uint64_t key, void *value);
void *cbm_int_ht_get(const cbm_int_ht_t *ht, uint64_t key);
void *cbm_int_ht_delete(cbm_int_ht_t *ht, uint64_t key);
bool cbm_int_ht_has(const cbm_int_ht_t *ht, uint64_t key);

#endif
```

`tests/test_int_hash_table.c`:
```c
#include "test_framework.h"
#include "foundation/int_hash_table.h"

TEST(int_ht_basic) {
    cbm_int_ht_t *ht = cbm_int_ht_create(16);
    ASSERT_NOT_NULL(ht);
    
    int val1 = 42;
    int val2 = 99;
    
    cbm_int_ht_set(ht, 12345, &val1);
    cbm_int_ht_set(ht, 67890, &val2);
    
    ASSERT_EQ(cbm_int_ht_get(ht, 12345), &val1);
    ASSERT_EQ(cbm_int_ht_get(ht, 67890), &val2);
    ASSERT_NULL(cbm_int_ht_get(ht, 11111));
    
    ASSERT_EQ(cbm_int_ht_delete(ht, 12345), &val1);
    ASSERT_NULL(cbm_int_ht_get(ht, 12345));
    
    cbm_int_ht_free(ht);
    PASS();
}

SUITE(int_hash_table) {
    RUN_TEST(int_ht_basic);
}
```

- [ ] **Step 2: Update CMakeLists and verify test fails**

Add `int_hash_table.c` to `src/CMakeLists.txt` under `FOUNDATION_SRCS`.
Add `test_int_hash_table.c` to `tests/CMakeLists.txt` and register the suite in `tests/test_main.c`.
Run `cmake --build build` and `build/tests/Release/run_test_fqn.exe` (or equivalent) to see linker errors or test failure.

- [ ] **Step 3: Implement `cbm_int_ht_t`**

`src/foundation/int_hash_table.c`: Implement Robin Hood hashing with `XXH3_64bits` or a fast integer mixer (e.g. `key * 0x9E3779B185EBCA87ULL`).

- [ ] **Step 4: Run tests to verify PASS**

Run tests and ensure `int_ht_basic` passes.

- [ ] **Step 5: Commit**

```bash
git add src/foundation/int_hash_table.* tests/test_int_hash_table.c src/CMakeLists.txt tests/CMakeLists.txt tests/test_main.c
git commit -m "feat: implement cbm_int_ht_t for zero-allocation integer hashing"
```

---

### Task 2: Implement Edge Hash Set (`cbm_edge_set_t`)

We need a set to deduplicate edges based on a compact 18-byte key.

**Files:**
- Create: `src/foundation/edge_set.h`
- Create: `src/foundation/edge_set.c`
- Create: `tests/test_edge_set.c`
- Modify: `src/CMakeLists.txt`, `tests/CMakeLists.txt`, `tests/test_main.c`

- [ ] **Step 1: Write the interface and failing test**

`src/foundation/edge_set.h`:
```c
#ifndef CBM_EDGE_SET_H
#define CBM_EDGE_SET_H

#include <stdint.h>
#include <stdbool.h>

#pragma pack(push, 1)
typedef struct {
    uint64_t src;
    uint64_t tgt;
    uint16_t type_id;
} cbm_edge_key_t;
#pragma pack(pop)

typedef struct cbm_gbuf_edge cbm_gbuf_edge_t; // Forward decl

typedef struct {
    cbm_edge_key_t key;
    cbm_gbuf_edge_t *edge;
    uint32_t psl;
} cbm_edge_set_entry_t;

typedef struct {
    cbm_edge_set_entry_t *entries;
    uint32_t capacity;
    uint32_t count;
    uint32_t mask;
} cbm_edge_set_t;

cbm_edge_set_t *cbm_edge_set_create(uint32_t initial_capacity);
void cbm_edge_set_free(cbm_edge_set_t *set);
void cbm_edge_set_insert(cbm_edge_set_t *set, cbm_edge_key_t key, cbm_gbuf_edge_t *edge);
cbm_gbuf_edge_t *cbm_edge_set_get(const cbm_edge_set_t *set, cbm_edge_key_t key);
void cbm_edge_set_delete(cbm_edge_set_t *set, cbm_edge_key_t key);

#endif
```

Write a basic test in `tests/test_edge_set.c` that inserts two edge keys and retrieves them.

- [ ] **Step 2: Verify test fails**

- [ ] **Step 3: Implement `cbm_edge_set_t`**

Use `XXH3_64bits(&key, sizeof(key), 0)` to hash the 18-byte struct in `src/foundation/edge_set.c` using Robin Hood probing. 

- [ ] **Step 4: Run test to verify passes**

- [ ] **Step 5: Commit**

```bash
git add src/foundation/edge_set.* tests/test_edge_set.c src/CMakeLists.txt tests/CMakeLists.txt tests/test_main.c
git commit -m "feat: implement cbm_edge_set_t for fast edge deduplication"
```

---

### Task 3: Integrate CBMInternPool into Graph Buffer

**Files:**
- Modify: `src/graph_buffer/graph_buffer.h`
- Modify: `src/graph_buffer/graph_buffer.c`

- [ ] **Step 1: Add Intern Pool and Integer Hash Tables**

Update `struct cbm_gbuf` in `graph_buffer.c`:
- Add `CBMInternPool *intern_pool;`
- Add `cbm_int_ht_t *edge_type_to_id;` (map interned type string ptr to `uint16_t`)
- Add `uint16_t next_type_id;`

Update `cbm_gbuf_new()` to initialize these, and `cbm_gbuf_free()` to free them. Remove `heap_strdup` helper.

- [ ] **Step 2: Update Node Creation to Use Interning**

In `cbm_gbuf_upsert_node`:
- `node->label = cbm_intern(gb->intern_pool, label);`
- Do the same for `name`, `file_path`, `properties_json`, `qualified_name`.

Update `free_node_strings` to do **nothing** (or remove it entirely), because strings are owned by the intern pool.

- [ ] **Step 3: Update Edge Creation to Use Interning and Type IDs**

In `cbm_gbuf_upsert_edge`:
- `edge->type = cbm_intern(gb->intern_pool, type);`
- `edge->properties_json = cbm_intern(gb->intern_pool, properties_json);`

Update `free_edge_strings` to do nothing.

- [ ] **Step 4: Build and test**

Run `tests/test_graph_buffer.c` (or run all tests) to ensure interning doesn't break string retrieval.

- [ ] **Step 5: Commit**

```bash
git add src/graph_buffer/graph_buffer.c
git commit -m "refactor(graph_buffer): replace heap_strdup with cbm_intern"
```

---

### Task 4: Switch Graph Buffer Node Indexes to Integer Hash Tables

**Files:**
- Modify: `src/graph_buffer/graph_buffer.c`

- [ ] **Step 1: Replace Node ID hash table**

Change `CBMHashTable *node_by_id;` to `cbm_int_ht_t *node_by_id;`
In `cbm_gbuf_upsert_node` and `unindex_node`, use `cbm_int_ht_set(gb->node_by_id, node->id, node);` instead of formatting `snprintf` strings.

- [ ] **Step 2: Replace Label and Name hash tables**

Change `nodes_by_label` and `nodes_by_name` to `cbm_int_ht_t *`.
Since `node->label` is an interned string pointer, we can cast it to `uint64_t`:
`uint64_t label_key = (uint64_t)(uintptr_t)node->label;`

Update `cbm_gbuf_find_by_label` and `cbm_gbuf_find_by_name` to intern the query string first (if it's not already interned, it won't exist in the table, but `cbm_intern` handles that), then cast to `uint64_t` to lookup the `node_ptr_array_t`.

- [ ] **Step 3: Build and test**

Run all tests to ensure node retrieval by ID, label, and name still works perfectly.

- [ ] **Step 4: Commit**

```bash
git add src/graph_buffer/graph_buffer.c
git commit -m "refactor(graph_buffer): transition node indexes to cbm_int_ht_t"
```

---

### Task 5: Switch Graph Buffer Edge Indexes to Integer Hash Tables

**Files:**
- Modify: `src/graph_buffer/graph_buffer.c`

- [ ] **Step 1: Replace Edge Hash Set**

Change `CBMHashTable *edge_by_key;` to `cbm_edge_set_t *edge_by_key;`.
In `cbm_gbuf_upsert_edge`, get or assign the 16-bit `type_id` for the edge type:
```c
uint16_t type_id = (uint16_t)(uintptr_t)cbm_int_ht_get(gb->edge_type_to_id, (uint64_t)(uintptr_t)edge->type);
if (!type_id) {
    type_id = ++gb->next_type_id;
    cbm_int_ht_set(gb->edge_type_to_id, (uint64_t)(uintptr_t)edge->type, (void*)(uintptr_t)type_id);
}
```
Construct `cbm_edge_key_t key = {source_id, target_id, type_id}` and check/insert into `edge_by_key`.

- [ ] **Step 2: Replace Edge Source/Target indexes**

Change `edges_by_source_type` and `edges_by_target_type` to `cbm_int_ht_t`.
Construct packed keys: `uint64_t src_key = ((uint64_t)source_id << 16) | type_id;`
Use this to lookup and push to the `edge_ptr_array_t`.

- [ ] **Step 3: Update `cbm_gbuf_find_edges`**

Update `cbm_gbuf_find_edges` and deletion functions to use the integer tables and packed keys instead of `snprintf`.

- [ ] **Step 4: Build and test**

Run all tests. `ctest` or `./build/tests/Release/run_test_fqn.exe` to ensure edge traversal and incremental deletion passes.

- [ ] **Step 5: Commit**

```bash
git add src/graph_buffer/graph_buffer.c
git commit -m "refactor(graph_buffer): transition edge indexes to packed integers and edge set"
```

---

### Task 6: Final Cleanup and Memory Verification

**Files:**
- Modify: `src/graph_buffer/graph_buffer.c`

- [ ] **Step 1: Verify `cbm_gbuf_dump_to_sqlite`**

Ensure that `cbm_gbuf_dump_to_sqlite` does not rely on any string hash tables. Since it iterates over `gb->nodes` and `gb->edges` arrays directly, it should work perfectly with the new internal indexes.

- [ ] **Step 2: Verify `cbm_gbuf_free`**

Ensure `cbm_gbuf_free` calls `cbm_int_ht_free` and `cbm_edge_set_free` on all new structures, and frees the `CBMInternPool`. Ensure `free_node_strings` and `free_edge_strings` are completely removed to prevent double-freeing interned strings.

- [ ] **Step 3: Run OOM Stress Test**

Run the PowerShell OOM test or the standard test suite to ensure zero memory leaks and massive reduction in peak allocation.

- [ ] **Step 4: Commit**

```bash
git commit -a -m "test: verify graph buffer memory optimization and cleanup"
```
