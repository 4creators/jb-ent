#include "test_framework.h"
#include "foundation/int_hash_table.h"

TEST(int_ht_basic) {
    cbm_int_ht_t *ht = cbm_int_ht_create(16);
    ASSERT_NOT_NULL(ht);
    
    int val1 = 42;
    int val2 = 99;
    
    cbm_int_ht_set(ht, 12345, &val1);
    cbm_int_ht_set(ht, 67890, &val2);
    
    ASSERT_EQ(cbm_int_ht_get(ht, 12345), &val1);
    ASSERT_EQ(cbm_int_ht_get(ht, 67890), &val2);
    ASSERT_NULL(cbm_int_ht_get(ht, 11111));
    
    ASSERT_EQ(cbm_int_ht_delete(ht, 12345), &val1);
    ASSERT_NULL(cbm_int_ht_get(ht, 12345));
    
    cbm_int_ht_free(ht);
    PASS();
}

SUITE(int_hash_table) {
    RUN_TEST(int_ht_basic);
}