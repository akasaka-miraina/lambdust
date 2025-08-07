# Chibi-Scheme Integration Test Suite

A comprehensive R7RS compliance validation system for Lambdust using the Chibi-Scheme test suite as a reference implementation.

## 🎯 Overview

This integration system provides:

- **Automated R7RS Compliance Testing** - Run Chibi-Scheme tests against Lambdust
- **Comprehensive Analysis** - Detailed feature gap identification and prioritization
- **Multi-Format Reporting** - JSON, HTML, Markdown, and CSV reports
- **Implementation Roadmap** - Data-driven development priorities
- **CI/CD Integration** - Automated compliance monitoring

## 🏗️ Architecture

```
tests/chibi_integration/
├── mod.rs                      # Main integration framework
├── test_adapter.rs             # Chibi-Scheme to Lambdust adaptation
├── test_runner.rs              # Test execution engine
├── compliance_analyzer.rs      # R7RS compliance analysis
├── report_generator.rs         # Multi-format report generation
├── simple_test_runner.rs       # Basic test execution demo
├── adapted_tests/              # Converted test files
└── reports/                    # Generated compliance reports
```

### Core Components

#### 1. Test Adapter (`test_adapter.rs`)
Converts Chibi-Scheme tests to Lambdust-compatible format:
- **Import Statement Adaptation** - Maps Chibi imports to Lambdust equivalents
- **Test Framework Conversion** - Converts `(chibi test)` to native format
- **Syntax Compatibility** - Handles R7RS/R5RS syntax differences
- **Extension Handling** - Manages Chibi-specific extensions

#### 2. Test Runner (`test_runner.rs`)
Executes adapted tests with comprehensive error handling:
- **Timeout Management** - Prevents hanging tests
- **Parallel Execution** - Efficient test processing
- **Error Capture** - Detailed failure analysis
- **Performance Metrics** - Execution time tracking

#### 3. Compliance Analyzer (`compliance_analyzer.rs`)
Analyzes results for R7RS compliance assessment:
- **Feature Categorization** - Groups results by R7RS sections
- **Gap Identification** - Finds missing/incomplete features
- **Priority Matrix** - Ranks features by importance
- **Implementation Roadmap** - Phased development plan

#### 4. Report Generator (`report_generator.rs`)
Produces comprehensive compliance reports:
- **HTML Reports** - Interactive dashboards with charts
- **JSON Data** - Machine-readable compliance metrics
- **Markdown Summaries** - Human-readable documentation
- **CSV Exports** - Data analysis and tracking

## 🚀 Quick Start

### Prerequisites

1. **Chibi-Scheme Tests Available**:
   ```bash
   # Tests should be available at /tmp/chibi-scheme/tests/
   ls /tmp/chibi-scheme/tests/
   ```

2. **Lambdust Build Environment**:
   ```bash
   cd /Users/makasaka/lambdust
   cargo build
   ```

### Running Tests

#### Comprehensive Compliance Test
```bash
cargo test comprehensive_r7rs_compliance_test_suite -- --nocapture
```

#### Basic Validation
```bash
cargo test basic_r7rs_compliance_test -- --nocapture
```

#### Component Tests
```bash
# Test adaptation functionality
cargo test test_chibi_test_adaptation

# Test compliance analysis
cargo test test_compliance_analysis

# Test report generation
cargo test test_report_generation
```

### Manual Execution

```rust
use chibi_integration::*;

// Create and configure test suite
let config = ChibiIntegrationConfig {
    chibi_test_path: "/tmp/chibi-scheme/tests".to_string(),
    adapted_test_path: "tests/chibi_integration/adapted_tests".to_string(),
    report_path: "tests/chibi_integration/reports".to_string(),
    include_performance: true,
    detailed_errors: true,
    continue_on_error: true,
    timeout_seconds: 30,
    adapt_extensions: true,
};

// Run comprehensive test suite
let mut suite = ChibiIntegrationSuite::with_config(config);
let results = suite.run_complete_suite()?;

// Generate reports
let analyzer = ComplianceAnalyzer::new();
let analysis = analyzer.analyze_compliance(&results);
let generator = ReportGenerator::with_analysis(&results, analysis);

generator.generate_html_report(&Path::new("compliance_report.html"))?;
generator.generate_json_report(&Path::new("compliance_report.json"))?;
```

## 📊 Report Formats

### HTML Report (`compliance_report.html`)
Interactive dashboard featuring:
- Executive summary with compliance grade
- Test results table with filtering
- Feature coverage analysis
- Implementation recommendations
- Progress charts and visualizations

### JSON Report (`compliance_report.json`)
Machine-readable data including:
```json
{
  "overall_compliance": {
    "overall_percentage": 75.2,
    "feature_completeness": 68.5,
    "total_features": 14,
    "complete_features": 8,
    "weighted_score": 72.1
  },
  "feature_analysis": { /* detailed per-feature results */ },
  "critical_gaps": [ /* prioritized implementation gaps */ ],
  "recommendations": [ /* actionable development priorities */ ]
}
```

### Markdown Summary (`compliance_summary.md`)
Documentation-ready summary:
- Executive overview
- Compliance grade and metrics
- Critical gaps and recommendations
- Implementation roadmap

### CSV Exports
- `test_results.csv` - Individual test outcomes
- `feature_coverage.csv` - Per-feature compliance metrics

## 🎯 Key Features

### R7RS Feature Coverage

