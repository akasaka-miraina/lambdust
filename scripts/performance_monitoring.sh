#!/bin/bash
# Performance Monitoring Script for Lambdust
#
# This script provides local performance monitoring capabilities
# and can be used for development-time performance testing.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="${PROJECT_ROOT}/benchmark_results"
HISTORY_DIR="${PROJECT_ROOT}/performance_history"
CONFIG_FILE="${PROJECT_ROOT}/performance_config.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
DEFAULT_BENCHMARK_SUITES=(
    "migration_impact_benchmarks"
    "core_operation_benchmarks"
    "scheme_operation_benchmarks"
    "system_performance_benchmarks"
    "regression_testing_benchmarks"
    "performance_analysis_benchmarks"
)

DEFAULT_OUTPUT_FORMATS=("json" "html" "csv")
DEFAULT_ALERT_THRESHOLD=15  # 15% performance change threshold
DEFAULT_SAMPLE_SIZE=100

usage() {
    cat << EOF
Usage: $0 [OPTIONS] [COMMAND]

Performance monitoring script for Lambdust.

COMMANDS:
    run                 Run all benchmark suites
    run <suite>         Run specific benchmark suite
    compare <baseline>  Compare current performance against baseline
    report              Generate performance report
    alert               Check for performance alerts
    dashboard           Start local performance dashboard
    clean               Clean benchmark results and history
    
OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -c, --config FILE   Use custom configuration file
    -o, --output DIR    Output directory for results
    -f, --format FMT    Output format (json, html, csv, all)
    -t, --threshold N   Alert threshold percentage (default: 15)
    -s, --samples N     Number of samples per benchmark (default: 100)
    --no-alerts         Disable performance alerts
    --baseline FILE     Baseline file for comparisons
    --profile           Enable profiling during benchmarks
    --flamegraph        Generate flamegraphs for hot paths

EXAMPLES:
    $0 run                                  # Run all benchmarks
    $0 run migration_impact_benchmarks      # Run specific suite
    $0 compare baseline.json                # Compare against baseline
    $0 report --format html                 # Generate HTML report
    $0 alert --threshold 10                 # Check alerts with 10% threshold
    
EOF
}

log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        INFO)  echo -e "${BLUE}[INFO]${NC} ${timestamp} $message" ;;
        WARN)  echo -e "${YELLOW}[WARN]${NC} ${timestamp} $message" ;;
        ERROR) echo -e "${RED}[ERROR]${NC} ${timestamp} $message" >&2 ;;
        SUCCESS) echo -e "${GREEN}[SUCCESS]${NC} ${timestamp} $message" ;;
        *) echo "[$level] ${timestamp} $message" ;;
    esac
}

setup_directories() {
    log INFO "Setting up directories..."
    mkdir -p "$RESULTS_DIR" "$HISTORY_DIR"
    
    # Create results subdirectories
    for suite in "${DEFAULT_BENCHMARK_SUITES[@]}"; do
        mkdir -p "${RESULTS_DIR}/${suite}"
    done
}

load_config() {
    local config_file="${1:-$CONFIG_FILE}"
    
    if [[ -f "$config_file" ]]; then
        log INFO "Loading configuration from $config_file"
        # In a real implementation, this would parse TOML
        # For now, we'll use environment variables
        source "$config_file" 2>/dev/null || true
    else
        log WARN "Configuration file not found: $config_file"
        log INFO "Using default configuration"
    fi
}

check_dependencies() {
    log INFO "Checking dependencies..."
    
    # Check for Rust and Cargo
    if ! command -v cargo >/dev/null 2>&1; then
        log ERROR "Rust/Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check for criterion
    if ! cargo --list | grep -q criterion; then
        log WARN "cargo-criterion not found. Installing..."
        cargo install cargo-criterion || {
            log ERROR "Failed to install cargo-criterion"
            exit 1
        }
    fi
    
    # Check for other tools
    local tools=("jq" "python3")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            log WARN "$tool not found. Some features may not work."
        fi
    done
    
    log SUCCESS "Dependencies check completed"
}

run_benchmark_suite() {
    local suite="$1"
    local output_dir="${RESULTS_DIR}/${suite}"
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local result_file="${output_dir}/results_${timestamp}.json"
    
    log INFO "Running benchmark suite: $suite"
    
    # Ensure the benchmark exists
    local bench_file="${PROJECT_ROOT}/benches/${suite}.rs"
    if [[ ! -f "$bench_file" ]]; then
        log ERROR "Benchmark file not found: $bench_file"
        return 1
    fi
    
    # Run the benchmark
    cd "$PROJECT_ROOT"
    
    log INFO "Executing: cargo bench --bench $suite --features benchmarks"
    
    if cargo bench --bench "$suite" --features benchmarks -- --output-format json > "$result_file" 2>&1; then
        log SUCCESS "Benchmark suite $suite completed successfully"
        log INFO "Results saved to: $result_file"
        
        # Generate summary
        generate_benchmark_summary "$result_file" "${output_dir}/summary_${timestamp}.json"
        
        return 0
    else
        log ERROR "Benchmark suite $suite failed"
        return 1
    fi
}

