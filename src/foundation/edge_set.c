#include "foundation/edge_set.h"
#include "foundation/allocator.h"
#include <xxhash.h>
#include <string.h>

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

static inline uint32_t hash_edge_key(cbm_edge_key_t key) {
    /* Use XXH3_64bits for high dispersion of the 18-byte struct */
    return (uint32_t)XXH3_64bits(&key, sizeof(cbm_edge_key_t));
}

static inline bool keys_equal(cbm_edge_key_t a, cbm_edge_key_t b) {
    return a.src == b.src && a.tgt == b.tgt && a.type_id == b.type_id;
}

cbm_edge_set_t *cbm_edge_set_create(uint32_t initial_capacity) {
    cbm_edge_set_t *set = (cbm_edge_set_t *)CBM_CALLOC(1, sizeof(cbm_edge_set_t));
    if (!set) return NULL;
    
    set->capacity = next_pow2(initial_capacity);
    set->mask = set->capacity - 1;
    set->entries = (cbm_edge_set_entry_t *)CBM_CALLOC(set->capacity, sizeof(cbm_edge_set_entry_t));
    if (!set->entries) {
        CBM_FREE(set);
        return NULL;
    }
    return set;
}

void cbm_edge_set_free(cbm_edge_set_t *set) {
    if (!set) return;
    CBM_FREE(set->entries);
    CBM_FREE(set);
}

static void set_resize(cbm_edge_set_t *set) {
    uint32_t new_cap = set->capacity * 2;
    uint32_t new_mask = new_cap - 1;
    cbm_edge_set_entry_t *new_entries = (cbm_edge_set_entry_t *)CBM_CALLOC(new_cap, sizeof(cbm_edge_set_entry_t));
    if (!new_entries) return; /* Keep old table on OOM */

    for (uint32_t i = 0; i < set->capacity; i++) {
        if (set->entries[i].psl > 0) {
            cbm_edge_set_entry_t cur = set->entries[i];
            cur.psl = 1;
            uint32_t idx = hash_edge_key(cur.key) & new_mask;
            
            for (;;) {
                if (new_entries[idx].psl == 0) {
                    new_entries[idx] = cur;
                    break;
                }
                if (cur.psl > new_entries[idx].psl) {
                    cbm_edge_set_entry_t tmp = new_entries[idx];
                    new_entries[idx] = cur;
                    cur = tmp;
                }
                cur.psl++;
                idx = (idx + 1) & new_mask;
            }
        }
    }
    
    CBM_FREE(set->entries);
    set->entries = new_entries;
    set->capacity = new_cap;
    set->mask = new_mask;
}

void cbm_edge_set_insert(cbm_edge_set_t *set, cbm_edge_key_t key, cbm_gbuf_edge_t *edge) {
    if (!set) return;
    
    if (set->count * 4 >= set->capacity * 3) {
        set_resize(set);
    }
    
    cbm_edge_set_entry_t cur;
    cur.key = key;
    cur.edge = edge;
    cur.psl = 1;
    
    uint32_t idx = hash_edge_key(key) & set->mask;
    
    for (;;) {
        if (set->entries[idx].psl == 0) {
            set->entries[idx] = cur;
            set->count++;
            return;
        }
        if (keys_equal(set->entries[idx].key, key)) {
            /* Replace existing */
            set->entries[idx].edge = edge;
            return;
        }
        if (cur.psl > set->entries[idx].psl) {
            cbm_edge_set_entry_t tmp = set->entries[idx];
            set->entries[idx] = cur;
            cur = tmp;
        }
        cur.psl++;
        idx = (idx + 1) & set->mask;
    }
}

cbm_gbuf_edge_t *cbm_edge_set_get(const cbm_edge_set_t *set, cbm_edge_key_t key) {
    if (!set) return NULL;
    
    uint32_t idx = hash_edge_key(key) & set->mask;
    uint32_t psl = 1;
    
    for (;;) {
        if (set->entries[idx].psl == 0 || psl > set->entries[idx].psl) {
            return NULL;
        }
        if (keys_equal(set->entries[idx].key, key)) {
            return set->entries[idx].edge;
        }
        psl++;
        idx = (idx + 1) & set->mask;
    }
}

void cbm_edge_set_delete(cbm_edge_set_t *set, cbm_edge_key_t key) {
    if (!set) return;
    
    uint32_t idx = hash_edge_key(key) & set->mask;
    uint32_t psl = 1;
    
    for (;;) {
        if (set->entries[idx].psl == 0 || psl > set->entries[idx].psl) {
            return;
        }
        if (keys_equal(set->entries[idx].key, key)) {
            set->entries[idx].psl = 0;
            set->count--;
            
            /* Backward shift */
            uint32_t cur_idx = idx;
            for (;;) {
                uint32_t next_idx = (cur_idx + 1) & set->mask;
                if (set->entries[next_idx].psl <= 1) {
                    break;
                }
                set->entries[cur_idx] = set->entries[next_idx];
                set->entries[cur_idx].psl--;
                set->entries[next_idx].psl = 0;
                cur_idx = next_idx;
            }
            return;
        }
        psl++;
        idx = (idx + 1) & set->mask;
    }
}

void cbm_edge_set_foreach(const cbm_edge_set_t *set, cbm_edge_set_iter_fn fn, void *userdata) {
    if (!set || !fn) return;
    for (uint32_t i = 0; i < set->capacity; i++) {
        if (set->entries[i].psl > 0) {
            fn(set->entries[i].key, set->entries[i].edge, userdata);
        }
    }
}