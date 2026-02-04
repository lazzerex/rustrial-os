# Development Scripts

Automation scripts for building, testing, and running RustrialOS.

## Root Scripts

### `./run.sh`
Quick QEMU runner with options:
- `-d, --debug` - Enable GDB debugging (port 1234)
- `-n, --no-kvm` - Disable KVM acceleration
- `-s, --serial` - Show serial output in terminal
- `-h, --help` - Show help

**Example:**
```bash
./run.sh              # Run with default settings
./run.sh --debug      # Run with GDB server
```

### `./build.sh`
Build automation with options:
- `-r, --release` - Build in release mode
- `-c, --check` - Only check compilation (no bootimage)
- `-t, --test` - Build and run tests
- `-v, --verbose` - Verbose output
- `-h, --help` - Show help

**Example:**
```bash
./build.sh            # Debug build
./build.sh --release  # Optimized build
./build.sh --check    # Quick compile check
```

### `./test.sh`
Test automation:
- Run all integration tests
- Or run specific test: `./test.sh tcp_test`
- Shows pass/fail summary

**Example:**
```bash
./test.sh             # Run all tests
./test.sh tcp_test    # Run specific test
```

### `./clean.sh`
Clean build artifacts:
- `-a, --all` - Deep clean (including Cargo cache)
- `-n, --native` - Clean only native C/ASM objects
- `-h, --help` - Show help

**Example:**
```bash
./clean.sh            # Standard clean
./clean.sh --all      # Deep clean
```

## Scripts Directory

### `scripts/gdb-debug.sh`
GDB debugging helper:
1. First terminal: `./run.sh --debug`
2. Second terminal: `./scripts/gdb-debug.sh`

Automatically connects to QEMU's GDB server with proper settings.

### `scripts/make-iso.sh`
Create bootable ISO image:
- Requires: `grub-mkrescue`, `xorriso`
- Output: `rustrial-os.iso`

**Example:**
```bash
./scripts/make-iso.sh
qemu-system-x86_64 -cdrom rustrial-os.iso
```

### `scripts/generate-config.py`
Generate configuration headers:
- Creates `src/config.rs` with build constants
- Memory sizes, network buffers, display settings
- Automatically called by build system

**Example:**
```bash
./scripts/generate-config.py
```

## Quick Start Workflow

```bash
# Clean build
./clean.sh

# Build the OS
./build.sh

# Run in QEMU
./run.sh

# Run tests
./test.sh

# Debug with GDB
./run.sh --debug    # Terminal 1
./scripts/gdb-debug.sh  # Terminal 2
```

## CI/CD Integration

These scripts are designed to work in CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Build
  run: ./build.sh --release

- name: Test
  run: ./test.sh

- name: Create ISO
  run: ./scripts/make-iso.sh
```

## Script Dependencies

- **QEMU**: `qemu-system-x86_64`
- **GDB**: `gdb` (for debugging)
- **GRUB**: `grub-mkrescue` (for ISO creation)
- **Xorriso**: `xorriso` (for ISO creation)
- **Python 3**: For config generation
- **Clang/NASM**: For native C/ASM compilation

## Notes

- All scripts have proper error handling (`set -e`)
- Scripts are self-documenting (use `--help`)
- Network configuration in `run.sh` includes RTL8139 NIC
- GDB script includes Intel syntax and panic breakpoints
