#ifndef CBM_MEM_CONTEXT_H
#define CBM_MEM_CONTEXT_H

#include <stddef.h>

typedef struct CBMContext CBMContext;

CBMContext* cbm_ctx_new(void);
void cbm_ctx_free(CBMContext *ctx);
void* cbm_ctx_alloc(CBMContext *ctx, size_t size);
const char* cbm_ctx_intern(CBMContext *ctx, const char *s);

#endif /* CBM_MEM_CONTEXT_H */
