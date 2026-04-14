#include "../tests/test_mem_context.c"
#include "../tests/test_framework.h"

/* Framework variables */
int tf_pass_count = 0;
int tf_fail_count = 0;
int tf_skip_count = 0;

int main(void) {
    RUN_SUITE(mem_context);
    TEST_SUMMARY();
}
