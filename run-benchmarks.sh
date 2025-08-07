#!/bin/bash

# Comprehensive Scheme Benchmarking Runner
# Main script to execute performance comparison across implementations

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="/benchmarks"
CONFIG_FILE="${SCRIPT_DIR}/benchmark-config.yaml"
RESULTS_DIR="${BENCHMARK_DIR}/results"
LOGS_DIR="${BENCHMARK_DIR}/logs"
TESTS_DIR="${BENCHMARK_DIR}/tests"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "${LOGS_DIR}/benchmark.log"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" | tee -a "${LOGS_DIR}/benchmark.log"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" | tee -a "${LOGS_DIR}/benchmark.log"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" | tee -a "${LOGS_DIR}/benchmark.log"
}

# Initialize benchmark environment
initialize_environment() {
    log_info "Initializing benchmark environment..."
    
    # Create necessary directories
    mkdir -p "${RESULTS_DIR}"/{raw,processed,reports}
    mkdir -p "${LOGS_DIR}"
    mkdir -p "${TESTS_DIR}"
    
    # Clear old logs
    > "${LOGS_DIR}/benchmark.log"
    
    # Check for required tools
    local required_tools=("jq" "bc" "time" "valgrind" "python3")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool '$tool' is not installed"
            exit 1
        fi
    done
    
    log_success "Environment initialized"
}

# Verify Scheme implementations
verify_implementations() {
    log_info "Verifying Scheme implementations..."
    
    local implementations_available=0
    local implementations_total=0
    
    # Check each implementation
    for impl in chez racket gambit gauche chicken mit-scheme guile cyclone; do
        implementations_total=$((implementations_total + 1))
        
        case "$impl" in
            "chez")
                if command -v scheme &> /dev/null; then
                    log_success "Chez Scheme available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "Chez Scheme not available"
                fi
                ;;
            "racket")
                if command -v racket &> /dev/null; then
                    log_success "Racket available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "Racket not available"
                fi
                ;;
            "gambit")
                if command -v gsi &> /dev/null; then
                    log_success "Gambit Scheme available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "Gambit Scheme not available"
                fi
                ;;
            "gauche")
                if command -v gosh &> /dev/null; then
                    log_success "Gauche available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "Gauche not available"
                fi
                ;;
            "chicken")
                if command -v csi &> /dev/null; then
                    log_success "Chicken Scheme available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "Chicken Scheme not available"
                fi
                ;;
            "mit-scheme")
                if command -v mit-scheme &> /dev/null; then
                    log_success "MIT/GNU Scheme available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "MIT/GNU Scheme not available"
                fi
                ;;
            "guile")
                if command -v guile &> /dev/null; then
                    log_success "Guile available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "Guile not available"
                fi
                ;;
            "cyclone")
                if command -v cyclone &> /dev/null; then
                    log_success "Cyclone Scheme available"
                    implementations_available=$((implementations_available + 1))
                else
                    log_warn "Cyclone Scheme not available"
                fi
                ;;
        esac
    done
    
    log_info "Found $implementations_available of $implementations_total Scheme implementations"
    
    if [ "$implementations_available" -eq 0 ]; then
        log_error "No Scheme implementations found"
        exit 1
    fi
}

# Build Lambdust if needed
build_lambdust() {
    log_info "Building Lambdust..."
    
    if [ ! -f "${SCRIPT_DIR}/Cargo.toml" ]; then
        log_error "Lambdust source not found in ${SCRIPT_DIR}"
        exit 1
    fi
    
    cd "${SCRIPT_DIR}"
    
    # Build in release mode
    if ! cargo build --release --features benchmarks; then
        log_error "Failed to build Lambdust"
        exit 1
    fi
    
    # Copy binary to benchmarks directory
    mkdir -p "${BENCHMARK_DIR}/lambdust/target/release"
    cp target/release/lambdust "${BENCHMARK_DIR}/lambdust/target/release/"
    
    log_success "Lambdust built successfully"
}

