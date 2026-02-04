#!/bin/bash
# GDB debugging helper for RustrialOS
# First run: ./run.sh --debug
# Then in another terminal: ./scripts/gdb-debug.sh

set -e

# Find project root (where Cargo.toml is)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/../Cargo.toml" ]; then
    PROJECT_ROOT="$SCRIPT_DIR/.."
elif [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    PROJECT_ROOT="$SCRIPT_DIR"
else
    echo "Error: Cannot find project root (Cargo.toml)"
    exit 1
fi

cd "$PROJECT_ROOT"

KERNEL="target/x86_64-rustrial_os/debug/rustrial_os"

if [ ! -f "$KERNEL" ]; then
    echo "Kernel binary not found at $KERNEL"
    echo "Building kernel first..."
    echo ""
    cargo build --target x86_64-rustrial_os.json
    echo ""
    if [ ! -f "$KERNEL" ]; then
        echo "Error: Build succeeded but kernel binary still not found"
        exit 1
    fi
fi

echo "Connecting to QEMU GDB server on localhost:1234..."
echo "Kernel: $KERNEL"
echo ""

# Create GDB script
cat > /tmp/gdb-rustrial.txt << 'EOF'
# Connect to QEMU
target remote localhost:1234

# Set architecture
set architecture i386:x86-64:intel

# Useful settings
set disassembly-flavor intel
set print pretty on
set pagination off

# Try to set breakpoints at key locations
# Use 'hbreak' for hardware breakpoints which work better for kernel debugging
hbreak _start
hbreak rustrial_os::kernel_main
hbreak rustrial_os::kernel_main_custom

# Show current location
echo \n=== Current CPU State ===\n
info registers rip rsp rbp
echo \n=== Next Instructions ===\n
x/10i $rip

# List breakpoints
echo \n=== Breakpoints ===\n
info breakpoints

# Ready to debug
echo \n
echo ====================================\n
echo GDB connected! CPU is PAUSED.\n
echo ====================================\n
echo Useful commands:\n
echo   c        - Continue execution\n
echo   si       - Step one instruction\n
echo   ni       - Next instruction (skip calls)\n
echo   bt       - Backtrace (call stack)\n
echo   info br  - List breakpoints\n
echo   info reg - Show all registers\n
echo   p expr   - Print expression\n
echo   x/10i $rip - Disassemble at current location\n
echo   q        - Quit GDB\n
echo ====================================\n
echo Type 'c' to start the kernel!\n
echo \n
EOF

gdb -x /tmp/gdb-rustrial.txt "$KERNEL"

rm -f /tmp/gdb-rustrial.txt
