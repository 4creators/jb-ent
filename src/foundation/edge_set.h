#ifndef CBM_EDGE_SET_H
#define CBM_EDGE_SET_H

#include <stdint.h>
#include <stdbool.h>

#pragma pack(push, 1)
typedef struct {
    uint64_t src;
    uint64_t tgt;
    uint16_t type_id;
} cbm_edge_key_t;
#pragma pack(pop)

typedef struct cbm_gbuf_edge cbm_gbuf_edge_t; /* Forward declaration */

typedef struct {
    cbm_edge_key_t key;
    cbm_gbuf_edge_t *edge;
    uint32_t psl; /* probe sequence length */
} cbm_edge_set_entry_t;

typedef struct {
    cbm_edge_set_entry_t *entries;
    uint32_t capacity;
    uint32_t count;
    uint32_t mask;
} cbm_edge_set_t;

cbm_edge_set_t *cbm_edge_set_create(uint32_t initial_capacity);
void cbm_edge_set_free(cbm_edge_set_t *set);
void cbm_edge_set_insert(cbm_edge_set_t *set, cbm_edge_key_t key, cbm_gbuf_edge_t *edge);
cbm_gbuf_edge_t *cbm_edge_set_get(const cbm_edge_set_t *set, cbm_edge_key_t key);
void cbm_edge_set_delete(cbm_edge_set_t *set, cbm_edge_key_t key);

/* Iteration: call fn(key, edge, userdata) for each entry. */
typedef void (*cbm_edge_set_iter_fn)(cbm_edge_key_t key, cbm_gbuf_edge_t *edge, void *userdata);
void cbm_edge_set_foreach(const cbm_edge_set_t *set, cbm_edge_set_iter_fn fn, void *userdata);

#endif /* CBM_EDGE_SET_H */