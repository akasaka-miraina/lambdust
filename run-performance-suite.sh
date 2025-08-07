#!/bin/bash

# Master Performance Testing Suite for Lambdust
# This script coordinates native benchmarks, monitoring, and analysis
# to provide comprehensive performance insights with Docker compatibility

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Default configuration
SUITE_MODE="comprehensive"
OUTPUT_BASE_DIR="performance-results"
ENABLE_MONITORING=false
ENABLE_ANALYSIS=true
DOCKER_COMPATIBLE=true
COMPARISON_BASELINE=""

# Function to print colored output
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_section() {
    echo -e "${PURPLE}>>> $1${NC}"
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
    echo "Lambdust Master Performance Testing Suite"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -m, --mode MODE           Test mode: quick, comprehensive, development (default: comprehensive)"
    echo "  -o, --output DIR          Base output directory (default: performance-results)"
    echo "  -M, --monitor            Enable continuous performance monitoring"
    echo "  -A, --no-analysis        Disable automatic analysis"
    echo "  -D, --no-docker          Disable Docker compatibility features"
    echo "  -b, --baseline FILE       Baseline file for performance comparison"
    echo "  -r, --regression-only     Only run regression detection against baseline"
    echo "  -c, --continuous          Run continuous benchmarking (for CI/CD)"
    echo "  -h, --help               Show this help message"
    echo ""
    echo "Modes:"
    echo "  quick         - Fast benchmark run (reduced iterations)"
    echo "  comprehensive - Full benchmark suite with detailed analysis"
    echo "  development   - Developer-focused quick tests"
    echo ""
    echo "Examples:"
    echo "  $0                        # Run comprehensive benchmark suite"
    echo "  $0 -m quick              # Run quick benchmarks"
    echo "  $0 -M                    # Run with monitoring enabled"
    echo "  $0 -b baseline.json      # Compare against baseline"
    echo "  $0 -r -b baseline.json   # Only check for regressions"
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--mode)
            SUITE_MODE="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_BASE_DIR="$2"
            shift 2
            ;;
        -M|--monitor)
            ENABLE_MONITORING=true
            shift
            ;;
        -A|--no-analysis)
            ENABLE_ANALYSIS=false
            shift
            ;;
        -D|--no-docker)
            DOCKER_COMPATIBLE=false
            shift
            ;;
        -b|--baseline)
            COMPARISON_BASELINE="$2"
            shift 2
            ;;
        -r|--regression-only)
            REGRESSION_ONLY=true
            shift
            ;;
        -c|--continuous)
            CONTINUOUS_MODE=true
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

# Validate mode
if [[ ! "$SUITE_MODE" =~ ^(quick|comprehensive|development)$ ]]; then
    print_error "Invalid mode: $SUITE_MODE"
    print_info "Valid modes: quick, comprehensive, development"
    exit 1
fi

# Main execution function
main() {
    print_header "🎯 LAMBDUST MASTER PERFORMANCE TESTING SUITE"
    
    print_info "Configuration:"
    echo "  • Mode: $SUITE_MODE"
    echo "  • Output Directory: $OUTPUT_BASE_DIR"
    echo "  • Monitoring: $([ "$ENABLE_MONITORING" == true ] && echo "enabled" || echo "disabled")"
    echo "  • Analysis: $([ "$ENABLE_ANALYSIS" == true ] && echo "enabled" || echo "disabled")"
    echo "  • Docker Compatible: $([ "$DOCKER_COMPATIBLE" == true ] && echo "yes" || echo "no")"
    if [[ -n "$COMPARISON_BASELINE" ]]; then
        echo "  • Baseline: $COMPARISON_BASELINE"
    fi
    echo ""
    
    # Create output directory structure
    setup_output_directories
    
    # Check prerequisites
    check_prerequisites
    
    # Handle different execution modes
    if [[ "$REGRESSION_ONLY" == true ]]; then
        run_regression_check_only
    elif [[ "$CONTINUOUS_MODE" == true ]]; then
        run_continuous_benchmarking
    else
        run_standard_benchmark_suite
    fi
    
    print_success "🏆 Performance testing suite completed successfully!"
}

# Setup output directory structure
setup_output_directories() {
    print_section "Setting up output directories"
    
    mkdir -p "$OUTPUT_BASE_DIR"/{native,monitoring,analysis,docker-compatible}
    
    # Create docker-compatible structure if enabled
    if [[ "$DOCKER_COMPATIBLE" == true ]]; then
        mkdir -p "$OUTPUT_BASE_DIR/docker-compatible"/{results,config,logs}
        
        # Create Docker compatibility metadata
        cat > "$OUTPUT_BASE_DIR/docker-compatible/metadata.json" << EOF
{
  "suite_version": "1.0.0",
  "lambdust_version": "0.1.0",
  "native_benchmark_format": "native_v1",
  "docker_compatibility": "enabled",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "system_info": {
    "platform": "$(uname -s)",
    "architecture": "$(uname -m)",
    "cpu_cores": $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
  }
}
EOF
    fi
    
    print_success "Output directories created"
}

