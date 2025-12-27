cmake_minimum_required(VERSION 3.21)
project(QoS LANGUAGES CXX C)

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    include(CheckIPOSupported)
    check_ipo_supported(RESULT result OUTPUT output)
    if(result)
        set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
        message(STATUS ">> LTO (Link Time Optimization): ENABLED")
    else()
        message(WARNING ">> LTO not supported: ${output}")
    endif()

    message(STATUS ">> Release Mode: Enabling optimizations...")
    add_compile_options(-Oz -ffunction-sections -fdata-sections)
    add_link_options(-Wl,--gc-sections -Wl,--icf=all -Wl,-s)
else()
    message(STATUS ">> Debug Mode: LTO Disabled")
endif()

find_program(CARGO_EXECUTABLE cargo REQUIRED)

set(RUST_TARGET "aarch64-linux-android")
set(ANDROID_ABI "arm64-v8a")

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
            ${RUST_CRATE_PATH}/src/algorithms/mod.rs
            ${RUST_CRATE_PATH}/src/algorithms/cpu_math.rs
            ${RUST_CRATE_PATH}/src/algorithms/memory_math.rs
            ${RUST_CRATE_PATH}/src/algorithms/poll_math.rs
            ${RUST_CRATE_PATH}/src/algorithms/storage_math.rs
            ${RUST_CRATE_PATH}/src/algorithms/thermal_math.rs
            ${RUST_CRATE_PATH}/src/bindings/mod.rs
            ${RUST_CRATE_PATH}/src/bindings/api_entry.rs
            ${RUST_CRATE_PATH}/src/bindings/sys.rs
            ${RUST_CRATE_PATH}/src/config/mod.rs
            ${RUST_CRATE_PATH}/src/config/loop_settings.rs
            ${RUST_CRATE_PATH}/src/config/tunables.rs
            ${RUST_CRATE_PATH}/src/controllers/mod.rs
            ${RUST_CRATE_PATH}/src/controllers/cpu_impl.rs
            ${RUST_CRATE_PATH}/src/controllers/memory_impl.rs
            ${RUST_CRATE_PATH}/src/controllers/signal_impl.rs
            ${RUST_CRATE_PATH}/src/controllers/storage_impl.rs
            ${RUST_CRATE_PATH}/src/core/mod.rs
            ${RUST_CRATE_PATH}/src/core/interfaces.rs
            ${RUST_CRATE_PATH}/src/core/lifecycle.rs
            ${RUST_CRATE_PATH}/src/core/logging.rs
            ${RUST_CRATE_PATH}/src/core/state.rs
            ${RUST_CRATE_PATH}/src/core/types.rs
            ${RUST_CRATE_PATH}/src/hal/mod.rs
            ${RUST_CRATE_PATH}/src/hal/bridge.rs
            ${RUST_CRATE_PATH}/src/hal/cached_file.rs
            ${RUST_CRATE_PATH}/src/hal/filesystem.rs
            ${RUST_CRATE_PATH}/src/hal/kernel.rs
            ${RUST_CRATE_PATH}/src/hal/properties.rs
            ${RUST_CRATE_PATH}/src/monitors/mod.rs
            ${RUST_CRATE_PATH}/src/monitors/psi_monitor.rs
            ${RUST_CRATE_PATH}/src/registry/mod.rs
            ${RUST_CRATE_PATH}/src/registry/boot_tweaks.rs
            ${RUST_CRATE_PATH}/src/resources/mod.rs
            ${RUST_CRATE_PATH}/src/resources/sys_paths.rs
    COMMENT "Building Rust static library (${CMAKE_BUILD_TYPE}) for ${RUST_TARGET}..."
)

add_custom_target(RustLib DEPENDS ${RUST_STATIC_LIB})

set(NATIVE_DIR ${CMAKE_SOURCE_DIR}/native)
set(NATIVE_INCLUDE_DIR ${NATIVE_DIR}/include)
set(NATIVE_SRC_DIR ${NATIVE_DIR}/src)

file(GLOB_RECURSE ALL_CPP_SOURCES CONFIGURE_DEPENDS "${NATIVE_SRC_DIR}/*.cpp")
file(GLOB_RECURSE CPP_HEADERS CONFIGURE_DEPENDS "${NATIVE_INCLUDE_DIR}/*.h")

set(LIB_CPP_SOURCES ${ALL_CPP_SOURCES})
list(REMOVE_ITEM LIB_CPP_SOURCES "${NATIVE_SRC_DIR}/main.cpp")

add_library(qos_native_lib
    STATIC
    ${LIB_CPP_SOURCES}
    ${CPP_HEADERS}
)

set_target_properties(qos_native_lib PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
)

target_include_directories(qos_native_lib PUBLIC ${NATIVE_INCLUDE_DIR})

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_definitions(qos_native_lib PUBLIC DEBUG)
endif()

add_executable(qos_daemon
    ${NATIVE_SRC_DIR}/main.cpp
)

add_dependencies(qos_daemon RustLib)

target_link_libraries(qos_daemon
    qos_native_lib
    ${RUST_STATIC_LIB}
    android
    log
    c
    m
    dl
)

install(TARGETS qos_daemon DESTINATION bin)