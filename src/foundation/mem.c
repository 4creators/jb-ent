/*
 * mem.c — Unified memory management via mimalloc.
 *
 * Budget tracking based on actual RSS via mi_process_info().
 * When MI_OVERRIDE=0 (ASan builds), falls back to OS-specific
 * RSS queries (task_info on macOS, /proc/self/statm on Linux,
 * GetProcessMemoryInfo on Windows).
 */
#include "mem.h"
#include "platform.h"
#include "log.h"
#include "str_util.h"

#include "foundation/constants.h"
#include "sqlite3.h"
#include "foundation/str_util.h"

#define MAX_RAM_FRACTION 1.0
#define DEFAULT_RAM_FRACTION 0.5
#include <mimalloc.h>
#include <stdatomic.h>
#include <stdio.h>
#include <inttypes.h>

#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <psapi.h>
#elif defined(__APPLE__)
#include <mach/mach.h>
#else
#include <unistd.h>
#endif

/* ── Static state ─────────────────────────────────────────────── */

static size_t g_budget;          /* budget in bytes */
static atomic_int g_initialized; /* init guard */
static atomic_int g_was_over;    /* pressure hysteresis */

#define MB_DIVISOR ((size_t)(CBM_SZ_1K * CBM_SZ_1K))

/* ── OS fallback for RSS (ASan builds where MI_OVERRIDE=0) ──── */

static size_t os_rss(void) {
#if defined(_WIN32)
    PROCESS_MEMORY_COUNTERS pmc;
    if (GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof(pmc))) {
        return (size_t)pmc.PagefileUsage; /* Commit charge, not WorkingSetSize, to prevent OS freeze */
    }
    return 0;
#elif defined(__APPLE__)
    struct mach_task_basic_info info = {0};
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO, (task_info_t)&info, &count) ==
        KERN_SUCCESS) {
        return (size_t)info.resident_size;
    }
    return 0;
#else
    FILE *f = fopen("/proc/self/statm", "r");
    if (!f) {
        return 0;
    }
    unsigned long pages = 0;
    unsigned long rss_pages = 0;
    if (fscanf(f, "%lu %lu", &pages, &rss_pages) != 2) {
        rss_pages = 0;
    }
    (void)fclose(f);
    long ps = sysconf(_SC_PAGESIZE);
    return rss_pages * (ps > 0 ? (size_t)ps : CBM_SZ_4K);
#endif
}

/* ── Pressure logging (hysteresis) ────────────────────────────── */

static void check_pressure(size_t rss) {
    if (g_budget == 0) {
        return;
    }

    bool over = rss > g_budget;
    int was = atomic_load(&g_was_over);

    if (over && !was) {
        atomic_store(&g_was_over, 1);
        char rss_mb[CBM_SZ_32];
        char budget_mb[CBM_SZ_32];
        char pct_str[CBM_SZ_16];
        snprintf(rss_mb, sizeof(rss_mb), "%zu", rss / MB_DIVISOR);
        snprintf(budget_mb, sizeof(budget_mb), "%zu", g_budget / MB_DIVISOR);
        snprintf(pct_str, sizeof(pct_str), "%zu",
                 g_budget > 0 ? (rss * CBM_PERCENT) / g_budget : 0);
        cbm_log_warn("mem.pressure.warn", "rss_mb", rss_mb, "budget_mb", budget_mb, "pct", pct_str);
    } else if (!over && was) {
        atomic_store(&g_was_over, 0);
        char rss_mb[CBM_SZ_32];
        char budget_mb[CBM_SZ_32];
        char pct_str[CBM_SZ_16];
        snprintf(rss_mb, sizeof(rss_mb), "%zu", rss / MB_DIVISOR);
        snprintf(budget_mb, sizeof(budget_mb), "%zu", g_budget / MB_DIVISOR);
        snprintf(pct_str, sizeof(pct_str), "%zu",
                 g_budget > 0 ? (rss * CBM_PERCENT) / g_budget : 0);
        cbm_log_info("mem.pressure.ok", "rss_mb", rss_mb, "budget_mb", budget_mb, "pct", pct_str);
    }
}

/* ── Public API ────────────────────────────────────────────────── */

