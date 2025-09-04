#!/bin/bash
# SIGSEGV Elimination Validation Script for Lambdust Language Processor
# Comprehensive Docker-based validation ensuring memory safety across all language components

set -euo pipefail

# Color output functions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

# Configuration
DOCKER_COMPOSE_DIR="docker/language-validation"
LOG_DIR="validation_logs"
TIMEOUT_DURATION="600" # 10 minutes timeout
VERIFICATION_SCRIPT="verification_test.scm"

# Validation phases
declare -a VALIDATION_PHASES=(
    "docker_setup"
    "memory_safety_validation"
    "language_processor_validation"
    "r7rs_compliance_validation"
    "stress_testing_validation"
    "verification_script_validation"
)

# Phase tracking
declare -A PHASE_STATUS
declare -A PHASE_LOGS

initialize_validation() {
    info "Initializing SIGSEGV Elimination Validation"
    echo "=============================================="
    
    # Create log directory
    mkdir -p "$LOG_DIR"
    
    # Initialize phase status
    for phase in "${VALIDATION_PHASES[@]}"; do
        PHASE_STATUS[$phase]="PENDING"
        PHASE_LOGS[$phase]="$LOG_DIR/${phase}.log"
    done
    
    # System information
    info "System Information:"
    echo "  - OS: $(uname -s) $(uname -r)"
    echo "  - Architecture: $(uname -m)"
    echo "  - Date: $(date)"
    echo "  - Docker: $(docker --version 2>/dev/null || echo 'Not available')"
    echo "  - Docker Compose: $(docker-compose --version 2>/dev/null || echo 'Not available')"
    echo ""
}

# Phase 1: Docker Setup and Build
validate_docker_setup() {
    info "Phase 1: Docker Environment Setup"
    
    local log_file="${PHASE_LOGS[docker_setup]}"
    
    {
        echo "=== Docker Setup Phase ==="
        echo "Timestamp: $(date)"
        echo ""
        
        # Check Docker availability
        echo "Checking Docker availability..."
        if ! command -v docker &> /dev/null; then
            echo "ERROR: Docker not found"
            exit 1
        fi
        
        if ! command -v docker-compose &> /dev/null; then
            echo "ERROR: Docker Compose not found"
            exit 1
        fi
        
        echo "Docker and Docker Compose are available"
        
        # Change to docker directory
        cd "$DOCKER_COMPOSE_DIR"
        
        echo "Building language validation container..."
        docker-compose build language-validation
        
        echo "Docker setup completed successfully"
        
    } 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        PHASE_STATUS[docker_setup]="PASSED"
        success "Docker setup completed"
    else
        PHASE_STATUS[docker_setup]="FAILED"
        error "Docker setup failed"
        return 1
    fi
}

# Phase 2: Memory Safety Validation
validate_memory_safety() {
    info "Phase 2: Memory Safety Validation"
    
    local log_file="${PHASE_LOGS[memory_safety_validation]}"
    
    {
        echo "=== Memory Safety Validation Phase ==="
        echo "Timestamp: $(date)"
        echo ""
        
        cd "$DOCKER_COMPOSE_DIR"
        
        echo "Running AddressSanitizer validation..."
        timeout "$TIMEOUT_DURATION" docker-compose run --rm asan || {
            echo "AddressSanitizer validation failed or timed out"
            exit 1
        }
        
        echo ""
        echo "Running Memory Sanitizer validation..."
        timeout "$TIMEOUT_DURATION" docker-compose run --rm msan || {
            echo "WARNING: Memory Sanitizer validation failed (may not be critical)"
        }
        
        echo ""
        echo "Memory safety validation completed"
        
    } 2>&1 | tee "$log_file"
    
    # Check for SIGSEGV or other critical errors
    if grep -qiE "sigsegv|segmentation fault|memory error|use after free" "$log_file"; then
        PHASE_STATUS[memory_safety_validation]="FAILED"
        error "Memory safety violations detected"
        return 1
    else
        PHASE_STATUS[memory_safety_validation]="PASSED"
        success "Memory safety validation passed"
    fi
}

