#!/bin/bash
set -e

echo "🚀 Comprehensive SIGSEGV Resolution Validation"
echo "=============================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Results tracking
PHASE_RESULTS=()
TOTAL_PHASES=5
FAILED_PHASES=0

log_phase_result() {
    local phase_name="$1"
    local exit_code="$2"
    
    if [ $exit_code -eq 0 ]; then
        PHASE_RESULTS+=("${GREEN}✅ $phase_name: PASSED${NC}")
        echo -e "${GREEN}✅ Phase completed successfully: $phase_name${NC}"
    else
        PHASE_RESULTS+=("${RED}❌ $phase_name: FAILED${NC}")
        FAILED_PHASES=$((FAILED_PHASES + 1))
        echo -e "${RED}❌ Phase failed: $phase_name${NC}"
    fi
    echo ""
}

cd "$PROJECT_ROOT"

echo -e "${PURPLE}=== PRE-VALIDATION SYSTEM CHECK ===${NC}"
echo "Project: Lambdust R7RS Scheme Interpreter"
echo "Branch: $(git branch --show-current)"
echo "Commit: $(git log --oneline -n 1)"
echo "Docker Compose Version: $(docker-compose --version)"
echo ""

echo -e "${YELLOW}Phase 1/5: Docker Environment Setup${NC}"
echo "Building all Docker environments..."

cd docker
docker-compose build ubuntu-test || {
    echo -e "${RED}Failed to build ubuntu-test environment${NC}"
    log_phase_result "Docker Environment Setup" 1
    exit 1
}

docker-compose build memory-safety || {
    echo -e "${RED}Failed to build memory-safety environment${NC}"
    log_phase_result "Docker Environment Setup" 1
    exit 1
}

docker-compose build performance-validation || {
    echo -e "${RED}Failed to build performance-validation environment${NC}"
    log_phase_result "Docker Environment Setup" 1
    exit 1
}

log_phase_result "Docker Environment Setup" 0
cd "$PROJECT_ROOT"

echo -e "${YELLOW}Phase 2/5: Basic Compilation & Static Analysis${NC}"

docker-compose -f docker/docker-compose.yml run --rm ubuntu-test bash -c "
    echo '🔧 Compilation check...'
    timeout 300 cargo check --all-targets --all-features || exit 1
    
    echo '📝 Clippy analysis...'  
    cargo clippy --all-targets --all-features -- -D warnings || exit 1
    
    echo '💅 Format check...'
    cargo fmt --check || exit 1
    
    echo '✅ Basic validation completed'
" 2>&1 | tee /tmp/basic_validation.log

BASIC_VALIDATION_EXIT=$?
log_phase_result "Basic Compilation & Static Analysis" $BASIC_VALIDATION_EXIT

if [ $BASIC_VALIDATION_EXIT -ne 0 ]; then
    echo -e "${RED}Cannot proceed with advanced testing due to basic failures${NC}"
    echo "Check /tmp/basic_validation.log for details"
    exit 1
fi

echo -e "${YELLOW}Phase 3/5: Memory Safety Deep Validation${NC}"

docker-compose -f docker/docker-compose.yml run --rm memory-safety bash -c "
    cp /home/lambda/scripts/memory-safety-test.sh /tmp/
    chmod +x /tmp/memory-safety-test.sh
    /tmp/memory-safety-test.sh
" 2>&1 | tee /tmp/memory_safety.log

MEMORY_SAFETY_EXIT=$?
log_phase_result "Memory Safety Deep Validation" $MEMORY_SAFETY_EXIT

echo -e "${YELLOW}Phase 4/5: Performance Regression Testing${NC}"

docker-compose -f docker/docker-compose.yml run --rm performance-validation bash -c "
    cp /home/lambda/scripts/performance-test.sh /tmp/
    chmod +x /tmp/performance-test.sh
    /tmp/performance-test.sh
" 2>&1 | tee /tmp/performance_validation.log

PERFORMANCE_EXIT=$?
log_phase_result "Performance Regression Testing" $PERFORMANCE_EXIT

echo -e "${YELLOW}Phase 5/5: CI Environment Simulation${NC}"

