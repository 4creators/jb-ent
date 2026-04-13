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

# Glob all source files
file(GLOB_RECURSE FOUNDATION_SRCS "src/foundation/*.c")
file(GLOB_RECURSE STORE_SRCS "src/store/*.c")
file(GLOB_RECURSE CYPHER_SRCS "src/cypher/*.c")
file(GLOB_RECURSE MCP_SRCS "src/mcp/*.c")
file(GLOB_RECURSE DISCOVER_SRCS "src/discover/*.c")
file(GLOB_RECURSE GRAPH_BUFFER_SRCS "src/graph_buffer/*.c")
file(GLOB_RECURSE PIPELINE_SRCS "src/pipeline/*.c")
file(GLOB_RECURSE SIMHASH_SRCS "src/simhash/*.c")
file(GLOB_RECURSE SEMANTIC_SRCS "src/semantic/*.c")
file(GLOB_RECURSE TRACES_SRCS "src/traces/*.c")
file(GLOB_RECURSE WATCHER_SRCS "src/watcher/*.c")
file(GLOB_RECURSE CLI_SRCS "src/cli/*.c")
file(GLOB_RECURSE UI_SRCS "src/ui/*.c")

# Tree-sitter extraction and grammar sources
file(GLOB CBM_SRCS "internal/cbm/*.c")
list(REMOVE_ITEM CBM_SRCS "${CMAKE_CURRENT_SOURCE_DIR}/internal/cbm/arena.c")
file(GLOB CBM_CXX_SRCS "internal/cbm/*.cpp")

# Vendored C files not in vcpkg
set(VENDORED_SRCS
    vendored/tre/tre_all.c
    vendored/yyjson/yyjson.c
    vendored/mongoose/mongoose.c
)

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

# Create the main executable target
add_executable(codebase-memory-mcp
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
