#!/usr/bin/env python3
import os
import sys
import subprocess
import platform
import argparse
import shutil
import time
from pathlib import Path

DEFAULT_ANDROID_API = "33"
DEFAULT_BUILD_TYPE = "Release"
PROJECT_NAME = "QoS"

ABI_MAP = {
    "arm64-v8a": "aarch64-linux-android",
    "armeabi-v7a": "armv7-linux-androideabi",
    "x86_64": "x86_64-linux-android",
    "x86": "i686-linux-android"
}

class Style:
    HEADER = '\033[95m'
    SUCCESS = '\033[92m'
    INPUT = '\033[96m'
    WARNING = '\033[93m'
    ERROR = '\033[91m'
    DIM = '\033[90m'
    BOLD = '\033[1m'
    RESET = '\033[0m'

def log_header(msg):
    print(f"\n{Style.BOLD}{Style.HEADER}=== {msg} ==={Style.RESET}")

def log_section(msg):
    print(f"\n{Style.BOLD}{Style.DIM}=== {msg} ==={Style.RESET}")

def log_step(msg):
    print(f"{Style.WARNING}>>{Style.RESET} {msg}...")

def log_kv(key, value):
    print(f"   {Style.DIM}{key:<15}{Style.RESET} : {Style.INPUT}{value}{Style.RESET}")

def log_info(msg):
    print(f"   {Style.DIM}{msg}{Style.RESET}")

def log_success(msg):
    print(f"   {Style.SUCCESS}[OK]{Style.RESET} {msg}")

def log_error(msg):
    print(f"\n{Style.ERROR}[ERROR] {msg}{Style.RESET}")
    sys.exit(1)

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
        log_error(f"Tool '{tool_name}' missing. Please install it.")


def run_command(cmd, cwd=None, shell=False):
    try:
        subprocess.run(cmd, check=True, cwd=cwd, shell=shell)
    except subprocess.CalledProcessError:
        log_error(f"Command failed: {' '.join(cmd)}")


def clean_specific_build(build_dir):
    if build_dir.exists():
        try:
            shutil.rmtree(build_dir)
        except Exception as e:
            log_error(f"Failed to clean directory {build_dir}: {e}")


def clean_rust_target(rust_target, build_type):
    crate_path = Path("src")
    if crate_path.exists():
        cmd = ["cargo", "clean", "--target", rust_target]
        if build_type == "Release":
            cmd.append("--release")
            
        run_command(cmd, cwd=crate_path)


def clean_all_builds():
    log_header("Cleaning Workspace")
    build_path = Path("build")
    if build_path.exists():
        try:
            shutil.rmtree(build_path)
            log_success("Folder 'build/' removed.")
        except Exception as e:
            log_error(f"Failed to remove build folder: {e}")
            
    target_path = Path("target")
    if target_path.exists():
        try:
            shutil.rmtree(target_path)
            log_success("Folder 'target/' (Rust) removed.")
        except Exception as e:
            log_error(f"Failed to remove target folder: {e}")

    if not build_path.exists() and not target_path.exists():
        log_info("Workspace cleaned.")


def get_abi_selection():
    log_section("Select Architecture")
    abis = list(ABI_MAP.keys())
    
    for i, abi in enumerate(abis):
        print(f"   {Style.SUCCESS}[{i+1}]{Style.RESET} {abi}")
    print(f"   {Style.SUCCESS}[a]{Style.RESET} Build All")
    print(f"   {Style.ERROR}[q]{Style.RESET} Quit")

    choice = input(f"\n   {Style.DIM}>{Style.RESET} Enter choice: ").strip().lower()

    if choice == 'q':
        sys.exit(0)
    if choice == 'a':
        return "all"
    
    try:
        idx = int(choice) - 1
        if 0 <= idx < len(abis):
            return abis[idx]
        else:
            log_error("Invalid selection.")
    except ValueError:
        log_error("Invalid input.")


def get_api_selection():
    log_section("Select Android API Level")
    print(f"   {Style.SUCCESS}[Enter]{Style.RESET} Default ({DEFAULT_ANDROID_API})")
    
    choice = input(f"\n   {Style.DIM}>{Style.RESET} Enter API level: ").strip()
    
    if not choice:
        return DEFAULT_ANDROID_API
    
    if choice.isdigit() and int(choice) >= 21:
        return choice
    else:
        print(f"{Style.WARNING}   Invalid API. Using default: {DEFAULT_ANDROID_API}{Style.RESET}")
        return DEFAULT_ANDROID_API
    