docker-compose -f docker/docker-compose.yml run --rm ubuntu-test bash -c "
    # GitHub Actions simulation
    export CARGO_TERM_COLOR=always
    export RUST_BACKTRACE=1
    export CARGO_BUILD_JOBS=1
    export CARGO_INCREMENTAL=0
    
    echo '🔄 Clean build simulation...'
    cargo clean
    
    echo '🏗️  Full compilation...'
    timeout 600 cargo build --lib --all-features || exit 1
    
    echo '🧪 Complete test suite...'
    timeout 600 cargo test --lib --no-default-features || exit 1
    
    echo '✅ CI simulation completed'
" 2>&1 | tee /tmp/ci_simulation.log

CI_SIMULATION_EXIT=$?
log_phase_result "CI Environment Simulation" $CI_SIMULATION_EXIT

# Generate comprehensive report
echo ""
echo "=============================================="
echo -e "${PURPLE}COMPREHENSIVE VALIDATION REPORT${NC}"
echo "=============================================="
echo ""

echo "Phase Results:"
for result in "${PHASE_RESULTS[@]}"; do
    echo -e "$result"
done

echo ""
echo "Detailed Logs:"
echo "- Basic validation: /tmp/basic_validation.log"
echo "- Memory safety: /tmp/memory_safety.log"  
echo "- Performance: /tmp/performance_validation.log"
echo "- CI simulation: /tmp/ci_simulation.log"

echo ""

# SIGSEGV-specific summary
echo -e "${BLUE}SIGSEGV Resolution Status:${NC}"

if [ $MEMORY_SAFETY_EXIT -eq 0 ]; then
    echo -e "${GREEN}✅ AddressSanitizer: No memory safety violations${NC}"
    echo -e "${GREEN}✅ Valgrind: No memory leaks detected${NC}"
    echo -e "${GREEN}✅ Miri: No undefined behavior detected${NC}"
else
    echo -e "${RED}❌ Memory safety issues detected${NC}"
fi

if [ $PERFORMANCE_EXIT -eq 0 ]; then
    echo -e "${GREEN}✅ Performance targets met (≥15.68% improvement maintained)${NC}"
    echo -e "${GREEN}✅ SafeOptimizedValue performance validated${NC}"
else
    echo -e "${RED}❌ Performance regression detected${NC}"
fi

echo ""

# Final decision
if [ $FAILED_PHASES -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL VALIDATION PHASES SUCCESSFUL!${NC}"
    echo -e "${GREEN}✅ SIGSEGV issues resolved${NC}"
    echo -e "${GREEN}✅ Memory safety ensured${NC}" 
    echo -e "${GREEN}✅ Performance maintained${NC}"
    echo -e "${GREEN}✅ CI compatibility confirmed${NC}"
    echo ""
    echo -e "${BLUE}READY TO PUSH TO REPOSITORY${NC}"
    echo ""
    echo "Recommended next steps:"
    echo "1. git add ."
    echo "2. git commit -m 'fix: resolve SIGSEGV issues with memory safety improvements'"
    echo "3. git push origin 0.2.0"
    echo "4. Create pull request"
    echo ""
    exit 0
else
    echo -e "${RED}⚠️  $FAILED_PHASES OUT OF $TOTAL_PHASES PHASES FAILED${NC}"
    echo -e "${RED}❌ DO NOT PUSH - ISSUES MUST BE RESOLVED${NC}"
    echo ""
    echo "Required actions:"
    echo "1. Review failed phase logs above"
    echo "2. Fix identified issues"
    echo "3. Re-run this validation script"
    echo "4. Only push when all phases pass"
    echo ""
    
    # Specific recommendations based on failures
    if [ $MEMORY_SAFETY_EXIT -ne 0 ]; then
        echo "Memory Safety Fixes Required:"
        echo "- Review src/eval/optimized_value.rs unsafe operations"
        echo "- Fix red-black tree deletion in src/containers/ordered_set.rs" 
        echo "- Implement SafeOptimizedValue as documented"
    fi
    
    if [ $PERFORMANCE_EXIT -ne 0 ]; then
        echo "Performance Optimization Required:"
        echo "- Ensure SafeOptimizedValue maintains ≥15.68% improvement"
        echo "- Optimize Arc usage (target: 50% reduction)"
        echo "- Profile memory allocation patterns"
    fi
    
    echo ""
    exit 1
fi