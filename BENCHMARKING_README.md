# Lambdust Native Benchmarking System

A comprehensive, standalone performance benchmarking suite for the Lambdust Scheme interpreter that provides immediate performance insights while maintaining compatibility with Docker-based benchmark frameworks.

## 🎯 Overview

This benchmarking system provides:

- **Native Performance Testing**: Run benchmarks directly without Docker dependencies
- **Comprehensive Analysis**: Statistical analysis with detailed performance insights
- **Docker Compatibility**: Results integrate seamlessly with Docker benchmark frameworks
- **Regression Detection**: Compare performance across different runs
- **Real-time Monitoring**: Continuous performance tracking
- **Cross-Implementation Comparison**: Ready for comparison with other Scheme implementations

## 🚀 Quick Start

### Basic Usage

```bash
# Run comprehensive benchmark suite
./run-native-benchmarks.sh

# Run quick benchmarks (development mode)
./run-native-benchmarks.sh --quick

# Run with CSV output
./run-native-benchmarks.sh --csv

# Save results to specific location
./run-native-benchmarks.sh --output /path/to/results.json
```

### Master Performance Suite

```bash
# Run complete performance testing suite
./run-performance-suite.sh

# Quick development testing
./run-performance-suite.sh --mode development

# Enable performance monitoring
./run-performance-suite.sh --monitor

# Compare against baseline
./run-performance-suite.sh --baseline previous_results.json

# Continuous integration mode
./run-performance-suite.sh --continuous
```

## 📊 Benchmark Categories

### 1. Arithmetic Operations
- Integer arithmetic (addition, multiplication, division)
- Floating-point operations
- Complex number computations
- Rational number handling
- Numeric tower promotions

### 2. List Operations
- List construction and traversal
- Cons, car, cdr operations
- List append and reverse
- Large list processing
- Memory-intensive list operations

### 3. Recursion Performance
- Classic recursive algorithms (fibonacci, factorial)
- Tail recursion optimization
- Deep recursion handling
- Mutual recursion patterns
- Tree recursion algorithms

### 4. Memory Allocation
- Small object allocation patterns
- Large object allocation
- Mixed allocation scenarios
- Nested structure creation
- Memory fragmentation analysis

### 5. Function Calls
- Direct function call overhead
- Environment lookup performance
- Symbol interning efficiency
- Variable binding operations
- Closure creation and invocation

### 6. Realistic Programs
- Scheme algorithm implementations
- Higher-order functions (map, fold, filter)
- Data structure manipulations
- Sorting algorithms
- String processing operations

## 🔧 Tools and Scripts

### Core Executables

1. **Native Benchmark Runner** (`native-benchmark-runner`)
   ```bash
   cargo run --bin native-benchmark-runner --features benchmarks
   ```

2. **Performance Monitor** (`performance-monitor`)
   ```bash
   cargo run --bin performance-monitor --features benchmarks -- --continuous
   ```

3. **Scheme Comparison** (`scheme-comparison`)
   ```bash
   cargo run --bin scheme-comparison --features benchmarks
   ```

### Analysis Tools

1. **Performance Analysis Script** (`analyze-performance.py`)
   ```bash
   # Analyze single run
   python3 analyze-performance.py --file results.json
   
   # Compare two runs
   python3 analyze-performance.py --compare baseline.json current.json
   
   # Analyze entire directory
   python3 analyze-performance.py --directory benchmark-results/
   
   # Export to CSV
   python3 analyze-performance.py --file results.json --export analysis.csv
   ```

2. **Benchmark Shell Scripts**
   - `run-native-benchmarks.sh`: Simple benchmark execution
   - `run-performance-suite.sh`: Comprehensive testing suite

## 📈 Performance Metrics

### Primary Metrics
- **Operations per Second**: Throughput measurement
- **Execution Time**: Total and per-operation timing
- **Memory Usage**: Heap consumption and allocation patterns
- **Statistical Analysis**: Mean, median, standard deviation, percentiles
- **Performance Score**: Composite 0-100 performance rating

### Advanced Metrics
- **SIMD Optimization Effectiveness**: Vectorization impact
- **Memory Pool Efficiency**: Allocation optimization
- **Environment Optimization**: Variable lookup performance
- **Fast Path Coverage**: Primitive operation optimization

## 🐳 Docker Integration

### Compatibility Features

The benchmarking system is designed for seamless Docker integration:

