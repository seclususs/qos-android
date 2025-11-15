@echo off
setlocal

SET "DEFAULT_ABI=arm64-v8a"
SET "ANDROID_API=23"
SET "BUILD_TYPE=Release"

SET "NDK_PATH=%ANDROID_NDK_HOME%"
IF NOT DEFINED NDK_PATH (
    SET "DEFAULT_NDK_SEARCH_PATH=%LOCALAPPDATA%\Android\Sdk\ndk"
    IF EXIST "%DEFAULT_NDK_SEARCH_PATH%" (
        echo ANDROID_NDK_HOME not set, searching default path...
        FOR /f "delims=" %%i IN ('dir /b /ad /o-n "%DEFAULT_NDK_SEARCH_PATH%"') DO (
            SET "NDK_PATH=%DEFAULT_NDK_SEARCH_PATH%\%%i"
        )
    )
)
IF NOT DEFINED NDK_PATH (
    echo Error: ANDROID_NDK_HOME is not set and default NDK path was not found.
    echo Please set ANDROID_NDK_HOME to your NDK directory.
    echo Example: set ANDROID_NDK_HOME=C:\Users\HP\AppData\Local\Android\Sdk\ndk\android-ndk-r28c
    exit /b 1
)
echo Using NDK at: %NDK_PATH%
SET "TOOLCHAIN_FILE=%NDK_PATH%\build\cmake\android.toolchain.cmake"
IF NOT EXIST "%TOOLCHAIN_FILE%" (
    echo Error: CMake toolchain file not found at %TOOLCHAIN_FILE%
    exit /b 1
)

SET "TARGET_ABI=%DEFAULT_ABI%"
IF "%1"=="clean" (
    echo Cleaning build directory...
    IF EXIST build (
        rmdir /s /q build
    )
    echo Done.
    exit /b 0
)
IF NOT "%1"=="" (
    SET "TARGET_ABI=%1"
)

SET "RUST_TARGET="
IF "%TARGET_ABI%"=="arm64-v8a" SET "RUST_TARGET=aarch64-linux-android"
IF "%TARGET_ABI%"=="armeabi-v7a" SET "RUST_TARGET=armv7-linux-androideabi"
IF "%TARGET_ABI%"=="x86_64" SET "RUST_TARGET=x86_64-linux-android"
IF "%TARGET_ABI%"=="x86" SET "RUST_TARGET=i686-linux-android"
IF NOT DEFINED RUST_TARGET (
    echo Error: ABI '%TARGET_ABI%' is not supported.
    exit /b 1
)

echo --- Ensuring Rust target '%RUST_TARGET%' is installed ---
rustup target add %RUST_TARGET%

SET "BUILD_DIR=build\%TARGET_ABI%"
echo --- Starting build for %TARGET_ABI% in %BUILD_DIR% ---
mkdir "%BUILD_DIR%"
cd "%BUILD_DIR%"
cmake -DANDROID_ABI=%TARGET_ABI% ^
      -DANDROID_PLATFORM=android-%ANDROID_API% ^
      -DCMAKE_TOOLCHAIN_FILE="%TOOLCHAIN_FILE%" ^
      -DCMAKE_BUILD_TYPE=%BUILD_TYPE% ^
      -G "Ninja" ^
      ..\..
ninja
cd ..\..
echo.
echo --- Build Complete! ---
echo Your binary is at: %BUILD_DIR%\adaptive_daemon