#!/bin/bash

# Value Optimization Performance Verification Script
# 
# This script runs comprehensive performance verification for the Value enum
# optimizations and generates detailed reports.

set -e

echo "🚀 Value Optimization Performance Verification"
echo "════════════════════════════════════════════════════════════════"
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
CONFIG_TYPE="standard"
REPORT_FORMAT="both"
CLEAN_BUILD=false
RUN_BENCHMARKS=true
VERBOSE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --config)
            CONFIG_TYPE="$2"
            shift 2
            ;;
        --report-format)
            REPORT_FORMAT="$2"
            shift 2
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --no-benchmarks)
            RUN_BENCHMARKS=false
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "OPTIONS:"
            echo "  --config TYPE           Configuration type (fast|standard|production|comprehensive)"
            echo "  --report-format FORMAT  Report format (text|json|both)"
            echo "  --clean                 Clean build before running"
            echo "  --no-benchmarks        Skip Criterion benchmarks"
            echo "  --verbose              Verbose output"
            echo "  --help                 Show this help"
            echo
            echo "EXAMPLES:"
            echo "  $0                                    # Standard verification"
            echo "  $0 --config fast                     # Quick verification"
            echo "  $0 --config production --clean       # Full production verification"
            echo "  $0 --report-format json --verbose    # JSON output with verbose logging"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}Configuration:${NC}"
echo "  Config Type: $CONFIG_TYPE"
echo "  Report Format: $REPORT_FORMAT"
echo "  Clean Build: $CLEAN_BUILD"
echo "  Run Benchmarks: $RUN_BENCHMARKS"
echo "  Verbose: $VERBOSE"
echo

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo -e "${RED}Error: Must be run from the lambdust project root${NC}"
    exit 1
fi

# Clean build if requested
if [[ "$CLEAN_BUILD" == "true" ]]; then
    echo -e "${YELLOW}🧹 Cleaning build artifacts...${NC}"
    cargo clean
    echo
fi

# Build the project
echo -e "${YELLOW}🔨 Building project...${NC}"
if [[ "$VERBOSE" == "true" ]]; then
    cargo build --release --bins
else
    cargo build --release --bins > /dev/null 2>&1
fi

if [[ $? -ne 0 ]]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Build completed successfully${NC}"
echo

# Run Criterion benchmarks if requested
if [[ "$RUN_BENCHMARKS" == "true" ]]; then
    echo -e "${YELLOW}⚡ Running Criterion benchmarks...${NC}"
    
    # Create benchmarks directory if it doesn't exist
    mkdir -p target/criterion
    
    if [[ "$VERBOSE" == "true" ]]; then
        cargo bench --bench comprehensive_value_optimization_benchmarks
    else
        echo "  Running comprehensive optimization benchmarks..."
        cargo bench --bench comprehensive_value_optimization_benchmarks > /dev/null 2>&1
    fi
    
    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}✅ Benchmarks completed successfully${NC}"
        echo "  📊 Benchmark results saved to target/criterion/"
    else
        echo -e "${YELLOW}⚠️  Benchmarks completed with warnings${NC}"
    fi
    echo
fi

# Run the performance verification suite
echo -e "${YELLOW}🔍 Running performance verification suite...${NC}"

VERIFICATION_ARGS="--config $CONFIG_TYPE --report-format $REPORT_FORMAT"

if [[ "$VERBOSE" == "true" ]]; then
    cargo run --release --bin value_optimization_verifier -- $VERIFICATION_ARGS
else
    cargo run --release --bin value_optimization_verifier -- $VERIFICATION_ARGS
fi

VERIFICATION_EXIT_CODE=$?

echo

# Check results and provide feedback
if [[ $VERIFICATION_EXIT_CODE -eq 0 ]]; then
    echo -e "${GREEN}✅ Performance verification completed successfully${NC}"
    
    # Check if reports were generated
    if [[ -f "value_optimization_report.txt" ]]; then
        echo -e "${GREEN}📄 Text report generated: value_optimization_report.txt${NC}"
    fi
    
    if [[ -f "value_optimization_report.json" ]]; then
        echo -e "${GREEN}📄 JSON report generated: value_optimization_report.json${NC}"
    fi
    
    echo
    echo -e "${BLUE}📊 Quick Stats Check:${NC}"
    
    # Extract key metrics if JSON report exists
    if [[ -f "value_optimization_report.json" ]] && command -v jq &> /dev/null; then
        echo "  Arc Reduction: $(jq -r '.arc_analysis.arc_reduction_percentage' value_optimization_report.json 2>/dev/null || echo 'N/A')%"
        echo "  Memory Savings: $(jq -r '.memory_analysis.average_savings_percentage' value_optimization_report.json 2>/dev/null || echo 'N/A')%"
        echo "  Semantic Compliance: $(jq -r '.semantic_verification.compliance_percentage' value_optimization_report.json 2>/dev/null || echo 'N/A')%"
        echo "  Production Readiness: $(jq -r '.production_assessment.overall_readiness_score' value_optimization_report.json 2>/dev/null || echo 'N/A')/100"
    else
        echo "  (Install 'jq' for detailed metric extraction)"
    fi
    
    echo
    echo -e "${GREEN}🎉 Verification Complete!${NC}"
    echo
    echo "Next steps:"
    echo "  1. Review the generated reports"
    echo "  2. Address any failed optimization targets"
    echo "  3. Run verification again after fixes"
    echo "  4. Consider production deployment when all targets are met"
    
else
    echo -e "${RED}❌ Performance verification failed${NC}"
    echo
    echo "Troubleshooting:"
    echo "  1. Check the error messages above"
    echo "  2. Try running with --verbose for more details"
    echo "  3. Try --config fast for a quicker test"
    echo "  4. Check that all dependencies are properly installed"
    
    exit $VERIFICATION_EXIT_CODE
fi

# Offer to open reports
if [[ "$REPORT_FORMAT" == "text" || "$REPORT_FORMAT" == "both" ]] && [[ -f "value_optimization_report.txt" ]]; then
    echo
    read -p "Would you like to view the text report? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if command -v less &> /dev/null; then
            less value_optimization_report.txt
        elif command -v more &> /dev/null; then
            more value_optimization_report.txt
        else
            cat value_optimization_report.txt
        fi
    fi
fi

echo
echo -e "${BLUE}🔧 Additional Tools:${NC}"
echo "  View Criterion results: open target/criterion/report/index.html"
echo "  Compare benchmark runs: cargo bench -- --save-baseline <name>"
echo "  Profile memory usage: cargo run --release --bin value_optimization_verifier -- --config production"
echo
echo -e "${GREEN}Performance verification session complete!${NC}"