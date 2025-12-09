cmake_minimum_required(VERSION 3.21)
project(QoS LANGUAGES CXX C)
find_program(CARGO_EXECUTABLE cargo REQUIRED)

if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(RUST_TARGET "aarch64-linux-android")
elseif(ANDROID_ABI STREQUAL "armeabi-v7a")
    set(RUST_TARGET "armv7-linux-androideabi")
elseif(ANDROID_ABI STREQUAL "x86_64")
    set(RUST_TARGET "x86_64-linux-android")
elseif(ANDROID_ABI STREQUAL "x86")
    set(RUST_TARGET "i686-linux-android")
else()
    message(FATAL_ERROR "Unsupported Android ABI: ${ANDROID_ABI}")
endif()

message(STATUS "NDK ABI: ${ANDROID_ABI}")
message(STATUS "Rust Target: ${RUST_TARGET}")
message(STATUS "Build Type: ${CMAKE_BUILD_TYPE}")

set(RUST_CRATE_PATH ${CMAKE_SOURCE_DIR}/src)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_BUILD_TYPE_DIR "debug")
    set(CARGO_FLAGS "")
    set(RUST_LIB_DIR ${RUST_CRATE_PATH}/target/${RUST_TARGET}/debug)
else()
    set(RUST_BUILD_TYPE_DIR "release")
    set(CARGO_FLAGS "--release")
    set(RUST_LIB_DIR ${RUST_CRATE_PATH}/target/${RUST_TARGET}/release)
endif()

set(RUST_LIB_NAME "qos_logic")
set(RUST_STATIC_LIB ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.a)

add_custom_command(
    OUTPUT ${RUST_STATIC_LIB}
    COMMAND ${CARGO_EXECUTABLE} build ${CARGO_FLAGS} --target ${RUST_TARGET}
    WORKING_DIRECTORY ${RUST_CRATE_PATH}
    DEPENDS ${RUST_CRATE_PATH}/Cargo.toml
            ${RUST_CRATE_PATH}/src/lib.rs
            ${RUST_CRATE_PATH}/src/c_api.rs
            ${RUST_CRATE_PATH}/src/ffi.rs
            ${RUST_CRATE_PATH}/src/system_utils.rs
            ${RUST_CRATE_PATH}/src/tweaker_logic.rs
            ${RUST_CRATE_PATH}/src/memory_logic.rs
            ${RUST_CRATE_PATH}/src/refresh_logic.rs
            ${RUST_CRATE_PATH}/src/storage_logic.rs
            ${RUST_CRATE_PATH}/src/vm_logic.rs
            ${RUST_CRATE_PATH}/src/logger.rs
    COMMENT "Building Rust static library (${CMAKE_BUILD_TYPE}) for ${RUST_TARGET}..."
)

add_custom_target(RustLib
    DEPENDS ${RUST_STATIC_LIB}
)

set(JNI_DIR ${CMAKE_SOURCE_DIR}/src/jni)
set(JNI_INCLUDE_DIR ${JNI_DIR}/include)
set(JNI_SRC_DIR ${JNI_DIR}/src)

file(GLOB CPP_HEADERS
    "${JNI_INCLUDE_DIR}/*.h"
)

file(GLOB CPP_SOURCES
    "${JNI_SRC_DIR}/*.cpp"
)

add_library(qos_daemon_cpp
    STATIC
    ${CPP_SOURCES}
    ${CPP_HEADERS}
)

set_target_properties(qos_daemon_cpp PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_definitions(qos_daemon_cpp PUBLIC DEBUG)
endif()

target_include_directories(qos_daemon_cpp
    PUBLIC
    ${JNI_INCLUDE_DIR}
)

add_executable(qos_daemon
    ${JNI_SRC_DIR}/main.cpp
)

add_dependencies(qos_daemon RustLib)

target_link_libraries(qos_daemon
    qos_daemon_cpp
    ${RUST_STATIC_LIB}
    android
    log
    c
    m
    dl
)

target_include_directories(qos_daemon
    PRIVATE
    ${JNI_INCLUDE_DIR}
)

install(TARGETS qos_daemon
    DESTINATION bin
)