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

target_link_libraries(run_test_mem_context PRIVATE mimalloc unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_mem_context PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
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

target_link_libraries(run_test_fqn PRIVATE xxHash::xxhash mimalloc unofficial::sqlite3::sqlite3)
if(WIN32)
    target_link_libraries(run_test_fqn PRIVATE ws2_32 psapi bcrypt crypt32 winhttp rpcrt4)
    if(MSVC)
        set_target_properties(run_test_fqn PROPERTIES LINK_FLAGS "/STACK:8388608")
    endif()
endif()
