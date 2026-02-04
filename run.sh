#!/bin/bash
# Quick QEMU runner for RustrialOS
# Usage: ./run.sh [options]
#   -d, --debug     Enable GDB debugging (port 1234)
#   -n, --no-kvm    Disable KVM acceleration
#   -s, --serial    Show serial output in terminal
#   -g, --graphics  Enable graphics mode (default)
#   -h, --help      Show this help

set -e

# Default options
USE_KVM="-enable-kvm"
DEBUG_MODE=""
SERIAL_MODE="-serial mon:stdio"
MEMORY="256M"
CPU="2"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--debug)
            DEBUG_MODE="-s -S"
            echo "Debug mode enabled. Connect GDB to localhost:1234"
            shift
            ;;
        -n|--no-kvm)
            USE_KVM=""
            echo "KVM disabled"
            shift
            ;;
        -s|--serial)
            SERIAL_MODE="-serial stdio"
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

# Check if bootimage exists
BOOTIMAGE="target/x86_64-rustrial_os/debug/bootimage-rustrial_os.bin"

if [ ! -f "$BOOTIMAGE" ]; then
    echo "Bootimage not found. Building..."
    cargo bootimage --target x86_64-rustrial_os.json
fi

echo "Starting RustrialOS in QEMU..."
echo "Memory: $MEMORY, CPUs: $CPU"

# Run QEMU
qemu-system-x86_64 \
    $USE_KVM \
    $DEBUG_MODE \
    -drive format=raw,file=$BOOTIMAGE \
    -m $MEMORY \
    -smp $CPU \
    $SERIAL_MODE \
    -device rtl8139,netdev=net0 \
    -netdev user,id=net0,hostfwd=tcp::8080-:80

echo "QEMU exited"
