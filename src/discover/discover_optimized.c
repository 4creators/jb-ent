#include "discover.h"
#include "foundation/mem_context.h"
#include "pipeline/worker_pool.h"
#include "foundation/platform.h"
#include "foundation/compat_fs.h"
#include "foundation/log.h"
#include "foundation/allocator.h"
#include "foundation/constants.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/stat.h>

#ifdef _WIN32
#include <windows.h>
#ifndef S_ISDIR
#define S_ISDIR(mode)  (((mode) & S_IFMT) == S_IFDIR)
#endif
#ifndef S_ISREG
#define S_ISREG(mode)  (((mode) & S_IFMT) == S_IFREG)
#endif
#else
#include <pthread.h>
#endif

/* ── Helpers ───────────────────────────────────────────────────────── */

static const char *OPT_IGNORED_JSON_FILES[] = {
    "package.json",       "package-lock.json", "tsconfig.json",
    "jsconfig.json",      "composer.json",     "composer.lock",
    "yarn.lock",          "openapi.json",      "swagger.json",
    "jest.config.json",   ".eslintrc.json",    ".prettierrc.json",
    ".babelrc.json",      "tslint.json",       "angular.json",
    "firebase.json",      "renovate.json",     "lerna.json",
    "turbo.json",         ".stylelintrc.json", "pnpm-lock.json",
    "deno.json",          "biome.json",        "devcontainer.json",
    ".devcontainer.json", "launch.json",       "settings.json",
    "extensions.json",    "tasks.json",        NULL};

static bool opt_str_in_list(const char *s, const char *const *list) {
    for (int i = 0; list[i]; i++) {
        if (strcmp(s, list[i]) == 0) return true;
    }
    return false;
}

static CBMLanguage opt_detect_file_language(const char *entry_name, const char *abs_path) {
    CBMLanguage lang = cbm_language_for_filename(entry_name);
    if (lang == CBM_LANG_COUNT) return CBM_LANG_COUNT;
    const char *dot = strrchr(entry_name, '.');
    if (dot && strcmp(dot, ".m") == 0) {
        lang = cbm_disambiguate_m(abs_path);
    }
    if (lang == CBM_LANG_JSON && opt_str_in_list(entry_name, OPT_IGNORED_JSON_FILES)) {
        return CBM_LANG_COUNT;
    }
    return lang;
}

static int opt_safe_stat(const char *abs_path, struct stat *st) {
#ifdef _WIN32
    if (stat(abs_path, st) != 0) return -1;
#else
    if (lstat(abs_path, st) != 0) return -1;
    if (S_ISLNK(st->st_mode)) return -1;
#endif
    return 0;
}

/* ── Data Structures ───────────────────────────────────────────────── */

typedef struct {
    char *abs_dir;
    char *rel_dir;
} dir_task_t;

typedef struct {
    dir_task_t *tasks;
    int head;
    int tail;
    int capacity;
    int active_workers;
    bool done;

#ifdef _WIN32
    CRITICAL_SECTION cs;
    CONDITION_VARIABLE cv;
#else
    pthread_mutex_t mutex;
    pthread_cond_t cond;
#endif
} work_queue_t;

typedef struct {
    char *rel_dir_path;
    char *filename;
    CBMLanguage language;
    int64_t size;
} internal_file_info_t;

typedef struct {
    internal_file_info_t *files;
    int count;
    int capacity;
} thread_local_files_t;

typedef struct {
    int worker_id;
    work_queue_t *q;
    thread_local_files_t *local_files;
    const cbm_discover_opts_t *opts;
    const cbm_gitignore_t *gitignore;
    const cbm_gitignore_t *cbmignore;
} worker_ctx_t;

/* ── Queue Implementation ──────────────────────────────────────────── */

static void queue_init(work_queue_t *q, int capacity) {
    q->capacity = capacity;
    q->head = 0;
    q->tail = 0;
    q->active_workers = 0;
    q->done = false;
    q->tasks = CBM_MALLOC(capacity * sizeof(dir_task_t));
#ifdef _WIN32
    InitializeCriticalSection(&q->cs);
    InitializeConditionVariable(&q->cv);
#else
    pthread_mutex_init(&q->mutex, NULL);
    pthread_cond_init(&q->cond, NULL);
#endif
}