# Generate test files for all implementations
generate_test_files() {
    log_info "Generating test files..."
    
    python3 "${SCRIPT_DIR}/scripts/generate_tests.py" \
        --config "${CONFIG_FILE}" \
        --output-dir "${TESTS_DIR}" \
        --implementations all
        
    log_success "Test files generated"
}

# Run individual benchmark
run_single_benchmark() {
    local implementation="$1"
    local test_name="$2" 
    local test_file="$3"
    local output_file="$4"
    
    log_info "Running $test_name on $implementation..."
    
    # Create temporary script for execution
    local temp_script="${BENCHMARK_DIR}/tmp_run_${implementation}_${test_name}.sh"
    
    case "$implementation" in
        "chez")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec scheme --quiet --script \"$test_file\"" >> "$temp_script"
            ;;
        "racket")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec racket -t \"$test_file\"" >> "$temp_script"
            ;;
        "gambit")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec gsi -:d- -f \"$test_file\"" >> "$temp_script"
            ;;
        "gauche")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec gosh -b \"$test_file\"" >> "$temp_script"
            ;;
        "chicken")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec csi -script \"$test_file\"" >> "$temp_script"
            ;;
        "mit-scheme")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec mit-scheme --quiet --load \"$test_file\" --eval '(exit)'" >> "$temp_script"
            ;;
        "guile")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec guile --no-auto-compile -s \"$test_file\"" >> "$temp_script"
            ;;
        "cyclone")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec cyclone -i \"$test_file\"" >> "$temp_script"
            ;;
        "lambdust")
            echo "#!/bin/bash" > "$temp_script"
            echo "exec \"${BENCHMARK_DIR}/lambdust/target/release/lambdust\" --batch \"$test_file\"" >> "$temp_script"
            ;;
        *)
            log_error "Unknown implementation: $implementation"
            return 1
            ;;
    esac
    
    chmod +x "$temp_script"
    
    # Run with timing and memory profiling
    local start_time=$(date +%s.%N)
    
    if timeout 300 /usr/bin/time -v "$temp_script" > "${output_file}.stdout" 2> "${output_file}.stderr"; then
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc)
        
        # Extract timing and memory information
        local max_memory=$(grep "Maximum resident set size" "${output_file}.stderr" | awk '{print $6}' || echo "0")
        local user_time=$(grep "User time" "${output_file}.stderr" | awk '{print $4}' || echo "0")
        local sys_time=$(grep "System time" "${output_file}.stderr" | awk '{print $4}' || echo "0")
        local cpu_percent=$(grep "Percent of CPU" "${output_file}.stderr" | sed 's/%//' | awk '{print $7}' || echo "0")
        
        # Create JSON result
        cat > "$output_file" << EOF
{
    "implementation": "$implementation",
    "test": "$test_name", 
    "duration": $duration,
    "user_time": $user_time,
    "system_time": $sys_time,
    "max_memory_kb": $max_memory,
    "cpu_percent": "$cpu_percent",
    "success": true,
    "timestamp": "$(date -Iseconds)"
}
EOF
        
        log_success "$test_name completed on $implementation (${duration}s)"
    else
        log_error "$test_name failed on $implementation"
        cat > "$output_file" << EOF
{
    "implementation": "$implementation", 
    "test": "$test_name",
    "success": false,
    "error": "Test failed or timed out",
    "timestamp": "$(date -Iseconds)"
}
EOF
    fi
    
    # Clean up
    rm -f "$temp_script"
}

