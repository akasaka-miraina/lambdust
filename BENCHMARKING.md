# Lambdust Scheme Benchmarking Suite

A comprehensive Docker-based benchmarking environment for comparing Lambdust performance against major Scheme implementations.

## Overview

This benchmarking suite provides automated, reproducible performance comparison between Lambdust and the following Scheme implementations:

- **Chez Scheme** - Cross-module optimizing native-code compiler, R6 compliant
- **Racket** - Native code compiler, R5/R6/R7 compliant
- **Gambit** - Fast, concurrent, retargetable optimizing compiler, R5/R7 compliant
- **Gauche** - Fast R7RS Scheme implementation (widely used)
- **Chicken** - Scheme-to-C compiler with good performance
- **MIT/GNU Scheme** - Traditional, mature implementation
- **Guile** - GNU's official Scheme, widely deployed
- **Cyclone** - R7RS compliant Scheme-to-C compiler
- **Lambdust** - Our implementation with gradual typing and effects

## Quick Start

### Prerequisites

- **Docker** (version 20.10+)
- **Docker Compose** (version 2.0+)
- **10GB+ free disk space**
- **4GB+ RAM recommended**

### Running Benchmarks

```bash
# Full benchmark suite (takes 30-60 minutes)
./benchmark.sh

# Quick benchmark with web interface
./benchmark.sh --quick --web

# Test specific implementations only
./benchmark.sh --implementations "lambdust,chez,racket"

# View help
./benchmark.sh --help
```

### Viewing Results

After benchmarks complete:

```bash
# Start web server to view HTML reports
./benchmark.sh --no-build --no-benchmarks --web

# Access results directly
docker-compose -f docker-compose.benchmarks.yml exec scheme-benchmarks bash
cd /benchmarks/results
```

Results are available at `http://localhost:8080` when web server is running.

## Architecture

### Docker Environment

The benchmarking suite uses a multi-stage Docker build to install all Scheme implementations in isolated environments:

```
Base System (Ubuntu 22.04)
├── Development tools (gcc, cmake, etc.)
├── Python analysis tools
├── Performance monitoring (valgrind, time, etc.)
└── Individual Scheme implementations
    ├── Chez Scheme
    ├── Racket
    ├── Gambit
    ├── Gauche
    ├── Chicken
    ├── MIT/GNU Scheme
    ├── Guile
    ├── Cyclone
    └── Lambdust (built from source)
```

### Benchmark Categories

#### Micro-benchmarks
- **arithmetic_ops** - Basic arithmetic operations
- **list_operations** - List creation and traversal
- **vector_operations** - Vector creation and access
- **string_operations** - String manipulation
- **function_calls** - Function call overhead
- **closure_creation** - Closure creation and invocation
- **recursion_depth** - Deep recursion performance
- **tail_call_optimization** - Tail call optimization

#### Algorithm Benchmarks
- **fibonacci** - Recursive Fibonacci calculation
- **factorial** - Factorial calculation
- **quicksort** - Quicksort implementation
- **mergesort** - Merge sort implementation
- **binary_search** - Binary search algorithm
- **tree_traversal** - Tree traversal algorithms
- **dynamic_programming** - Dynamic programming solutions

#### Data Structure Performance
- **list_creation_access** - List performance
- **vector_creation_access** - Vector performance
- **hash_table_operations** - Hash table operations
- **tree_operations** - Tree operations
- **queue_operations** - Queue operations

#### R7RS Compliance Performance
- **r7rs_arithmetic** - R7RS arithmetic procedures
- **r7rs_lists** - R7RS list procedures
- **r7rs_vectors** - R7RS vector procedures
- **r7rs_strings** - R7RS string procedures
- **r7rs_io** - R7RS I/O procedures
- **r7rs_macros** - R7RS macro system

### Performance Metrics

For each test, the suite collects:

- **Execution Time**
  - Wall clock time
  - User CPU time
  - System CPU time
  - Statistical measures (mean, median, std dev, percentiles)

- **Memory Usage**
  - Peak RSS (Resident Set Size)
  - Virtual memory usage
  - Memory allocations (via Valgrind when enabled)

