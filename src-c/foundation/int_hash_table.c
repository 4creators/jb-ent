#include "foundation/int_hash_table.h"
#include "foundation/allocator.h"
#include <string.h>

/* Use a fast integer hash (splitmix64-style or similar) */
static inline uint32_t hash_int64(uint64_t key) {
    key ^= key >> 30;
    key *= 0xbf58476d1ce4e5b9ULL;
    key ^= key >> 27;
    key *= 0x94d049bb133111ebULL;
    key ^= key >> 31;
    return (uint32_t)key;
}

static uint32_t next_pow2(uint32_t v) {
    if (v == 0) return 16;
    v--;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    return v + 1;
}

cbm_int_ht_t *cbm_int_ht_create(uint32_t initial_capacity) {
    cbm_int_ht_t *ht = (cbm_int_ht_t *)CBM_CALLOC(1, sizeof(cbm_int_ht_t));
    if (!ht) return NULL;
    
    ht->capacity = next_pow2(initial_capacity);
    ht->mask = ht->capacity - 1;
    ht->entries = (cbm_int_ht_entry_t *)CBM_CALLOC(ht->capacity, sizeof(cbm_int_ht_entry_t));
    if (!ht->entries) {
        CBM_FREE(ht);
        return NULL;
    }
    return ht;
}

void cbm_int_ht_free(cbm_int_ht_t *ht) {
    if (!ht) return;
    CBM_FREE(ht->entries);
    CBM_FREE(ht);
}

static void ht_resize(cbm_int_ht_t *ht) {
    uint32_t new_cap = ht->capacity * 2;
    uint32_t new_mask = new_cap - 1;
    cbm_int_ht_entry_t *new_entries = (cbm_int_ht_entry_t *)CBM_CALLOC(new_cap, sizeof(cbm_int_ht_entry_t));
    if (!new_entries) return; // Keep old table on OOM

    for (uint32_t i = 0; i < ht->capacity; i++) {
        if (ht->entries[i].psl > 0) {
            cbm_int_ht_entry_t cur = ht->entries[i];
            cur.psl = 1;
            uint32_t idx = hash_int64(cur.key) & new_mask;
            
            for (;;) {
                if (new_entries[idx].psl == 0) {
                    new_entries[idx] = cur;
                    break;
                }
                if (cur.psl > new_entries[idx].psl) {
                    cbm_int_ht_entry_t tmp = new_entries[idx];
                    new_entries[idx] = cur;
                    cur = tmp;
                }
                cur.psl++;
                idx = (idx + 1) & new_mask;
            }
        }
    }
    
    CBM_FREE(ht->entries);
    ht->entries = new_entries;
    ht->capacity = new_cap;
    ht->mask = new_mask;
}

void *cbm_int_ht_set(cbm_int_ht_t *ht, uint64_t key, void *value) {
    if (!ht) return NULL;
    
    if (ht->count * 4 >= ht->capacity * 3) {
        ht_resize(ht);
    }
    
    cbm_int_ht_entry_t cur;
    cur.key = key;
    cur.value = value;
    cur.psl = 1;
    
    uint32_t idx = hash_int64(key) & ht->mask;
    
    for (;;) {
        if (ht->entries[idx].psl == 0) {
            ht->entries[idx] = cur;
            ht->count++;
            return NULL;
        }
        if (ht->entries[idx].key == key) {
            void *old_val = ht->entries[idx].value;
            ht->entries[idx].value = value;
            return old_val;
        }
        if (cur.psl > ht->entries[idx].psl) {
            cbm_int_ht_entry_t tmp = ht->entries[idx];
            ht->entries[idx] = cur;
            cur = tmp;
        }
        cur.psl++;
        idx = (idx + 1) & ht->mask;
    }
}

void *cbm_int_ht_get(const cbm_int_ht_t *ht, uint64_t key) {
    if (!ht) return NULL;
    
    uint32_t idx = hash_int64(key) & ht->mask;
    uint32_t psl = 1;
    
    for (;;) {
        if (ht->entries[idx].psl == 0 || psl > ht->entries[idx].psl) {
            return NULL;
        }
        if (ht->entries[idx].key == key) {
            return ht->entries[idx].value;
        }
        psl++;
        idx = (idx + 1) & ht->mask;
    }
}

bool cbm_int_ht_has(const cbm_int_ht_t *ht, uint64_t key) {
    return cbm_int_ht_get(ht, key) != NULL;
}

void *cbm_int_ht_delete(cbm_int_ht_t *ht, uint64_t key) {
    if (!ht) return NULL;
    
    uint32_t idx = hash_int64(key) & ht->mask;
    uint32_t psl = 1;
    
    for (;;) {
        if (ht->entries[idx].psl == 0 || psl > ht->entries[idx].psl) {
            return NULL;
        }
        if (ht->entries[idx].key == key) {
            void *val = ht->entries[idx].value;
            ht->entries[idx].psl = 0;
            ht->count--;
            
            // Backward shift
            uint32_t cur_idx = idx;
            for (;;) {
                uint32_t next_idx = (cur_idx + 1) & ht->mask;
                if (ht->entries[next_idx].psl <= 1) {
                    break;
                }
                ht->entries[cur_idx] = ht->entries[next_idx];
                ht->entries[cur_idx].psl--;
                ht->entries[next_idx].psl = 0;
                cur_idx = next_idx;
            }
            return val;
        }
        psl++;
        idx = (idx + 1) & ht->mask;
    }
}

void cbm_int_ht_foreach(const cbm_int_ht_t *ht, cbm_int_ht_iter_fn fn, void *userdata) {
    if (!ht || !fn) return;
    for (uint32_t i = 0; i < ht->capacity; i++) {
        if (ht->entries[i].psl > 0) {
            fn(ht->entries[i].key, ht->entries[i].value, userdata);
        }
    }
}