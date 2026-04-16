add_executable(run_test_edge_set
    ../run_test_edge_set.c
    ../src/foundation/edge_set.c
    ../src/foundation/platform.c
    ../src/foundation/str_util.c
    ../src/foundation/arena.c
    ../src/foundation/mem.c
    ../src/foundation/log.c
    ../src/foundation/system_info.c
    ../src/foundation/compat.c
)

target_link_libraries(run_test_edge_set PRIVATE ${MIMALLOC_LIB} xxHash::xxhash unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_edge_set PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_executable(run_test_int_hash_table
    ../run_test_int_hash_table.c
    ../src/foundation/int_hash_table.c
    ../src/foundation/platform.c
    ../src/foundation/str_util.c
    ../src/foundation/arena.c
    ../src/foundation/mem.c
    ../src/foundation/log.c
    ../src/foundation/system_info.c
    ../src/foundation/compat.c
)

target_link_libraries(run_test_int_hash_table PRIVATE ${MIMALLOC_LIB} unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_int_hash_table PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_executable(run_test_mem_context
    ../run_test_mem_context.c
    ../src/foundation/mem_context.c
    ../src/foundation/str_intern.c
    ../src/foundation/arena.c
    ../src/foundation/mem.c
    ../src/foundation/log.c
    ../src/foundation/platform.c
    ../src/foundation/compat.c
    ../src/foundation/system_info.c
)

target_link_libraries(run_test_mem_context PRIVATE ${MIMALLOC_LIB} unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_mem_context PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_executable(run_test_graph_buffer
    ../run_test_graph_buffer.c
    ../src/graph_buffer/graph_buffer.c
    ../src/foundation/edge_set.c
    ../src/foundation/int_hash_table.c
    ../src/foundation/hash_table.c
    ../src/foundation/str_intern.c
    ../src/foundation/mem_context.c
    ../src/foundation/arena.c
    ../src/foundation/mem.c
    ../src/foundation/log.c
    ../src/foundation/platform.c
    ../src/foundation/system_info.c
    ../src/foundation/compat.c
    ../src/foundation/compat_thread.c
    ../src/foundation/compat_regex.c
    ../src/foundation/str_util.c
    ../src/foundation/profile.c
    ../src/store/store.c
    ../internal/cbm/sqlite_writer.c
    ../vendored/tre/tre_all.c
)

target_link_libraries(run_test_graph_buffer PRIVATE ${MIMALLOC_LIB} xxHash::xxhash unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_graph_buffer PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_executable(run_test_fqn
    test_fqn.c
    ../run_fqn.c
    ../src/pipeline/fqn.c
    ../src/foundation/platform.c
    ../src/foundation/str_util.c
    ../src/foundation/arena.c
    ../src/foundation/mem.c
    ../src/foundation/log.c
    ../src/foundation/system_info.c
    ../src/foundation/compat.c
)

target_link_libraries(run_test_fqn PRIVATE ${MIMALLOC_LIB} xxHash::xxhash unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_fqn PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
    if(MSVC)
        set_target_properties(run_test_fqn PROPERTIES LINK_FLAGS "/STACK:8388608")
    endif()
endif()

add_executable(run_roundtrip_proof run_roundtrip_proof.c)
target_include_directories(run_roundtrip_proof PRIVATE ../src ../vendored/tre)
target_link_libraries(run_roundtrip_proof PRIVATE codebase-memory-mcp-lib)
if(NOT CBM_ENABLE_ASAN)
    target_link_libraries(run_roundtrip_proof PRIVATE ${MIMALLOC_LIB})
endif()

target_link_libraries(run_roundtrip_proof PRIVATE
    unofficial::sqlite3::sqlite3
    xxHash::xxhash
    lz4::lz4
    libgit2::libgit2package
)

if(WIN32)
    target_link_libraries(run_roundtrip_proof PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
    if(MSVC)
        set_target_properties(run_roundtrip_proof PROPERTIES LINK_FLAGS "/STACK:8388608")
    endif()
endif()
