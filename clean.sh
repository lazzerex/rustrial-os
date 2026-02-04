#!/bin/bash
# Clean build artifacts for RustrialOS
# Usage: ./clean.sh [options]
#   -a, --all       Deep clean (including Cargo cache)
#   -n, --native    Clean only native artifacts
#   -h, --help      Show this help

set -e

DEEP_CLEAN=0
NATIVE_ONLY=0

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--all)
            DEEP_CLEAN=1
            shift
            ;;
        -n|--native)
            NATIVE_ONLY=1
            shift
            ;;
        -h|--help)
            grep '^#' "$0" | grep -v '#!/bin/bash' | sed 's/^# //'
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Cleaning RustrialOS build artifacts..."

if [ $NATIVE_ONLY -eq 1 ]; then
    echo "Cleaning native C/ASM objects..."
    rm -f target/*.o
    if [ -f "src/native/Makefile" ]; then
        cd src/native && make clean && cd ../..
    fi
    echo "Native artifacts cleaned!"
    exit 0
fi

if [ $DEEP_CLEAN -eq 1 ]; then
    echo "Deep clean mode - removing all build artifacts..."
    cargo clean
    rm -rf target/
    rm -f Cargo.lock
    echo "Deep clean complete!"
else
    echo "Standard clean..."
    cargo clean
    rm -f target/*.o
    echo "Clean complete!"
fi

echo "Done!"