static void queue_destroy(work_queue_t *q) {
    for (int i = q->head; i < q->tail; i++) {
        CBM_FREE(q->tasks[i % q->capacity].abs_dir);
        CBM_FREE(q->tasks[i % q->capacity].rel_dir);
    }
    CBM_FREE(q->tasks);
#ifdef _WIN32
    DeleteCriticalSection(&q->cs);
#else
    pthread_mutex_destroy(&q->mutex);
    pthread_cond_destroy(&q->cond);
#endif
}

static void queue_lock(work_queue_t *q) {
#ifdef _WIN32
    EnterCriticalSection(&q->cs);
#else
    pthread_mutex_lock(&q->mutex);
#endif
}

static void queue_unlock(work_queue_t *q) {
#ifdef _WIN32
    LeaveCriticalSection(&q->cs);
#else
    pthread_mutex_unlock(&q->mutex);
#endif
}

static void queue_signal_all(work_queue_t *q) {
#ifdef _WIN32
    WakeAllConditionVariable(&q->cv);
#else
    pthread_cond_broadcast(&q->cond);
#endif
}

static void queue_wait(work_queue_t *q) {
#ifdef _WIN32
    SleepConditionVariableCS(&q->cv, &q->cs, INFINITE);
#else
    pthread_cond_wait(&q->cond, &q->mutex);
#endif
}

static void queue_push(work_queue_t *q, const char *abs_dir, const char *rel_dir) {
    queue_lock(q);
    int count = q->tail - q->head;
    if (count >= q->capacity) {
        int new_cap = q->capacity * 2;
        dir_task_t *new_tasks = CBM_MALLOC(new_cap * sizeof(dir_task_t));
        for (int i = 0; i < count; i++) {
            new_tasks[i] = q->tasks[(q->head + i) % q->capacity];
        }
        CBM_FREE(q->tasks);
        q->tasks = new_tasks;
        q->head = 0;
        q->tail = count;
        q->capacity = new_cap;
    }
    q->tasks[q->tail % q->capacity].abs_dir = strdup(abs_dir);
    q->tasks[q->tail % q->capacity].rel_dir = strdup(rel_dir);
    q->tail++;
    queue_signal_all(q);
    queue_unlock(q);
}

static bool queue_pop(work_queue_t *q, dir_task_t *out) {
    queue_lock(q);
    while (q->head == q->tail && !q->done && q->active_workers > 0) {
        queue_wait(q);
    }
    if (q->head == q->tail) {
        q->done = true;
        queue_signal_all(q);
        queue_unlock(q);
        return false;
    }
    *out = q->tasks[q->head % q->capacity];
    q->head++;
    q->active_workers++;
    queue_unlock(q);
    return true;
}

static void queue_task_done(work_queue_t *q) {
    queue_lock(q);
    q->active_workers--;
    if (q->head == q->tail && q->active_workers == 0) {
        q->done = true;
    }
    queue_signal_all(q);
    queue_unlock(q);
}

/* ── Thread-local Files ────────────────────────────────────────────── */

static void tl_files_init(thread_local_files_t *tl) {
    tl->count = 0;
    tl->capacity = 1024;
    tl->files = CBM_MALLOC(tl->capacity * sizeof(internal_file_info_t));
}

static void tl_files_add(thread_local_files_t *tl, const char *rel_dir, const char *filename, CBMLanguage lang, int64_t size) {
    if (tl->count >= tl->capacity) {
        tl->capacity *= 2;
        tl->files = CBM_REALLOC(tl->files, tl->capacity * sizeof(internal_file_info_t));
    }
    tl->files[tl->count].rel_dir_path = CBM_STRDUP(rel_dir);
    tl->files[tl->count].filename = CBM_STRDUP(filename);
    tl->files[tl->count].language = lang;
    tl->files[tl->count].size = size;
    tl->count++;
}

static void tl_files_destroy(thread_local_files_t *tl) {
    for (int i = 0; i < tl->count; i++) {
        CBM_FREE(tl->files[i].rel_dir_path);
        CBM_FREE(tl->files[i].filename);
    }
    CBM_FREE(tl->files);
}

/* ── Worker Function ───────────────────────────────────────────────── */

