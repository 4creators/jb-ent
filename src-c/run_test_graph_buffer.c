#include "tests/test_graph_buffer.c"
#include "tests/test_framework.h"
#if MI_OVERRIDE
#include <mimalloc.h>
#endif
#include <stdlib.h>
#include "foundation/allocator.h"
#include "foundation/mem.h"

/* Framework variables */
int tf_pass_count = 0;
int tf_fail_count = 0;
int tf_skip_count = 0;

int main(void) {
#if MI_OVERRIDE
    mi_version();
#endif
    RUN_SUITE(graph_buffer);
    TEST_SUMMARY();
}