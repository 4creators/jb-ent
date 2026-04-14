cmake_minimum_required(VERSION 3.15)
project(CodebaseMemoryMCP C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 14)

# Global compiler definitions
add_definitions(
    -D_CRT_SECURE_NO_WARNINGS 
    -D_CRT_NONSTDC_NO_DEPRECATE 
    -D_DEFAULT_SOURCE 
    -D_GNU_SOURCE
    -DMG_ENABLE_LOG=0
    -DHAVE_LIBGIT2=1
    -DMI_OVERRIDE=1
    -DCBM_HARDEN_MEMORY
    -DMG_ENABLE_CUSTOM_CALLOC=1
)

if(MSVC)
    add_compile_options(/utf-8 /W3 /wd4005 /wd4996 /wd4018 /wd4244 /wd4267 /wd4146 /wd4090 /experimental:c11atomics)
endif()

# Find vcpkg dependencies
find_package(unofficial-sqlite3 CONFIG REQUIRED)
find_package(xxHash CONFIG REQUIRED)
find_package(lz4 CONFIG REQUIRED)
find_package(libgit2 CONFIG REQUIRED)
find_package(mimalloc CONFIG REQUIRED)

# Include directories
include_directories(
    src
    internal/cbm
    vendored
    vendored/tre
    internal/cbm/vendored/ts_runtime/include
    internal/cbm/vendored/ts_runtime/src
)

# Glob all source files AND headers to make them show up in IDE
file(GLOB_RECURSE FOUNDATION_SRCS "src/foundation/*.c" "src/foundation/*.h")
file(GLOB_RECURSE STORE_SRCS "src/store/*.c" "src/store/*.h")
file(GLOB_RECURSE CYPHER_SRCS "src/cypher/*.c" "src/cypher/*.h")
file(GLOB_RECURSE MCP_SRCS "src/mcp/*.c" "src/mcp/*.h")
file(GLOB_RECURSE DISCOVER_SRCS "src/discover/*.c" "src/discover/*.h")
file(GLOB_RECURSE GRAPH_BUFFER_SRCS "src/graph_buffer/*.c" "src/graph_buffer/*.h")
file(GLOB_RECURSE PIPELINE_SRCS "src/pipeline/*.c" "src/pipeline/*.h")
file(GLOB_RECURSE SIMHASH_SRCS "src/simhash/*.c" "src/simhash/*.h")
file(GLOB_RECURSE SEMANTIC_SRCS "src/semantic/*.c" "src/semantic/*.h")
file(GLOB_RECURSE TRACES_SRCS "src/traces/*.c" "src/traces/*.h")
file(GLOB_RECURSE WATCHER_SRCS "src/watcher/*.c" "src/watcher/*.h")
file(GLOB_RECURSE CLI_SRCS "src/cli/*.c" "src/cli/*.h")
file(GLOB_RECURSE UI_SRCS "src/ui/*.c" "src/ui/*.h")

# Tree-sitter extraction and grammar sources
file(GLOB CBM_SRCS "internal/cbm/*.c" "internal/cbm/*.h")
list(REMOVE_ITEM CBM_SRCS "${CMAKE_CURRENT_SOURCE_DIR}/internal/cbm/arena.c")
file(GLOB CBM_CXX_SRCS "internal/cbm/*.cpp" "internal/cbm/*.hpp")

# Vendored C files not in vcpkg
file(GLOB_RECURSE VENDORED_SRCS
    "vendored/yyjson/*.c" "vendored/yyjson/*.h"
    "vendored/mongoose/*.c" "vendored/mongoose/*.h"
)
list(APPEND VENDORED_SRCS "vendored/tre/tre_all.c")
file(GLOB VENDORED_TRE_HEADERS "vendored/tre/*.h")
list(APPEND VENDORED_SRCS ${VENDORED_TRE_HEADERS})

# Build bin2c tool
add_executable(bin2c tools/bin2c.c)

# Generate C array from Nomic vector binary blob
set(VECTOR_BIN_FILE "${CMAKE_CURRENT_SOURCE_DIR}/vendored/nomic/code_vectors.bin")
set(VECTOR_C_FILE "${CMAKE_CURRENT_BINARY_DIR}/code_vectors_blob.c")

add_custom_command(
    OUTPUT ${VECTOR_C_FILE}
    COMMAND bin2c ${VECTOR_BIN_FILE} ${VECTOR_C_FILE} PRETRAINED_VECTOR_BLOB
    DEPENDS bin2c ${VECTOR_BIN_FILE}
    COMMENT "Converting Nomic vector blob to C array..."
)

# Group all sources to recreate directory structure in IDE
set(ALL_SOURCES
    src/main.c
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

source_group(TREE ${CMAKE_CURRENT_SOURCE_DIR} FILES ${ALL_SOURCES})

# Create the main executable target
add_executable(codebase-memory-mcp ${ALL_SOURCES})

# Increase Stack Size on MSVC to 8MB to prevent silent stack overflow
if(MSVC)
    set_target_properties(codebase-memory-mcp PROPERTIES LINK_FLAGS "/STACK:8388608")
endif()

# Link vcpkg libraries
target_link_libraries(codebase-memory-mcp PRIVATE
    mimalloc
    unofficial::sqlite3::sqlite3
    xxHash::xxhash
    lz4::lz4
    libgit2::libgit2package
)

# Link Windows system libraries
if(WIN32)
    target_link_libraries(codebase-memory-mcp PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_subdirectory(tests)