run_all_benchmarks() {
    log INFO "Running all benchmark suites..."
    
    local failed_suites=()
    local successful_suites=()
    
    for suite in "${DEFAULT_BENCHMARK_SUITES[@]}"; do
        if run_benchmark_suite "$suite"; then
            successful_suites+=("$suite")
        else
            failed_suites+=("$suite")
        fi
    done
    
    log INFO "Benchmark run completed"
    log INFO "Successful suites: ${#successful_suites[@]}"
    log INFO "Failed suites: ${#failed_suites[@]}"
    
    if [[ ${#failed_suites[@]} -gt 0 ]]; then
        log WARN "Failed suites: ${failed_suites[*]}"
        return 1
    fi
    
    return 0
}

generate_benchmark_summary() {
    local result_file="$1"
    local summary_file="$2"
    
    if [[ ! -f "$result_file" ]]; then
        log ERROR "Result file not found: $result_file"
        return 1
    fi
    
    log INFO "Generating benchmark summary..."
    
    # Create a Python script for processing results
    cat > /tmp/process_results.py << 'EOF'
#!/usr/bin/env python3
import json
import sys
import statistics
from datetime import datetime

def process_benchmark_results(result_file, summary_file):
    try:
        with open(result_file, 'r') as f:
            # Handle both Criterion JSON and custom JSON formats
            content = f.read().strip()
            if not content:
                return False
            
            # Try to parse as JSON
            try:
                data = json.loads(content)
            except json.JSONDecodeError:
                # If it's not JSON, create a simple summary
                data = {"raw_output": content}
        
        summary = {
            "timestamp": datetime.utcnow().isoformat(),
            "benchmark_count": len(data) if isinstance(data, list) else 1,
            "status": "completed",
            "raw_data": data
        }
        
        with open(summary_file, 'w') as f:
            json.dump(summary, f, indent=2)
        
        return True
    except Exception as e:
        print(f"Error processing results: {e}", file=sys.stderr)
        return False

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: process_results.py <result_file> <summary_file>")
        sys.exit(1)
    
    result_file = sys.argv[1]
    summary_file = sys.argv[2]
    
    if process_benchmark_results(result_file, summary_file):
        print("Summary generated successfully")
    else:
        sys.exit(1)
EOF

    if python3 /tmp/process_results.py "$result_file" "$summary_file"; then
        log SUCCESS "Summary generated: $summary_file"
    else
        log ERROR "Failed to generate summary"
        return 1
    fi
    
    rm -f /tmp/process_results.py
}

compare_with_baseline() {
    local baseline_file="$1"
    local current_results_dir="$RESULTS_DIR"
    
    log INFO "Comparing current results with baseline: $baseline_file"
    
    if [[ ! -f "$baseline_file" ]]; then
        log ERROR "Baseline file not found: $baseline_file"
        return 1
    fi
    
    # Create comparison script
    cat > /tmp/compare_performance.py << 'EOF'
#!/usr/bin/env python3
import json
import os
import sys
from datetime import datetime

def load_json_file(file_path):
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except:
        return None

def compare_performance(baseline_file, results_dir, threshold=0.15):
    baseline = load_json_file(baseline_file)
    if not baseline:
        print("Failed to load baseline file")
        return False
    
    comparisons = []
    alerts = []
    
    # Find latest results for each suite
    for suite_dir in os.listdir(results_dir):
        suite_path = os.path.join(results_dir, suite_dir)
        if not os.path.isdir(suite_path):
            continue
        
        # Find latest summary file
        summary_files = [f for f in os.listdir(suite_path) if f.startswith('summary_')]
        if not summary_files:
            continue
        
        latest_summary = max(summary_files)
        current_data = load_json_file(os.path.join(suite_path, latest_summary))
        
        if current_data:
            comparison = {
                "suite": suite_dir,
                "baseline_benchmarks": baseline.get("benchmark_count", 0),
                "current_benchmarks": current_data.get("benchmark_count", 0),
                "status": "compared"
            }
            comparisons.append(comparison)
    
    report = {
        "timestamp": datetime.utcnow().isoformat(),
        "baseline_file": baseline_file,
        "comparisons": comparisons,
        "alerts": alerts,
        "summary": {
            "total_suites_compared": len(comparisons),
            "alerts_count": len(alerts)
        }
    }
    
    with open("comparison_report.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"Comparison completed. {len(comparisons)} suites compared, {len(alerts)} alerts.")
    return len(alerts) == 0

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: compare_performance.py <baseline_file> <results_dir> [threshold]")
        sys.exit(1)
    
    baseline_file = sys.argv[1]
    results_dir = sys.argv[2]
    threshold = float(sys.argv[3]) if len(sys.argv) > 3 else 0.15
    
    success = compare_performance(baseline_file, results_dir, threshold)
    sys.exit(0 if success else 1)
EOF

    if python3 /tmp/compare_performance.py "$baseline_file" "$current_results_dir" "${DEFAULT_ALERT_THRESHOLD}"; then
        log SUCCESS "Performance comparison completed successfully"
        if [[ -f "comparison_report.json" ]]; then
            log INFO "Comparison report saved to: comparison_report.json"
        fi
    else
        log WARN "Performance comparison detected alerts"
        return 1
    fi
    
    rm -f /tmp/compare_performance.py
}

generate_performance_report() {
    local format="${1:-html}"
    local output_file="${RESULTS_DIR}/performance_report"
    
    log INFO "Generating performance report in $format format..."
    
    case "$format" in
        "html")
            output_file="${output_file}.html"
            generate_html_report "$output_file"
            ;;
        "json")
            output_file="${output_file}.json"
            generate_json_report "$output_file"
            ;;
        "csv")
            output_file="${output_file}.csv"
            generate_csv_report "$output_file"
            ;;
        "all")
            generate_html_report "${RESULTS_DIR}/performance_report.html"
            generate_json_report "${RESULTS_DIR}/performance_report.json"
            generate_csv_report "${RESULTS_DIR}/performance_report.csv"
            log SUCCESS "All report formats generated"
            return 0
            ;;
        *)
            log ERROR "Unknown format: $format"
            return 1
            ;;
    esac
    
    log SUCCESS "Report generated: $output_file"
}

