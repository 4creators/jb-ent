add_library(cbm_core STATIC
    pipeline/fqn.c
    foundation/platform.c
    ../vendored/xxhash/xxhash.c
)

target_include_directories(cbm_core PUBLIC 
    ${CMAKE_CURRENT_SOURCE_DIR} 
    ${CMAKE_CURRENT_SOURCE_DIR}/../vendored
)

target_link_libraries(cbm_core PUBLIC ws2_32 psapi)
