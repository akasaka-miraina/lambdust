#!/bin/bash

# Comprehensive Scheme Benchmarking Automation Script
# Main entry point for the complete benchmarking pipeline

set -euo pipefail

# Colors and formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_NAME="lambdust-benchmarks"
DOCKER_IMAGE="lambdust/benchmarks:latest"
COMPOSE_FILE="docker-compose.benchmarks.yml"

# Default options
BUILD_IMAGE=true
RUN_BENCHMARKS=true
GENERATE_REPORTS=true
START_WEB_SERVER=false
CLEANUP_AFTER=false
VERBOSE=false
PARALLEL_BUILDS=true

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

log_header() {
    echo
    echo -e "${BOLD}${BLUE}================================${NC}"
    echo -e "${BOLD}${BLUE} $*${NC}"
    echo -e "${BOLD}${BLUE}================================${NC}"
    echo
}

# Help function
show_help() {
    cat << EOF
${BOLD}Lambdust Scheme Benchmarking Suite${NC}

Comprehensive performance comparison between Lambdust and major Scheme implementations.

${BOLD}USAGE:${NC}
    $0 [OPTIONS]

${BOLD}OPTIONS:${NC}
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    --no-build              Skip Docker image build
    --no-benchmarks         Skip running benchmarks
    --no-reports            Skip report generation
    --web                   Start web server for reports
    --cleanup               Clean up containers and volumes after completion
    --quick                 Quick benchmarking (reduced iterations)
    --implementations LIST  Comma-separated list of implementations to test
    --tests LIST           Comma-separated list of test suites to run

${BOLD}EXAMPLES:${NC}
    # Run full benchmark suite
    $0

    # Quick benchmark with web server
    $0 --quick --web

    # Test only specific implementations
    $0 --implementations "lambdust,chez,racket"

    # Skip building and run reports only
    $0 --no-build --no-benchmarks --web

    # Verbose mode with cleanup
    $0 --verbose --cleanup

${BOLD}DOCKER PROFILES:${NC}
    default     - Run benchmarks only
    analysis    - Include results analysis
    web         - Include web server for reports

${BOLD}OUTPUT:${NC}
    Results will be stored in Docker volumes and can be accessed via:
    - docker compose -f $COMPOSE_FILE exec scheme-benchmarks ls /benchmarks/results
    - Web interface at http://localhost:8080 (if --web is used)

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            --no-build)
                BUILD_IMAGE=false
                shift
                ;;
            --no-benchmarks)
                RUN_BENCHMARKS=false
                shift
                ;;
            --no-reports)
                GENERATE_REPORTS=false
                shift
                ;;
            --web)
                START_WEB_SERVER=true
                shift
                ;;
            --cleanup)
                CLEANUP_AFTER=true
                shift
                ;;
            --quick)
                export BENCHMARK_ITERATIONS=3
                export BENCHMARK_WARMUP=1
                shift
                ;;
            --implementations)
                export BENCHMARK_IMPLEMENTATIONS="$2"
                shift 2
                ;;
            --tests)
                export BENCHMARK_TESTS="$2"
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check for Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is required but not installed"
        exit 1
    fi
    
    # Check for Docker Compose (v2 uses 'docker compose' instead of 'docker compose')
    if ! docker compose version &> /dev/null; then
        log_error "Docker Compose is required but not available"
        exit 1
    fi
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check available disk space (need at least 10GB)
    available_space=$(df "$SCRIPT_DIR" | awk 'NR==2 {print $4}')
    required_space=$((10 * 1024 * 1024))  # 10GB in KB
    
    if [[ $available_space -lt $required_space ]]; then
        log_warn "Available disk space may be insufficient ($(($available_space / 1024 / 1024))GB available, 10GB recommended)"
    fi
    
    log_success "Prerequisites check passed"
}

# Build Docker image
build_image() {
    if [[ "$BUILD_IMAGE" != true ]]; then
        log "Skipping Docker image build"
        return
    fi
    
    log_header "Building Docker Image"
    
    local build_args=""
    if [[ "$VERBOSE" == true ]]; then
        build_args="--progress=plain"
    fi
    
    if [[ "$PARALLEL_BUILDS" == true ]]; then
        build_args="$build_args --parallel"
    fi
    
    log "Building Docker image: $DOCKER_IMAGE"
    log "This may take 20-30 minutes for first build..."
    
    if docker compose -f "$COMPOSE_FILE" build $build_args; then
        log_success "Docker image built successfully"
    else
        log_error "Failed to build Docker image"
        exit 1
    fi
}

