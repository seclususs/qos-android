@echo off

if not defined ANDROID_NDK_HOME (
    echo Error: The ANDROID_NDK_HOME environment variable is not set.
    echo Please set the path to your Android NDK root directory.
    echo Example: setx ANDROID_NDK_HOME "C:\path\to\android-ndk"
    exit /b 1
)

set "NDK_BUILD=%ANDROID_NDK_HOME%\ndk-build.cmd"

if not exist "%NDK_BUILD%" (
    echo Error: ndk-build.cmd not found at "%NDK_BUILD%"
    exit /b 1
)

set "PROJECT_DIR=%~dp0"
if "%PROJECT_DIR:~-1%"=="\" set "PROJECT_DIR=%PROJECT_DIR:~0,-1%"

set "OUTPUT_DIR=%PROJECT_DIR%\build"

set "TARGET_ABIS=arm64-v8a armeabi-v7a x86_64 x86"

echo Cleaning previous builds...
if exist "%OUTPUT_DIR%" rd /s /q "%OUTPUT_DIR%"
if exist "%PROJECT_DIR%\libs" rd /s /q "%PROJECT_DIR%\libs"
if exist "%PROJECT_DIR%\obj" rd /s /q "%PROJECT_DIR%\obj"

for %%a in (%TARGET_ABIS%) do (
    echo.
    echo =================================================
    echo      Starting compilation for architecture: %%a
    echo =================================================
    
    call "%NDK_BUILD%" ^
    NDK_PROJECT_PATH="%PROJECT_DIR%" ^
    APP_BUILD_SCRIPT="%PROJECT_DIR%\jni\Android.mk" ^
    APP_ABI=%%a ^
    NDK_OUT="%OUTPUT_DIR%"
    
    if errorlevel 1 (
        echo Error: Compilation for %%a failed.
        exit /b 1
    )
)

echo.
echo =================================================
echo               Compilation Complete!
echo =================================================
echo Binary files can be found at: %OUTPUT_DIR%\libs\
echo.

tree /F "%OUTPUT_DIR%\libs"
