cmake_minimum_required(VERSION 3.14...3.22)

project(lb_tests LANGUAGES CXX)

set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    message(STATUS "Using '${CMAKE_CXX_COMPILER_ID}' compiler")
    link_directories("${CMAKE_PREFIX_PATH}/lib/c++")
endif()

if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    message(STATUS "Using '${CMAKE_CXX_COMPILER_ID}' compiler")
    link_directories("${CMAKE_PREFIX_PATH}/lib/gcc/current")
endif()


# ---- Options ----

option(ENABLE_TEST_COVERAGE "Enable test coverage" OFF)
option(TEST_INSTALLED_VERSION "Test the version found by find_package" OFF)

# --- Import tools ----

include(../cmake/tools.cmake)

# ---- Dependencies ----

include(../cmake/CPM.cmake)

CPMAddPackage("gh:doctest/doctest@2.4.11")
CPMAddPackage("gh:TheLartians/Format.cmake@1.7.3")

if(TEST_INSTALLED_VERSION)
  find_package(lb_lib REQUIRED)
else()
  CPMAddPackage(NAME lb_lib SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/..)
endif()

# ---- Create binary ----

file(GLOB sources CONFIGURE_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/source/*.cpp)
add_executable(${PROJECT_NAME} ${sources})
target_link_libraries(${PROJECT_NAME}
    doctest::doctest
    drogon
    fmt::fmt
    spdlog::spdlog
    CLI11::CLI11
    lb_lib
    )
set_target_properties(${PROJECT_NAME} PROPERTIES CXX_STANDARD 20)

# enable compiler warnings
if(NOT TEST_INSTALLED_VERSION)
  if(CMAKE_CXX_COMPILER_ID MATCHES "Clang" OR CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    target_compile_options(lb_lib PUBLIC -Wall -Wpedantic -Wextra)
  elseif(MSVC)
    target_compile_options(lb_lib PUBLIC /W4 /WX)
    target_compile_definitions(${PROJECT_NAME} PUBLIC DOCTEST_CONFIG_USE_STD_HEADERS)
  endif()
endif()

# ---- Add Tests ----

enable_testing()

# Note: doctest and similar testing frameworks can automatically configure CMake tests. For other
# testing frameworks add the tests target instead: add_test(NAME ${PROJECT_NAME} COMMAND
# ${PROJECT_NAME})

include(${doctest_SOURCE_DIR}/scripts/cmake/doctest.cmake)
doctest_discover_tests(${PROJECT_NAME})

# ---- code coverage ----

if(ENABLE_TEST_COVERAGE)
  target_compile_options(lb_lib PUBLIC -O0 -g -fprofile-arcs -ftest-coverage)
  target_link_options(lb_lib PUBLIC -fprofile-arcs -ftest-coverage)
endif()