- **System Resources**
  - CPU utilization
  - Context switches
  - I/O operations
  - Cache performance (when available)

## Configuration

### Benchmark Configuration (`benchmark-config.yaml`)

```yaml
# Execution settings
execution:
  iterations: 10          # Number of test runs
  warmup_iterations: 3    # Warm-up runs (not counted)
  timeout: 300           # Test timeout (seconds)
  memory_limit: 2048     # Memory limit (MB)

# Output settings
output:
  formats: ["json", "csv", "html"]
  baseline: "chez"       # Baseline for comparisons
  
# Implementations to test
implementations:
  lambdust:
    binary: "/benchmarks/lambdust/target/release/lambdust"
    args: ["--batch"]
    file_extension: ".ldust"
  # ... other implementations
```

### Environment Variables

```bash
# Benchmark execution
BENCHMARK_ITERATIONS=10      # Number of iterations per test
BENCHMARK_WARMUP=3          # Warm-up iterations
BENCHMARK_TIMEOUT=300       # Test timeout in seconds
BENCHMARK_MEMORY_LIMIT=2048 # Memory limit in MB

# Scheme runtime settings
SCHEME_HEAP_SIZE=256m       # Initial heap size
GC_INITIAL_HEAP_SIZE=64m    # Initial GC heap size
```

## Usage Examples

### Basic Usage

```bash
# Run all benchmarks with default settings
./benchmark.sh

# Quick test (reduced iterations)
./benchmark.sh --quick

# Verbose output
./benchmark.sh --verbose
```

### Selective Testing

```bash
# Test specific implementations
./benchmark.sh --implementations "lambdust,chez,gambit"

# Test specific benchmark suites
./benchmark.sh --tests "micro_benchmarks,algorithm_benchmarks"

# Skip building Docker image (if already built)
./benchmark.sh --no-build
```

### Results and Analysis

```bash
# Generate reports only (skip benchmarking)
./benchmark.sh --no-build --no-benchmarks

# Start web server for interactive results
./benchmark.sh --no-build --no-benchmarks --web

# Clean up after completion
./benchmark.sh --cleanup
```

### Docker Compose Usage

For more control, use Docker Compose directly:

```bash
# Build the benchmarking environment
docker-compose -f docker-compose.benchmarks.yml build

# Run benchmarks
docker-compose -f docker-compose.benchmarks.yml up scheme-benchmarks

# Run analysis
docker-compose -f docker-compose.benchmarks.yml --profile analysis up results-analyzer

# Start web server
docker-compose -f docker-compose.benchmarks.yml --profile web up -d report-server

# Clean up
docker-compose -f docker-compose.benchmarks.yml down --volumes
```

## Results Interpretation

### Overall Performance Rankings

Results are ranked by:
1. **Average Rank** across all tests (lower is better)
2. **Win Percentage** (percentage of tests where implementation was fastest)
3. **Statistical Significance** of performance differences

### Performance Categories

- **Speed Champion** - Fastest overall execution time
- **Memory Efficient** - Lowest memory usage
- **Most Consistent** - Smallest performance variance
- **Best Scaling** - Best performance on large datasets

### Relative Performance

All results are normalized against a baseline implementation (default: Chez Scheme):
- **1.0** - Same performance as baseline
- **< 1.0** - Faster than baseline (e.g., 0.8 = 20% faster)
- **> 1.0** - Slower than baseline (e.g., 1.5 = 50% slower)

## Advanced Usage

### Custom Test Development

Add new tests by:

1. **Define test in `scripts/generate_tests.py`**:
```python
'my_test': {
    'description': 'My custom test',
    'setup': '(define setup-code ...)',
    'main_code': '''
    (define (my-test-function n)
      ; Test implementation
      )
    
    (define (run-test)
      (my-test-function 10000))
    ''',
    'expected_type': 'number'
}
```

2. **Add to benchmark configuration**:
```yaml
benchmark_suites:
  custom_tests:
    name: "Custom Test Suite"
    tests:
      - my_test
```

### Profiling Integration

Enable advanced profiling:

