#!/bin/bash

# Native Benchmark Runner Script for Lambdust
# This script provides an easy way to run comprehensive performance benchmarks
# without requiring Docker or external dependencies.

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
QUICK_MODE=false
OUTPUT_DIR="benchmark-results"
GENERATE_CSV=true
VERBOSE=false

# Function to print colored output
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -q, --quick           Run benchmarks in quick mode (fewer iterations)"
    echo "  -o, --output DIR      Output directory for benchmark results (default: benchmark-results)"
    echo "  -n, --no-csv          Don't generate CSV output"
    echo "  -v, --verbose         Enable verbose output"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    Run full benchmark suite"
    echo "  $0 --quick           Run quick benchmarks"
    echo "  $0 -o /tmp/results   Save results to /tmp/results"
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -q|--quick)
            QUICK_MODE=true
            shift
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -n|--no-csv)
            GENERATE_CSV=false
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    print_header "Lambdust Native Performance Benchmarking Suite"
    
    print_info "Initializing benchmark environment..."
    
    # Check if we're in the right directory
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "Cargo.toml not found. Please run this script from the Lambdust project root."
        exit 1
    fi
    
    # Verify that the binary exists in Cargo.toml
    if ! grep -q "native-benchmark-runner" Cargo.toml; then
        print_error "Native benchmark runner binary not found in Cargo.toml"
        exit 1
    fi
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    print_info "Results will be saved to: $OUTPUT_DIR"
    
    # Generate timestamp for this run
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    
    # Build the project with benchmarks feature
    print_info "Building Lambdust with benchmarks feature..."
    if [[ "$VERBOSE" == true ]]; then
        cargo build --release --features benchmarks --bin native-benchmark-runner
    else
        cargo build --release --features benchmarks --bin native-benchmark-runner > /dev/null 2>&1
    fi
    
    if [[ $? -ne 0 ]]; then
        print_error "Failed to build the benchmark runner"
        exit 1
    fi
    
    print_success "Build completed successfully"
    
    # Prepare benchmark arguments
    BENCHMARK_ARGS=""
    OUTPUT_FILE="${OUTPUT_DIR}/lambdust_benchmark_results_${TIMESTAMP}.json"
    
    if [[ "$QUICK_MODE" == true ]]; then
        BENCHMARK_ARGS="$BENCHMARK_ARGS --quick"
        print_info "Running in quick mode (reduced iterations for faster execution)"
    else
        print_info "Running full benchmark suite (this may take several minutes)"
    fi
    
    BENCHMARK_ARGS="$BENCHMARK_ARGS --output '$OUTPUT_FILE'"
    
    if [[ "$GENERATE_CSV" == true ]]; then
        BENCHMARK_ARGS="$BENCHMARK_ARGS --csv"
    fi
    
    # Run the benchmarks
    print_header "Executing Performance Benchmarks"
    
    print_info "Starting benchmark execution..."
    print_info "This will test: arithmetic, lists, recursion, memory allocation, function calls"
    
    # Display system information
    print_info "System Information:"
    echo "  • CPU Cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 'Unknown')"
    echo "  • Platform: $(uname -s)"
    echo "  • Architecture: $(uname -m)"
    echo "  • Rust Version: $(rustc --version)"
    echo ""
    
    # Run the actual benchmark
    if [[ "$VERBOSE" == true ]]; then
        eval "cargo run --release --features benchmarks --bin native-benchmark-runner -- $BENCHMARK_ARGS"
    else
        eval "cargo run --release --features benchmarks --bin native-benchmark-runner -- $BENCHMARK_ARGS" 2>/dev/null
    fi
    
    BENCHMARK_EXIT_CODE=$?
    
    if [[ $BENCHMARK_EXIT_CODE -ne 0 ]]; then
        print_error "Benchmark execution failed with exit code: $BENCHMARK_EXIT_CODE"
        exit $BENCHMARK_EXIT_CODE
    fi
    
    print_success "Benchmark execution completed successfully"
    
    # Display results summary
    print_header "Benchmark Results Summary"
    
    if [[ -f "$OUTPUT_FILE" ]]; then
        print_success "Results saved to: $OUTPUT_FILE"
        
        # Extract key metrics using jq if available
        if command -v jq >/dev/null 2>&1; then
            print_info "Performance Summary:"
            
            # Overall performance score
            OVERALL_SCORE=$(jq -r '.overall_summary.overall_performance_score' "$OUTPUT_FILE" 2>/dev/null || echo "N/A")
            echo "  • Overall Performance Score: ${OVERALL_SCORE}/100"
            
            # Total tests
            TOTAL_TESTS=$(jq -r '.overall_summary.total_tests' "$OUTPUT_FILE" 2>/dev/null || echo "N/A")
            echo "  • Total Tests Executed: ${TOTAL_TESTS}"
            
            # Execution time
            EXEC_TIME=$(jq -r '.overall_summary.total_execution_time_seconds' "$OUTPUT_FILE" 2>/dev/null || echo "N/A")
            echo "  • Total Execution Time: ${EXEC_TIME}s"
            
            # Category performance
            echo ""
            print_info "Category Performance:"
            jq -r '.categories[] | "  • \(.name): \(.summary.avg_ops_per_second | floor) ops/sec (\(.summary.performance_grade))"' "$OUTPUT_FILE" 2>/dev/null || echo "  Unable to parse category details"
            
            # Top recommendations
            echo ""
            print_info "Top Optimization Recommendations:"
            jq -r '.performance_recommendations[:3][] | "  • \(.)"' "$OUTPUT_FILE" 2>/dev/null || echo "  Unable to parse recommendations"
            
        else
            print_warning "jq not found. Install jq for detailed result parsing."
            print_info "Raw results are available in JSON format: $OUTPUT_FILE"
        fi
        
        # Check for CSV output
        CSV_FILE="${OUTPUT_FILE%.*}.csv"
        if [[ -f "$CSV_FILE" ]]; then
            print_success "CSV results saved to: $CSV_FILE"
        fi
        
    else
        print_error "Output file not found: $OUTPUT_FILE"
        exit 1
    fi
    
    # Additional analysis and recommendations
    print_header "Next Steps and Integration"
    
    print_info "Integration with Docker Benchmarks:"
    echo "  • The generated JSON/CSV files are compatible with Docker benchmark framework"
    echo "  • Results can be compared with other Scheme implementations"
    echo "  • Use the CSV file for spreadsheet analysis and visualization"
    echo ""
    
    print_info "Performance Analysis:"
    echo "  • Review the JSON file for detailed statistical metrics"
    echo "  • Focus on categories marked as 'Needs Improvement'"
    echo "  • Consider the optimization recommendations for targeted improvements"
    echo ""
    
    print_info "Running Regular Benchmarks:"
    echo "  • Use --quick for development cycle testing"
    echo "  • Run full benchmarks before releases"
    echo "  • Track performance trends over time using the JSON outputs"
    echo ""
    
    print_success "Native benchmarking complete! 🎯"
    
    # Provide final summary
    if [[ "$QUICK_MODE" == true ]]; then
        print_info "Quick mode completed. For comprehensive analysis, run without --quick flag."
    else
        print_info "Full benchmark suite completed. Results ready for analysis and comparison."
    fi
}

# Trap to handle interruption
trap 'print_error "Benchmark interrupted by user"; exit 1' INT

# Run the main function
main "$@"