# Run all benchmarks
run_benchmarks() {
    log_info "Starting benchmark execution..."
    
    local total_tests=0
    local completed_tests=0
    
    # Get available implementations
    local implementations=()
    for impl in chez racket gambit gauche chicken mit-scheme guile cyclone lambdust; do
        case "$impl" in
            "chez") command -v scheme &> /dev/null && implementations+=("$impl") ;;
            "racket") command -v racket &> /dev/null && implementations+=("$impl") ;;
            "gambit") command -v gsi &> /dev/null && implementations+=("$impl") ;;
            "gauche") command -v gosh &> /dev/null && implementations+=("$impl") ;;
            "chicken") command -v csi &> /dev/null && implementations+=("$impl") ;;
            "mit-scheme") command -v mit-scheme &> /dev/null && implementations+=("$impl") ;;
            "guile") command -v guile &> /dev/null && implementations+=("$impl") ;;
            "cyclone") command -v cyclone &> /dev/null && implementations+=("$impl") ;;
            "lambdust") [ -f "${BENCHMARK_DIR}/lambdust/target/release/lambdust" ] && implementations+=("$impl") ;;
        esac
    done
    
    # Get test files
    local test_suites=("micro_benchmarks" "data_structure_benchmarks" "algorithm_benchmarks")
    
    # Calculate total tests
    for suite in "${test_suites[@]}"; do
        local suite_tests=$(find "${TESTS_DIR}/${suite}" -name "*.scm" -o -name "*.ldust" -o -name "*.rkt" 2>/dev/null | wc -l || echo "0")
        total_tests=$((total_tests + suite_tests * ${#implementations[@]}))
    done
    
    log_info "Running $total_tests total benchmark combinations..."
    
    # Execute benchmarks
    for suite in "${test_suites[@]}"; do
        log_info "Running $suite test suite..."
        
        for impl in "${implementations[@]}"; do
            local test_files
            case "$impl" in
                "lambdust") test_files=$(find "${TESTS_DIR}/${suite}" -name "*.ldust" 2>/dev/null || true) ;;
                "racket") test_files=$(find "${TESTS_DIR}/${suite}" -name "*.rkt" 2>/dev/null || true) ;;
                *) test_files=$(find "${TESTS_DIR}/${suite}" -name "*.scm" 2>/dev/null || true) ;;
            esac
            
            for test_file in $test_files; do
                local test_name=$(basename "$test_file" | sed 's/\.[^.]*$//')
                local output_file="${RESULTS_DIR}/raw/${impl}_${suite}_${test_name}.json"
                
                run_single_benchmark "$impl" "${suite}_${test_name}" "$test_file" "$output_file"
                completed_tests=$((completed_tests + 1))
                
                # Progress indicator
                local progress=$((completed_tests * 100 / total_tests))
                log_info "Progress: $completed_tests/$total_tests ($progress%)"
            done
        done
    done
    
    log_success "All benchmarks completed: $completed_tests tests"
}

# Process results and generate reports
process_results() {
    log_info "Processing benchmark results..."
    
    python3 "${SCRIPT_DIR}/scripts/process_results.py" \
        --input-dir "${RESULTS_DIR}/raw" \
        --output-dir "${RESULTS_DIR}/processed" \
        --config "${CONFIG_FILE}"
    
    # Generate HTML report
    python3 "${SCRIPT_DIR}/scripts/generate_report.py" \
        --input-dir "${RESULTS_DIR}/processed" \
        --output-dir "${RESULTS_DIR}/reports" \
        --format html
    
    log_success "Results processed and reports generated"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_info "Starting Scheme benchmarking suite..."
    log_info "Timestamp: $(date -Iseconds)"
    
    # Parse command line arguments
    local run_build=true
    local run_tests=true
    local skip_lambdust=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-build)
                run_build=false
                shift
                ;;
            --no-tests)
                run_tests=false
                shift
                ;;
            --skip-lambdust)
                skip_lambdust=true
                shift
                ;;
            --help)
                echo "Usage: $0 [--no-build] [--no-tests] [--skip-lambdust] [--help]"
                echo "  --no-build      Skip building Lambdust"
                echo "  --no-tests      Skip running benchmarks"
                echo "  --skip-lambdust Skip Lambdust in benchmarks"
                echo "  --help          Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Execute benchmark pipeline
    initialize_environment
    verify_implementations
    
    if [ "$run_build" = true ] && [ "$skip_lambdust" = false ]; then
        build_lambdust
    fi
    
    if [ "$run_tests" = true ]; then
        generate_test_files
        run_benchmarks
        process_results
    fi
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    log_success "Benchmark suite completed in ${total_duration} seconds"
    log_info "Results available in: ${RESULTS_DIR}/reports/"
}

# Execute main function with all arguments
main "$@"