The system analyzes compliance across all major R7RS areas:

| Feature Category | Priority | Coverage Analysis |
|-----------------|----------|-------------------|
| **Core Language** | Critical | Basic syntax, evaluation model |
| **Numeric Operations** | Critical | Arithmetic, numeric tower |
| **Lists & Pairs** | Critical | Fundamental data structures |
| **Procedures** | Critical | Lambda, application, closures |
| **Binding Constructs** | High | let, let*, letrec variations |
| **Strings** | High | String manipulation procedures |
| **Input/Output** | High | Port operations, read/write |
| **Vectors** | Medium | Vector operations and procedures |
| **Exception Handling** | Medium | Error handling, guards |
| **Macro System** | Medium | syntax-rules, hygiene |
| **Libraries** | Medium | Module system, imports |
| **Continuations** | Low | call/cc, dynamic wind |
| **Bytevectors** | Low | Binary data operations |
| **Records** | Low | Record type definitions |

### Implementation Roadmap

The system generates a phased implementation plan:

#### Phase 1: Critical Core Features (8 weeks)
- Core language constructs
- Basic arithmetic operations  
- List manipulation
- Essential procedures

#### Phase 2: Standard Library (12 weeks)
- String operations
- I/O procedures
- Vector operations
- Exception handling

#### Phase 3: Advanced Features (6 weeks)
- Macro system
- Continuation support
- Library system
- Performance optimization

### Compliance Grading

| Grade | Percentage | Description |
|-------|------------|-------------|
| **A+** | 95%+ | Excellent - Production ready |
| **A** | 90-94% | Very Good - Minor gaps |
| **B+** | 85-89% | Good - Some work needed |
| **B** | 80-84% | Satisfactory - Moderate gaps |
| **C+** | 70-79% | Needs Work - Significant issues |
| **C** | 60-69% | Major Gaps - Substantial work |
| **D** | <60% | Incomplete - Extensive development |

## 🔧 Configuration Options

### ChibiIntegrationConfig

```rust
pub struct ChibiIntegrationConfig {
    pub chibi_test_path: String,        // Path to Chibi-Scheme tests
    pub adapted_test_path: String,      // Output for adapted tests
    pub report_path: String,            // Report output directory
    pub include_performance: bool,      // Run performance comparisons
    pub detailed_errors: bool,          // Capture detailed error info
    pub continue_on_error: bool,        // Continue after test failures
    pub timeout_seconds: u64,           // Per-test timeout
    pub adapt_extensions: bool,         // Handle Chibi extensions
}
```

### Environment Variables

- `SKIP_CHIBI_TESTS` - Skip integration tests in CI
- `GENERATE_FULL_REPORT` - Enable comprehensive reporting
- `CI` - Detect CI environment for adjusted behavior

## 📈 Performance Integration

The system integrates with Lambdust's existing benchmarking infrastructure:

```rust
// Combined compliance and performance testing
cargo test integration_with_performance_benchmarks -- --ignored --nocapture
```

Performance metrics include:
- Test execution time
- Memory usage patterns
- Throughput measurements
- Comparison with reference implementation

## 🔍 Troubleshooting

### Common Issues

#### Tests Not Found
```
⚠️ Chibi-Scheme tests not found at: /tmp/chibi-scheme/tests/
```
**Solution**: Ensure Chibi-Scheme tests are available at expected location

#### Adaptation Failures
```
❌ Test adaptation failed: Parse error in macro expansion
```
**Solution**: Check `test_adapter.rs` for syntax pattern handling

#### Execution Timeouts
```
⏱️ TIMEOUT (30.00s)
```
**Solution**: Increase `timeout_seconds` or investigate infinite loops

#### Report Generation Errors
```
❌ HTML report generation failed: Permission denied
```
**Solution**: Ensure write permissions for report directory

### Debugging

Enable detailed logging:
```rust
let config = ChibiIntegrationConfig {
    detailed_errors: true,
    // ... other settings
};
```

Check individual test results:
```rust
for result in &test_results {
    if result.status != TestStatus::Passed {
        println!("Failed test: {} - {}", 
                result.test_name, 
                result.error_message.as_deref().unwrap_or("Unknown error"));
    }
}
```

## 🤝 Contributing

### Adding New Test Categories

1. **Identify Test Source**: Locate relevant Chibi-Scheme tests
2. **Create Adapter**: Add adaptation logic in `test_adapter.rs`
3. **Update Runner**: Extend test categorization in `test_runner.rs`
4. **Add Analysis**: Update feature mapping in `compliance_analyzer.rs`
5. **Test Integration**: Verify end-to-end functionality

### Improving Adaptation

1. **Pattern Analysis**: Study Chibi-Scheme syntax differences
2. **Mapping Rules**: Create conversion patterns
3. **Edge Cases**: Handle special syntax constructs
4. **Validation**: Test adapted code for correctness

### Enhancing Reports

1. **Data Collection**: Identify useful metrics
2. **Visualization**: Create charts and graphs
3. **Format Support**: Add new output formats
4. **Interactivity**: Enhance HTML reports

## 📝 License

This integration system is part of the Lambdust project and follows the same licensing terms.

## 🙏 Acknowledgments

- **Chibi-Scheme Project** - For providing comprehensive R7RS test suite
- **R7RS Working Group** - For the Scheme language specification
- **Lambdust Contributors** - For the core language implementation

---

For more information about Lambdust, see the main project documentation.