# Build Guide

This project is built **exclusively for ARM64 (aarch64)**.

## Quick Start

Build interactively:

```bash
python3 build.py
```

## Command Line Usage

Use command-line options to skip interactive prompts.

```bash
python3 build.py [--api LEVEL] [--type Release|Debug|All] [--clean]
```

### Arguments

| Argument | Explanation |
|---------|-------------|
| `--api` | Specifies the Android API level to build against. Default is **33**. |
| `--type` | Selects build type: **Release**, **Debug**, or **All** |
| `--clean` | Removes the existing build directory before compiling. |

## Build Output

After a successful build, binaries will be located at:

| Build Type | Output Path |
|-----------|-------------|
| Release | `build/Release/arm64-v8a/qos_daemon` |
| Debug | `build/Debug/arm64-v8a/qos_daemon` |

## Common Commands

Build both Release and Debug binaries:

```bash
python3 build.py --type All
```

Build Debug only (faster, no LTO):

```bash
python3 build.py --type Debug
```

Build using a specific Android API level:

```bash
python3 build.py --api 29
```

Clean previous build and rebuild Release:

```bash
python3 build.py --clean --type Release
```

Clean all build artifacts:

```bash
python3 build.py --clean
```