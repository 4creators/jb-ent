/*
 * allocator.h — Global memory auditing and decisive OOM abortion.
 *
 * Provides macros to replace standard memory allocations.
 * When CBM_HARDEN_MEMORY is defined, allocations are tracked via
 * a global atomic counter and any OOM instantly aborts the process.
 */
#ifndef CBM_ALLOCATOR_H
#define CBM_ALLOCATOR_H

#include <stddef.h>

#ifdef CBM_HARDEN_MEMORY

extern void* cbm_malloc_safe(size_t size, const char *file, int line);
extern void* cbm_calloc_safe(size_t count, size_t size, const char *file, int line);
extern void* cbm_realloc_safe(void* ptr, size_t size, const char *file, int line);
extern void  cbm_free_safe(void* ptr, const char *file, int line);
extern char* cbm_strdup_safe(const char* s, const char *file, int line);
extern char* cbm_strndup_safe(const char* s, size_t n, const char *file, int line);

#define CBM_MALLOC(size)      cbm_malloc_safe((size), __FILE__, __LINE__)
#define CBM_CALLOC(c, s)      cbm_calloc_safe((c), (s), __FILE__, __LINE__)
#define CBM_REALLOC(p, s)     cbm_realloc_safe((p), (s), __FILE__, __LINE__)
#define CBM_FREE(p)           cbm_free_safe((p), __FILE__, __LINE__)
#define CBM_STRDUP(s)         cbm_strdup_safe((s), __FILE__, __LINE__)
#define CBM_STRNDUP(s, n)     cbm_strndup_safe((s), (n), __FILE__, __LINE__)

/* Expose the audit counter for main() to print at exit */
extern void cbm_mem_print_audit(void);

#else /* Normal allocation (No Harden Memory) */

#include "foundation/compat.h"
#include <stdlib.h>
#include <string.h>

/* 1. Microsoft Visual C++ (and Windows toolchains targeting MSVC ABI) */
#if defined(_MSC_VER) || defined(_WIN32) || defined(__CYGWIN__) || defined(__MINGW32__)
    #if MI_OVERRIDE
        #include <mimalloc.h>
        #define CBM_MALLOC(size)      mi_malloc(size)
        #define CBM_CALLOC(c, s)      mi_calloc(c, s)
        #define CBM_REALLOC(p, s)     mi_realloc(p, s)
        #define CBM_FREE(p)           mi_free(p)
        #define CBM_STRDUP(s)         mi_strdup(s)
        #define CBM_STRNDUP(s, n)     mi_strndup(s, n)
    #else
        #define CBM_MALLOC(size)      malloc(size)
        #define CBM_CALLOC(c, s)      calloc(c, s)
        #define CBM_REALLOC(p, s)     realloc(p, s)
        #define CBM_FREE(p)           free(p)
        #ifdef _MSC_VER
            #define CBM_STRDUP(s)         _strdup(s)
        #else
            #define CBM_STRDUP(s)         strdup(s)
        #endif
        #define CBM_STRNDUP(s, n)     cbm_strndup(s, n) /* Shim required for cross-platform fallback */
    #endif

/* 2. GCC or Clang (Linux/macOS/POSIX) */
#elif defined(__GNUC__) || defined(__clang__)
    #if MI_OVERRIDE
        #include <mimalloc.h>
        #define CBM_MALLOC(size)      mi_malloc(size)
        #define CBM_CALLOC(c, s)      mi_calloc(c, s)
        #define CBM_REALLOC(p, s)     mi_realloc(p, s)
        #define CBM_FREE(p)           mi_free(p)
        #define CBM_STRDUP(s)         mi_strdup(s)
        #define CBM_STRNDUP(s, n)     mi_strndup(s, n)
    #else
        #define CBM_MALLOC(size)      malloc(size)
        #define CBM_CALLOC(c, s)      calloc(c, s)
        #define CBM_REALLOC(p, s)     realloc(p, s)
        #define CBM_FREE(p)           free(p)
        #define CBM_STRDUP(s)         strdup(s)
        #define CBM_STRNDUP(s, n)     strndup(s, n)
    #endif

/* X. Unknown Toolchain */
#else
    #error "Unsupported compiler. Please add memory allocation macros for your toolchain in allocator.h"
#endif

#define cbm_mem_print_audit() ((void)0)

#endif /* CBM_HARDEN_MEMORY */

#endif /* CBM_ALLOCATOR_H */