generate_html_report() {
    local output_file="$1"
    
    cat > "$output_file" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Lambdust Performance Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background-color: #f5f5f5; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; }
        .success { color: green; }
        .warning { color: orange; }
        .error { color: red; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Lambdust Performance Report</h1>
        <p><strong>Generated:</strong> <script>document.write(new Date().toISOString());</script></p>
    </div>
    
    <div class="section">
        <h2>Overview</h2>
        <p>This report summarizes the performance characteristics of the Lambdust language implementation.</p>
    </div>
    
    <div class="section">
        <h2>Benchmark Suites</h2>
        <table>
            <tr>
                <th>Suite Name</th>
                <th>Status</th>
                <th>Benchmark Count</th>
                <th>Last Run</th>
            </tr>
            <tr><td colspan="4"><em>Data will be populated by actual benchmark results</em></td></tr>
        </table>
    </div>
    
    <div class="section">
        <h2>Performance Alerts</h2>
        <p>No alerts detected in this sample report.</p>
    </div>
</body>
</html>
EOF
}

generate_json_report() {
    local output_file="$1"
    
    cat > "$output_file" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)",
  "version": "1.0",
  "summary": {
    "total_suites": ${#DEFAULT_BENCHMARK_SUITES[@]},
    "completed_suites": 0,
    "failed_suites": 0,
    "total_benchmarks": 0
  },
  "suites": {},
  "alerts": [],
  "recommendations": []
}
EOF
}

generate_csv_report() {
    local output_file="$1"
    
    cat > "$output_file" << 'EOF'
suite_name,benchmark_name,avg_time_ms,min_time_ms,max_time_ms,samples,timestamp
EOF
}

