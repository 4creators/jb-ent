#include "test_framework.h"
#include "cbm.h"
#include "foundation/allocator.h"
#include "foundation/mem.h"
#include "foundation/slab_alloc.h"
#if MI_OVERRIDE
#include <mimalloc.h>
#endif
#include <stdio.h>
#include <stdlib.h>

int tf_pass_count = 0;
int tf_fail_count = 0;
int tf_skip_count = 0;

static int extract_file_path(const char *path, uint64_t budget) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        printf("File not found: %s\n", path);
        return 1; // Ignore if file not found locally
    }
    
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    char *src = CBM_MALLOC(size + 1);
    if (!src) return 0;
    fread(src, 1, size, f);
    src[size] = '\0';
    fclose(f);

    CBMFileResult *r = cbm_extract_file(src, size, CBM_LANG_C, "test", path, budget, NULL, NULL);
    if (!r) {
        CBM_FREE(src);
        return 0;
    }
    if (r->has_error) {
        printf("EXTRACTION ERROR: %s\n", r->error_msg);
        cbm_free_result(r);
        CBM_FREE(src);
        return 0;
    }
    
    if (r->defs.count == 0 && size > 1000) {
        cbm_free_result(r);
        CBM_FREE(src);
        return 0;
    }

    cbm_free_result(r);
    CBM_FREE(src);
    return 1;
}

int main(int argc, char **argv) {
#if MI_OVERRIDE
    mi_version();
#endif
    cbm_mem_init(0, 0.5);
    cbm_slab_install();
    cbm_init();

    if (argc > 1) {
        if (!extract_file_path(argv[1], 600000000ULL)) {
            printf("FAIL: %s\n", argv[1]);
        } else {
            printf("PASS: %s\n", argv[1]);
        }
    } else {
        printf("Usage: %s <path>\n", argv[0]);
    }

    cbm_shutdown();
    return 0;
}