```yaml
# Docker-compatible output structure
docker_output_schema:
  version: "docker_benchmark_v2"
  implementation: "lambdust"
  benchmark_results: [...] 
```

### Integration Commands

```bash
# Convert native results to Docker format
python3 scripts/convert_native_to_docker.py \
  --input native_results.json \
  --output docker_results.json

# Run comparison with other Scheme implementations
docker-compose -f docker-compose.benchmarks.yml up

# Generate cross-implementation report
python3 scripts/generate_cross_implementation_report.py \
  --native-results lambdust_results.json \
  --docker-results docker_results/ \
  --output comparison_report.html
```

### File Structure for Docker

```
benchmark-results/
├── native/                    # Native benchmark results
├── docker-compatible/         # Docker-formatted results
│   ├── results/              # Converted result files
│   ├── config/               # Integration configuration
│   └── logs/                 # Execution logs
├── analysis/                 # Analysis reports and data
└── monitoring/               # Performance monitoring data
```

## 📊 Statistical Analysis

### Metrics Calculated
- **Descriptive Statistics**: Mean, median, mode, standard deviation
- **Distribution Analysis**: Percentiles (P95, P99), skewness, kurtosis
- **Trend Analysis**: Performance evolution over time
- **Regression Detection**: Automatic identification of performance drops
- **Comparative Analysis**: Baseline vs. current performance

### Report Formats

1. **JSON Output**: Machine-readable detailed results
2. **CSV Export**: Spreadsheet-compatible data
3. **Text Reports**: Human-readable analysis
4. **Markdown Reports**: Documentation-friendly summaries

## 🔍 Performance Monitoring

### Real-time Monitoring

```bash
# Start continuous monitoring
cargo run --bin performance-monitor -- --continuous --interval 60

# Single snapshot
cargo run --bin performance-monitor

# Generate monitoring report
cargo run --bin performance-monitor -- --report
```

### Monitoring Features
- **System Resource Tracking**: CPU, memory, load average
- **Performance Trend Analysis**: Track performance over time
- **Regression Alerts**: Automatic detection of performance drops
- **Historical Data**: Retain monitoring data with configurable retention

## 🚨 Regression Detection

### Automated Regression Checking

```bash
# Check for regressions against baseline
./run-performance-suite.sh --regression-only --baseline baseline.json

# Set custom regression thresholds
./run-performance-suite.sh --threshold 5.0 --baseline baseline.json
```

### Regression Criteria
- Overall performance drop > 10%
- Category-specific drops > 15%
- Memory usage increase > 20%
- Statistical significance testing

## 🏗️ Development Integration

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Run Performance Benchmarks
  run: |
    ./run-performance-suite.sh --mode quick
    ./run-performance-suite.sh --regression-only --baseline baseline.json

- name: Generate Performance Report
  run: |
    python3 analyze-performance.py --file results.json --output performance_report.md
```

### Development Workflow

1. **Pre-commit**: Quick performance check
   ```bash
   ./run-performance-suite.sh --mode development
   ```

2. **Pull Request**: Regression detection
   ```bash
   ./run-performance-suite.sh --regression-only --baseline main_baseline.json
   ```

3. **Release**: Comprehensive benchmarking
   ```bash
   ./run-performance-suite.sh --mode comprehensive --baseline previous_release.json
   ```

## 📁 Output Files and Structure

### Generated Files

```
performance-results/
├── PERFORMANCE_REPORT_20240106_143022.md    # Comprehensive report
├── native/
│   ├── lambdust_benchmark_comprehensive_20240106_143022.json
│   └── lambdust_benchmark_comprehensive_20240106_143022.csv
├── analysis/
│   ├── analysis_report_20240106_143022.txt
│   ├── analysis_data_20240106_143022.csv
│   └── comparison_report_20240106_143022.txt
├── monitoring/
│   └── performance_monitoring_20240106_143022.json
└── docker-compatible/
    ├── metadata.json
    └── results/
        ├── lambdust_benchmark_comprehensive_20240106_143022.json
        └── analysis_report_20240106_143022.txt
