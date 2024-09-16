cmake_minimum_required(VERSION 3.14...3.22)

# ---- Project ----

# Note: update this to your new project's name and version
project(
  lb
  VERSION 0.1
  LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
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


# ---- Include guards ----

if(PROJECT_SOURCE_DIR STREQUAL PROJECT_BINARY_DIR)
  message(
    FATAL_ERROR
      "In-source builds not allowed. Please make a new directory (called a build directory) and run CMake from there."
  )
endif()

# ---- Add dependencies via CPM ----
# see https://github.com/TheLartians/CPM.cmake for more info

include(cmake/CPM.cmake)

# PackageProject.cmake will be used to make our target installable
CPMAddPackage("gh:TheLartians/PackageProject.cmake@1.12.0")

CPMAddPackage("gh:fmtlib/fmt#11.0.2")

set(SPDLOG_FMT_EXTERNAL ON)
CPMAddPackage("gh:gabime/spdlog@1.14.1")

CPMAddPackage("gh:CLIUtils/CLI11@2.4.2")

CPMAddPackage(
    NAME drogon
    VERSION 1.9.5
    GITHUB_REPOSITORY drogonframework/drogon
    GIT_TAG v1.9.5
)

# ---- Add source files ----

# Note: globbing sources is considered bad practice as CMake's generators may not detect new files
# automatically. Keep that in mind when changing files, or explicitly mention them here.
file(GLOB_RECURSE headers CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/include/*.h")
file(GLOB_RECURSE sources CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/source/*.cpp")

# ---- Create standalone executable ----

# Note: for header-only libraries change all PUBLIC flags to INTERFACE and create an interface
# target: add_library(${PROJECT_NAME} INTERFACE)
add_executable(${PROJECT_NAME} ${headers} ${sources})
add_library(${PROJECT_NAME}_lib ${headers} ${sources})

set_target_properties(${PROJECT_NAME} PROPERTIES CXX_STANDARD 20)
set_target_properties(${PROJECT_NAME}_lib PROPERTIES CXX_STANDARD 20)

# being a cross-platform target, we enforce standards conformance on MSVC
target_compile_options(${PROJECT_NAME} PUBLIC "$<$<COMPILE_LANG_AND_ID:CXX,MSVC>:/permissive->")
target_compile_options(${PROJECT_NAME}_lib PUBLIC "$<$<COMPILE_LANG_AND_ID:CXX,MSVC>:/permissive->")

# Link dependencies
target_link_libraries(${PROJECT_NAME} PRIVATE
    drogon
    fmt::fmt
    spdlog::spdlog
    CLI11::CLI11)

target_link_libraries(${PROJECT_NAME}_lib PRIVATE
    drogon
    fmt::fmt
    spdlog::spdlog
    CLI11::CLI11)

target_include_directories(
  ${PROJECT_NAME} PUBLIC $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include>
                         $<INSTALL_INTERFACE:include/${PROJECT_NAME}-${PROJECT_VERSION}>
)

target_include_directories(
  ${PROJECT_NAME}_lib PUBLIC $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include>
                         $<INSTALL_INTERFACE:include/${PROJECT_NAME}_lib-${PROJECT_VERSION}>
)

# ---- Create an installable target ----
# this allows users to install and find the library via `find_package()`.

# the location where the project's version header will be placed should match the project's regular
# header paths
# string(TOLOWER ${PROJECT_NAME}_lib/version.h VERSION_HEADER_LOCATION)

# packageProject(
#   NAME ${PROJECT_NAME}_lib
#   VERSION ${PROJECT_VERSION}
#   NAMESPACE ${PROJECT_NAME}_lib
#   BINARY_DIR ${PROJECT_BINARY_DIR}
#   INCLUDE_DIR ${PROJECT_SOURCE_DIR}/include
#   INCLUDE_DESTINATION include/${PROJECT_NAME}_lib-${PROJECT_VERSION}
#   VERSION_HEADER "${VERSION_HEADER_LOCATION}"
#   COMPATIBILITY SameMajorVersion
#   DEPENDENCIES "fmt 11.0.2;spdlog 1.14.1;CLI11 2.4.2;drogon 1.9.5"
# )