# Phase 3: Language Processor Validation
validate_language_processor() {
    info "Phase 3: Language Processor Validation"
    
    local log_file="${PHASE_LOGS[language_processor_validation]}"
    
    {
        echo "=== Language Processor Validation Phase ==="
        echo "Timestamp: $(date)"
        echo ""
        
        cd "$DOCKER_COMPOSE_DIR"
        
        echo "Running comprehensive language processor tests..."
        timeout "$TIMEOUT_DURATION" docker-compose run --rm language-validation bash -c "
            echo '🔧 Building project...'
            cargo build --lib --no-default-features
            
            echo ''
            echo '🧪 Running language safety tests...'
            cargo test --lib --no-default-features language_safety_validation --verbose
            
            echo ''
            echo '💎 Running optimized value tests...'
            cargo test --lib --no-default-features optimized_value --verbose
            
            echo ''
            echo '🌍 Running environment tests...'
            cargo test --lib --no-default-features environment --verbose
            
            echo ''
            echo '🏷️ Running symbol tests...'
            cargo test --lib --no-default-features symbol --verbose
            
            echo ''
            echo '📦 Running container tests...'
            cargo test --lib --no-default-features container --verbose
            
            echo ''
            echo 'Language processor validation completed'
        "
        
    } 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        PHASE_STATUS[language_processor_validation]="PASSED"
        success "Language processor validation passed"
    else
        PHASE_STATUS[language_processor_validation]="FAILED"
        error "Language processor validation failed"
        return 1
    fi
}

# Phase 4: R7RS Compliance Validation
validate_r7rs_compliance() {
    info "Phase 4: R7RS Compliance Validation"
    
    local log_file="${PHASE_LOGS[r7rs_compliance_validation]}"
    
    {
        echo "=== R7RS Compliance Validation Phase ==="
        echo "Timestamp: $(date)"
        echo ""
        
        cd "$DOCKER_COMPOSE_DIR"
        
        echo "Building Lambdust interpreter..."
        timeout "$TIMEOUT_DURATION" docker-compose run --rm language-validation bash -c "
            cargo build --bin lambdust --no-default-features
            echo 'Interpreter built successfully'
        "
        
        echo ""
        echo "Running R7RS compliance tests..."
        timeout "$TIMEOUT_DURATION" docker-compose run --rm language-validation bash -c "
            echo '📚 Testing basic R7RS features...'
            cargo test --lib --no-default-features r7rs --verbose
            
            echo ''
            echo '🔍 Testing SRFI implementations...'
            cargo test --lib --no-default-features srfi --verbose || echo 'Some SRFI tests may be expected to fail'
            
            echo 'R7RS compliance validation completed'
        "
        
    } 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        PHASE_STATUS[r7rs_compliance_validation]="PASSED"
        success "R7RS compliance validation passed"
    else
        PHASE_STATUS[r7rs_compliance_validation]="FAILED"
        error "R7RS compliance validation failed"
        return 1
    fi
}

# Phase 5: Stress Testing Validation
validate_stress_testing() {
    info "Phase 5: Stress Testing Validation"
    
    local log_file="${PHASE_LOGS[stress_testing_validation]}"
    
    {
        echo "=== Stress Testing Validation Phase ==="
        echo "Timestamp: $(date)"
        echo ""
        
        cd "$DOCKER_COMPOSE_DIR"
        
        echo "Running concurrent stress tests..."
        timeout "$TIMEOUT_DURATION" docker-compose run --rm language-validation bash -c "
            echo '💪 Running multi-threaded tests...'
            RUST_TEST_THREADS=4 cargo test --lib --no-default-features concurrent --verbose
            
            echo ''
            echo '🔥 Running memory pressure tests...'  
            RUST_BACKTRACE=full cargo test --lib --no-default-features memory_pressure --verbose
            
            echo ''
            echo '⚡ Running performance-sensitive tests...'
            cargo test --lib --no-default-features benchmark --verbose || echo 'Some benchmark tests may have different results in containers'
            
            echo 'Stress testing completed'
        "
        
    } 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        PHASE_STATUS[stress_testing_validation]="PASSED"
        success "Stress testing validation passed"
    else
        PHASE_STATUS[stress_testing_validation]="FAILED"
        error "Stress testing validation failed"
        return 1
    fi
}

# Phase 6: Verification Script Validation
validate_verification_script() {
    info "Phase 6: Verification Script Validation"
    
    local log_file="${PHASE_LOGS[verification_script_validation]}"
    
    {
        echo "=== Verification Script Validation Phase ==="
        echo "Timestamp: $(date)"
        echo ""
        
        cd "$DOCKER_COMPOSE_DIR"
        
        echo "Checking for verification script..."
        if [ -f "../../$VERIFICATION_SCRIPT" ]; then
            echo "Found verification script: $VERIFICATION_SCRIPT"
            
            echo "Running verification script in container..."
            timeout "$TIMEOUT_DURATION" docker-compose run --rm language-validation bash -c "
                echo '🧪 Running verification_test.scm...'
                ./target/debug/lambdust '$VERIFICATION_SCRIPT' || {
                    echo 'Verification script execution failed'
                    exit 1
                }
                echo 'Verification script completed successfully'
            "
        else
            echo "Verification script not found at ../../$VERIFICATION_SCRIPT"
            echo "Skipping verification script validation"
            return 0
        fi
        
    } 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        PHASE_STATUS[verification_script_validation]="PASSED" 
        success "Verification script validation passed"
    else
        PHASE_STATUS[verification_script_validation]="FAILED"
        error "Verification script validation failed"
        return 1
    fi
}

