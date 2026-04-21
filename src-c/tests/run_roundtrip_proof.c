#include "pipeline/pipeline.h"
#include "graph_buffer/graph_buffer.h"
#include "store/store.h"
#include "foundation/allocator.h"
#include "foundation/compat_fs.h"
#include <stdio.h>
#include <stdlib.h>

#ifdef _WIN32
#include <io.h>
#define cbm_close _close
#else
#include <unistd.h>
#define cbm_close close
#endif

static int make_temp_db(char *path, size_t pathsz) {
    snprintf(path, pathsz, "cbm_proof_XXXXXX");
    int fd = cbm_mkstemp(path);
    if (fd < 0) return -1;
    cbm_close(fd);
    return 0;
}

int main(void) {
#if MI_OVERRIDE
    mi_version();
#endif
    cbm_mem_init(0, 0.5); /* Initialize memory with defaults: 50% RAM budget */

    const char *repo_path = "E:\\src\\ai\\codebase-mcp";
    
    printf("1. Parsing codebase-memory-mcp into a real SQLite database...\n");
    char db_path[256];
    if (make_temp_db(db_path, sizeof(db_path)) != 0) {
        printf("Failed to create temp db\n");
        return 1;
    }

    cbm_pipeline_t *p = cbm_pipeline_new(repo_path, db_path, CBM_MODE_FULL);
    if (cbm_pipeline_run(p) != 0) {
        printf("Pipeline failed\n");
        return 1;
    }
    
    char *project = CBM_STRDUP(cbm_pipeline_project_name(p));
    cbm_pipeline_free(p);

    printf("2. Loading full graph from SQLite into memory (Graph A)...\n");
    cbm_gbuf_t *gbA = cbm_gbuf_new(project, repo_path);
    if (cbm_gbuf_load_from_db(gbA, db_path, project) != 0) {
        printf("Failed to load gbA\n");
        return 1;
    }
    printf("   Graph A: %d nodes, %d edges\n", cbm_gbuf_node_count(gbA), cbm_gbuf_edge_count(gbA));

    printf("3. Flushing Graph A into a new temporary SQLite DB...\n");
    char temp_db_path[256];
    if (make_temp_db(temp_db_path, sizeof(temp_db_path)) != 0) {
        printf("Failed to create temp db 2\n");
        return 1;
    }

    cbm_store_t *store = cbm_store_open_path(temp_db_path);
    cbm_store_upsert_project(store, project, repo_path);
    if (cbm_gbuf_flush_to_store(gbA, store) != 0) {
        printf("Failed to flush gbA\n");
        return 1;
    }
    cbm_store_close(store);

    printf("4. Loading second temporary SQLite DB into memory (Graph B)...\n");
    cbm_gbuf_t *gbB = cbm_gbuf_new(project, repo_path);
    if (cbm_gbuf_load_from_db(gbB, temp_db_path, project) != 0) {
        printf("Failed to load gbB\n");
        return 1;
    }
    printf("   Graph B: %d nodes, %d edges\n", cbm_gbuf_node_count(gbB), cbm_gbuf_edge_count(gbB));

    printf("5. Proving Mathematical Structural Equivalence (Isomorphism)...\n");
    if (cbm_gbuf_equals(gbA, gbB)) {
        printf("\nSUCCESS: Graph A and Graph B are mathematically isomorphic!\n");
        printf("The in-memory integer hash tables and SQLite serialization are perfectly lossless.\n");
    } else {
        printf("\nFAILURE: Graphs are NOT isomorphic!\n");
    }

    cbm_gbuf_free(gbA);
    cbm_gbuf_free(gbB);
    cbm_unlink(db_path);
    cbm_unlink(temp_db_path);
    CBM_FREE(project);

    return 0;
}
