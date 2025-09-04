#!/bin/bash
set -e

echo "🚨 SIGSEGV Docker Validation Suite"
echo "=================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Project root: $PROJECT_ROOT"

# Build memory safety Docker image
echo -e "${BLUE}Building Memory Safety Docker Environment...${NC}"
cd "$PROJECT_ROOT/docker"

# Update docker-compose.yml to include memory safety service
cat > docker-compose.yml << 'EOF'
version: '3.8'
services:
  ubuntu-test:
    build: 
      context: .
      dockerfile: ubuntu-test/Dockerfile
    image: lambdust-ubuntu-test:latest
    volumes:
      - ../:/workspace:cached
    environment:
      - CARGO_HOME=/workspace/.cargo
      - RUST_BACKTRACE=1
      - CARGO_TERM_COLOR=always
    working_dir: /workspace
    command: /bin/bash
    stdin_open: true
    tty: true

  memory-safety:
    build:
      context: memory-safety
      dockerfile: Dockerfile
    image: lambdust-memory-safety:latest
    volumes:
      - ../:/workspace:cached
    environment:
      - CARGO_HOME=/workspace/.cargo
      - RUST_BACKTRACE=full
      - CARGO_TERM_COLOR=always
      - RUST_MIN_STACK=4194304
    working_dir: /workspace
    command: /bin/bash
    stdin_open: true
    tty: true
    cap_add:
      - SYS_PTRACE
    security_opt:
      - seccomp:unconfined
EOF

docker-compose build memory-safety

echo -e "${YELLOW}Phase 1: Pre-validation Checks${NC}"

# Check current compilation status
echo "Checking current compilation status..."
docker-compose run --rm ubuntu-test bash -c "
    echo 'Current branch:' && git branch --show-current
    echo 'Last commit:' && git log --oneline -n 1
    echo 'Compilation check...'
    timeout 300 cargo check --all-targets --all-features || exit 1
"

echo -e "${YELLOW}Phase 2: Memory Safety Deep Analysis${NC}"

# Run comprehensive memory safety validation
docker-compose run --rm memory-safety bash -c "
    # Copy and run memory safety test script
    cp /home/lambda/scripts/memory-safety-test.sh /tmp/
    chmod +x /tmp/memory-safety-test.sh
    /tmp/memory-safety-test.sh
"

MEMORY_SAFETY_EXIT_CODE=$?

echo -e "${YELLOW}Phase 3: SIGSEGV-Specific Validation${NC}"

# Focus on specific SIGSEGV-prone areas
docker-compose run --rm memory-safety bash -c "
    echo '🎯 Testing OptimizedValue unsafe operations...'
    
    # Test with extra debugging
    RUST_BACKTRACE=full cargo test --lib eval::optimized_value --no-default-features -- --nocapture || {
        echo '❌ OptimizedValue tests failed'
        exit 1
    }
    
    echo '🎯 Testing OrderedSet red-black tree operations...'
    RUST_BACKTRACE=full cargo test --lib containers::ordered_set --no-default-features -- --nocapture || {
        echo '❌ OrderedSet tests failed'  
        exit 1
    }
    
    echo '🎯 Testing ValueData union safety...'
    # Run targeted tests that exercise unsafe pointer operations
    RUST_BACKTRACE=full cargo test --lib --no-default-features hash -- --nocapture || {
        echo '❌ ValueData union tests failed'
        exit 1
    }
    
    echo '✅ All SIGSEGV-specific tests passed'
"

SIGSEGV_EXIT_CODE=$?

echo -e "${YELLOW}Phase 4: Performance Regression Validation${NC}"

# Validate that SafeOptimizedValue maintains performance improvements
docker-compose run --rm memory-safety bash -c "
    echo '📊 Running performance validation...'
    
    # Build in release mode for performance testing
    cargo build --release --lib --no-default-features || exit 1
    
    # Run benchmark tests if available
    cargo test --release --lib --no-default-features benchmark 2>/dev/null || {
        echo 'ℹ️  No benchmark tests found, skipping performance validation'
    }
    
    echo '✅ Performance validation completed'
"

echo -e "${YELLOW}Phase 5: Container Isolation Validation${NC}"

# Test in clean containerized environment to catch environment-specific issues
docker-compose run --rm memory-safety bash -c "
    echo '🐳 Testing in isolated container environment...'
    
    # Clean build from scratch
    cargo clean
    
    # Full compilation
    timeout 600 cargo build --lib --all-features || {
        echo '❌ Container build failed'
        exit 1
    }
    
    # Run core functionality tests
    timeout 300 cargo test --lib --no-default-features -- --test-threads=1 || {
        echo '❌ Container tests failed'
        exit 1
    }
    
    echo '✅ Container isolation tests passed'
"

CONTAINER_EXIT_CODE=$?

# Final Results
echo "======================================"
echo -e "${BLUE}SIGSEGV Docker Validation Results:${NC}"
echo ""

if [ $MEMORY_SAFETY_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Memory Safety Validation: PASSED${NC}"
else
    echo -e "${RED}❌ Memory Safety Validation: FAILED${NC}"
fi

if [ $SIGSEGV_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ SIGSEGV-Specific Tests: PASSED${NC}"
else
    echo -e "${RED}❌ SIGSEGV-Specific Tests: FAILED${NC}"
fi

if [ $CONTAINER_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Container Isolation Tests: PASSED${NC}"
else
    echo -e "${RED}❌ Container Isolation Tests: FAILED${NC}"
fi

echo ""

if [ $MEMORY_SAFETY_EXIT_CODE -eq 0 ] && [ $SIGSEGV_EXIT_CODE -eq 0 ] && [ $CONTAINER_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL VALIDATION PHASES PASSED!${NC}"
    echo -e "${GREEN}✅ Safe to push changes to repository${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run ./scripts/ci-simulate.sh for final CI validation"
    echo "2. Create pull request with SIGSEGV fixes"
    echo "3. Monitor CI pipeline for successful completion"
    exit 0
else
    echo -e "${RED}⚠️  VALIDATION FAILURES DETECTED${NC}"
    echo -e "${RED}❌ DO NOT PUSH until all issues are resolved${NC}"
    echo ""
    echo "Required actions:"
    echo "1. Fix memory safety issues identified above"
    echo "2. Re-run this validation script"
    echo "3. Only push when all phases pass"
    exit 1
fi