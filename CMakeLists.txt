cmake_minimum_required(VERSION 3.21)

project(QoS
    VERSION 1.0
    LANGUAGES C CXX
)

# ==============================================================================
# Global Configuration
# ==============================================================================

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Default build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release CACHE STRING "" FORCE)
endif()

# Detect IPO / LTO support once
include(CheckIPOSupported)
check_ipo_supported(RESULT LTO_SUPPORTED OUTPUT LTO_ERROR)

# Root paths
set(RUST_ROOT_DIR   "${CMAKE_SOURCE_DIR}/core")
set(NATIVE_ROOT_DIR "${CMAKE_SOURCE_DIR}/native")
set(RUST_TARGET_TRIPLE "aarch64-linux-android")

# ==============================================================================
# Rust Build
# ==============================================================================

find_program(CARGO_EXECUTABLE cargo REQUIRED)

# Determine Rust build mode
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_PROFILE "debug")
    set(CARGO_ARGS "")
else()
    set(RUST_PROFILE "release")
    set(CARGO_ARGS "--release")
endif()

message(STATUS ">> Rust profile: ${RUST_PROFILE}")

# Track all Rust sources
file(GLOB_RECURSE RUST_SOURCES CONFIGURE_DEPENDS
    "${RUST_ROOT_DIR}/src/*.rs"
    "${RUST_ROOT_DIR}/Cargo.toml"
)

# Rust library output
set(RUST_LIB_NAME "qos_logic")
set(RUST_STATIC_LIB
    "${CMAKE_SOURCE_DIR}/target/${RUST_TARGET_TRIPLE}/${RUST_PROFILE}/lib${RUST_LIB_NAME}.a"
)

# Cargo build step
add_custom_command(
    OUTPUT  "${RUST_STATIC_LIB}"
    COMMAND "${CARGO_EXECUTABLE}" build ${CARGO_ARGS} --target ${RUST_TARGET_TRIPLE}
    WORKING_DIRECTORY "${RUST_ROOT_DIR}"
    DEPENDS ${RUST_SOURCES}
    COMMENT "Building Rust crate '${RUST_LIB_NAME}' (${RUST_PROFILE})"
    VERBATIM
)

add_custom_target(rust_build ALL DEPENDS "${RUST_STATIC_LIB}")

# Imported Rust library
add_library(rust_static_lib STATIC IMPORTED GLOBAL)
set_target_properties(rust_static_lib PROPERTIES
    IMPORTED_LOCATION "${RUST_STATIC_LIB}"
)
add_dependencies(rust_static_lib rust_build)

# ==============================================================================
# C++ Library
# ==============================================================================

file(GLOB_RECURSE NATIVE_SOURCES CONFIGURE_DEPENDS
    "${NATIVE_ROOT_DIR}/src/*.cpp"
)
file(GLOB_RECURSE NATIVE_HEADERS CONFIGURE_DEPENDS
    "${NATIVE_ROOT_DIR}/include/*.h"
)

list(REMOVE_ITEM NATIVE_SOURCES
    "${NATIVE_ROOT_DIR}/src/main.cpp"
)

add_library(qos_native STATIC
    ${NATIVE_SOURCES}
    ${NATIVE_HEADERS}
)

target_include_directories(qos_native
    PUBLIC "${NATIVE_ROOT_DIR}/include"
)

# Release-only optimizations
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    target_compile_options(qos_native PRIVATE
        -O3
        -ffunction-sections
        -fdata-sections
        $<$<COMPILE_LANGUAGE:CXX>:-fno-exceptions>
        $<$<COMPILE_LANGUAGE:CXX>:-fno-rtti>
        -fno-unwind-tables
        -fno-asynchronous-unwind-tables
        -fvisibility=hidden
        -fvisibility-inlines-hidden
    )

    if(LTO_SUPPORTED)
        set_property(TARGET qos_native PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
        message(STATUS ">> LTO enabled for qos_native")
    endif()
else()
    target_compile_definitions(qos_native PRIVATE DEBUG)
endif()

# ==============================================================================
# Daemon Executable
# ==============================================================================

add_executable(qos_daemon
    "${NATIVE_ROOT_DIR}/src/main.cpp"
)

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    target_link_options(qos_daemon PRIVATE
        -Wl,--gc-sections
        -Wl,--icf=all
        -Wl,-s
        -Wl,--build-id=none
    )

    if(LTO_SUPPORTED)
        set_property(TARGET qos_daemon PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
    endif()
endif()

target_link_libraries(qos_daemon
    PRIVATE
    qos_native
    rust_static_lib
    android
    log
    c
    m
    dl
)

install(TARGETS qos_daemon DESTINATION bin)