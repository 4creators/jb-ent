#include "test_framework.h"
#include "foundation/mem_context.h"
#include <string.h>
#include <stdio.h>

TEST(cbm_context_lifecycle) {
    CBMContext *ctx = cbm_ctx_new();
    ASSERT_NOT_NULL(ctx);

    /* Test allocation */
    char *buf1 = (char*)cbm_ctx_alloc(ctx, 1024);
    ASSERT_NOT_NULL(buf1);
    memset(buf1, 'A', 1024);
    ASSERT_EQ(buf1[0], 'A');
    ASSERT_EQ(buf1[1023], 'A');

    /* Test interning (deduplication) */
    const char *str1 = cbm_ctx_intern(ctx, "hello_world");
    const char *str2 = cbm_ctx_intern(ctx, "hello_world");
    const char *str3 = cbm_ctx_intern(ctx, "different");

    ASSERT_NOT_NULL(str1);
    ASSERT_NOT_NULL(str2);
    ASSERT_NOT_NULL(str3);

    ASSERT_STR_EQ(str1, "hello_world");
    ASSERT_STR_EQ(str3, "different");

    /* Pointer equality proves deduplication */
    ASSERT_EQ((void*)str1, (void*)str2);
    ASSERT_NEQ((void*)str1, (void*)str3);

    /* Bulk free destroys everything */
    cbm_ctx_free(ctx);
    PASS();
}

SUITE(mem_context) {
    RUN_TEST(cbm_context_lifecycle);
}
