add_executable(run_test_fqn
    test_fqn.c
    ../run_fqn.c
    ../src/pipeline/fqn.c
    ../src/foundation/platform.c
    ../src/foundation/str_util.c
    ../src/foundation/arena.c
)

target_link_libraries(run_test_fqn PRIVATE xxHash::xxhash)