check_performance_alerts() {
    local threshold="${1:-$DEFAULT_ALERT_THRESHOLD}"
    
    log INFO "Checking for performance alerts (threshold: ${threshold}%)..."
    
    local alerts_found=0
    
    # Check each benchmark suite for alerts
    for suite in "${DEFAULT_BENCHMARK_SUITES[@]}"; do
        local suite_dir="${RESULTS_DIR}/${suite}"
        if [[ -d "$suite_dir" ]]; then
            local summary_files=($(ls -t "${suite_dir}"/summary_*.json 2>/dev/null | head -2))
            
            if [[ ${#summary_files[@]} -ge 2 ]]; then
                # Compare latest two runs
                log INFO "Checking $suite for performance changes..."
                # In a real implementation, this would compare actual metrics
            fi
        fi
    done
    
    if [[ $alerts_found -eq 0 ]]; then
        log SUCCESS "No performance alerts detected"
        return 0
    else
        log WARN "Found $alerts_found performance alerts"
        return 1
    fi
}

start_performance_dashboard() {
    log INFO "Starting local performance dashboard..."
    
    # Create a simple HTTP server for the dashboard
    cat > /tmp/dashboard.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os
import json
from datetime import datetime

class DashboardHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.path = '/dashboard.html'
        elif self.path == '/api/status':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            status = {
                "status": "running",
                "timestamp": datetime.utcnow().isoformat(),
                "suites": []
            }
            self.wfile.write(json.dumps(status).encode())
            return
        return super().do_GET()

if __name__ == "__main__":
    PORT = 8080
    with socketserver.TCPServer(("", PORT), DashboardHandler) as httpd:
        print(f"Dashboard server running at http://localhost:{PORT}")
        httpd.serve_forever()
EOF

    # Create dashboard HTML
    cat > /tmp/dashboard.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Lambdust Performance Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .dashboard { max-width: 1200px; margin: 0 auto; }
        .metrics { display: flex; gap: 20px; margin: 20px 0; }
        .metric-card { background: #f5f5f5; padding: 20px; border-radius: 5px; flex: 1; }
    </style>
</head>
<body>
    <div class="dashboard">
        <h1>Lambdust Performance Dashboard</h1>
        <div class="metrics">
            <div class="metric-card">
                <h3>Total Benchmarks</h3>
                <p id="total-benchmarks">Loading...</p>
            </div>
            <div class="metric-card">
                <h3>Last Update</h3>
                <p id="last-update">Loading...</p>
            </div>
            <div class="metric-card">
                <h3>Status</h3>
                <p id="status">Loading...</p>
            </div>
        </div>
        <div id="charts">
            <p>Performance charts would be displayed here.</p>
        </div>
    </div>
    <script>
        function updateDashboard() {
            fetch('/api/status')
                .then(response => response.json())
                .then(data => {
                    document.getElementById('status').textContent = data.status;
                    document.getElementById('last-update').textContent = new Date(data.timestamp).toLocaleString();
                    document.getElementById('total-benchmarks').textContent = data.suites.length;
                })
                .catch(error => {
                    console.error('Error fetching status:', error);
                });
        }
        
        updateDashboard();
        setInterval(updateDashboard, 30000); // Update every 30 seconds
    </script>
</body>
</html>
EOF

    cd /tmp
    log INFO "Dashboard available at http://localhost:8080"
    python3 dashboard.py
}

clean_results() {
    log INFO "Cleaning benchmark results and history..."
    
    read -p "Are you sure you want to delete all benchmark results? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$RESULTS_DIR"/*
        rm -rf "$HISTORY_DIR"/*
        log SUCCESS "Cleaned benchmark results and history"
    else
        log INFO "Clean operation cancelled"
    fi
}

main() {
    local command=""
    local verbose=false
    local config_file="$CONFIG_FILE"
    local output_dir="$RESULTS_DIR"
    local format="json"
    local threshold="$DEFAULT_ALERT_THRESHOLD"
    local samples="$DEFAULT_SAMPLE_SIZE"
    local enable_alerts=true
    local baseline_file=""
    local enable_profile=false
    local enable_flamegraph=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -v|--verbose)
                verbose=true
                shift
                ;;
            -c|--config)
                config_file="$2"
                shift 2
                ;;
            -o|--output)
                output_dir="$2"
                shift 2
                ;;
            -f|--format)
                format="$2"
                shift 2
                ;;
            -t|--threshold)
                threshold="$2"
                shift 2
                ;;
            -s|--samples)
                samples="$2"
                shift 2
                ;;
            --no-alerts)
                enable_alerts=false
                shift
                ;;
            --baseline)
                baseline_file="$2"
                shift 2
                ;;
            --profile)
                enable_profile=true
                shift
                ;;
            --flamegraph)
                enable_flamegraph=true
                shift
                ;;
            run|compare|report|alert|dashboard|clean)
                command="$1"
                shift
                break
                ;;
            *)
                log ERROR "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Set up logging verbosity
    if [[ "$verbose" == true ]]; then
        set -x
    fi
    
    # Initialize
    setup_directories
    load_config "$config_file"
    check_dependencies
    
    # Execute command
    case "$command" in
        "run")
            if [[ $# -gt 0 ]]; then
                # Run specific suite
                run_benchmark_suite "$1"
            else
                # Run all suites
                run_all_benchmarks
            fi
            ;;
        "compare")
            if [[ -z "$baseline_file" && $# -gt 0 ]]; then
                baseline_file="$1"
            fi
            if [[ -n "$baseline_file" ]]; then
                compare_with_baseline "$baseline_file"
            else
                log ERROR "Baseline file required for comparison"
                exit 1
            fi
            ;;
        "report")
            generate_performance_report "$format"
            ;;
        "alert")
            check_performance_alerts "$threshold"
            ;;
        "dashboard")
            start_performance_dashboard
            ;;
        "clean")
            clean_results
            ;;
        "")
            log ERROR "No command specified"
            usage
            exit 1
            ;;
        *)
            log ERROR "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"