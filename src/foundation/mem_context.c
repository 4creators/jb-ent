#include "foundation/mem_context.h"
#include "foundation/allocator.h"
#include "foundation/str_intern.h"

#if MI_OVERRIDE
#include <mimalloc.h>
#endif

#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#endif

#ifndef CBM_HEAP_MALLOC
#if MI_OVERRIDE
#define CBM_HEAP_MALLOC(heap, size) mi_heap_malloc((heap), (size))
#else
#define CBM_HEAP_MALLOC(heap, size) CBM_MALLOC(size)
#endif
#endif

struct CBMContext {
#if MI_OVERRIDE
    mi_heap_t *heap;
#else
    void *heap;
#endif
#ifdef _WIN32
    SRWLOCK lock;
#else
    pthread_rwlock_t lock;
#endif
    CBMInternPool *intern_pool;
};

CBMContext* cbm_ctx_new(void) {
    CBMContext *ctx = (CBMContext*)CBM_MALLOC(sizeof(CBMContext));
    if (!ctx) {
        return NULL;
    }

#ifdef _WIN32
    InitializeSRWLock(&ctx->lock);
#else
    pthread_rwlock_init(&ctx->lock, NULL);
#endif

#if MI_OVERRIDE
    ctx->heap = mi_heap_new();
#else
    ctx->heap = NULL;
#endif
    ctx->intern_pool = cbm_intern_create();

    return ctx;
}

void cbm_ctx_free(CBMContext *ctx) {
    if (!ctx) {
        return;
    }

#ifdef _WIN32
    AcquireSRWLockExclusive(&ctx->lock);
#else
    pthread_rwlock_wrlock(&ctx->lock);
#endif

    if (ctx->intern_pool) {
        cbm_intern_free(ctx->intern_pool);
        ctx->intern_pool = NULL;
    }
#if MI_OVERRIDE
    if (ctx->heap) {
        mi_heap_destroy(ctx->heap);
        ctx->heap = NULL;
    }
#endif

#ifdef _WIN32
    ReleaseSRWLockExclusive(&ctx->lock);
#else
    pthread_rwlock_unlock(&ctx->lock);
    pthread_rwlock_destroy(&ctx->lock);
#endif

    CBM_FREE(ctx);
}

void* cbm_ctx_alloc(CBMContext *ctx, size_t size) {
#if MI_OVERRIDE
    if (!ctx || !ctx->heap) {
        return NULL;
    }
#else
    if (!ctx) {
        return NULL;
    }
#endif

    void *ptr;

#ifdef _WIN32
    AcquireSRWLockExclusive(&ctx->lock);
#else
    pthread_rwlock_wrlock(&ctx->lock);
#endif

    ptr = CBM_HEAP_MALLOC(ctx->heap, size);

#ifdef _WIN32
    ReleaseSRWLockExclusive(&ctx->lock);
#else
    pthread_rwlock_unlock(&ctx->lock);
#endif

    return ptr;
}

const char* cbm_ctx_intern(CBMContext *ctx, const char *s) {
    if (!ctx || !ctx->intern_pool || !s) {
        return NULL;
    }

    const char *result;

#ifdef _WIN32
    AcquireSRWLockExclusive(&ctx->lock);
#else
    pthread_rwlock_wrlock(&ctx->lock);
#endif

    result = cbm_intern(ctx->intern_pool, s);

#ifdef _WIN32
    ReleaseSRWLockExclusive(&ctx->lock);
#else
    pthread_rwlock_unlock(&ctx->lock);
#endif

    return result;
}
