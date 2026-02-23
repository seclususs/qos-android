#!/usr/bin/env python3
import argparse
import os
import platform
import shutil
import subprocess
import sys
import time
from pathlib import Path

DEFAULT_ANDROID_API = "33"
DEFAULT_BUILD_TYPE = "Release"
PROJECT_NAME = "QoS"
ARCH_ABI = "arm64-v8a"
RUST_TARGET = "aarch64-linux-android"


class Style:
    GREEN = "\033[92m"
    CYAN = "\033[96m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    BOLD = "\033[1m"
    RESET = "\033[0m"


def log_info(msg):
    print(f"{Style.BOLD}[+]{Style.RESET} {msg}")


def log_sub(msg):
    print(f" {Style.CYAN}->{Style.RESET} {msg}")


def log_ok(msg):
    print(f" {Style.GREEN}[OK]{Style.RESET} {msg}")


def log_warn(msg):
    print(f"{Style.YELLOW}[WARN]{Style.RESET} {msg}")


def log_err(msg):
    print(f"\n{Style.RED}[ERROR]{Style.RESET} {msg}")
    sys.exit(1)


def find_ndk():
    ndk_env = os.environ.get("ANDROID_NDK_HOME")

    if ndk_env and os.path.exists(ndk_env):
        return Path(ndk_env)
    
    home = Path.home()

    if platform.system() == "Windows":
        search_paths = [
            Path(os.environ.get("LOCALAPPDATA", "")) / "Android/Sdk/ndk",
            home / "AppData/Local/Android/Sdk/ndk",
        ]

    elif platform.system() == "Darwin":
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
        log_err(f"Missing tool: '{tool_name}'. Please install it.")


def run_cmd(cmd, cwd=None, silent=False):
    try:
        if silent:

            process = subprocess.run(
                cmd,
                cwd=cwd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
                errors="replace",
            )

            if process.returncode != 0:
                print(process.stdout)
                raise subprocess.CalledProcessError(process.returncode, cmd)
            
        else:
            subprocess.run(cmd, check=True, cwd=cwd)

    except subprocess.CalledProcessError as exc:
        log_err(f"Command failed: {' '.join(cmd)}")
        raise exc


def clean_build(build_type=None):
    if build_type:
        targets = [Path("build") / build_type / ARCH_ABI]

    else:
        targets = [Path("build"), Path("target")]

    for path in targets:
        if path.exists():

            try:
                shutil.rmtree(path)
                log_sub(f"Removed: {path}")

            except OSError as exc:
                log_warn(f"Failed to clean {path}: {exc}")


def run_quality_checks(ndk_path, api_level, do_check, do_lint):
    if not do_check and not do_lint:
        return
    
    log_info("Running code analysis...")

    rust_path = Path("core")

    if rust_path.exists():

        if do_check:
            run_cmd(
                ["cargo", "check", "--target", RUST_TARGET, "--release"],
                cwd=rust_path,
                silent=True,
            )
            log_ok("Rust syntax")

        if do_lint:
            run_cmd(
                ["cargo", "clippy", "--target", RUST_TARGET, "--release"],
                cwd=rust_path,
                silent=True,
            )
            log_ok("Rust lint")

    build_dir = Path("build") / "Release" / ARCH_ABI
    build_dir.mkdir(parents=True, exist_ok=True)

    toolchain = ndk_path / "build/cmake/android.toolchain.cmake"

    cmake_cmd = [
        "cmake",
        "-Wno-dev",
        f"-DANDROID_ABI={ARCH_ABI}",
        f"-DANDROID_PLATFORM=android-{api_level}",
        f"-DCMAKE_TOOLCHAIN_FILE={toolchain}",
        "-DCMAKE_BUILD_TYPE=Release",
        "-G",
        "Ninja",
        "../../..",
    ]

    try:
        run_cmd(cmake_cmd, cwd=build_dir, silent=True)

    except subprocess.CalledProcessError:
        log_err("Failed to configure CMake for analysis.")

    if do_check:
        run_cmd(["ninja", "syntax"], cwd=build_dir, silent=True)
        log_ok("C++ syntax")

    if do_lint:
        run_cmd(["ninja", "lint"], cwd=build_dir, silent=True)
        log_ok("C++ lint")


def build_project(ndk_path, api_level, build_type):
    log_info(f"Building [{build_type}] for {ARCH_ABI} (API {api_level})")

    build_dir = Path("build") / build_type / ARCH_ABI

    if build_dir.exists():
        shutil.rmtree(build_dir)

    build_dir.mkdir(parents=True, exist_ok=True)

    try:
        run_cmd(["rustup", "target", "add", RUST_TARGET], silent=True)

    except subprocess.CalledProcessError:
        log_err(f"Failed to add Rust target: {RUST_TARGET}")

    log_sub("Configuring project...")

    toolchain = ndk_path / "build/cmake/android.toolchain.cmake"

    cmake_cmd = [
        "cmake",
        "-Wno-dev",
        "-Wno-deprecated",
        f"-DANDROID_ABI={ARCH_ABI}",
        f"-DANDROID_PLATFORM=android-{api_level}",
        f"-DCMAKE_TOOLCHAIN_FILE={toolchain}",
        f"-DCMAKE_BUILD_TYPE={build_type}",
        "-G",
        "Ninja",
        "../../..",
    ]

    run_cmd(cmake_cmd, cwd=build_dir, silent=True)

    log_sub("Compiling...")

    run_cmd(["ninja"], cwd=build_dir)
    binary = build_dir / "qos_daemon"

    if binary.exists():
        log_ok(f"Artifact: {Style.BOLD}{binary}{Style.RESET}")

    else:
        log_err("Build failed: binary not found.")


def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )

    parser.add_argument("--api", default=DEFAULT_ANDROID_API, help="Android API level")
    parser.add_argument(
        "--type",
        choices=["Release", "Debug", "All"],
        default=DEFAULT_BUILD_TYPE,
        help="Build type",
    )
    parser.add_argument("--clean", action="store_true", help="Clean workspace")
    parser.add_argument("--check", action="store_true", help="Run syntax checks")
    parser.add_argument("--lint", action="store_true", help="Run linter")

    args = parser.parse_args()

    for tool in ["cmake", "ninja", "rustup", "cargo"]:
        check_tool(tool)

    ndk_path = find_ndk()

    if not ndk_path:
        log_err("Android NDK not found. Set ANDROID_NDK_HOME.")

    if args.clean:
        log_info("Cleaning workspace...")
        clean_build()

        if not (args.check or args.lint):
            return
        
    do_check = args.check or not (args.check or args.lint)
    do_lint = args.lint or not (args.check or args.lint)

    run_quality_checks(ndk_path, args.api, do_check, do_lint)

    explicit_analysis = args.check or args.lint

    explicit_build_args = any(x in sys.argv for x in ["--type", "--api"])

    if explicit_analysis and not explicit_build_args:
        return
    
    types = ["Release", "Debug"] if args.type == "All" else [args.type]

    start = time.time()

    try:
        for b_type in types:
            build_project(ndk_path, args.api, b_type)

        elapsed = time.time() - start
        print(f"\n{Style.GREEN}Done in {elapsed:.2f}s{Style.RESET}")

    except KeyboardInterrupt:
        print("\nCancelled.")
        sys.exit(0)

    except Exception as exc:
        log_err(f"Error: {exc}")


if __name__ == "__main__":
    main()