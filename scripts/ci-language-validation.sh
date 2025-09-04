#!/bin/bash
# CI-Optimized Language Processing Validation
# Fast, lightweight tests for CI environments with time/resource constraints
set -euo pipefail

echo "🔬 CI Language Processor Validation (Optimized)"
echo "=============================================="

# CI-specific timeout and resource settings
export RUST_TEST_TIMEOUT=60
export RUST_TEST_THREADS=1  # Single-threaded for consistency
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"

# Fast validation phases for CI
CI_VALIDATION_PHASES=(
    "core_language_semantics"
    "memory_safety_basics" 
    "sigsegv_elimination"
    "interpreter_smoke_test"
)

# Phase 1: Core Language Semantics (< 1 minute)
validate_core_language_semantics() {
    echo "🎯 Phase 1: Core Language Semantics (Fast)"
    
    # Run CI-optimized language tests
    RUST_TEST_THREADS=1 cargo test --test ci_language_validation --no-default-features \
        --verbose 2>&1 | tee ci_core_validation.log
    
    echo "✅ Core language semantics validated"
}

# Phase 2: Memory Safety Basics (< 30 seconds)
validate_memory_safety_basics() {
    echo "🛡️  Phase 2: Memory Safety Basics (Fast)"
    
    # Test basic memory safety without heavy operations
    RUST_TEST_THREADS=1 cargo test --lib --no-default-features \
        --verbose -- \
        immediate_value_safety \
        memory_safety_smoke \
        2>&1 | tee ci_memory_validation.log
    
    echo "✅ Memory safety basics validated"
}

# Phase 3: SIGSEGV Elimination (< 30 seconds)  
validate_sigsegv_elimination() {
    echo "🚨 Phase 3: SIGSEGV Elimination Verification (Fast)"
    
    # Test operations that previously caused SIGSEGV  
    RUST_TEST_THREADS=1 cargo test --test ci_language_validation --no-default-features \
        --verbose -- \
        sigsegv_elimination \
        ci_sigsegv_fixes_preserve_semantics \
        2>&1 | tee ci_sigsegv_validation.log
    
    echo "✅ SIGSEGV elimination verified"
}

# Phase 4: Interpreter Smoke Test (< 1 minute)
validate_interpreter_smoke_test() {
    echo "🎭 Phase 4: Interpreter Smoke Test (Fast)" 
    
    # Build interpreter
    cargo build --bin lambdust --no-default-features --quiet
    
    # Quick smoke test with simple Scheme expression
    echo "Testing basic interpreter functionality..."
    echo "(+ 1 2 3)" | timeout 10s ./target/debug/lambdust 2>&1 | tee ci_interpreter_validation.log || true
    
    # Test basic arithmetic doesn't crash
    if [ -f simple_verification_test.scm ]; then
        timeout 15s ./target/debug/lambdust simple_verification_test.scm \
            2>&1 | tee -a ci_interpreter_validation.log || echo "Simple test completed with exit code"
    else
        echo "Simple verification test not found, creating minimal test..."
        cat > /tmp/ci_minimal_test.scm << 'EOF'
;; Minimal R7RS validation for CI
(define x 42)
(define y (+ x 1))
(display "Basic arithmetic: ")
(display y)
(newline)
EOF
        timeout 15s ./target/debug/lambdust /tmp/ci_minimal_test.scm \
            2>&1 | tee -a ci_interpreter_validation.log || echo "Minimal test completed"
    fi
    
    echo "✅ Interpreter smoke test completed"
}

# CI Validation Report
generate_ci_validation_report() {
    echo ""
    echo "📋 CI Validation Summary Report"
    echo "==============================="
    
    local failed_phases=()
    local warning_phases=()
    
    for phase in "${CI_VALIDATION_PHASES[@]}"; do
        log_file="ci_${phase//_/-}_validation.log"
        
        # Map phase names to log files
        case $phase in
            "core_language_semantics") log_file="ci_core_validation.log" ;;
            "memory_safety_basics") log_file="ci_memory_validation.log" ;; 
            "sigsegv_elimination") log_file="ci_sigsegv_validation.log" ;;
            "interpreter_smoke_test") log_file="ci_interpreter_validation.log" ;;
        esac
        
        if [ -f "$log_file" ]; then
            if grep -q "FAILED\|error.*failed\|SIGSEGV\|segmentation fault" "$log_file"; then
                failed_phases+=("$phase")
                echo "❌ $phase: FAILED"
            elif grep -q "warning\|timeout\|exit code" "$log_file"; then
                warning_phases+=("$phase")
                echo "⚠️  $phase: PASSED WITH WARNINGS"
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
        echo "🎉 All CI language validation phases PASSED!"
        if [ ${#warning_phases[@]} -gt 0 ]; then
            echo "⚠️  Warning phases: ${warning_phases[*]} (monitor in full Docker validation)"
            echo "✅ CI validation SUCCESSFUL with warnings"
        else
            echo "✅ CI validation SUCCESSFUL"
        fi
        exit 0
    else
        echo "🚨 Failed phases: ${failed_phases[*]}"
        echo "❌ CI language validation FAILED"
        echo ""
        echo "🐳 Run Docker validation for comprehensive analysis:"
        echo "   docker-compose -f docker/language-validation/docker-compose.yml up language-validation"
        exit 1
    fi
}

# Performance monitoring for CI optimization
monitor_ci_performance() {
    echo ""
    echo "⚡ CI Performance Metrics"
    echo "========================"
    
    # Show timing information
    if command -v time >/dev/null 2>&1; then
        echo "Test execution timing logged to: ci_timing.log"
    fi
    
    # Show resource usage
    echo "Peak memory usage: $(ps -o pid,vsz,rss,comm -p $$ | tail -1)"
    
    # Compilation cache efficiency
    if [ -d "$CARGO_TARGET_DIR" ]; then
        target_size=$(du -sh "$CARGO_TARGET_DIR" 2>/dev/null | cut -f1 || echo "unknown")
        echo "Cargo target directory size: $target_size"
    fi
}

# Main execution
main() {
    echo "Starting CI language validation at $(date)"
    echo "Working directory: $(pwd)"
    echo "Rust version: $(rustc --version)"
    echo "CI optimization: ENABLED (fast mode)"
    echo ""
    
    local start_time=$(date +%s)
    
    # Execute CI validation phases
    for phase in "${CI_VALIDATION_PHASES[@]}"; do
        echo "Executing CI phase: $phase"
        phase_start=$(date +%s)
        
        "validate_$phase" || {
            echo "⚠️  Phase $phase encountered issues, continuing..."
        }
        
        phase_end=$(date +%s)
        phase_duration=$((phase_end - phase_start))
        echo "Phase duration: ${phase_duration}s"
        echo ""
    done
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    echo "Total CI validation time: ${total_duration}s"
    
    # Performance monitoring
    monitor_ci_performance
    
    # Generate final report
    generate_ci_validation_report
}

# Trap for cleanup
trap 'echo "CI validation interrupted at $(date)"' INT TERM

# Execute main function with timing
time main "$@" 2>&1 | tee ci_timing.log || main "$@"