# Check prerequisites
check_prerequisites() {
    print_section "Checking prerequisites"
    
    # Check if we're in the right directory
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "Cargo.toml not found. Please run from Lambdust project root."
        exit 1
    fi
    
    # Check for required binaries in Cargo.toml
    if ! grep -q "native-benchmark-runner" Cargo.toml; then
        print_error "Native benchmark runner not found in Cargo.toml"
        exit 1
    fi
    
    # Check Python for analysis (optional)
    if [[ "$ENABLE_ANALYSIS" == true ]] && ! command -v python3 >/dev/null 2>&1; then
        print_warning "Python3 not found. Analysis will be limited."
        ENABLE_ANALYSIS=false
    fi
    
    # Check for monitoring binary
    if [[ "$ENABLE_MONITORING" == true ]] && ! grep -q "performance-monitor" Cargo.toml; then
        print_warning "Performance monitor not found. Monitoring disabled."
        ENABLE_MONITORING=false
    fi
    
    print_success "Prerequisites checked"
}

# Run standard benchmark suite
run_standard_benchmark_suite() {
    print_section "Running benchmark suite in $SUITE_MODE mode"
    
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    NATIVE_OUTPUT="$OUTPUT_BASE_DIR/native/lambdust_benchmark_${SUITE_MODE}_${TIMESTAMP}.json"
    
    # Build project
    print_info "Building Lambdust with benchmarks..."
    cargo build --release --features benchmarks --bin native-benchmark-runner >/dev/null 2>&1
    if [[ $? -ne 0 ]]; then
        print_error "Build failed"
        exit 1
    fi
    print_success "Build completed"
    
    # Prepare benchmark arguments
    BENCHMARK_ARGS="--output '$NATIVE_OUTPUT' --csv"
    
    if [[ "$SUITE_MODE" == "quick" || "$SUITE_MODE" == "development" ]]; then
        BENCHMARK_ARGS="$BENCHMARK_ARGS --quick"
    fi
    
    # Run native benchmarks
    print_info "Executing native performance benchmarks..."
    eval "cargo run --release --features benchmarks --bin native-benchmark-runner -- $BENCHMARK_ARGS"
    
    if [[ $? -ne 0 ]]; then
        print_error "Native benchmark execution failed"
        exit 1
    fi
    print_success "Native benchmarks completed"
    
    # Copy results to Docker-compatible location
    if [[ "$DOCKER_COMPATIBLE" == true ]]; then
        cp "$NATIVE_OUTPUT" "$OUTPUT_BASE_DIR/docker-compatible/results/"
        cp "${NATIVE_OUTPUT%.*}.csv" "$OUTPUT_BASE_DIR/docker-compatible/results/" 2>/dev/null || true
        print_success "Results copied to Docker-compatible location"
    fi
    
    # Run monitoring if enabled
    if [[ "$ENABLE_MONITORING" == true ]]; then
        run_performance_monitoring "$TIMESTAMP"
    fi
    
    # Run analysis if enabled
    if [[ "$ENABLE_ANALYSIS" == true ]]; then
        run_performance_analysis "$NATIVE_OUTPUT" "$TIMESTAMP"
    fi
    
    # Run comparison if baseline provided
    if [[ -n "$COMPARISON_BASELINE" ]]; then
        run_baseline_comparison "$NATIVE_OUTPUT" "$COMPARISON_BASELINE" "$TIMESTAMP"
    fi
    
    # Generate final report
    generate_final_report "$NATIVE_OUTPUT" "$TIMESTAMP"
}

# Run performance monitoring
run_performance_monitoring() {
    local timestamp="$1"
    print_section "Running performance monitoring"
    
    local monitoring_output="$OUTPUT_BASE_DIR/monitoring/performance_monitoring_${timestamp}.json"
    
    print_info "Taking performance snapshot..."
    cargo run --release --features benchmarks --bin performance-monitor -- \
        --output "$monitoring_output" >/dev/null 2>&1
    
    if [[ $? -eq 0 ]]; then
        print_success "Performance monitoring completed"
    else
        print_warning "Performance monitoring failed"
    fi
}

