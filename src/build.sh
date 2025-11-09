#!/bin/bash

if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "Error: The ANDROID_NDK_HOME environment variable is not set."
    echo "Please set the path to your Android NDK root directory."
    echo "Example: export ANDROID_NDK_HOME=/path/to/android-ndk"
    exit 1
fi

NDK_BUILD="$ANDROID_NDK_HOME/ndk-build"

if [ ! -x "$NDK_BUILD" ]; then
    echo "Error: ndk-build not found or not executable at $NDK_BUILD"
    exit 1
fi

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
OUTPUT_DIR="$PROJECT_DIR/build"
TARGET_ABIS=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")

echo "Cleaning previous builds..."
rm -rf "$OUTPUT_DIR"
rm -rf "$PROJECT_DIR/libs"
rm -rf "$PROJECT_DIR/obj"

for abi in "${TARGET_ABIS[@]}"; do
    echo ""
    echo "================================================="
    echo "      Starting compilation for architecture: $abi"
    echo "================================================="
    
    "$NDK_BUILD" \
    NDK_PROJECT_PATH="$PROJECT_DIR" \
    APP_BUILD_SCRIPT="$PROJECT_DIR/jni/Android.mk" \
    APP_ABI="$abi" \
    NDK_OUT="$OUTPUT_DIR"
    
    if [ $? -ne 0 ]; then
        echo "Error: Compilation for $abi failed."
        exit 1
    fi
done

echo ""
    echo "================================================="
    echo "              Compilation Complete               "
    echo "================================================="
    echo "Binary files can be found at: $OUTPUT_DIR/libs/"
    echo ""

ls -R "$OUTPUT_DIR/libs/"
