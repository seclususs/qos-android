cmake_minimum_required(VERSION 3.21)
project(QoS LANGUAGES CXX C)

include(CheckIPOSupported)
check_ipo_supported(RESULT result OUTPUT output)
if(result)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
    message(STATUS ">> LTO (Link Time Optimization): DIAKTIFKAN")
else()
    message(WARNING ">> LTO tidak didukung: ${output}")
endif()

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    message(STATUS ">> Mode Release...")
    add_compile_options(-Oz -ffunction-sections -fdata-sections)
    add_link_options(-Wl,--gc-sections -Wl,--icf=all -Wl,-s)
endif()

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

set(RUST_CRATE_PATH ${CMAKE_SOURCE_DIR}/core)
set(RUST_WORKSPACE_TARGET_DIR ${CMAKE_SOURCE_DIR}/target)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_BUILD_TYPE_DIR "debug")
    set(CARGO_FLAGS "")
else()
    set(RUST_BUILD_TYPE_DIR "release")
    set(CARGO_FLAGS "--release")
endif()

set(RUST_LIB_NAME "qos_logic")
set(RUST_LIB_DIR ${RUST_WORKSPACE_TARGET_DIR}/${RUST_TARGET}/${RUST_BUILD_TYPE_DIR})
set(RUST_STATIC_LIB ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.a)

add_custom_command(
    OUTPUT ${RUST_STATIC_LIB}
    COMMAND ${CARGO_EXECUTABLE} build ${CARGO_FLAGS} --target ${RUST_TARGET}
    WORKING_DIRECTORY ${RUST_CRATE_PATH}
    DEPENDS ${RUST_CRATE_PATH}/Cargo.toml
            ${RUST_CRATE_PATH}/src/lib.rs
            ${RUST_CRATE_PATH}/src/bindings/mod.rs
            ${RUST_CRATE_PATH}/src/bindings/c_api.rs
            ${RUST_CRATE_PATH}/src/bindings/ffi.rs
            ${RUST_CRATE_PATH}/src/common/mod.rs
            ${RUST_CRATE_PATH}/src/common/error.rs
            ${RUST_CRATE_PATH}/src/common/fs.rs
            ${RUST_CRATE_PATH}/src/common/logger.rs
            ${RUST_CRATE_PATH}/src/common/psi.rs
            ${RUST_CRATE_PATH}/src/common/state.rs
            ${RUST_CRATE_PATH}/src/common/traits.rs
            ${RUST_CRATE_PATH}/src/controllers/mod.rs
            ${RUST_CRATE_PATH}/src/controllers/memory_logic.rs
            ${RUST_CRATE_PATH}/src/controllers/signal_logic.rs
            ${RUST_CRATE_PATH}/src/controllers/storage_logic.rs
            ${RUST_CRATE_PATH}/src/controllers/tweaker_logic.rs
            ${RUST_CRATE_PATH}/src/controllers/display_logic.rs
            ${RUST_CRATE_PATH}/src/controllers/cpu_logic.rs
    COMMENT "Building Rust static library (${CMAKE_BUILD_TYPE}) for ${RUST_TARGET}..."
)

add_custom_target(RustLib DEPENDS ${RUST_STATIC_LIB})

set(JNI_DIR ${CMAKE_SOURCE_DIR}/core/jni)
set(JNI_INCLUDE_DIR ${JNI_DIR}/include)
set(JNI_SRC_DIR ${JNI_DIR}/src)

file(GLOB CPP_HEADERS "${JNI_INCLUDE_DIR}/*.h")
file(GLOB ALL_CPP_SOURCES "${JNI_SRC_DIR}/*.cpp")

set(LIB_CPP_SOURCES ${ALL_CPP_SOURCES})
list(REMOVE_ITEM LIB_CPP_SOURCES "${JNI_SRC_DIR}/main.cpp")

add_library(qos_daemon_cpp
    STATIC
    ${LIB_CPP_SOURCES}
    ${CPP_HEADERS}
)

set_target_properties(qos_daemon_cpp PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
)

target_include_directories(qos_daemon_cpp PUBLIC ${JNI_INCLUDE_DIR})

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_definitions(qos_daemon_cpp PUBLIC DEBUG)
endif()

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

install(TARGETS qos_daemon DESTINATION bin)