# Run performance analysis
run_performance_analysis() {
    local results_file="$1"
    local timestamp="$2"
    
    print_section "Running performance analysis"
    
    local analysis_output="$OUTPUT_BASE_DIR/analysis/analysis_report_${timestamp}.txt"
    local analysis_csv="$OUTPUT_BASE_DIR/analysis/analysis_data_${timestamp}.csv"
    
    print_info "Generating analysis report..."
    python3 analyze-performance.py --file "$results_file" --output "$analysis_output" --export "$analysis_csv"
    
    if [[ $? -eq 0 ]]; then
        print_success "Performance analysis completed"
        
        # Copy analysis to Docker-compatible location
        if [[ "$DOCKER_COMPATIBLE" == true ]]; then
            cp "$analysis_output" "$OUTPUT_BASE_DIR/docker-compatible/results/"
            cp "$analysis_csv" "$OUTPUT_BASE_DIR/docker-compatible/results/"
        fi
    else
        print_warning "Performance analysis failed"
    fi
}

# Run baseline comparison
run_baseline_comparison() {
    local current_file="$1"
    local baseline_file="$2"
    local timestamp="$3"
    
    print_section "Running baseline comparison"
    
    if [[ ! -f "$baseline_file" ]]; then
        print_error "Baseline file not found: $baseline_file"
        return 1
    fi
    
    local comparison_output="$OUTPUT_BASE_DIR/analysis/comparison_report_${timestamp}.txt"
    local comparison_csv="$OUTPUT_BASE_DIR/analysis/comparison_data_${timestamp}.csv"
    
    print_info "Comparing against baseline: $baseline_file"
    python3 analyze-performance.py --compare "$baseline_file" "$current_file" \
        --output "$comparison_output" --export "$comparison_csv"
    
    if [[ $? -eq 0 ]]; then
        print_success "Baseline comparison completed"
        
        # Check for regressions
        if grep -q "PERFORMANCE REGRESSIONS:" "$comparison_output"; then
            print_warning "Performance regressions detected! Check the comparison report."
            
            # Extract and display major regressions
            echo ""
            print_warning "Major regressions found:"
            grep -A 5 "PERFORMANCE REGRESSIONS:" "$comparison_output" | tail -n +2 | head -n 3
        fi
        
        # Copy to Docker-compatible location
        if [[ "$DOCKER_COMPATIBLE" == true ]]; then
            cp "$comparison_output" "$OUTPUT_BASE_DIR/docker-compatible/results/"
            cp "$comparison_csv" "$OUTPUT_BASE_DIR/docker-compatible/results/"
        fi
    else
        print_warning "Baseline comparison failed"
    fi
}

# Run regression check only
run_regression_check_only() {
    print_section "Running regression check against baseline"
    
    if [[ -z "$COMPARISON_BASELINE" ]]; then
        print_error "Baseline file required for regression check"
        exit 1
    fi
    
    # Run quick benchmark for comparison
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    TEMP_OUTPUT="$OUTPUT_BASE_DIR/native/temp_regression_check_${TIMESTAMP}.json"
    
    print_info "Running quick benchmark for regression check..."
    cargo build --release --features benchmarks --bin native-benchmark-runner >/dev/null 2>&1
    cargo run --release --features benchmarks --bin native-benchmark-runner -- \
        --quick --output "$TEMP_OUTPUT" >/dev/null 2>&1
    
    # Run comparison
    run_baseline_comparison "$TEMP_OUTPUT" "$COMPARISON_BASELINE" "$TIMESTAMP"
    
    # Clean up temp file
    rm -f "$TEMP_OUTPUT"
}

# Run continuous benchmarking
run_continuous_benchmarking() {
    print_section "Starting continuous benchmarking mode"
    
    print_info "Continuous benchmarking will run every 30 minutes"
    print_info "Press Ctrl+C to stop"
    
    while true; do
        TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
        print_info "Running benchmark cycle at $(date)"
        
        # Run quick benchmark
        OUTPUT_FILE="$OUTPUT_BASE_DIR/native/continuous_${TIMESTAMP}.json"
        cargo run --release --features benchmarks --bin native-benchmark-runner -- \
            --quick --output "$OUTPUT_FILE" >/dev/null 2>&1
        
        # Run analysis if enabled
        if [[ "$ENABLE_ANALYSIS" == true ]]; then
            python3 analyze-performance.py --file "$OUTPUT_FILE" \
                --output "$OUTPUT_BASE_DIR/analysis/continuous_analysis_${TIMESTAMP}.txt" >/dev/null 2>&1
        fi
        
        print_success "Benchmark cycle completed: $OUTPUT_FILE"
        
        # Wait 30 minutes
        sleep 1800
    done
}

