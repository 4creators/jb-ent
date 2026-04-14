#include "tests/test_framework.h"
#include "foundation/allocator.h"
#include <stdlib.h>

int tf_pass_count = 0;
int tf_fail_count = 0;
int tf_skip_count = 0;

extern void suite_fqn(void);

int main(void) {
#ifdef CBM_HARDEN_MEMORY
    atexit(cbm_mem_print_audit);
#endif
    RUN_SUITE(fqn);
    TEST_SUMMARY();
}
