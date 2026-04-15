#ifndef CBM_INT_HASH_TABLE_H
#define CBM_INT_HASH_TABLE_H

#include <stdint.h>
#include <stdbool.h>

typedef struct {
    uint64_t key;
    void *value;
    uint32_t psl; /* probe sequence length */
} cbm_int_ht_entry_t;

typedef struct {
    cbm_int_ht_entry_t *entries;
    uint32_t capacity;
    uint32_t count;
    uint32_t mask;
} cbm_int_ht_t;

cbm_int_ht_t *cbm_int_ht_create(uint32_t initial_capacity);
void cbm_int_ht_free(cbm_int_ht_t *ht);
void *cbm_int_ht_set(cbm_int_ht_t *ht, uint64_t key, void *value);
void *cbm_int_ht_get(const cbm_int_ht_t *ht, uint64_t key);
void *cbm_int_ht_delete(cbm_int_ht_t *ht, uint64_t key);
bool cbm_int_ht_has(const cbm_int_ht_t *ht, uint64_t key);

/* Iteration: call fn(key, value, userdata) for each entry. */
typedef void (*cbm_int_ht_iter_fn)(uint64_t key, void *value, void *userdata);
void cbm_int_ht_foreach(const cbm_int_ht_t *ht, cbm_int_ht_iter_fn fn, void *userdata);

#endif /* CBM_INT_HASH_TABLE_H */