void cbm_mem_init(size_t explicit_budget_mb, double ram_fraction) {
    int expected = 0;
    if (!atomic_compare_exchange_strong(&g_initialized, &expected, 1)) {
        return;
    }

    if (ram_fraction <= 0.0 || ram_fraction > MAX_RAM_FRACTION) {
        ram_fraction = DEFAULT_RAM_FRACTION;
    }

    /* Reduce upfront memory: don't eagerly commit arenas.
     * Force decommit on purge (MADV_FREE_REUSABLE on macOS) so RSS
     * drops immediately instead of staying high until memory pressure. */
    mi_option_set(mi_option_arena_eager_commit, 0);
    mi_option_set(mi_option_purge_decommits, SKIP_ONE);
    mi_option_set(mi_option_purge_delay, 0); /* immediate purge, no 1s delay */

    cbm_system_info_t info = cbm_system_info();
    if (explicit_budget_mb > 0) {
        g_budget = explicit_budget_mb * MB_DIVISOR;
    } else {
        g_budget = (size_t)((double)info.total_ram * ram_fraction);
    }

    char budget_mb[CBM_SZ_32];
    char ram_mb[CBM_SZ_32];
    snprintf(budget_mb, sizeof(budget_mb), "%zu", g_budget / MB_DIVISOR);
    snprintf(ram_mb, sizeof(ram_mb), "%zu", info.total_ram / MB_DIVISOR);
    cbm_log_info("mem.init", "budget_mb", budget_mb, "total_ram_mb", ram_mb);
}

size_t cbm_mem_rss(void) {
#ifdef _WIN32
    return os_rss();
#else
    size_t current_rss = 0;
    size_t peak_rss = 0;
    mi_process_info(NULL, NULL, NULL, &current_rss, &peak_rss, NULL, NULL, NULL);
    if (current_rss > 0) {
        return current_rss;
    }
    /* Fallback for ASan builds (MI_OVERRIDE=0) */
    return os_rss();
#endif
}

size_t cbm_mem_peak_rss(void) {
#ifdef _WIN32
    PROCESS_MEMORY_COUNTERS pmc;
    if (GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof(pmc))) {
        return (size_t)pmc.PeakPagefileUsage;
    }
    return 0;
#else
    size_t peak_rss = 0;
    mi_process_info(NULL, NULL, NULL, NULL, &peak_rss, NULL, NULL, NULL);
    if (peak_rss > 0) {
        return peak_rss;
    }
    /* No OS fallback for peak — return current as best approximation */
    return os_rss();
#endif
}

size_t cbm_mem_budget(void) {
    return g_budget;
}

bool cbm_mem_over_budget(void) {
    size_t rss = cbm_mem_rss();
    check_pressure(rss);
    return rss > g_budget;
}

size_t cbm_mem_worker_budget(int num_workers) {
    if (num_workers <= 0) {
        num_workers = SKIP_ONE;
    }
    return g_budget / (size_t)num_workers;
}

void cbm_mem_collect(void) {
    mi_collect(true);
}

/* ── Hardened Memory Auditing & OOM Aborts ────────────────────────── */

#ifdef CBM_HARDEN_MEMORY

#include "allocator.h"

#ifdef _WIN32
#include <windows.h>
static SRWLOCK g_tracker_lock = SRWLOCK_INIT;
#define TRACKER_LOCK() AcquireSRWLockExclusive(&g_tracker_lock)
#define TRACKER_UNLOCK() ReleaseSRWLockExclusive(&g_tracker_lock)
#else
#include <pthread.h>
static pthread_mutex_t g_tracker_lock = PTHREAD_MUTEX_INITIALIZER;
#define TRACKER_LOCK() pthread_mutex_lock(&g_tracker_lock)
#define TRACKER_UNLOCK() pthread_mutex_unlock(&g_tracker_lock)
#endif

#define TRACKER_CAPACITY (1 << 26) /* 67 million allocations (2GB table) */
#define TRACKER_MASK (TRACKER_CAPACITY - 1)
#define TOMBSTONE ((void*)(~0ULL))

typedef struct {
    void *ptr;
    size_t size;
    const char *file;
    int line;
} alloc_record_t;

static alloc_record_t *g_tracker = NULL;
static atomic_int g_tracker_init_flag = 0;
static _Atomic int64_t g_audit_bytes = 0;

