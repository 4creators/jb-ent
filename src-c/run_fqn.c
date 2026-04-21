#include "tests/test_framework.h"
#include "foundation/allocator.h"
#include <stdlib.h>

int tf_pass_count = 0;
int tf_fail_count = 0;
int tf_skip_count = 0;

extern void suite_fqn(void);

int main(void) {
#if MI_OVERRIDE
    mi_version();
#endif
    cbm_mem_init(0, 0.5);
    RUN_SUITE(fqn);
    TEST_SUMMARY();
}