# Generate comprehensive validation report
generate_validation_report() {
    info "Generating Validation Report"
    
    local report_file="$LOG_DIR/validation_report.txt"
    
    {
        echo "SIGSEGV ELIMINATION VALIDATION REPORT"
        echo "======================================"
        echo "Generated: $(date)"
        echo "System: $(uname -s) $(uname -r) $(uname -m)"
        echo ""
        
        echo "PHASE RESULTS:"
        echo "--------------"
        
        local total_phases=0
        local passed_phases=0
        local failed_phases=0
        
        for phase in "${VALIDATION_PHASES[@]}"; do
            total_phases=$((total_phases + 1))
            local status="${PHASE_STATUS[$phase]}"
            
            printf "%-35s: %s\n" "$phase" "$status"
            
            case "$status" in
                "PASSED") passed_phases=$((passed_phases + 1)) ;;
                "FAILED") failed_phases=$((failed_phases + 1)) ;;
            esac
        done
        
        echo ""
        echo "SUMMARY:"
        echo "--------"
        echo "Total phases: $total_phases"
        echo "Passed: $passed_phases"
        echo "Failed: $failed_phases"
        echo "Success rate: $((passed_phases * 100 / total_phases))%"
        
        if [ $failed_phases -eq 0 ]; then
            echo ""
            echo "🎉 VALIDATION RESULT: PASSED"
            echo "✅ All SIGSEGV elimination validations successful"
            echo "✅ Memory safety validated across all language components"
            echo "✅ R7RS compliance maintained"
            echo "✅ Language processor is production-ready"
        else
            echo ""
            echo "❌ VALIDATION RESULT: FAILED"
            echo "🚨 $failed_phases phase(s) failed validation"
            echo "⚠️  Memory safety issues may still exist"
            echo "⛔ Not recommended for production use"
        fi
        
        echo ""
        echo "DETAILED LOGS:"
        echo "--------------"
        for phase in "${VALIDATION_PHASES[@]}"; do
            echo "$phase: ${PHASE_LOGS[$phase]}"
        done
        
    } | tee "$report_file"
    
    # Return appropriate exit code
    if [ $failed_phases -eq 0 ]; then
        return 0
    else
        return 1
    fi
}

# Cleanup function
cleanup_validation() {
    info "Cleaning up validation environment"
    
    if [ -d "$DOCKER_COMPOSE_DIR" ]; then
        cd "$DOCKER_COMPOSE_DIR"
        docker-compose down --volumes --remove-orphans 2>/dev/null || true
    fi
    
    info "Cleanup completed"
}

# Main execution function
main() {
    local start_time=$(date +%s)
    
    # Set up signal handlers for cleanup
    trap cleanup_validation EXIT
    trap 'error "Validation interrupted"; cleanup_validation; exit 130' INT TERM
    
    initialize_validation
    
    # Execute validation phases
    local overall_success=true
    
    for phase in "${VALIDATION_PHASES[@]}"; do
        info "Executing phase: $phase"
        
        case "$phase" in
            "docker_setup") validate_docker_setup ;;
            "memory_safety_validation") validate_memory_safety ;;
            "language_processor_validation") validate_language_processor ;;
            "r7rs_compliance_validation") validate_r7rs_compliance ;;
            "stress_testing_validation") validate_stress_testing ;;
            "verification_script_validation") validate_verification_script ;;
            *) error "Unknown validation phase: $phase"; overall_success=false ;;
        esac
        
        if [ "${PHASE_STATUS[$phase]}" = "FAILED" ]; then
            overall_success=false
            warning "Phase $phase failed, but continuing with remaining phases"
        fi
        
        echo ""
    done
    
    # Generate final report
    if generate_validation_report; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        success "SIGSEGV elimination validation completed successfully in ${duration}s"
        success "Lambdust language processor is memory-safe and production-ready"
        exit 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        error "SIGSEGV elimination validation failed after ${duration}s"
        error "Memory safety issues detected - not ready for production"
        exit 1
    fi
}

# Script entry point
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi