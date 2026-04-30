#!/bin/bash
# Start the host-side network test server and then launch QEMU.

set -e

ROOT_DIR="/home/lazzerex/rustrial-os"
SERVER_SCRIPT="$ROOT_DIR/scripts/network-test-server.py"
LOG_FILE="$(mktemp "${TMPDIR:-/tmp}/rustrial-network-test.XXXXXX.log")"

if ! command -v python3 >/dev/null 2>&1; then
    echo "Error: python3 not installed"
    exit 1
fi

python3 "$SERVER_SCRIPT" > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

cleanup() {
    kill "$SERVER_PID" >/dev/null 2>&1 || true
}

trap cleanup EXIT INT TERM

echo "Started host network test server (pid $SERVER_PID)"
echo "Log: $LOG_FILE"
echo "Launching QEMU..."

cd "$ROOT_DIR"
./run.sh --serial --no-kvm "$@"