# How to Build

## 1. Build Instructions

Use the provided automatic build script.

**(Linux/macOS only):** Make the script executable:

```bash
chmod +x build.sh
```

### A. Default Build (arm64-v8a)

**Windows:**

```powershell
.\build.bat
```

**Linux/macOS:**

```bash
./build.sh
```

### B. Build Other ABIs

**Windows:**

```powershell
.\build.bat armeabi-v7a
```

**Linux/macOS:**

```bash
./build.sh armeabi-v7a
```

### C. Clean Build

**Windows:**

```powershell
.\build.bat clean
```

**Linux/macOS:**

```bash
./build.sh clean
```

## 2. Build Output

The binary will be available in `build/<ABI>/adaptive_daemon` (e.g., `build/arm64-v8a/adaptive_daemon`).

Use `adb` to push and execute:

```bash
adb push build/arm64-v8a/adaptive_daemon /data/local/tmp/
adb shell
su
/data/local/tmp/adaptive_daemon
```