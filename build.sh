#!/bin/bash
# Build script for RustrialOS
# Usage: ./build.sh [options]
#   -r, --release   Build in release mode
#   -c, --check     Only check compilation (no bootimage)
#   -t, --test      Build and run tests
#   -v, --verbose   Verbose output
#   -h, --help      Show this help

set -e

# Default options
BUILD_MODE="debug"
CARGO_CMD="cargo bootimage"
TARGET="--target x86_64-rustrial_os.json"
VERBOSE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--release)
            BUILD_MODE="release"
            TARGET="$TARGET --release"
            echo "Building in RELEASE mode"
            shift
            ;;
        -c|--check)
            CARGO_CMD="cargo check"
            echo "Check mode (no bootimage)"
            shift
            ;;
        -t|--test)
            CARGO_CMD="cargo test"
            echo "Test mode"
            shift
            ;;
        -v|--verbose)
            VERBOSE="--verbose"
            shift
            ;;
        -h|--help)
            grep '^#' "$0" | grep -v '#!/bin/bash' | sed 's/^# //'
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "========================================"
echo "  RustrialOS Build System"
echo "========================================"
echo "Mode: $BUILD_MODE"
echo ""

# Build native C/ASM code first
echo "[1/3] Building native C/ASM components..."
cd src/native
if [ -f "Makefile" ]; then
    make
else
    echo "Compiling PCI driver..."
    clang -c -m32 -ffreestanding -fno-pie -O2 pci.c -o ../../target/pci.o
    echo "Compiling RTC driver..."
    clang -c -m32 -ffreestanding -fno-pie -O2 rtc.c -o ../../target/rtc.o
    echo "Assembling CPU ASM..."
    nasm -f elf32 cpu_asm.asm -o ../../target/cpu_asm.o
fi
cd ../..

echo ""
echo "[2/3] Building Rust kernel..."
$CARGO_CMD $TARGET $VERBOSE

echo ""
echo "[3/3] Build complete!"
if [ "$CARGO_CMD" = "cargo bootimage" ]; then
    BOOTIMAGE="target/x86_64-rustrial_os/$BUILD_MODE/bootimage-rustrial_os.bin"
    SIZE=$(du -h "$BOOTIMAGE" | cut -f1)
    echo "Bootimage: $BOOTIMAGE ($SIZE)"
    echo ""
    echo "Run with: ./run.sh"
fi

echo "========================================"
