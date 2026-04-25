add_executable(run_test_edge_set
    ../run_test_edge_set.c
    ../foundation/edge_set.c
    ../foundation/platform.c
    ../foundation/str_util.c
    ../foundation/arena.c
    ../foundation/mem.c
    ../foundation/log.c
    ../foundation/system_info.c
    ../foundation/compat.c
)

target_link_libraries(run_test_edge_set PRIVATE ${MIMALLOC_LIB} xxHash::xxhash unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_edge_set PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_executable(run_test_int_hash_table
    ../run_test_int_hash_table.c
    ../foundation/int_hash_table.c
    ../foundation/platform.c
    ../foundation/str_util.c
    ../foundation/arena.c
    ../foundation/mem.c
    ../foundation/log.c
    ../foundation/system_info.c
    ../foundation/compat.c
)

target_link_libraries(run_test_int_hash_table PRIVATE ${MIMALLOC_LIB} unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_int_hash_table PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_executable(run_test_mem_context
    ../run_test_mem_context.c
    ../foundation/mem_context.c
    ../foundation/str_intern.c
    ../foundation/arena.c
    ../foundation/mem.c
    ../foundation/log.c
    ../foundation/platform.c
    ../foundation/compat.c
    ../foundation/system_info.c
)

target_link_libraries(run_test_mem_context PRIVATE ${MIMALLOC_LIB} unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_mem_context PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
endif()

add_executable(run_test_graph_buffer
    ../run_test_graph_buffer.c
    ../graph_buffer/graph_buffer.c
    ../foundation/edge_set.c
    ../foundation/int_hash_table.c
    ../foundation/hash_table.c
    ../foundation/str_intern.c
    ../foundation/mem_context.c
    ../foundation/arena.c
    ../foundation/mem.c
    ../foundation/log.c
    ../foundation/platform.c
    ../foundation/system_info.c
    ../foundation/compat.c
    ../foundation/compat_thread.c
    ../foundation/compat_regex.c
    ../foundation/str_util.c
    ../foundation/profile.c
    ../store/store.c
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
    ../pipeline/fqn.c
    ../foundation/platform.c
    ../foundation/str_util.c
    ../foundation/arena.c
    ../foundation/mem.c
    ../foundation/log.c
    ../foundation/system_info.c
    ../foundation/compat.c
)

target_link_libraries(run_test_fqn PRIVATE ${MIMALLOC_LIB} xxHash::xxhash unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_fqn PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
    if(MSVC)
        set_target_properties(run_test_fqn PROPERTIES LINK_FLAGS "/STACK:8388608")
    endif()
endif()

add_executable(run_test_parse_lean
    run_test_parse_lean.c
)

target_link_libraries(run_test_parse_lean PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib unofficial::sqlite3::sqlite3 xxHash::xxhash lz4::lz4 libgit2::libgit2package)
if(WIN32)
    target_link_libraries(run_test_parse_lean PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
    if(MSVC)
        set_target_properties(run_test_parse_lean PROPERTIES LINK_FLAGS "/STACK:8388608")
    endif()
endif()

add_executable(run_test_openssl_version test_openssl_version.c)
target_link_libraries(run_test_openssl_version PRIVATE OpenSSL::Crypto)

add_executable(run_roundtrip_proof run_roundtrip_proof.c)
target_include_directories(run_roundtrip_proof PRIVATE ../src ../vendored/tre)
target_link_libraries(run_roundtrip_proof PRIVATE codebase-memory-mcp-lib)

add_executable(test_openssl_version test_openssl_version.c)
target_link_libraries(test_openssl_version PRIVATE OpenSSL::Crypto)

add_executable(test_mldsa_full test_mldsa_full.c)
target_include_directories(test_mldsa_full PRIVATE ../src)
target_link_libraries(test_mldsa_full PRIVATE codebase-memory-mcp-lib OpenSSL::Crypto)
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