static void init_tracker_memory(void) {
    int expected = 0;
    if (atomic_compare_exchange_strong(&g_tracker_init_flag, &expected, 1)) {
        size_t bytes = (size_t)TRACKER_CAPACITY * sizeof(alloc_record_t);
#ifdef _WIN32
        g_tracker = VirtualAlloc(NULL, bytes, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
#else
        g_tracker = mmap(NULL, bytes, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
#endif
        if (!g_tracker) {
            fprintf(stderr, "FATAL: Could not allocate 2GB memory tracker table!\n");
            exit(EXIT_FAILURE);
        }
    } else {
        /* Wait for other thread to finish allocating */
        while (!g_tracker) {
#ifdef _WIN32
            Sleep(1);
#else
            usleep(1000);
#endif
        }
    }
}

static inline uint32_t ptr_hash(void *ptr) {
    uint64_t h = (uint64_t)ptr;
    h ^= h >> 33;
    h *= 0xff51afd7ed558ccdULL;
    h ^= h >> 33;
    h *= 0xc4ceb9fe1a85ec53ULL;
    h ^= h >> 33;
    return (uint32_t)h;
}

static void trigger_oom_abort(const char *op, size_t size, const char *file, int line) {
    static char msg[CBM_SZ_1K];
    
    size_t current_allocated = (size_t)atomic_load(&g_audit_bytes);
    cbm_system_info_t info = cbm_system_info();
    
    const char *env_mb = getenv("CBM_BUDGET_MB");
    const char *env_frac = getenv("CBM_RAM_FRACTION");
    
    int len = snprintf(msg, sizeof(msg), 
        "FATAL: Out of Memory! Failed to %s %zu bytes at %s:%d\n"
        "  Total RAM Available : %zu Bytes\n"
        "  CBM_BUDGET_MB       : %s\n"
        "  CBM_RAM_FRACTION    : %s\n"
        "  Allocated Memory    : %zu Bytes\n"
        "  Active Budget       : %zu Bytes\n", 
        op, size, file, line,
        info.total_ram,
        env_mb ? env_mb : "(unset)",
        env_frac ? env_frac : "(unset)",
        current_allocated,
        g_budget);
        
#ifdef _WIN32
    HANDLE hErr = GetStdHandle(STD_ERROR_HANDLE);
    if (hErr && hErr != INVALID_HANDLE_VALUE) {
        DWORD written;
        WriteFile(hErr, msg, (DWORD)len, &written, NULL);
    }
#else
    fputs(msg, stderr);
    fflush(stderr);
#endif
    exit(EXIT_FAILURE);
}

static void trigger_tracker_abort(const char *reason, void *ptr, const char *file, int line) {
    char msg[CBM_SZ_256];
    int len = snprintf(msg, sizeof(msg), "TRACKER FATAL: %s [ptr=%p] at %s:%d\n", reason, ptr, file, line);
#ifdef _WIN32
    HANDLE hErr = GetStdHandle(STD_ERROR_HANDLE);
    if (hErr && hErr != INVALID_HANDLE_VALUE) {
        DWORD written;
        WriteFile(hErr, msg, (DWORD)len, &written, NULL);
    }
#else
    fputs(msg, stderr);
    fflush(stderr);
#endif
    exit(EXIT_FAILURE);
}

static void tracker_add(void *ptr, size_t size, const char *file, int line) {
    if (!ptr) return;
    if (!g_tracker) init_tracker_memory();
    TRACKER_LOCK();
    uint32_t idx = ptr_hash(ptr) & TRACKER_MASK;
    for (int i = 0; i < TRACKER_CAPACITY; i++) {
        void *p = g_tracker[idx].ptr;
        if (p == NULL || p == TOMBSTONE) {
            g_tracker[idx].ptr = ptr;
            g_tracker[idx].size = size;
            g_tracker[idx].file = file;
            g_tracker[idx].line = line;
            TRACKER_UNLOCK();
            return;
        }
        if (p == ptr) {
            TRACKER_UNLOCK();
            trigger_tracker_abort("Pointer already tracked", ptr, file, line);
        }
        idx = (idx + 1) & TRACKER_MASK;
    }
    TRACKER_UNLOCK();
    trigger_tracker_abort("Capacity exceeded", ptr, file, line);
}

static void tracker_remove(void *ptr, const char *file, int line) {
    if (!ptr) return;
    if (!g_tracker) init_tracker_memory();
    TRACKER_LOCK();
    uint32_t idx = ptr_hash(ptr) & TRACKER_MASK;
    for (int i = 0; i < TRACKER_CAPACITY; i++) {
        void *p = g_tracker[idx].ptr;
        if (p == ptr) {
            g_tracker[idx].ptr = TOMBSTONE;
            TRACKER_UNLOCK();
            return;
        }
        if (p == NULL) {
            TRACKER_UNLOCK();
            trigger_tracker_abort("Untracked pointer freed", ptr, file, line);
        }
        idx = (idx + 1) & TRACKER_MASK;
    }
    TRACKER_UNLOCK();
    trigger_tracker_abort("Double free or capacity scanned", ptr, file, line);
}

static atomic_int g_circuit_warned = 0;

static void check_circuit_breaker(size_t request_size) {
    if (g_budget > 0) {
        size_t current = (size_t)atomic_load(&g_audit_bytes);
        if (current + request_size > g_budget) {
            int expected = 0;
            if (atomic_compare_exchange_strong(&g_circuit_warned, &expected, 1)) {
                char warn_msg[CBM_SZ_256];
                int len = snprintf(warn_msg, sizeof(warn_msg), "level=warn msg=\"100%% memory budget exceeded (%zu Bytes)! Waiting for gracful autocancellation .... or 120%% abort.\"\n", g_budget);
#ifdef _WIN32
                HANDLE hErr = GetStdHandle(STD_ERROR_HANDLE);
                if (hErr && hErr != INVALID_HANDLE_VALUE) {
                    DWORD written;
                    WriteFile(hErr, warn_msg, (DWORD)len, &written, NULL);
                }
#else
                fputs(warn_msg, stderr);
                fflush(stderr);
#endif
            }
        }
        if (current + request_size > g_budget + (g_budget / 5)) {
            trigger_oom_abort("circuit_breaker", request_size, "mem.c", 0);
        }
    }
}

void* cbm_malloc_safe(size_t size, const char *file, int line) {
    check_circuit_breaker(size);
#ifdef _MSC_VER
    void *p = mi_malloc(size);
#else
    void *p = malloc(size);
#endif
    if (!p) trigger_oom_abort("malloc", size, file, line);
    tracker_add(p, mi_usable_size(p), file, line);
    atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    return p;
}

void* cbm_calloc_safe(size_t count, size_t size, const char *file, int line) {
    check_circuit_breaker(count * size);
#ifdef _MSC_VER
    void *p = mi_calloc(count, size);
#else
    void *p = calloc(count, size);
#endif
    if (!p && (count * size) > 0) trigger_oom_abort("calloc", count * size, file, line);
    if (p) {
        tracker_add(p, mi_usable_size(p), file, line);
        atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    }
    return p;
}

void* cbm_realloc_safe(void* ptr, size_t size, const char *file, int line) {
    check_circuit_breaker(size);
    size_t old_size = ptr ? mi_usable_size(ptr) : 0;
    if (ptr) {
        tracker_remove(ptr, file, line);
    }
#ifdef _MSC_VER
    void *p = mi_realloc(ptr, size);
#else
    void *p = realloc(ptr, size);
#endif
    if (!p && size > 0) trigger_oom_abort("realloc", size, file, line);
    if (p) {
        tracker_add(p, mi_usable_size(p), file, line);
        atomic_fetch_sub(&g_audit_bytes, old_size);
        atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    }
    return p;
}

void cbm_free_safe(void* ptr, const char *file, int line) {
    if (ptr) {
        tracker_remove(ptr, file, line);
        atomic_fetch_sub(&g_audit_bytes, mi_usable_size(ptr));
#ifdef _MSC_VER
        mi_free(ptr);
#else
        free(ptr);
#endif
    }
}

char* cbm_strdup_safe(const char* s, const char *file, int line) {
    if (!s) return NULL;
    check_circuit_breaker(strlen(s) + 1);
#ifdef _MSC_VER
    char *p = mi_strdup(s);
#else
    char *p = _strdup(s); /* Or standard strdup depending on platform, _strdup on MSVC */
#ifndef _WIN32
    if (!p) p = strdup(s);
#endif
#endif
    if (!p) trigger_oom_abort("strdup", strlen(s) + 1, file, line);
    tracker_add(p, mi_usable_size(p), file, line);
    atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    return p;
}

char* cbm_strndup_safe(const char* s, size_t n, const char *file, int line) {
    if (!s) return NULL;
    size_t len = 0;
    while (len < n && s[len]) len++;
    check_circuit_breaker(len + 1);
#ifdef _MSC_VER
    char *p = (char *)mi_malloc(len + 1);
#else
    char *p = malloc(len + 1);
#endif
    if (!p) trigger_oom_abort("strndup", len + 1, file, line);
    memcpy(p, s, len);
    p[len] = '\0';
    tracker_add(p, mi_usable_size(p), file, line);
    atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    return p;
}

void cbm_mem_print_audit(void) {
    if (!g_tracker) {
        fprintf(stderr, "MEMORY AUDIT OK: No allocations recorded.\n");
        return;
    }
    int64_t leaked = atomic_load(&g_audit_bytes);
    
    int leak_count = 0;
    size_t leaked_bytes = 0;
    for (int i = 0; i < TRACKER_CAPACITY; i++) {
        void *p = g_tracker[i].ptr;
        if (p != NULL && p != TOMBSTONE) {
            fprintf(stderr, "LEAK: %p (%zu bytes) allocated at %s:%d\n", p, g_tracker[i].size, g_tracker[i].file, g_tracker[i].line);
            leaked_bytes += g_tracker[i].size;
            leak_count++;
            if (leak_count > 100) {
                fprintf(stderr, "... and more leaks omitted.\n");
                break;
            }
        }
    }
    
    if (leak_count == 0 && leaked == 0) {
        fprintf(stderr, "MEMORY AUDIT OK: 0 bytes leaked.\n");
    } else {
        fprintf(stderr, "MEMORY AUDIT FAILED: %lld bytes leaked counter, %d allocations leaked in tracker totaling %zu bytes.\n", (long long)leaked, leak_count, leaked_bytes);
    }
}

#endif /* CBM_HARDEN_MEMORY */

/* ── Mongoose Override ────────────────────────────────────────── */
#if defined(MG_ENABLE_CUSTOM_CALLOC) && MG_ENABLE_CUSTOM_CALLOC == 1
#include "allocator.h"
void *mg_calloc(size_t count, size_t size) {
    return CBM_CALLOC(count, size);
}
void mg_free(void *ptr) {
    CBM_FREE(ptr);
}
#endif

/* ── SQLite Redirection ────────────────────────────────────────── */

static void* sqlite3_mi_malloc(int n) {
#ifdef CBM_HARDEN_MEMORY
    return cbm_malloc_safe((size_t)n, "sqlite", 0);
#else
    return mi_malloc((size_t)n);
#endif
}

static void sqlite3_mi_free(void *p) {
#ifdef CBM_HARDEN_MEMORY
    cbm_free_safe(p, "sqlite", 0);
#else
    mi_free(p);
#endif
}

static void* sqlite3_mi_realloc(void *p, int n) {
#ifdef CBM_HARDEN_MEMORY
    return cbm_realloc_safe(p, (size_t)n, "sqlite", 0);
#else
    return mi_realloc(p, (size_t)n);
#endif
}

static int sqlite3_mi_size(void *p) {
    return (int)mi_usable_size(p);
}

static int sqlite3_mi_roundup(int n) {
    return n;
}

static int sqlite3_mi_init(void *p) {
    (void)p;
    return SQLITE_OK;
}

static void sqlite3_mi_shutdown(void *p) {
    (void)p;
}

static sqlite3_mem_methods mi_methods = {
    sqlite3_mi_malloc,
    sqlite3_mi_free,
    sqlite3_mi_realloc,
    sqlite3_mi_size,
    sqlite3_mi_roundup,
    sqlite3_mi_init,
    sqlite3_mi_shutdown,
    NULL
};

void cbm_mem_hook_sqlite(void) {
    sqlite3_config(SQLITE_CONFIG_MALLOC, &mi_methods);
}