static bool opt_should_skip_directory(const char *entry_name, const char *rel_path,
                                  const cbm_discover_opts_t *opts, const cbm_gitignore_t *gitignore,
                                  const cbm_gitignore_t *cbmignore) {
    cbm_index_mode_t mode = opts ? opts->mode : CBM_MODE_FULL;
    if (cbm_should_skip_dir(entry_name, mode)) return true;
    if (gitignore && cbm_gitignore_matches(gitignore, rel_path, true)) return true;
    if (cbmignore && cbm_gitignore_matches(cbmignore, rel_path, true)) return true;
    return false;
}

static bool opt_should_skip_file(const char *entry_name, const char *rel_path,
                             const cbm_discover_opts_t *opts, const cbm_gitignore_t *gitignore,
                             const cbm_gitignore_t *cbmignore, off_t file_size) {
    cbm_index_mode_t mode = opts ? opts->mode : CBM_MODE_FULL;
    if (cbm_has_ignored_suffix(entry_name, mode)) return true;
    if (cbm_should_skip_filename(entry_name, mode)) return true;
    if (cbm_matches_fast_pattern(entry_name, mode)) return true;
    if (gitignore && cbm_gitignore_matches(gitignore, rel_path, false)) return true;
    if (cbmignore && cbm_gitignore_matches(cbmignore, rel_path, false)) return true;
    if (opts && opts->max_file_size > 0 && file_size > opts->max_file_size) return true;
    return false;
}

