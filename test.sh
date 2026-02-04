#!/bin/bash
# Test automation for RustrialOS
# Usage: ./test.sh [test-name]
#   If no test name provided, runs all tests

set -e

echo "========================================"
echo "  RustrialOS Test Suite"
echo "========================================"

# List of integration tests
TESTS=(
    "basic_boot"
    "heap_allocation"
    "stack_overflow"
    "should_panic"
    "tcp_test"
    "network_test"
    "loopback_test"
    "ethernet_test"
    "arp_test"
    "ipv4_test"
    "icmp_test"
    "udp_test"
)

# If argument provided, run specific test
if [ $# -eq 1 ]; then
    TEST_NAME=$1
    echo "Running test: $TEST_NAME"
    echo ""
    cargo test --test "$TEST_NAME"
    exit 0
fi

# Run all tests
echo "Running all integration tests..."
echo ""

PASSED=0
FAILED=0
FAILED_TESTS=()

for test in "${TESTS[@]}"; do
    echo "----------------------------------------"
    echo "Test: $test"
    echo "----------------------------------------"
    
    if cargo test --test "$test" 2>&1; then
        echo "[OK] $test"
        ((PASSED++))
    else
        echo "[FAIL] $test"
        ((FAILED++))
        FAILED_TESTS+=("$test")
    fi
    echo ""
done

# Summary
echo "========================================"
echo "  Test Summary"
echo "========================================"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total:  $((PASSED + FAILED))"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo "Failed tests:"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    exit 1
else
    echo ""
    echo "All tests passed! ✓"
    exit 0
fi
