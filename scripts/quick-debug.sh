#!/bin/bash
# Quick debug launcher - runs QEMU and GDB in split terminals
# Requires tmux

set -e

if ! command -v tmux &> /dev/null; then
    echo "Error: tmux not installed"
    echo "Install with: sudo apt install tmux"
    echo ""
    echo "Or manually run in two terminals:"
    echo "  Terminal 1: ./run.sh --debug --no-kvm"
    echo "  Terminal 2: ./scripts/gdb-debug.sh"
    exit 1
fi

# Create tmux session with split panes
echo "Starting debug session in tmux..."
echo "Controls: Ctrl+B then arrow keys to switch panes"
echo "Exit: Ctrl+B then 'x' and confirm"

tmux new-session -d -s rustrial-debug
tmux split-window -h -t rustrial-debug

# Left pane: QEMU
tmux send-keys -t rustrial-debug:0.0 'cd /home/lazzerex/rustrial-os && ./run.sh --debug --no-kvm' C-m

# Right pane: Wait a bit then start GDB
tmux send-keys -t rustrial-debug:0.1 'cd /home/lazzerex/rustrial-os && sleep 2 && ./scripts/gdb-debug.sh' C-m

# Attach to session
tmux attach-session -t rustrial-debug