```bash
# Enable Valgrind profiling (slower but detailed)
docker-compose -f docker-compose.benchmarks.yml run \
  -e ENABLE_VALGRIND=true \
  scheme-benchmarks

# Custom profiling with specific tools
python3 scripts/performance_monitor.py \
  --command "lambdust test.ldust" \
  --test-name "custom_test" \
  --implementation "lambdust" \
  --output "profile_results.json" \
  --valgrind
```

### Result Analysis

Process results programmatically:

```python
import json
from pathlib import Path

# Load processed results
with open('/benchmarks/results/processed/comparisons.json') as f:
    data = json.load(f)

# Extract Lambdust performance
rankings = data['summary']['overall_rankings']
lambdust_rank = next(
    (r for r in rankings if r['implementation'] == 'lambdust'),
    None
)

if lambdust_rank:
    print(f"Lambdust overall rank: {lambdust_rank['rank']}")
    print(f"Win percentage: {lambdust_rank['win_percentage']:.1f}%")
```

## Troubleshooting

### Common Issues

**Docker Build Failures**
```bash
# Clear Docker cache and rebuild
docker system prune -f
./benchmark.sh --verbose
```

**Out of Memory Errors**
```bash
# Increase Docker memory limits
# In Docker Desktop: Settings > Resources > Memory > 8GB
# Or reduce benchmark scope:
./benchmark.sh --implementations "lambdust,chez" --quick
```

**Slow Performance**
```bash
# Use quick mode for faster results
./benchmark.sh --quick

# Test fewer implementations
./benchmark.sh --implementations "lambdust,chez,racket"

# Skip Docker build if image exists
./benchmark.sh --no-build
```

**Permission Errors**
```bash
# Fix file permissions
sudo chown -R $USER:$USER .
chmod +x benchmark.sh scripts/*.py scripts/*.sh
```

### Debug Mode

For debugging test issues:

```bash
# Access the container directly
docker-compose -f docker-compose.benchmarks.yml exec scheme-benchmarks bash

# Check which implementations are available
/benchmarks/health_check.sh

# Run individual tests manually
cd /benchmarks
python3 lambdust/scripts/generate_tests.py --config lambdust/benchmark-config.yaml --output-dir tests --implementations lambdust

# Test specific implementation
lambdust/target/release/lambdust --batch tests/micro_benchmarks/arithmetic_ops.ldust
```

## Performance Optimization

### Docker Optimization

```bash
# Use BuildKit for faster builds
export DOCKER_BUILDKIT=1

# Enable parallel builds
./benchmark.sh --parallel-builds

# Use local registry for faster iterations
docker tag lambdust/benchmarks:latest localhost:5000/lambdust/benchmarks:latest
```

### Benchmark Tuning

```bash
# Reduce iterations for development
export BENCHMARK_ITERATIONS=3
export BENCHMARK_WARMUP=1

# Focus on specific test categories
./benchmark.sh --tests "micro_benchmarks"

# Disable memory profiling for speed
export DISABLE_MEMORY_PROFILING=true
```

## Contributing

### Adding New Implementations

To add a new Scheme implementation:

1. **Update `Dockerfile.benchmarks`** with installation steps
2. **Add implementation config** in `benchmark-config.yaml`
3. **Update test generator** in `scripts/generate_tests.py`
4. **Test the implementation** thoroughly

### Adding New Benchmarks

1. **Define test logic** in `scripts/generate_tests.py`
2. **Add to benchmark suites** in `benchmark-config.yaml`
3. **Ensure cross-implementation compatibility**
4. **Document expected behavior and results**

### Improving Analysis

1. **Enhance result processing** in `scripts/process_results.py`
2. **Add visualization capabilities** in `scripts/generate_report.py`
3. **Implement statistical significance testing**
4. **Add performance regression detection**

## License

This benchmarking suite is part of the Lambdust project and follows the same licensing terms.

## Support

For issues with the benchmarking suite:

1. **Check this documentation** for common solutions
2. **Review Docker logs** for error details
3. **Open an issue** with benchmark results and system details
4. **Contribute improvements** via pull requests

---

**Note**: Benchmark results are highly dependent on system configuration, Docker resource limits, and concurrent processes. For fair comparison, always run benchmarks on the same system with consistent resource allocation.