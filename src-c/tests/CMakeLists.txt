# src-c/tests/CMakeLists.txt - Simplified test executables using the core library

add_executable(run_test_edge_set ../run_test_edge_set.c)
target_link_libraries(run_test_edge_set PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib)

add_executable(run_test_int_hash_table ../run_test_int_hash_table.c)
target_link_libraries(run_test_int_hash_table PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib)

add_executable(run_test_mem_context ../run_test_mem_context.c)
target_link_libraries(run_test_mem_context PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib)

add_executable(run_test_graph_buffer ../run_test_graph_buffer.c)
target_link_libraries(run_test_graph_buffer PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib)

add_executable(run_test_fqn ../run_fqn.c test_fqn.c)
target_link_libraries(run_test_fqn PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib)

add_executable(run_test_parse_lean run_test_parse_lean.c)
target_link_libraries(run_test_parse_lean PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib)

add_executable(test_openssl_version test_openssl_version.c)
target_link_libraries(test_openssl_version PRIVATE OpenSSL::Crypto)

add_executable(test_mldsa_full test_mldsa_full.c)
target_include_directories(test_mldsa_full PRIVATE ../src)
target_link_libraries(test_mldsa_full PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib OpenSSL::Crypto)

add_executable(run_roundtrip_proof run_roundtrip_proof.c)
target_include_directories(run_roundtrip_proof PRIVATE ../src ../vendored/tre)
target_link_libraries(run_roundtrip_proof PRIVATE ${MIMALLOC_LIB} codebase-memory-mcp-lib)

# Register tests for CTest
add_test(NAME test_edge_set COMMAND run_test_edge_set)
add_test(NAME test_int_hash_table COMMAND run_test_int_hash_table)
add_test(NAME test_mem_context COMMAND run_test_mem_context)
add_test(NAME test_graph_buffer COMMAND run_test_graph_buffer)
add_test(NAME test_fqn COMMAND run_test_fqn)
add_test(NAME test_parse_lean COMMAND run_test_parse_lean)
add_test(NAME test_openssl_version COMMAND test_openssl_version)
add_test(NAME test_mldsa_full COMMAND test_mldsa_full)
add_test(NAME test_roundtrip_proof COMMAND run_roundtrip_proof)