# Generate final comprehensive report
generate_final_report() {
    local results_file="$1"
    local timestamp="$2"
    
    print_section "Generating final comprehensive report"
    
    local final_report="$OUTPUT_BASE_DIR/PERFORMANCE_REPORT_${timestamp}.md"
    
    cat > "$final_report" << EOF
# Lambdust Performance Testing Report

**Generated:** $(date)  
**Mode:** $SUITE_MODE  
**Suite Version:** 1.0.0  

## Summary

This report contains comprehensive performance analysis for Lambdust, including:

- Native benchmark results
- Performance monitoring data  
- Statistical analysis
- Optimization recommendations
$([ -n "$COMPARISON_BASELINE" ] && echo "- Baseline comparison analysis")

## Files Generated

### Native Benchmarks
- JSON Results: \`$(basename "$results_file")\`
- CSV Data: \`$(basename "${results_file%.*}").csv\`

### Analysis Reports
$([ "$ENABLE_ANALYSIS" == true ] && echo "- Analysis Report: \`analysis_report_${timestamp}.txt\`")
$([ "$ENABLE_ANALYSIS" == true ] && echo "- Analysis Data: \`analysis_data_${timestamp}.csv\`")

### Monitoring Data
$([ "$ENABLE_MONITORING" == true ] && echo "- Monitoring Snapshot: \`performance_monitoring_${timestamp}.json\`")

### Comparison Results
$([ -n "$COMPARISON_BASELINE" ] && echo "- Comparison Report: \`comparison_report_${timestamp}.txt\`")
$([ -n "$COMPARISON_BASELINE" ] && echo "- Comparison Data: \`comparison_data_${timestamp}.csv\`")

## Docker Compatibility

$([ "$DOCKER_COMPATIBLE" == true ] && echo "✅ Results are Docker-compatible and saved in \`docker-compatible/\` directory" || echo "❌ Docker compatibility disabled")

## Usage Instructions

### Quick Analysis
\`\`\`bash
# View benchmark results
cat $OUTPUT_BASE_DIR/analysis/analysis_report_${timestamp}.txt

# Import CSV data into spreadsheet
open ${results_file%.*}.csv
\`\`\`

### Integration with Docker Benchmarks
$([ "$DOCKER_COMPATIBLE" == true ] && echo "\`\`\`bash
# Copy results to Docker benchmark framework
cp $OUTPUT_BASE_DIR/docker-compatible/results/* /path/to/docker/benchmarks/

# Run Docker comparison
docker-compose -f docker-compose.benchmarks.yml up
\`\`\`")

### Continuous Monitoring
\`\`\`bash
# Start continuous performance monitoring
./run-performance-suite.sh -c

# Check for regressions
./run-performance-suite.sh -r -b baseline.json
\`\`\`

## Next Steps

1. Review the detailed analysis reports
2. Focus on categories marked as "Needs Improvement"
3. Implement recommended optimizations
4. Set up regular performance regression testing
$([ "$DOCKER_COMPATIBLE" == true ] && echo "5. Integrate results with Docker benchmark framework")

---

*Generated by Lambdust Master Performance Testing Suite v1.0.0*
EOF
    
    print_success "Final report generated: $final_report"
    
    # Display quick summary
    print_header "🏁 PERFORMANCE TESTING COMPLETE"
    
    print_info "Results Summary:"
    if [[ -f "$results_file" ]] && command -v jq >/dev/null 2>&1; then
        local overall_score=$(jq -r '.overall_summary.overall_performance_score' "$results_file" 2>/dev/null || echo "N/A")
        local total_tests=$(jq -r '.overall_summary.total_tests' "$results_file" 2>/dev/null || echo "N/A")
        local exec_time=$(jq -r '.overall_summary.total_execution_time_seconds' "$results_file" 2>/dev/null || echo "N/A")
        
        echo "  • Overall Performance Score: ${overall_score}/100"
        echo "  • Total Tests: ${total_tests}"
        echo "  • Execution Time: ${exec_time}s"
        echo "  • Mode: $SUITE_MODE"
    fi
    
    echo ""
    print_info "Generated Files:"
    echo "  • Comprehensive Report: $final_report"
    echo "  • JSON Results: $results_file"
    echo "  • CSV Data: ${results_file%.*}.csv"
    
    if [[ "$DOCKER_COMPATIBLE" == true ]]; then
        echo "  • Docker-compatible results: $OUTPUT_BASE_DIR/docker-compatible/"
    fi
    
    echo ""
    print_success "All performance testing artifacts ready for analysis and integration! 🎯"
}

# Handle interruption gracefully
trap 'print_error "Performance testing interrupted by user"; exit 1' INT

# Run the main function
main "$@"