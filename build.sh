#!/bin/bash
set -e

DEFAULT_ABI="arm64-v8a"
ANDROID_API="23"
BUILD_TYPE="Release"

if [ -n "$ANDROID_NDK_HOME" ]; then
    NDK_PATH="$ANDROID_NDK_HOME"
    echo "Using ANDROID_NDK_HOME from environment: $NDK_PATH"
elif [ -d "$HOME/Android/Sdk/ndk" ]; then
    NDK_PATH=$(find "$HOME/Android/Sdk/ndk" -maxdepth 1 -type d | sort -V | tail -n 1)
    echo "Found NDK at: $NDK_PATH"
elif [ -d "$HOME/Library/Android/sdk/ndk" ]; then
    NDK_PATH=$(find "$HOME/Library/Android/sdk/ndk" -maxdepth 1 -type d | sort -V | tail -n 1)
    echo "Found NDK at: $NDK_PATH"
else
    echo "Error: ANDROID_NDK_HOME is not set and default NDK path was not found."
    echo "Please set ANDROID_NDK_HOME to your NDK directory."
    exit 1
fi

TOOLCHAIN_FILE="$NDK_PATH/build/cmake/android.toolchain.cmake"
if [ ! -f "$TOOLCHAIN_FILE" ]; then
    echo "Error: CMake toolchain file not found at $TOOLCHAIN_FILE"
    exit 1
fi

TARGET_ABI=$DEFAULT_ABI
if [ "$1" == "clean" ]; then
    echo "Cleaning build directory..."
    rm -rf build
    echo "Done."
    exit 0
elif [ -n "$1" ]; then
    TARGET_ABI=$1
fi

RUST_TARGET=""
if [ "$TARGET_ABI" == "arm64-v8a" ]; then
    RUST_TARGET="aarch64-linux-android"
elif [ "$TARGET_ABI" == "armeabi-v7a" ]; then
    RUST_TARGET="armv7-linux-androideabi"
elif [ "$TARGET_ABI" == "x86_64" ]; then
    RUST_TARGET="x86_64-linux-android"
elif [ "$TARGET_ABI" == "x86" ]; then
    RUST_TARGET="i686-linux-android"
else
    echo "Error: ABI '$TARGET_ABI' is not supported."
    exit 1
fi

echo "--- Ensuring Rust target '$RUST_TARGET' is installed ---"
rustup target add $RUST_TARGET

BUILD_DIR="build/$TARGET_ABI"
echo "--- Starting build for $TARGET_ABI in $BUILD_DIR ---"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"
cmake -DANDROID_ABI=$TARGET_ABI \
      -DANDROID_PLATFORM=android-$ANDROID_API \
      -DCMAKE_TOOLCHAIN_FILE="$TOOLCHAIN_FILE" \
      -DCMAKE_BUILD_TYPE=$BUILD_TYPE \
      -G "Ninja" \
      ../..
ninja -j$(nproc)
cd ../..
echo ""
echo "--- Build Complete! ---"
echo "Your binary is at: $BUILD_DIR/adaptive_daemon"