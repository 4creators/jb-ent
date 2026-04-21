// Unity build: include simplecpp implementation directly since CGo only
// compiles .cpp files from the immediate package directory, not subdirs.
#include "foundation/allocator_tracker.h"

#include "vendored/simplecpp/simplecpp.cpp"

#include "preprocessor.h"
#include "vendored/simplecpp/simplecpp.h"

#include <sstream>
#include <string>
#include <vector>
#include <cstring>
#include <cstdlib>

extern "C" {
#include "foundation/allocator.h"
}

extern "C" {
char* cbm_preprocess(
    const char* source, int source_len,
    const char* filename,
    const char** extra_defines,
    const char** include_paths,
    int cpp_mode
) {
    if (!source || source_len <= 0) return NULL;

    // Fast-path: skip if no preprocessor directives worth expanding.
    bool has_macros = false;
    for (int i = 0; i < source_len - 1; i++) {
        if (source[i] == '#') {
            // Skip whitespace after #
            int j = i + 1;
            while (j < source_len && (source[j] == ' ' || source[j] == '\t')) j++;
            int remaining = source_len - j;
            if (remaining >= 6 && strncmp(source + j, "define", 6) == 0) {
                has_macros = true;
                break;
            }
            if (remaining >= 5 && strncmp(source + j, "ifdef", 5) == 0) {
                has_macros = true;
                break;
            }
            if (remaining >= 6 && strncmp(source + j, "ifndef", 6) == 0) {
                has_macros = true;
                break;
            }
            if (remaining >= 3 && strncmp(source + j, "if ", 3) == 0) {
                has_macros = true;
                break;
            }
        }
    }
    if (!has_macros) return NULL;  // NULL = no expansion needed, use original

    try {
        simplecpp::DUI dui;
        dui.clearIncludeCache = true; // Clear the static cache
        if (extra_defines) {
            for (int i = 0; extra_defines[i]; i++)
                dui.defines.push_back(extra_defines[i]);
        }
        if (include_paths) {
            for (int i = 0; include_paths[i]; i++)
                dui.includePaths.push_back(include_paths[i]);
        }
        dui.std = cpp_mode ? "c++17" : "c11";

        std::string src(source, source_len);
        std::istringstream istr(src);
        std::vector<std::string> files;
        files.push_back(filename ? filename : "<input>");

        simplecpp::TokenList rawtokens(istr, files, files[0]);
        simplecpp::TokenList output(files);
        simplecpp::FileDataCache filedata = simplecpp::load(rawtokens, files, dui);

        simplecpp::preprocess(output, rawtokens, files, filedata, dui);

        std::string result = output.stringify();

        // Clean up loaded file data
        simplecpp::cleanup(filedata);

        char* out = (char*)CBM_MALLOC(result.size() + 1);
        if (!out) return NULL;
        memcpy(out, result.c_str(), result.size() + 1);
        return out;
    } catch (...) {
        // Graceful fallback: return NULL = use original source
        return NULL;
    }
}

void cbm_preprocess_free(char* expanded) {
    CBM_FREE(expanded);
}

} // extern "C"

#include <new>

// Global operator new/delete overrides to ensure simplecpp allocations are tracked
// by the codebase-memory-mcp memory budget auditor (cbm_malloc_safe).
void* operator new(std::size_t size) {
    void* p = CBM_MALLOC(size);
    if (!p) throw std::bad_alloc();
    return p;
}

void* operator new[](std::size_t size) {
    void* p = CBM_MALLOC(size);
    if (!p) throw std::bad_alloc();
    return p;
}

void operator delete(void* p) noexcept {
    CBM_FREE(p);
}

void operator delete[](void* p) noexcept {
    CBM_FREE(p);
}

#if __cplusplus >= 201402L || (defined(_MSC_VER) && _MSC_VER >= 1916)
void operator delete(void* p, std::size_t size) noexcept {
    (void)size;
    CBM_FREE(p);
}

void operator delete[](void* p, std::size_t size) noexcept {
    (void)size;
    CBM_FREE(p);
}
#endif
