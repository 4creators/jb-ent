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

#include "foundation/constants.h"

#define MAX_RAM_FRACTION 1.0
#define DEFAULT_RAM_FRACTION 0.5
#include <mimalloc.h>
#include <stdatomic.h>
#include <stdio.h>

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
#ifdef _WIN32
    PROCESS_MEMORY_COUNTERS pmc;
    if (GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof(pmc))) {
        return (size_t)pmc.WorkingSetSize;
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

void cbm_mem_init(double ram_fraction) {
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
    g_budget = (size_t)((double)info.total_ram * ram_fraction);

    char budget_mb[CBM_SZ_32];
    char ram_mb[CBM_SZ_32];
    snprintf(budget_mb, sizeof(budget_mb), "%zu", g_budget / MB_DIVISOR);
    snprintf(ram_mb, sizeof(ram_mb), "%zu", info.total_ram / MB_DIVISOR);
    cbm_log_info("mem.init", "budget_mb", budget_mb, "total_ram_mb", ram_mb);
}

size_t cbm_mem_rss(void) {
    size_t current_rss = 0;
    size_t peak_rss = 0;
    mi_process_info(NULL, NULL, NULL, &current_rss, &peak_rss, NULL, NULL, NULL);
    if (current_rss > 0) {
        return current_rss;
    }
    /* Fallback for ASan builds (MI_OVERRIDE=0) */
    return os_rss();
}

size_t cbm_mem_peak_rss(void) {
    size_t peak_rss = 0;
    mi_process_info(NULL, NULL, NULL, NULL, &peak_rss, NULL, NULL, NULL);
    if (peak_rss > 0) {
        return peak_rss;
    }
    /* No OS fallback for peak — return current as best approximation */
    return os_rss();
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

static _Atomic int64_t g_audit_bytes = 0;

static void trigger_oom_abort(const char *op, size_t size, const char *file, int line) {
    /* Statically allocated message to guarantee we can print without any more RAM */
    char msg[CBM_SZ_256];
    snprintf(msg, sizeof(msg), "FATAL: Out of Memory! Failed to %s %zu bytes at %s:%d\n", op, size, file, line);
    fputs(msg, stderr);
    fflush(stderr);
    exit(EXIT_FAILURE);
}

void* cbm_malloc_safe(size_t size, const char *file, int line) {
    void *p = malloc(size);
    if (!p) trigger_oom_abort("malloc", size, file, line);
    atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    return p;
}

void* cbm_calloc_safe(size_t count, size_t size, const char *file, int line) {
    void *p = calloc(count, size);
    if (!p && (count * size) > 0) trigger_oom_abort("calloc", count * size, file, line);
    if (p) atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    return p;
}

void* cbm_realloc_safe(void* ptr, size_t size, const char *file, int line) {
    size_t old_size = ptr ? mi_usable_size(ptr) : 0;
    void *p = realloc(ptr, size);
    if (!p && size > 0) trigger_oom_abort("realloc", size, file, line);
    if (p) {
        atomic_fetch_sub(&g_audit_bytes, old_size);
        atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    }
    return p;
}

void cbm_free_safe(void* ptr, const char *file, int line) {
    (void)file; (void)line;
    if (ptr) {
        atomic_fetch_sub(&g_audit_bytes, mi_usable_size(ptr));
        free(ptr);
    }
}

char* cbm_strdup_safe(const char* s, const char *file, int line) {
    if (!s) return NULL;
    char *p = _strdup(s); /* Or standard strdup depending on platform, _strdup on MSVC */
#ifndef _WIN32
    if (!p) p = strdup(s);
#endif
    if (!p) trigger_oom_abort("strdup", strlen(s) + 1, file, line);
    atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    return p;
}

char* cbm_strndup_safe(const char* s, size_t n, const char *file, int line) {
    if (!s) return NULL;
    size_t len = 0;
    while (len < n && s[len]) len++;
    char *p = malloc(len + 1);
    if (!p) trigger_oom_abort("strndup", len + 1, file, line);
    memcpy(p, s, len);
    p[len] = '\0';
    atomic_fetch_add(&g_audit_bytes, mi_usable_size(p));
    return p;
}

void cbm_mem_print_audit(void) {
    int64_t leaked = atomic_load(&g_audit_bytes);
    if (leaked != 0) {
        fprintf(stderr, "MEMORY AUDIT FAILED: %lld bytes leaked!\n", (long long)leaked);
    } else {
        fprintf(stderr, "MEMORY AUDIT OK: 0 bytes leaked.\n");
    }
}

#endif /* CBM_HARDEN_MEMORY */
