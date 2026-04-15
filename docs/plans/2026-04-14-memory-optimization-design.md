# Memory Optimization: Integer Hash Tables & String Interning

## 1. Information Flow & Processing Stages (Analysis)

To understand the necessity and impact of memory optimization, we must first clearly outline how data moves through the pipeline, how it changes form, and the impact of the current vs. proposed architecture at each stage:

### Stage 1: Extraction & Parsing (Creation)
- **Current Flow:** AST nodes are extracted. Integer IDs are converted to strings via `snprintf`. Strings are hashed (FNV-1a). Millions of strings live on the heap.
- **New Flow:** `cbm_gbuf_new()` initializes the integer hash tables and the intern pool. Parsing extracts AST nodes; strings are passed to `cbm_intern()` (deduplicating them instantly). Edges are assigned a 16-bit `type_id`. Structs are indexed using raw/packed `uint64_t` integers.
- **Throughput Impact:** Massive CPU speedup. We bypass `snprintf`, `strlen`, and string hashing overheads, and drastically reduce CPU cache misses by using flat integer arrays (Robin Hood probing).

### Stage 2: SQLite Dump (Persistence)
- **Current Flow:** `cbm_gbuf_dump_to_sqlite` iterates over string-based hash tables, builds `CBMDumpNode` structs, and hands them to the fast B-tree SQLite writer.
- **New Flow:** The graph buffer iterates over the integer HTs and interned strings to build the identical `CBMDumpNode` arrays. The final data handed to SQLite is **bit-for-bit identical** to the old system.
- **Throughput Impact:** The transition from parsing to dumping becomes practically instantaneous. `cbm_gbuf_free()` destroys the intern pool and integer HT arrays in an `O(1)` sweep, eliminating the CPU stalls previously caused by `free()`-ing millions of individual heap strings.

### Stage 3: Incremental Updates (Deletion)
- **Current Flow:** Loads existing nodes, matches file paths, removes nodes/edges from string tables, and calls `CBM_FREE` on strings and structs.
- **New Flow:** The buffer calls `cbm_gbuf_delete_by_file()`. Nodes/edges are removed from the integer HTs and `CBM_FREE` is called on their struct pointer. We **do not** individually free the interned strings.
- **Throughput Impact:** Because strings are heavily deduplicated, the memory cost of "orphan" strings is virtually zero. The intern pool acts as a fast, append-only arena that is purged entirely at the end of the incremental run.

### Stage 4: Architectural Analysis & Data Retrieval (MCP Tools)
- **Current Flow:** Queries the SQLite database using SQL to build architecture overviews, resolve trace paths, and search the graph. Returns JSON strings to the LLM/User.
- **New Flow:** Because Stage 2 ensures the SQLite database schema and content are perfectly unchanged, this stage requires zero modifications.
- **Throughput Impact:** Zero change in latency or logic. The user experience remains identical.

## 2. Problem Statement (Current Architecture)
The AST parsing phase in `graph_buffer.c` generates millions of transient nodes and edges. Currently, these are indexed using string-based hash tables.
- Node IDs (`int64_t`) are formatted into strings (`snprintf`) and duplicated onto the heap (`strdup`).
- Composite edge keys (`srcID:tgtID:type`) are formatted and duplicated.
- Redundant node strings (`label`, `name`) are individually duplicated for every AST node.
This results in millions of small allocations that are rounded up by the memory allocator (`mimalloc`) to the nearest size class (e.g., 32/48 bytes). On Windows MSVC, this causes catastrophic memory bloat, spiking RAM usage to >4.4 GB for large projects like the Linux kernel.

## 3. Proposed Architecture & Components

The optimization relies on two major architectural shifts for the graph buffer:

### A. Layer 1: Integer Hash Tables & Packed Keys
We will entirely eradicate string formatting for numerical/composite IDs during the in-memory parsing phase.
- **cbm_int_ht_t:** A new open-addressing, Robin Hood hash table in `src/foundation/int_hash_table.c` that maps `uint64_t` keys to `void*` values.
- **Node Indexing:** `node_by_id` will use `cbm_int_ht_t` keyed directly on the `uint64_t` node ID.
- **Edge Type Encoding:** Edge types (e.g., "CALLS") will be mapped to a restricted `uint16_t` integer (`type_id` from 1 to 65535).
- **Edge Source/Target Indexing:** `edges_by_source_type` and `edges_by_target_type` will use `cbm_int_ht_t` by packing the 48-bit Node ID and 16-bit Type ID into a single 64-bit integer (`(node_id << 16) | type_id`).
- **Edge Deduplication (cbm_edge_set_t):** `edge_by_key` will be replaced with a custom open-addressing hash set storing `cbm_gbuf_edge_t*`. It will hash a compact 18-byte struct (`{ uint64_t src, tgt; uint16_t type_id; }`) using the `XXH3_64bits` algorithm to achieve high dispersion and cache locality.

### B. Layer 2: String Interning
We will deduplicate highly redundant AST string values.
- **CBMInternPool Integration:** `cbm_gbuf_t` will initialize and own a `CBMInternPool`.
- **Node Strings:** All calls to `heap_strdup` for `label`, `name`, `file_path`, and `type` will be replaced with `cbm_intern`. This guarantees that an AST label like `"Function"` is allocated exactly once across the entire graph buffer.

## 4. Expected Impact
- Eliminates >60% of all heap allocations during the parsing phase.
- Reduces active graph buffer memory footprint by 40-50% on all OSes.
- Significantly increases CPU parsing throughput by bypassing `snprintf`, `strlen`, and string hashing overheads.

## 5. Preliminary Analysis: Fully Integer-Based Database (Future Work)

While the current optimization isolates the integer packing to the in-memory parsing phase, we must analyze the feasibility of pushing this integer abstraction all the way down to the persistent SQLite database.

**The Vision:**
- The entire SQLite database schema switches from `TEXT` columns (for labels, names, files, edge types) to `INTEGER` columns.
- A separate `StringDictionary` table acts as a global intern pool (e.g., `ID -> String`).
- All analytical queries (e.g., `SELECT * FROM edges WHERE type_id = 5`) execute exclusively on integers.

**Information Flow (JIT Conversion):**
- Data remains entirely abstracted as packed integers from parsing, through SQLite storage, and into the analytical query engine.
- String conversion happens **"Just In Time" (JIT)** only at the absolute final boundary: when delivering the requested JSON payload to the user via the MCP server.

**Expected Benefits of JIT Integer Database:**
1. **Disk I/O:** Drastic reduction in the `.db` file size on disk, speeding up database loads and backups.
2. **Query Latency:** SQLite `INTEGER` comparisons and joins are exponentially faster than `TEXT` comparisons. Index sizes shrink dramatically, allowing the entire DB to fit in memory (RAM cache).
3. **Memory Mapped Operations:** Allows for future memory-mapped binary graph representations that load instantly without SQL overhead.

*Note: The current architecture in this document lays the exact in-memory foundation required to easily transition to the JIT Integer Database in the future.*
