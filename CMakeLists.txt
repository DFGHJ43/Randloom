cmake_minimum_required(VERSION 3.10)
project(Randloom C)

set(CMAKE_C_STANDARD 99)

add_executable(randloom
    src/tui.c
    src/term.c
    src/controls.c
    src/random.c
    src/stats.c
    src/output.c
    src/graph/graph.c
    src/graph/block.c
    src/graph/graph_uniform.c
    src/graph/graph_normal.c
    src/graph/graph_bernoulli.c
    src/graph/graph_poisson.c
)

# `m` (libm) is a Unix library; MSVC/clang-cl have no m.lib
if(NOT MSVC)
    target_link_libraries(randloom m)
endif()
