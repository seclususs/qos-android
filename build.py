#!/usr/bin/env python3
import os
import sys
import subprocess
import platform
import argparse
import shutil
import time
from pathlib import Path


ANDROID_API = "23"
BUILD_TYPE = "Release"
PROJECT_NAME = "AdaptiveDaemon"

ABI_MAP = {
    "arm64-v8a": "aarch64-linux-android",
    "armeabi-v7a": "armv7-linux-androideabi",
    "x86_64": "x86_64-linux-android",
    "x86": "i686-linux-android"
}

class Style:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

def log_header(msg):
    print(f"\n{Style.HEADER}{Style.BOLD}=== {msg} ==={Style.ENDC}")

def log_info(msg):
    print(f"{Style.BLUE}ℹ {msg}{Style.ENDC}")

def log_success(msg):
    print(f"{Style.GREEN}✔ {msg}{Style.ENDC}")

def log_error(msg):
    print(f"{Style.FAIL}✖ {msg}{Style.ENDC}")
    sys.exit(1)

def log_step(msg):
    print(f"{Style.CYAN}➜ {msg}{Style.ENDC}")

def clear_screen():
    os.system('cls' if os.name == 'nt' else 'clear')


def find_ndk():
    ndk_env = os.environ.get("ANDROID_NDK_HOME")
    if ndk_env and os.path.exists(ndk_env):
        return Path(ndk_env)
    
    home = Path.home()
    search_paths = []
    
    system = platform.system()
    if system == "Windows":
        search_paths = [
            Path(os.environ.get("LOCALAPPDATA", "")) / "Android/Sdk/ndk",
            home / "AppData/Local/Android/Sdk/ndk"
        ]
    elif system == "Darwin":
        search_paths = [home / "Library/Android/sdk/ndk"]
    else:
        search_paths = [home / "Android/Sdk/ndk"]
        
    for path in search_paths:
        if path.exists():
            versions = sorted([d for d in path.iterdir() if d.is_dir()], reverse=True)
            if versions:
                return versions[0]
    
    return None


def check_tool(tool_name):
    if not shutil.which(tool_name):
        log_error(f"Tool '{tool_name}' not found in PATH. Please install it first.")


def run_command(cmd, cwd=None, shell=False):
    try:
        subprocess.run(cmd, check=True, cwd=cwd, shell=shell)
    except subprocess.CalledProcessError:
        log_error(f"Failed to execute command: {' '.join(cmd)}")


def clean_specific_build(build_dir):
    if build_dir.exists():
        try:
            # log_info(f"Cleaning previous build artifacts in {build_dir.name}...")
            shutil.rmtree(build_dir)
        except Exception as e:
            log_error(f"Failed to clean directory {build_dir}: {e}")


def clean_all_builds():
    log_header("Cleaning All Builds")
    build_path = Path("build")
    if build_path.exists():
        try:
            shutil.rmtree(build_path)
            log_success("Folder 'build/' successfully removed.")
        except Exception as e:
            log_error(f"Failed to remove build folder: {e}")
    else:
        log_info("Folder 'build/' not found, nothing to clean.")


def get_user_selection():
    print(f"\n{Style.BOLD}Select Action:{Style.ENDC}")
    abis = list(ABI_MAP.keys())
    
    for i, abi in enumerate(abis):
        print(f"  {Style.GREEN}[{i+1}]{Style.ENDC} Build {abi}")
    
    print(f"  {Style.GREEN}[A]{Style.ENDC} Build All")
    print(f"  {Style.GREEN}[Q]{Style.ENDC} Quit")

    choice = input(f"\n{Style.CYAN}Enter choice: {Style.ENDC}").strip().lower()

    if choice == 'q':
        sys.exit(0)
    if choice == 'a':
        return "all"
    
    try:
        idx = int(choice) - 1
        if 0 <= idx < len(abis):
            return abis[idx]
        else:
            log_error("Invalid selection number.")
    except ValueError:
        log_error("Invalid input.")


def build_architecture(abi, ndk_path):
    rust_target = ABI_MAP[abi]
    toolchain_file = ndk_path / "build/cmake/android.toolchain.cmake"
    build_dir = Path("build") / abi
    
    log_header(f"Building for {abi}")
    log_info(f"Rust Target : {rust_target}")

    clean_specific_build(build_dir)

    log_step("Checking Rust target...")
    run_command(["rustup", "target", "add", rust_target])

    build_dir.mkdir(parents=True, exist_ok=True)
    log_info(f"Build Dir   : {build_dir}")

    log_step("Configuring CMake...")
    cmake_cmd = [
        "cmake",
        f"-DANDROID_ABI={abi}",
        f"-DANDROID_PLATFORM=android-{ANDROID_API}",
        f"-DCMAKE_TOOLCHAIN_FILE={toolchain_file}",
        f"-DCMAKE_BUILD_TYPE={BUILD_TYPE}",
        "-G", "Ninja",
        "../.." 
    ]
    run_command(cmake_cmd, cwd=build_dir)

    log_step("Compiling...")
    run_command(["ninja"], cwd=build_dir)

    binary_path = build_dir / "adaptive_daemon"
    
    if binary_path.exists():
        log_success(f"Build Successful! Binary located at:\n  {Style.BOLD}{binary_path.absolute()}{Style.ENDC}")
    else:
        log_error(f"Build finished, but binary not found at: {binary_path}")


def main():
    parser = argparse.ArgumentParser(description=f"Smart Builder for {PROJECT_NAME}")
    parser.add_argument(
        "--abi", 
        choices=["all"] + list(ABI_MAP.keys()), 
        help="Select build architecture"
    )
    parser.add_argument(
        "--clean", 
        action="store_true", 
        help="Clean ALL build folders"
    )
    
    args = parser.parse_args()

    print(f"\n{Style.BOLD}{Style.CYAN}=== Build ==={Style.ENDC}")
    print(f"{Style.CYAN}Platform: {platform.system()} | Python: {platform.python_version()}{Style.ENDC}")

    check_tool("cmake")
    check_tool("ninja")
    check_tool("rustup")
    check_tool("cargo")

    if args.clean:
        clean_all_builds()
        if not args.abi:
            return
        
    log_step("Detecting NDK...")
    ndk_path = find_ndk()
    if not ndk_path:
        log_error("Android NDK not found! Set ANDROID_NDK_HOME environment variable.")

    toolchain = ndk_path / "build/cmake/android.toolchain.cmake"
    if not toolchain.exists():
        log_error(f"Invalid toolchain file at: {toolchain}")

    log_success(f"NDK Found: {ndk_path}")

    selected_abi = args.abi
    if not selected_abi:
        selected_abi = get_user_selection()
        clear_screen()

    start_time = time.time()
    abis_to_build = []

    if selected_abi == "all":
        abis_to_build = list(ABI_MAP.keys())
    else:
        abis_to_build = [selected_abi]

    for abi in abis_to_build:
        try:
            build_architecture(abi, ndk_path)
        except KeyboardInterrupt:
            log_error("\nBuild cancelled by user.")
        except Exception as e:
            log_error(f"Unexpected error: {e}")

    elapsed = time.time() - start_time
    print(f"\n{Style.BOLD}{Style.GREEN}Completed in {elapsed:.2f}s{Style.ENDC}\n")

if __name__ == "__main__":
    main()