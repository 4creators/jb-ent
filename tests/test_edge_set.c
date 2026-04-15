#include "test_framework.h"
#include "foundation/edge_set.h"

/* Dummy edge structure just for testing */
struct cbm_gbuf_edge {
    int id;
};

TEST(edge_set_basic) {
    cbm_edge_set_t *set = cbm_edge_set_create(16);
    ASSERT_NOT_NULL(set);
    
    struct cbm_gbuf_edge edge1 = {1};
    struct cbm_gbuf_edge edge2 = {2};
    
    cbm_edge_key_t key1 = { .src = 100, .tgt = 200, .type_id = 5 };
    cbm_edge_key_t key2 = { .src = 300, .tgt = 400, .type_id = 10 };
    cbm_edge_key_t missing_key = { .src = 100, .tgt = 200, .type_id = 6 };
    
    cbm_edge_set_insert(set, key1, &edge1);
    cbm_edge_set_insert(set, key2, &edge2);
    
    ASSERT_EQ(cbm_edge_set_get(set, key1), &edge1);
    ASSERT_EQ(cbm_edge_set_get(set, key2), &edge2);
    ASSERT_NULL(cbm_edge_set_get(set, missing_key));
    
    cbm_edge_set_delete(set, key1);
    ASSERT_NULL(cbm_edge_set_get(set, key1));
    ASSERT_EQ(cbm_edge_set_get(set, key2), &edge2); /* Other key still exists */
    
    cbm_edge_set_free(set);
    PASS();
}

SUITE(edge_set) {
    RUN_TEST(edge_set_basic);
}