# Run benchmarks
run_benchmarks() {
    if [[ "$RUN_BENCHMARKS" != true ]]; then
        log "Skipping benchmark execution"
        return
    fi
    
    log_header "Running Benchmarks"
    
    # Start benchmark container
    log "Starting benchmark container..."
    
    local compose_profiles="default"
    local compose_args=""
    
    if [[ "$VERBOSE" == true ]]; then
        compose_args="-f"
    fi
    
    # Run the benchmarking container
    if docker compose -f "$COMPOSE_FILE" up $compose_args scheme-benchmarks; then
        log_success "Benchmarks completed successfully"
    else
        log_error "Benchmark execution failed"
        
        # Show container logs for debugging
        log "Container logs:"
        docker compose -f "$COMPOSE_FILE" logs scheme-benchmarks
        
        exit 1
    fi
}

# Generate reports
generate_reports() {
    if [[ "$GENERATE_REPORTS" != true ]]; then
        log "Skipping report generation"
        return
    fi
    
    log_header "Generating Reports"
    
    log "Processing benchmark results and generating reports..."
    
    # Run analysis container
    if docker compose -f "$COMPOSE_FILE" --profile analysis up results-analyzer; then
        log_success "Reports generated successfully"
    else
        log_error "Report generation failed"
        exit 1
    fi
}

# Start web server
start_web_server() {
    if [[ "$START_WEB_SERVER" != true ]]; then
        return
    fi
    
    log_header "Starting Web Server"
    
    log "Starting web server to view reports..."
    
    # Start web server in background
    docker compose -f "$COMPOSE_FILE" --profile web up -d report-server
    
    if [[ $? -eq 0 ]]; then
        log_success "Web server started at http://localhost:8080"
        log "Press Ctrl+C to stop the web server"
        
        # Wait for user interrupt
        trap 'echo && log "Stopping web server..."; docker compose -f "$COMPOSE_FILE" --profile web down; exit 0' INT
        
        # Show server logs
        docker compose -f "$COMPOSE_FILE" --profile web logs -f report-server
    else
        log_error "Failed to start web server"
    fi
}

# Show results summary
show_results() {
    log_header "Benchmark Results Summary"
    
    # Try to extract some key results
    log "Extracting results summary..."
    
    # Check if results exist
    local results_exist=$(docker compose -f "$COMPOSE_FILE" exec -T scheme-benchmarks test -d /benchmarks/results/processed && echo "yes" || echo "no")
    
    if [[ "$results_exist" == "yes" ]]; then
        log "Results found. Key statistics:"
        
        # Try to show summary from JSON results
        docker compose -f "$COMPOSE_FILE" exec -T scheme-benchmarks \
            python3 -c "
import json
import os
try:
    with open('/benchmarks/results/processed/comparisons.json', 'r') as f:
        data = json.load(f)
    
    summary = data.get('summary', {}).get('overall_rankings', [])
    if summary:
        print('\nOverall Performance Rankings:')
        for i, impl in enumerate(summary[:5], 1):
            name = impl['implementation']
            avg_rank = impl['average_rank']
            win_pct = impl['win_percentage']
            print(f'{i:2d}. {name:12s} - Avg Rank: {avg_rank:.2f}, Wins: {win_pct:.1f}%')
    else:
        print('No summary data available')
except Exception as e:
    print(f'Could not extract summary: {e}')
" 2>/dev/null || log_warn "Could not extract results summary"
        
        log ""
        log "Full results available in Docker volume 'benchmark-results'"
        log "View with: docker compose -f $COMPOSE_FILE exec scheme-benchmarks ls -la /benchmarks/results/"
        
    else
        log_warn "No processed results found"
    fi
}

# Cleanup resources
cleanup() {
    if [[ "$CLEANUP_AFTER" != true ]]; then
        return
    fi
    
    log_header "Cleanup"
    
    log "Stopping and removing containers..."
    docker compose -f "$COMPOSE_FILE" down --remove-orphans
    
    log "Removing volumes..."
    docker compose -f "$COMPOSE_FILE" down --volumes
    
    log "Cleaning up Docker images..."
    docker image prune -f --filter "label=project=lambdust"
    
    log_success "Cleanup completed"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_header "Lambdust Scheme Benchmarking Suite"
    log "Starting comprehensive performance benchmarking..."
    
    # Parse arguments and check prerequisites
    parse_args "$@"
    check_prerequisites
    
    # Set verbose mode for Docker commands if requested
    if [[ "$VERBOSE" == true ]]; then
        set -x
    fi
    
    # Execute pipeline
    build_image
    run_benchmarks
    generate_reports
    show_results
    start_web_server
    
    # Cleanup if requested
    cleanup
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_header "Benchmark Suite Completed"
    log_success "Total execution time: ${duration} seconds"
    
    if [[ "$START_WEB_SERVER" != true ]]; then
        log ""
        log "To view results via web interface, run:"
        log "  $0 --no-build --no-benchmarks --web"
        log ""
        log "To access raw results:"
        log "  docker compose -f $COMPOSE_FILE exec scheme-benchmarks bash"
    fi
}

# Execute main function with all arguments
main "$@"