def get_build_type_selection():
    log_section("Select Build Type")
    print(f"   {Style.SUCCESS}[1]{Style.RESET} Release (Default)")
    print(f"   {Style.SUCCESS}[2]{Style.RESET} Debug")
    
    choice = input(f"\n   {Style.DIM}>{Style.RESET} Enter choice: ").strip()
    return "Debug" if choice == '2' else "Release"


def build_architecture(abi, ndk_path, api_level, build_type):
    rust_target = ABI_MAP[abi]
    toolchain_file = ndk_path / "build/cmake/android.toolchain.cmake"
    build_dir = Path("build") / abi
    
    log_header(f"Building: {abi}")
    log_kv("Build Type", build_type)
    log_kv("Rust Target", rust_target)
    
    log_step("Cleaning previous artifacts (CMake)")
    clean_specific_build(build_dir)
    
    log_step("Checking Rust environment")
    try:
        subprocess.run(
            ["rustup", "target", "add", rust_target], 
            check=True, 
            stdout=subprocess.DEVNULL, 
            stderr=subprocess.DEVNULL
        )
    except subprocess.CalledProcessError:
        log_error(f"Failed to add rust target: {rust_target}")

    build_dir.mkdir(parents=True, exist_ok=True)

    log_step("Configuring CMake")
    cmake_cmd = [
        "cmake",
        "-Wno-dev",
        f"-DANDROID_ABI={abi}",
        f"-DANDROID_PLATFORM=android-{api_level}",
        f"-DCMAKE_TOOLCHAIN_FILE={toolchain_file}",
        f"-DCMAKE_BUILD_TYPE={build_type}",
        "-G", "Ninja",
        "../.." 
    ]
    run_command(cmake_cmd, cwd=build_dir)

    log_step("Compiling Native Code")
    run_command(["ninja"], cwd=build_dir)

    binary_path = build_dir / "qos_daemon"
    
    if binary_path.exists():
        log_success(f"Artifact created: {Style.BOLD}{binary_path.name}{Style.RESET}")
    else:
        log_error(f"Binary not found at: {binary_path}")


def main():
    parser = argparse.ArgumentParser(description=f"Builder for {PROJECT_NAME}")
    parser.add_argument("--abi", choices=["all"] + list(ABI_MAP.keys()))
    parser.add_argument("--api")
    parser.add_argument("--type", choices=["Release", "Debug"])
    parser.add_argument("--clean", action="store_true")
    
    args = parser.parse_args()

    if not any([args.abi, args.clean]):
        clear_screen()

    print(f"{Style.BOLD}{Style.HEADER}Builder{Style.RESET}")
    
    check_tool("cmake")
    check_tool("ninja")
    check_tool("rustup")
    check_tool("cargo")

    if args.clean:
        clean_all_builds()
        if not any([args.abi, args.api, args.type]):
            return

    ndk_path = find_ndk()
    if not ndk_path:
        log_error("Android NDK not found! Set ANDROID_NDK_HOME.")
    
    log_kv("NDK", ndk_path.name)

    toolchain = ndk_path / "build/cmake/android.toolchain.cmake"
    if not toolchain.exists():
        log_error(f"Invalid toolchain: {toolchain}")

    selected_abi = args.abi
    selected_api = args.api
    selected_type = args.type

    if not selected_abi:
        selected_abi = get_abi_selection()
        clear_screen()

    if not selected_api:
        selected_api = get_api_selection()
        clear_screen()

    if not selected_type:
        selected_type = get_build_type_selection()
        clear_screen()

    start_time = time.time()
    abis_to_build = []

    if selected_abi == "all":
        abis_to_build = list(ABI_MAP.keys())
    else:
        abis_to_build = [selected_abi]

    for abi in abis_to_build:
        try:
            build_architecture(abi, ndk_path, selected_api, selected_type)
        except KeyboardInterrupt:
            print(f"\n{Style.ERROR}Build cancelled.{Style.RESET}")
            sys.exit(0)
        except Exception as e:
            log_error(f"Unexpected error: {e}")

    elapsed = time.time() - start_time
    print(f"\n{Style.SUCCESS}{Style.BOLD}Build Completed in {elapsed:.2f}s{Style.RESET}\n")

if __name__ == "__main__":
    main()