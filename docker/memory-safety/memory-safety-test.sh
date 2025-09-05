#!/bin/bash
set -e

echo "🛡️  Memory Safety Validation Suite for Lambdust"
echo "================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

FAILED_TESTS=0
TOTAL_TESTS=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}Running: $test_name${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASS: $test_name${NC}"
    else
        echo -e "${RED}❌ FAIL: $test_name${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

echo "🔧 System Information:"
uname -a
rustc --version
cargo --version
echo ""

# 1. Basic Compilation Check
echo -e "${YELLOW}Phase 1: Basic Compilation & Static Analysis${NC}"
run_test "Clean Build" "cargo clean && cargo check --all-targets --all-features"
run_test "Clippy Analysis" "cargo clippy --all-targets --all-features -- -D warnings"
run_test "Format Check" "cargo fmt --check"

# 2. Memory Safety with AddressSanitizer
echo -e "${YELLOW}Phase 2: AddressSanitizer Validation${NC}"
export RUSTFLAGS="-Zsanitizer=address"
run_test "AddressSanitizer Build" "+nightly cargo build --target x86_64-unknown-linux-gnu"
run_test "AddressSanitizer Tests - Core" "+nightly cargo test --target x86_64-unknown-linux-gnu --lib eval::optimized_value"
run_test "AddressSanitizer Tests - Containers" "+nightly cargo test --target x86_64-unknown-linux-gnu --lib containers::ordered_set"

# 3. Valgrind Memory Leak Detection
echo -e "${YELLOW}Phase 3: Valgrind Memory Leak Detection${NC}"
unset RUSTFLAGS
cargo build --lib --no-default-features
run_test "Valgrind Memcheck - Unit Tests" "valgrind --tool=memcheck --leak-check=full --show-leak-kinds=all --track-origins=yes --error-exitcode=1 cargo test --lib --no-default-features optimized_value 2>&1 | tee /tmp/valgrind.log"

# 4. Miri Undefined Behavior Detection
echo -e "${YELLOW}Phase 4: Miri Undefined Behavior Detection${NC}"
run_test "Miri UB Check - OptimizedValue" "+nightly cargo miri test optimized_value --lib"
run_test "Miri UB Check - OrderedSet" "+nightly cargo miri test ordered_set --lib"

# 5. Stress Testing
echo -e "${YELLOW}Phase 5: Stress Testing${NC}"
run_test "High-Load Memory Operations" "timeout 300 cargo test --lib --release --no-default-features -- --test-threads=1"

# 6. SIGSEGV Specific Tests
echo -e "${YELLOW}Phase 6: SIGSEGV-Specific Validation${NC}"
run_test "Red-Black Tree Stress Test" "cargo test --lib containers::ordered_set::tests::test_stress_removal --no-default-features -- --exact"
run_test "Unsafe Pointer Validation" "cargo test --lib eval::optimized_value::tests --no-default-features"

# 7. Stack Overflow Detection
echo -e "${YELLOW}Phase 7: Stack Overflow Detection${NC}"
export RUST_MIN_STACK=2097152  # 2MB stack
run_test "Large Stack Test" "cargo test --lib --no-default-features -- --test-threads=1"

# Results Summary
echo "================================================="
echo -e "${BLUE}Memory Safety Validation Results:${NC}"
echo "Total Tests: $TOTAL_TESTS"
echo "Failed Tests: $FAILED_TESTS"
echo "Passed Tests: $((TOTAL_TESTS - FAILED_TESTS))"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL MEMORY SAFETY TESTS PASSED!${NC}"
    echo -e "${GREEN}✅ SIGSEGV issues appear to be resolved${NC}"
    exit 0
else
    echo -e "${RED}⚠️  $FAILED_TESTS MEMORY SAFETY TESTS FAILED${NC}"
    echo -e "${RED}❌ SIGSEGV issues may still exist${NC}"
    exit 1
fi