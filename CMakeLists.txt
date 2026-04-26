cmake_minimum_required(VERSION 3.15)
project(CodebaseMemoryMCP C CXX)

enable_testing()

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 14)

option(CBM_ENABLE_ASAN "Enable AddressSanitizer" OFF)
option(CBM_HARDEN_MEMORY "Enable memory hardening" ON)

if(CBM_ENABLE_ASAN)
    set(MI_OVERRIDE_VAL 0)
    if(MSVC)
        add_compile_options(/fsanitize=address)
        # MSVC requires /MD or /MT to link ASan runtime properly. Default is /MD.
    else()
        add_compile_options(-fsanitize=address -fno-omit-frame-pointer)
        add_link_options(-fsanitize=address)
    endif()
else()
    set(MI_OVERRIDE_VAL 1)
endif()

if(CBM_HARDEN_MEMORY)
    add_definitions(-DCBM_HARDEN_MEMORY)
endif()
# Global compiler definitions
add_definitions(
    -D_CRT_SECURE_NO_WARNINGS
    -DTREE_SITTER_REUSE_ALLOCATOR
    -D_CRT_NONSTDC_NO_DEPRECATE 
    -D_DEFAULT_SOURCE 
    -D_GNU_SOURCE
    -DMG_ENABLE_LOG=0
    -DMG_TLS=2
    -DHAVE_LIBGIT2=1
    -DMI_OVERRIDE=${MI_OVERRIDE_VAL}
    -DCBM_ENABLE_OPTIMIZED=1
    -DMG_ENABLE_CUSTOM_CALLOC=1
)

if(MSVC)
    add_compile_options(/utf-8 /W3 /wd4005 /wd4996 /wd4018 /wd4244 /wd4267 /wd4146 /wd4090 /experimental:c11atomics)
    add_compile_options("$<$<CONFIG:Release>:/WX>")
    add_compile_options("$<$<CONFIG:RelWithDebInfo>:/WX>")
else()
    add_compile_options("$<$<CONFIG:Release>:-Werror>")
    add_compile_options("$<$<CONFIG:RelWithDebInfo>:-Werror>")
endif()

# Find vcpkg dependencies
find_package(unofficial-sqlite3 CONFIG REQUIRED)
find_package(xxHash CONFIG REQUIRED)
find_package(lz4 CONFIG REQUIRED)
find_package(libgit2 CONFIG REQUIRED)
find_package(mimalloc CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)

# Include directories
include_directories(
    src-c
    src-c/internal/cbm
    src-c/vendored
    src-c/vendored/tre
    src-c/internal/cbm/vendored/ts_runtime/include
    src-c/internal/cbm/vendored/ts_runtime/src
)

# Glob all source files AND headers to make them show up in IDE
file(GLOB_RECURSE FOUNDATION_SRCS "src-c/foundation/*.c" "src-c/foundation/*.h")
file(GLOB_RECURSE STORE_SRCS "src-c/store/*.c" "src-c/store/*.h")
file(GLOB_RECURSE CYPHER_SRCS "src-c/cypher/*.c" "src-c/cypher/*.h")
file(GLOB_RECURSE MCP_SRCS "src-c/mcp/*.c" "src-c/mcp/*.h")
file(GLOB_RECURSE DISCOVER_SRCS "src-c/discover/*.c" "src-c/discover/*.h")
file(GLOB_RECURSE GRAPH_BUFFER_SRCS "src-c/graph_buffer/*.c" "src-c/graph_buffer/*.h")
file(GLOB_RECURSE PIPELINE_SRCS "src-c/pipeline/*.c" "src-c/pipeline/*.h")
file(GLOB_RECURSE SIMHASH_SRCS "src-c/simhash/*.c" "src-c/simhash/*.h")
file(GLOB_RECURSE SEMANTIC_SRCS "src-c/semantic/*.c" "src-c/semantic/*.h")
file(GLOB_RECURSE TRACES_SRCS "src-c/traces/*.c" "src-c/traces/*.h")
file(GLOB_RECURSE WATCHER_SRCS "src-c/watcher/*.c" "src-c/watcher/*.h")
file(GLOB_RECURSE CLI_SRCS "src-c/cli/*.c" "src-c/cli/*.h")
file(GLOB_RECURSE UI_SRCS "src-c/ui/*.c" "src-c/ui/*.h")

# Tree-sitter extraction and grammar sources
file(GLOB CBM_SRCS "src-c/internal/cbm/*.c" "src-c/internal/cbm/*.h")
list(REMOVE_ITEM CBM_SRCS "${CMAKE_CURRENT_SOURCE_DIR}/src-c/internal/cbm/arena.c")
file(GLOB CBM_CXX_SRCS "src-c/internal/cbm/*.cpp" "src-c/internal/cbm/*.hpp")