```

### File Descriptions

- **JSON Results**: Complete benchmark data with statistical metrics
- **CSV Data**: Spreadsheet-compatible format for analysis
- **Analysis Reports**: Human-readable performance insights
- **Monitoring Data**: System performance snapshots
- **Docker-Compatible**: Formatted for Docker framework integration

## 🎛️ Configuration Options

### Benchmark Configuration

```rust
PerformanceTestConfig {
    test_duration: Duration::from_secs(5),
    warmup_duration: Duration::from_secs(1),
    micro_bench_iterations: 10000,
    macro_bench_iterations: 1000,
    test_simd_optimizations: true,
    test_memory_pools: true,
    test_environment_optimization: true,
    generate_detailed_reports: true,
}
```

### Quick Mode vs. Comprehensive Mode

| Feature | Quick Mode | Comprehensive Mode |
|---------|------------|-------------------|
| Iterations | 100-1000 | 1000-10000 |
| Test Duration | 1s | 5s |
| Warmup | 100ms | 1s |
| SIMD Testing | ✓ | ✓ |
| Memory Profiling | ❌ | ✓ |
| Detailed Analysis | ❌ | ✓ |

## 🔧 Troubleshooting

### Common Issues

1. **Build Failures**
   ```bash
   # Ensure benchmarks feature is enabled
   cargo build --features benchmarks
   ```

2. **Missing Dependencies**
   ```bash
   # Install required system dependencies
   sudo apt-get install build-essential python3
   ```

3. **Permission Issues**
   ```bash
   # Make scripts executable
   chmod +x *.sh
   ```

4. **Memory Issues**
   ```bash
   # Reduce iterations for memory-constrained systems
   ./run-native-benchmarks.sh --quick
   ```

### Performance Issues

1. **Slow Benchmarks**: Use `--quick` flag for development
2. **High Memory Usage**: Monitor with performance monitor
3. **Inconsistent Results**: Increase warmup iterations
4. **System Load**: Run benchmarks on idle system

## 🤝 Contributing

### Adding New Benchmarks

1. **Create Benchmark Function**
   ```rust
   fn benchmark_new_feature(&self, iterations: usize) -> BenchmarkResult {
       // Implementation
   }
   ```

2. **Add to Category**
   ```rust
   results.push(self.benchmark_new_feature(iterations));
   ```

3. **Update Documentation**
   - Add to category description
   - Update expected performance thresholds

### Benchmark Quality Guidelines

- **Repeatability**: Results should be consistent across runs
- **Isolation**: Each benchmark should test one specific feature
- **Scaling**: Benchmarks should scale with input size appropriately
- **Documentation**: Clear description of what is being measured

## 📚 API Reference

### Core Types

```rust
pub struct BenchmarkResult {
    pub test_name: String,
    pub iterations: usize,
    pub total_time_ms: f64,
    pub ops_per_second: f64,
    pub memory_usage_mb: f64,
    pub throughput_items_per_sec: Option<f64>,
    pub statistical_metrics: StatisticalMetrics,
}

pub struct StatisticalMetrics {
    pub mean_time_ms: f64,
    pub median_time_ms: f64,
    pub std_deviation_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub p95_time_ms: f64,
    pub p99_time_ms: f64,
}
```

### Analysis Functions

```python
# Python analysis API
analyzer = PerformanceAnalyzer()
analysis = analyzer.analyze_single_run("results.json")
comparison = analyzer.compare_benchmark_runs("baseline.json", "current.json")
trends = analyzer.analyze_performance_trends(results_list)
```

## 🏆 Performance Targets

### Target Performance Levels

| Category | Excellent | Good | Fair | Needs Improvement |
|----------|-----------|------|------|-------------------|
| Arithmetic | >1M ops/sec | >500K ops/sec | >100K ops/sec | <100K ops/sec |
| Lists | >100K ops/sec | >50K ops/sec | >10K ops/sec | <10K ops/sec |
| Recursion | >10K ops/sec | >5K ops/sec | >1K ops/sec | <1K ops/sec |
| Memory | >100K ops/sec | >50K ops/sec | >10K ops/sec | <10K ops/sec |
| Functions | >2M ops/sec | >1M ops/sec | >500K ops/sec | <500K ops/sec |

### Competitive Benchmarks

Target performance relative to other Scheme implementations:
- **Racket**: 80% in arithmetic operations
- **Guile**: 90% in list processing
- **Chicken**: 120% memory efficiency (lower is better)

## 📄 License

This benchmarking system is part of the Lambdust project and follows the same licensing terms.

---

## Getting Help

- **Issues**: Report bugs or request features via GitHub Issues
- **Documentation**: See `docs/` directory for detailed guides
- **Community**: Join discussions in project forums
- **Support**: Contact maintainers for enterprise support

**Happy benchmarking! 🎯**