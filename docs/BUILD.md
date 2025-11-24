# Build Guide

## How to Build

### Interactive
Run without any arguments:

```
python3 build.py
```

---

### Argument

**Syntax:**
```
python3 build.py [--abi ABI|all] [--api LEVEL] [--type Release|Debug] [--clean]
```

---

### Examples

Build for arm64:
```
python3 build.py --abi arm64-v8a
```

Build in debug mode:
```
python3 build.py --abi arm64-v8a --type Debug
```

Build with a specific API level:
```
python3 build.py --abi arm64-v8a --api 29
```

Build all ABIs:
```
python3 build.py --abi all
```

Clean and then build:
```
python3 build.py --clean --abi arm64-v8a
```

Clean only:
```
python3 build.py --clean
```