#include "tests/test_int_hash_table.c"
#include "tests/test_framework.h"
#if MI_OVERRIDE
#include <mimalloc.h>
#endif

/* Framework variables */
int tf_pass_count = 0;
int tf_fail_count = 0;
int tf_skip_count = 0;

int main(void) {
#if MI_OVERRIDE
    mi_version();
#endif
    RUN_SUITE(int_hash_table);
    TEST_SUMMARY();
}