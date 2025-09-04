#!/bin/bash
# Language-Specific SIGSEGV Validation Suite
set -euo pipefail

echo "🔬 Language Processor Memory Safety Validation"
echo "=============================================="

# Language processing validation phases
VALIDATION_PHASES=(
    "immediate_values"
    "symbol_interning" 
    "environment_chains"
    "value_operations"
    "r7rs_compliance"
    "stress_testing"
)

# Phase 1: Immediate Value Safety
validate_immediate_values() {
    echo "📊 Phase 1: Immediate Value Safety Testing"
    
    # Test immediate value operations without heap allocation
    cargo test --lib --no-default-features -p lambdust \
        --verbose -- immediate_value 2>&1 | tee immediate_validation.log
    
    # Verify no memory allocations for primitive operations
    echo "Testing zero-allocation immediate operations..."
    cargo test --lib --no-default-features -p lambdust \
        --verbose -- zero_alloc 2>&1 | tee -a immediate_validation.log
}

# Phase 2: Symbol Interning Safety
validate_symbol_interning() {
    echo "🏷️  Phase 2: Symbol Interning Safety Testing"
    
    # Test symbol table operations under concurrent access
    RUST_TEST_THREADS=4 cargo test --lib --no-default-features \
        --verbose -- symbol 2>&1 | tee symbol_validation.log
    
    # Test symbol memory safety with AddressSanitizer
    if rustc --version | grep -q nightly; then
        echo "Running AddressSanitizer on symbol operations..."
        RUSTFLAGS='-Zsanitizer=address' cargo +nightly test \
            --target x86_64-unknown-linux-gnu --lib \
            --verbose -- symbol 2>&1 | tee -a symbol_validation.log
    fi
}

# Phase 3: Environment Chain Safety  
validate_environment_chains() {
    echo "🌍 Phase 3: Environment Chain Safety Testing"
    
    # Test deep environment nesting without stack overflow
    cargo test --lib --no-default-features \
        --verbose -- environment 2>&1 | tee environment_validation.log
    
    # Test with reduced stack size to catch overflow issues
    echo "Testing with limited stack size..."
    RUST_MIN_STACK=1048576 cargo test --lib --no-default-features \
        --verbose -- environment_deep 2>&1 | tee -a environment_validation.log
}

# Phase 4: Value Operations Safety
validate_value_operations() {
    echo "💎 Phase 4: Value Operations Safety Testing"
    
    # Test OptimizedValue operations with Miri
    if rustc +nightly --version >/dev/null 2>&1; then
        echo "Running Miri undefined behavior detection..."
        cargo +nightly miri test --lib --no-default-features \
            --verbose -- optimized_value 2>&1 | tee value_validation.log || true
    fi
    
    # Test value operations with Valgrind
    echo "Running Valgrind memory safety check..."
    cargo build --lib --no-default-features
    valgrind --tool=memcheck --track-origins=yes --leak-check=full \
        --error-exitcode=1 \
        cargo test --lib --no-default-features \
        --verbose -- value_safety 2>&1 | tee -a value_validation.log || true
}

# Phase 5: R7RS Compliance Testing
validate_r7rs_compliance() {
    echo "📚 Phase 5: R7RS Compliance Safety Testing"
    
    # Build the interpreter
    cargo build --bin lambdust --no-default-features
    
    # Test verification_test.scm in containerized environment
    echo "Running R7RS compliance verification..."
    if [ -f "/workspace/verification_test.scm" ]; then
        timeout 60s ./target/debug/lambdust verification_test.scm \
            2>&1 | tee r7rs_validation.log || echo "Timeout or error in R7RS test"
    else
        echo "verification_test.scm not found, skipping R7RS validation"
    fi
    
    # Test container operations specifically
    echo "Testing container system safety..."
    cargo test --lib --no-default-features \
        --verbose -- container 2>&1 | tee -a r7rs_validation.log
}

# Phase 6: Stress Testing
validate_stress_testing() {
    echo "💪 Phase 6: Language Processor Stress Testing"
    
    # High-concurrency testing
    echo "Running concurrent stress test..."
    RUST_TEST_THREADS=8 cargo test --lib --no-default-features \
        --verbose -- concurrent 2>&1 | tee stress_validation.log
    
    # Memory pressure testing
    echo "Running memory pressure test..."
    RUST_BACKTRACE=full timeout 120s cargo test --lib --no-default-features \
        --verbose -- memory_pressure 2>&1 | tee -a stress_validation.log || true
}

# Validation Summary
generate_validation_report() {
    echo "📋 Validation Summary Report"
    echo "============================"
    
    local failed_phases=()
    
    for phase in "${VALIDATION_PHASES[@]}"; do
        log_file="${phase}_validation.log"
        if [ -f "$log_file" ]; then
            if grep -q "FAILED\|error\|SIGSEGV\|segmentation fault" "$log_file"; then
                failed_phases+=("$phase")
                echo "❌ $phase: FAILED"
            else
                echo "✅ $phase: PASSED"
            fi
        else
            failed_phases+=("$phase")
            echo "⚠️  $phase: NO LOG FOUND"
        fi
    done
    
    echo ""
    if [ ${#failed_phases[@]} -eq 0 ]; then
        echo "🎉 All language validation phases PASSED!"
        echo "✅ Memory safety validation SUCCESSFUL"
        exit 0
    else
        echo "🚨 Failed phases: ${failed_phases[*]}"
        echo "❌ Memory safety validation FAILED"
        exit 1
    fi
}

# Main execution
main() {
    echo "Starting language processor validation at $(date)"
    echo "Working directory: $(pwd)"
    echo "Rust version: $(rustc --version)"
    echo ""
    
    # Execute validation phases
    for phase in "${VALIDATION_PHASES[@]}"; do
        echo "Executing validation phase: $phase"
        "validate_$phase" || {
            echo "⚠️  Phase $phase encountered issues, continuing..."
        }
        echo ""
    done
    
    # Generate final report
    generate_validation_report
}

# Execute main function
main "$@"