# Vendored C files not in vcpkg
file(GLOB_RECURSE VENDORED_SRCS
    "src-c/vendored/yyjson/*.c" "src-c/vendored/yyjson/*.h"
    "src-c/vendored/mongoose/*.c" "src-c/vendored/mongoose/*.h"
)
list(APPEND VENDORED_SRCS "src-c/vendored/tre/tre_all.c")
file(GLOB VENDORED_TRE_HEADERS "src-c/vendored/tre/*.h")
list(APPEND VENDORED_SRCS ${VENDORED_TRE_HEADERS})

# Build bin2c tool
add_executable(bin2c src-c/tools/bin2c.c)

# Generate C array from Nomic vector binary blob
set(VECTOR_BIN_FILE "${CMAKE_CURRENT_SOURCE_DIR}/src-c/vendored/nomic/code_vectors.bin")
set(VECTOR_C_FILE "${CMAKE_CURRENT_BINARY_DIR}/code_vectors_blob.c")

add_custom_command(
    OUTPUT ${VECTOR_C_FILE}
    COMMAND bin2c ${VECTOR_BIN_FILE} ${VECTOR_C_FILE} PRETRAINED_VECTOR_BLOB
    DEPENDS bin2c ${VECTOR_BIN_FILE}
    COMMENT "Converting Nomic vector blob to C array..."
)

# Group all sources to recreate directory structure in IDE
set(LIB_SOURCES
    ${FOUNDATION_SRCS}
    ${STORE_SRCS}
    ${CYPHER_SRCS}
    ${MCP_SRCS}
    ${DISCOVER_SRCS}
    ${GRAPH_BUFFER_SRCS}
    ${PIPELINE_SRCS}
    ${SIMHASH_SRCS}
    ${SEMANTIC_SRCS}
    ${TRACES_SRCS}
    ${WATCHER_SRCS}
    ${CLI_SRCS}
    ${UI_SRCS}
    ${CBM_SRCS}
    ${CBM_CXX_SRCS}
    ${VENDORED_SRCS}
    ${VECTOR_C_FILE}
)

# source_group(TREE ${CMAKE_CURRENT_SOURCE_DIR}/src-c FILES src-c/main.c ${LIB_SOURCES})

# --- Shared Dependencies & Compiler Flags ---
add_library(codebase-memory-mcp-deps INTERFACE)

target_link_libraries(codebase-memory-mcp-deps INTERFACE
    unofficial::sqlite3::sqlite3
    xxHash::xxhash
    lz4::lz4
    libgit2::libgit2package
    OpenSSL::SSL
    OpenSSL::Crypto
)

# Increase Stack Size on MSVC to 8MB to prevent silent stack overflow
if(MSVC)
    target_link_options(codebase-memory-mcp-deps INTERFACE "/STACK:8388608")
endif()

# Link Windows system libraries
if(WIN32)
    target_link_libraries(codebase-memory-mcp-deps INTERFACE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

# --- Approach A: Static Library and Linked Executables ---
add_library(codebase-memory-mcp-lib OBJECT ${LIB_SOURCES})

# IMPORTANT: mimalloc MUST be the very first library linked so it initializes before ucrtbased.dll
if(NOT CBM_ENABLE_ASAN)
    set(MIMALLOC_LIB mimalloc)
else()
    set(MIMALLOC_LIB "")
endif()

# Guarantee mimalloc is first on the link line!
target_link_libraries(codebase-memory-mcp-lib PUBLIC ${MIMALLOC_LIB})
target_link_libraries(codebase-memory-mcp-lib PUBLIC codebase-memory-mcp-deps)

# Create the main executable target
add_executable(codebase-memory-mcp src-c/main.c)
target_link_libraries(codebase-memory-mcp PRIVATE codebase-memory-mcp-lib)

# --- Approach B: Standalone Unified Executable (for Profiling) ---
add_executable(codebase-memory-mcp-exe src-c/main.c ${LIB_SOURCES})
# Guarantee mimalloc is first on the link line!
target_link_libraries(codebase-memory-mcp-exe PRIVATE ${MIMALLOC_LIB})
target_link_libraries(codebase-memory-mcp-exe PRIVATE codebase-memory-mcp-deps)

# Visual Studio Profiler explicitly requires these overrides in RelWithDebInfo
if(MSVC)
    target_link_options(codebase-memory-mcp-exe PRIVATE
        "$<$<CONFIG:RelWithDebInfo>:/PROFILE>"
        "$<$<CONFIG:RelWithDebInfo>:/INCREMENTAL:NO>"
        "$<$<CONFIG:RelWithDebInfo>:/OPT:NOICF>"
        "$<$<CONFIG:RelWithDebInfo>:/DEBUG:FULL>"
    )
endif()

add_subdirectory(src-c/tests)


