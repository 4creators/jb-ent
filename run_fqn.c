#include "tests/test_framework.h"

int tf_pass_count = 0;
int tf_fail_count = 0;
int tf_skip_count = 0;

extern void suite_fqn(void);

int main(void) {
    RUN_SUITE(fqn);
    TEST_SUMMARY();
}