#ifdef _WIN32
static DWORD WINAPI worker_thread(LPVOID arg) {
#else
static void *worker_thread(void *arg) {
#endif
    worker_ctx_t *ctx = (worker_ctx_t *)arg;
    dir_task_t task;

    while (queue_pop(ctx->q, &task)) {
        cbm_dir_t *d = cbm_opendir(task.abs_dir);
        if (d) {
            cbm_dirent_t *entry;
            while ((entry = cbm_readdir(d)) != NULL) {
                char abs_path[CBM_SZ_4K];
                char rel_path[CBM_SZ_4K];
                snprintf(abs_path, sizeof(abs_path), "%s/%s", task.abs_dir, entry->name);
                if (task.rel_dir[0] != '\0') {
                    snprintf(rel_path, sizeof(rel_path), "%s/%s", task.rel_dir, entry->name);
                } else {
                    snprintf(rel_path, sizeof(rel_path), "%s", entry->name);
                }

                if (entry->is_dir) {
                    if (!opt_should_skip_directory(entry->name, rel_path, ctx->opts, ctx->gitignore, ctx->cbmignore)) {
                        queue_push(ctx->q, abs_path, rel_path);
                    }
                } else {
                    int64_t file_size = entry->file_size;
                    if (file_size < 0) {
                        struct stat st;
                        if (opt_safe_stat(abs_path, &st) != 0) continue;
                        if (!S_ISREG(st.st_mode)) continue;
                        file_size = (int64_t)st.st_size;
                    }
                    if (!opt_should_skip_file(entry->name, rel_path, ctx->opts, ctx->gitignore, ctx->cbmignore, file_size)) {
                        CBMLanguage lang = opt_detect_file_language(entry->name, abs_path);
                        if (lang != CBM_LANG_COUNT) {
                            tl_files_add(ctx->local_files, task.rel_dir, entry->name, lang, file_size);
                        }
                    }
                }
            }
            cbm_closedir(d);
        }
        free(task.abs_dir);
        free(task.rel_dir);
        queue_task_done(ctx->q);
    }
    
#ifdef _WIN32
    return 0;
#else
    return NULL;
#endif
}

/* ── Public API ────────────────────────────────────────────────────── */

static CBMContext *g_discover_ctx = NULL;

void cbm_discover_cleanup(void) {
    if (g_discover_ctx) {
        cbm_ctx_free(g_discover_ctx);
        g_discover_ctx = NULL;
    }
}

int cbm_discover_optimized(const char *repo_path, const cbm_discover_opts_t *opts, cbm_file_info_t **out, int *count) {
    if (!repo_path || !out || !count) return CBM_NOT_FOUND;

    *out = NULL;
    *count = 0;

    struct stat st;
    if (stat(repo_path, &st) != 0 || !S_ISDIR(st.st_mode)) {
        return CBM_NOT_FOUND;
    }

    cbm_gitignore_t *gitignore = NULL;
    char gi_path[CBM_SZ_4K];
    snprintf(gi_path, sizeof(gi_path), "%s/.gitignore", repo_path);
    if (stat(gi_path, &st) == 0 && S_ISREG(st.st_mode)) {
        gitignore = cbm_gitignore_load(gi_path);
    }

    cbm_gitignore_t *cbmignore = NULL;
    if (opts && opts->ignore_file && opts->ignore_file[0] != '\0') {
        cbmignore = cbm_gitignore_load(opts->ignore_file);
    } else {
        snprintf(gi_path, sizeof(gi_path), "%s/.cbmignore", repo_path);
        if (stat(gi_path, &st) == 0 && S_ISREG(st.st_mode)) {
            cbmignore = cbm_gitignore_load(gi_path);
        }
    }

    int num_workers = cbm_default_worker_count(true);
    if (num_workers < 1) num_workers = 1;

    work_queue_t q;
    queue_init(&q, 1024);
    
    // Initial push
    queue_push(&q, repo_path, "");

    worker_ctx_t *workers = CBM_MALLOC(num_workers * sizeof(worker_ctx_t));
#ifdef _WIN32
    HANDLE *threads = CBM_MALLOC(num_workers * sizeof(HANDLE));
#else
    pthread_t *threads = CBM_MALLOC(num_workers * sizeof(pthread_t));
#endif

    for (int i = 0; i < num_workers; i++) {
        workers[i].worker_id = i;
        workers[i].q = &q;
        workers[i].opts = opts;
        workers[i].gitignore = gitignore;
        workers[i].cbmignore = cbmignore;
        workers[i].local_files = CBM_MALLOC(sizeof(thread_local_files_t));
        tl_files_init(workers[i].local_files);
        
#ifdef _WIN32
        threads[i] = CreateThread(NULL, 0, worker_thread, &workers[i], 0, NULL);
#else
        pthread_create(&threads[i], NULL, worker_thread, &workers[i]);
#endif
    }

    for (int i = 0; i < num_workers; i++) {
#ifdef _WIN32
        WaitForSingleObject(threads[i], INFINITE);
        CloseHandle(threads[i]);
#else
        pthread_join(threads[i], NULL);
#endif
    }

    /* Phase 2: Merge and Deduplicate */
    int total_files = 0;
    for (int i = 0; i < num_workers; i++) {
        total_files += workers[i].local_files->count;
    }

    CBMContext *ctx = cbm_ctx_new();
    if (g_discover_ctx) cbm_ctx_free(g_discover_ctx);
    g_discover_ctx = ctx;

    cbm_file_info_t *final_files = CBM_MALLOC(total_files * sizeof(cbm_file_info_t));
    int f_idx = 0;

    for (int w = 0; w < num_workers; w++) {
        thread_local_files_t *tl = workers[w].local_files;
        for (int i = 0; i < tl->count; i++) {
            internal_file_info_t *fi = &tl->files[i];
            const char *interned_dir = cbm_ctx_intern(ctx, fi->rel_dir_path);
            
            cbm_file_info_t *out_fi = &final_files[f_idx++];
            out_fi->language = fi->language;
            out_fi->size = fi->size;
            out_fi->rel_dir_path = interned_dir;
            out_fi->filename = cbm_ctx_intern(ctx, fi->filename);
            
            size_t rel_len = strlen(interned_dir) + 1 + strlen(out_fi->filename) + 1;
            size_t abs_len = strlen(repo_path) + 1 + rel_len;
            
            char *p_abs = CBM_MALLOC(abs_len);
            
            if (interned_dir[0] != '\0') {
                snprintf(p_abs, abs_len, "%s/%s/%s", repo_path, interned_dir, out_fi->filename);
            } else {
                snprintf(p_abs, abs_len, "%s/%s", repo_path, out_fi->filename);
            }
            
            out_fi->path = p_abs;
            size_t repo_len = strlen(repo_path);
            if (p_abs[repo_len] == '/' || p_abs[repo_len] == '\\') {
                out_fi->rel_path = p_abs + repo_len + 1;
            } else {
                out_fi->rel_path = p_abs + repo_len;
            }
        }
        tl_files_destroy(tl);
        CBM_FREE(tl);
    }

    *out = final_files;
    *count = total_files;

    CBM_FREE(workers);
    CBM_FREE(threads);
    queue_destroy(&q);

    if (gitignore) cbm_gitignore_free(gitignore);
    if (cbmignore) cbm_gitignore_free(cbmignore);

    return 0;
}
