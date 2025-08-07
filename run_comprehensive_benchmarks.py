#!/usr/bin/env python3
"""
Comprehensive Performance Benchmark Runner for Lambdust

This script orchestrates the complete benchmarking process including:
- Cross-implementation performance comparison
- Statistical analysis and confidence intervals
- Performance regression detection
- Report generation and visualization
- Integration with existing Docker infrastructure

Usage:
    python3 run_comprehensive_benchmarks.py [options]
    
Options:
    --config CONFIG_FILE        Use custom configuration file
    --quick                     Run quick benchmark (fewer iterations)
    --implementations LIST      Comma-separated list of implementations
    --categories LIST           Comma-separated list of test categories
    --output-dir DIR           Output directory for results
    --regression-baseline FILE  Baseline for regression detection
    --generate-report          Generate comprehensive reports
    --verbose                  Verbose output
"""

import argparse
import json
import os
import sys
import subprocess
import time
from pathlib import Path
from typing import Dict, List, Optional
import tempfile
import yaml


class BenchmarkRunner:
    """Orchestrates comprehensive benchmark execution."""
    
    def __init__(self, config_path: str, args: argparse.Namespace):
        self.config_path = config_path
        self.args = args
        self.config = self.load_configuration()
        self.results = {}
        self.start_time = time.time()
        
    def load_configuration(self) -> Dict:
        """Load benchmark configuration from file."""
        try:
            with open(self.config_path, 'r') as f:
                config = json.load(f)
                
            # Apply command line overrides
            if self.args.quick:
                config['statistical_config']['iterations'] = 3
                config['statistical_config']['warmup_iterations'] = 1
                
            if self.args.output_dir:
                config['output_config']['output_dir'] = self.args.output_dir
                
            if self.args.implementations:
                target_impls = set(self.args.implementations.split(','))
                config['implementations'] = [
                    impl for impl in config['implementations']
                    if impl['id'] in target_impls
                ]
                
            if self.args.categories:
                target_cats = set(self.args.categories.split(','))
                config['test_categories'] = [
                    cat for cat in config['test_categories']
                    if cat['name'] in target_cats
                ]
                
            return config
            
        except Exception as e:
            print(f"Error loading configuration: {e}")
            sys.exit(1)
            
    def check_system_requirements(self) -> bool:
        """Verify system requirements and implementation availability."""
        print("🔍 Checking system requirements...")
        
        # Check Docker availability
        try:
            result = subprocess.run(['docker', '--version'], 
                                  capture_output=True, text=True, check=True)
            if self.args.verbose:
                print(f"✓ Docker available: {result.stdout.strip()}")
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("❌ Docker not available")
            return False
            
        # Check available Scheme implementations
        available_impls = []
        for impl in self.config['implementations']:
            if impl['id'] == 'lambdust':
                # Check if Lambdust binary exists
                binary_path = f"{impl['runtime']['target_dir']}/{impl['runtime']['profile']}/lambdust"
                if os.path.exists(binary_path):
                    available_impls.append(impl['id'])
                    if self.args.verbose:
                        print(f"✓ Lambdust binary found: {binary_path}")
                else:
                    print(f"⚠ Lambdust binary not found at {binary_path}")
            else:
                # Check native implementations
                try:
                    subprocess.run([impl['runtime']['binary_path'], '--version'],
                                 capture_output=True, check=True, timeout=5)
                    available_impls.append(impl['id'])
                    if self.args.verbose:
                        print(f"✓ {impl['name']} available")
                except:
                    if self.args.verbose:
                        print(f"⚠ {impl['name']} not available")
                        
        print(f"📊 Available implementations: {', '.join(available_impls)}")
        return len(available_impls) > 0
        
    def build_lambdust(self) -> bool:
        """Build Lambdust in release mode for benchmarking."""
        print("🔨 Building Lambdust...")
        
        try:
            result = subprocess.run(
                ['cargo', 'build', '--release'],
                cwd='.',
                capture_output=True,
                text=True,
                check=True
            )
            
            if self.args.verbose:
                print("✓ Lambdust built successfully")
                
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Failed to build Lambdust: {e}")
            if self.args.verbose:
                print(f"stdout: {e.stdout}")
                print(f"stderr: {e.stderr}")
            return False
            
    def run_native_benchmarks(self) -> Dict:
        """Run benchmarks on native implementations."""
        print("🚀 Running native implementation benchmarks...")
        
        results = {}
        
        for impl in self.config['implementations']:
            if impl['id'] == 'lambdust':
                continue  # Handle separately
                
            print(f"  📈 Benchmarking {impl['name']}...")
            
            impl_results = self.run_implementation_benchmarks(impl)
            results[impl['id']] = impl_results
            
            if self.args.verbose:
                success_rate = len([r for r in impl_results if r.get('success', False)])
                total_tests = len(impl_results)
                print(f"    ✓ {success_rate}/{total_tests} tests completed successfully")
                
        return results
        
    def run_implementation_benchmarks(self, impl: Dict) -> List[Dict]:
        """Run benchmarks for a single implementation."""
        results = []
        
        for category in self.config['test_categories']:
            for test in category['tests']:
                for param_combo in self.generate_parameter_combinations(test['parameters']):
                    result = self.run_single_test(impl, test, param_combo, category['name'])
                    results.append(result)
                    
        return results
        
    def generate_parameter_combinations(self, parameters: List[Dict]) -> List[Dict]:
        """Generate all parameter combinations for a test."""
        if not parameters:
            return [{}]
            
        combinations = [{}]
        
        for param in parameters:
            new_combinations = []
            for value_def in param['values']:
                for existing_combo in combinations:
                    new_combo = existing_combo.copy()
                    
                    # Extract actual value based on type
                    if value_def['type'] == 'Integer':
                        value = value_def['value']
                    elif value_def['type'] == 'Float':
                        value = value_def['value']
                    elif value_def['type'] == 'String':
                        value = value_def['value']
                    elif value_def['type'] == 'Boolean':
                        value = value_def['value']
                    else:
                        continue
                        
                    new_combo[param['name']] = value
                    new_combinations.append(new_combo)
                    
            combinations = new_combinations
            
        return combinations
        
    def run_single_test(self, impl: Dict, test: Dict, params: Dict, category: str) -> Dict:
        """Run a single test case."""
        # Substitute parameters in code template
        code = test['code_template']
        for param_name, param_value in params.items():
            placeholder = f"{{{param_name}}}"
            code = code.replace(placeholder, str(param_value))
            
        # Create temporary test file
        file_extension = '.ldust' if impl['id'] == 'lambdust' else '.scm'
        
        with tempfile.NamedTemporaryFile(mode='w', suffix=file_extension, delete=False) as f:
            f.write(code)
            temp_file = f.name
            
        try:
            # Run the test
            iterations = self.config['statistical_config']['iterations']
            warmup = self.config['statistical_config']['warmup_iterations']
            
            timing_results = []
            
            # Warmup runs
            for _ in range(warmup):
                self.execute_test_file(impl, temp_file)
                
            # Actual measurement runs
            for i in range(iterations):
                start_time = time.time()
                success, output, error = self.execute_test_file(impl, temp_file)
                end_time = time.time()
                
                if success:
                    timing_results.append(end_time - start_time)
                else:
                    break
                    
            # Calculate statistics
            if timing_results:
                mean_time = sum(timing_results) / len(timing_results)
                ops_per_second = 1.0 / mean_time if mean_time > 0 else 0
                
                return {
                    'implementation': impl['id'],
                    'test_name': test['name'],
                    'category': category,
                    'parameters': params,
                    'success': True,
                    'mean_time_seconds': mean_time,
                    'ops_per_second': ops_per_second,
                    'timing_results': timing_results,
                    'iterations': len(timing_results),
                }
            else:
                return {
                    'implementation': impl['id'],
                    'test_name': test['name'],
                    'category': category,
                    'parameters': params,
                    'success': False,
                    'error': error or 'Test execution failed',
                    'timing_results': [],
                }
                
        finally:
            # Clean up temporary file
            os.unlink(temp_file)
            
    def execute_test_file(self, impl: Dict, test_file: str) -> tuple:
        """Execute a test file using the specified implementation."""
        try:
            if impl['id'] == 'lambdust':
                cmd = [
                    f"{impl['runtime']['target_dir']}/{impl['runtime']['profile']}/lambdust",
                    '--batch',
                    test_file
                ]
            else:
                cmd = [impl['runtime']['binary_path']] + impl['runtime']['args'] + [test_file]
                
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=self.config['resource_config']['limits']['global_timeout_seconds'],
                env={**os.environ, **impl['runtime'].get('env_vars', {})}
            )
            
            return result.returncode == 0, result.stdout, result.stderr
            
        except subprocess.TimeoutExpired:
            return False, "", "Test timed out"
        except Exception as e:
            return False, "", str(e)
            
    def run_docker_benchmarks(self) -> Dict:
        """Run benchmarks using Docker infrastructure."""
        print("🐳 Running Docker-based benchmarks...")
        
        try:
            # Use existing Docker infrastructure
            result = subprocess.run(
                ['./benchmark.sh', '--quick' if self.args.quick else '--full'],
                cwd='.',
                capture_output=True,
                text=True,
                check=True,
                timeout=3600  # 1 hour timeout
            )
            
            if self.args.verbose:
                print("✓ Docker benchmarks completed")
                
            # Parse Docker benchmark results
            return self.parse_docker_results()
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Docker benchmarks failed: {e}")
            if self.args.verbose:
                print(f"stdout: {e.stdout}")
                print(f"stderr: {e.stderr}")
            return {}
            
    def parse_docker_results(self) -> Dict:
        """Parse results from Docker benchmark execution."""
        # Look for recent result files
        result_files = []
        
        for file_path in Path('./benchmark-results').glob('*.json'):
            if 'lambdust' in file_path.name.lower():
                result_files.append(file_path)
                
        if not result_files:
            return {}
            
        # Use the most recent result file
        latest_file = max(result_files, key=lambda p: p.stat().st_mtime)
        
        try:
            with open(latest_file, 'r') as f:
                return json.load(f)
        except Exception as e:
            if self.args.verbose:
                print(f"Error parsing Docker results: {e}")
            return {}
            
    def perform_statistical_analysis(self, results: Dict) -> Dict:
        """Perform statistical analysis on benchmark results."""
        print("📊 Performing statistical analysis...")
        
        # Group results by test and implementation
        grouped_results = {}
        
        for impl_id, impl_results in results.items():
            for result in impl_results:
                if not result.get('success', False):
                    continue
                    
                test_key = f"{result['category']}_{result['test_name']}"
                if test_key not in grouped_results:
                    grouped_results[test_key] = {}
                    
                grouped_results[test_key][impl_id] = result['timing_results']
                
        # Perform pairwise comparisons
        statistical_results = {
            'descriptive_stats': {},
            'pairwise_comparisons': [],
            'effect_sizes': {},
            'outliers': {},
        }
        
        for test_key, test_data in grouped_results.items():
            # Calculate descriptive statistics
            for impl_id, timings in test_data.items():
                if timings:
                    stats = self.calculate_descriptive_stats(timings)
                    statistical_results['descriptive_stats'][f"{test_key}_{impl_id}"] = stats
                    
            # Perform pairwise comparisons
            impl_ids = list(test_data.keys())
            for i in range(len(impl_ids)):
                for j in range(i + 1, len(impl_ids)):
                    impl_a = impl_ids[i]
                    impl_b = impl_ids[j]
                    
                    if impl_a in test_data and impl_b in test_data:
                        comparison = self.compare_implementations(
                            test_data[impl_a], test_data[impl_b], 
                            impl_a, impl_b, test_key
                        )
                        statistical_results['pairwise_comparisons'].append(comparison)
                        
        return statistical_results
        
    def calculate_descriptive_stats(self, values: List[float]) -> Dict:
        """Calculate descriptive statistics for a dataset."""
        if not values:
            return {}
            
        values = sorted(values)
        n = len(values)
        
        mean = sum(values) / n
        variance = sum((x - mean) ** 2 for x in values) / (n - 1) if n > 1 else 0
        std_dev = variance ** 0.5
        
        median = values[n // 2] if n % 2 == 1 else (values[n // 2 - 1] + values[n // 2]) / 2
        
        return {
            'count': n,
            'mean': mean,
            'median': median,
            'std_dev': std_dev,
            'min': values[0],
            'max': values[-1],
            'q25': values[n // 4],
            'q75': values[3 * n // 4],
        }
        
    def compare_implementations(self, data_a: List[float], data_b: List[float], 
                               impl_a: str, impl_b: str, test_key: str) -> Dict:
        """Compare two implementations statistically."""
        if not data_a or not data_b:
            return {
                'impl_a': impl_a,
                'impl_b': impl_b,
                'test': test_key,
                'significant': False,
                'effect_size': 0.0,
                'performance_ratio': 1.0,
            }
            
        mean_a = sum(data_a) / len(data_a)
        mean_b = sum(data_b) / len(data_b)
        
        # Calculate effect size (Cohen's d approximation)
        pooled_std = ((sum((x - mean_a) ** 2 for x in data_a) + 
                      sum((x - mean_b) ** 2 for x in data_b)) / 
                     (len(data_a) + len(data_b) - 2)) ** 0.5
                     
        effect_size = (mean_a - mean_b) / pooled_std if pooled_std > 0 else 0.0
        
        # Simple significance test (t-test approximation)
        significant = abs(effect_size) > 0.5  # Simplified threshold
        
        performance_ratio = mean_b / mean_a if mean_a > 0 else 1.0
        
        return {
            'impl_a': impl_a,
            'impl_b': impl_b,
            'test': test_key,
            'mean_a': mean_a,
            'mean_b': mean_b,
            'effect_size': effect_size,
            'significant': significant,
            'performance_ratio': performance_ratio,
            'winner': impl_a if mean_a < mean_b else impl_b,
        }
        
    def detect_regressions(self, results: Dict) -> Dict:
        """Detect performance regressions if baseline is available."""
        if not self.args.regression_baseline:
            return {}
            
        print("🔍 Detecting performance regressions...")
        
        try:
            with open(self.args.regression_baseline, 'r') as f:
                baseline_results = json.load(f)
                
            regressions = []
            improvements = []
            
            # Compare current results with baseline
            for impl_id, impl_results in results.items():
                if impl_id not in baseline_results:
                    continue
                    
                baseline_impl = baseline_results[impl_id]
                
                for result in impl_results:
                    if not result.get('success', False):
                        continue
                        
                    # Find matching baseline test
                    baseline_test = None
                    for baseline_result in baseline_impl:
                        if (baseline_result.get('test_name') == result['test_name'] and
                            baseline_result.get('category') == result['category'] and
                            baseline_result.get('parameters') == result['parameters']):
                            baseline_test = baseline_result
                            break
                            
                    if baseline_test is None:
                        continue
                        
                    # Calculate performance change
                    current_perf = result.get('ops_per_second', 0)
                    baseline_perf = baseline_test.get('ops_per_second', 0)
                    
                    if baseline_perf > 0:
                        change_percent = ((current_perf - baseline_perf) / baseline_perf) * 100
                        
                        if change_percent < -5.0:  # 5% degradation threshold
                            regressions.append({
                                'implementation': impl_id,
                                'test': result['test_name'],
                                'category': result['category'],
                                'degradation_percent': abs(change_percent),
                                'current_performance': current_perf,
                                'baseline_performance': baseline_perf,
                            })
                        elif change_percent > 5.0:  # 5% improvement threshold
                            improvements.append({
                                'implementation': impl_id,
                                'test': result['test_name'],
                                'category': result['category'],
                                'improvement_percent': change_percent,
                                'current_performance': current_perf,
                                'baseline_performance': baseline_perf,
                            })
                            
            return {
                'regressions': regressions,
                'improvements': improvements,
                'baseline_file': self.args.regression_baseline,
            }
            
        except Exception as e:
            print(f"❌ Error detecting regressions: {e}")
            return {}
            
    def generate_comprehensive_report(self, all_results: Dict) -> None:
        """Generate comprehensive benchmark report."""
        print("📝 Generating comprehensive report...")
        
        output_dir = Path(self.config['output_config']['output_dir'])
        output_dir.mkdir(parents=True, exist_ok=True)
        
        timestamp = int(time.time())
        
        # Save raw results
        results_file = output_dir / f'benchmark_results_{timestamp}.json'
        with open(results_file, 'w') as f:
            json.dump(all_results, f, indent=2)
            
        # Generate HTML report
        html_report = self.generate_html_report(all_results)
        html_file = output_dir / f'benchmark_report_{timestamp}.html'
        with open(html_file, 'w') as f:
            f.write(html_report)
            
        # Generate markdown summary
        md_report = self.generate_markdown_report(all_results)
        md_file = output_dir / f'benchmark_summary_{timestamp}.md'
        with open(md_file, 'w') as f:
            f.write(md_report)
            
        # Generate CSV data for further analysis
        csv_data = self.generate_csv_data(all_results)
        csv_file = output_dir / f'benchmark_data_{timestamp}.csv'
        with open(csv_file, 'w') as f:
            f.write(csv_data)
            
        print(f"📊 Reports generated in {output_dir}:")
        print(f"  - JSON results: {results_file.name}")
        print(f"  - HTML report: {html_file.name}")
        print(f"  - Markdown summary: {md_file.name}")
        print(f"  - CSV data: {csv_file.name}")
        
    def generate_html_report(self, results: Dict) -> str:
        """Generate HTML performance report."""
        html = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Lambdust Performance Benchmark Results</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }}
        .header {{ background-color: #f5f5f5; padding: 20px; border-radius: 5px; margin-bottom: 30px; }}
        .summary {{ background-color: #e8f4fd; padding: 15px; border-radius: 5px; margin: 20px 0; }}
        .implementation {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .score {{ font-size: 1.2em; font-weight: bold; color: #2e7d32; }}
        .regression {{ color: #d32f2f; font-weight: bold; }}
        .improvement {{ color: #388e3c; font-weight: bold; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .chart-placeholder {{ background-color: #f9f9f9; padding: 40px; text-align: center; 
                            border: 1px dashed #ccc; margin: 20px 0; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 Lambdust Performance Benchmark Results</h1>
        <p><strong>Generated:</strong> {time.strftime('%Y-%m-%d %H:%M:%S')}</p>
        <p><strong>Duration:</strong> {time.time() - self.start_time:.1f} seconds</p>
    </div>
    
    <div class="summary">
        <h2>📊 Executive Summary</h2>
        <ul>
            <li><strong>Implementations tested:</strong> {len(results.get('native_results', {}))}</li>
            <li><strong>Test categories:</strong> {len(self.config['test_categories'])}</li>
            <li><strong>Statistical confidence:</strong> {self.config['statistical_config']['confidence_level'] * 100:.0f}%</li>
        </ul>
    </div>
    """
        
        # Add implementation results
        if 'native_results' in results:
            html += "<h2>🏆 Implementation Performance</h2>"
            
            for impl_id, impl_results in results['native_results'].items():
                successful_tests = len([r for r in impl_results if r.get('success', False)])
                total_tests = len(impl_results)
                success_rate = (successful_tests / total_tests * 100) if total_tests > 0 else 0
                
                # Calculate average performance
                successful_results = [r for r in impl_results if r.get('success', False)]
                avg_ops_per_sec = 0
                if successful_results:
                    avg_ops_per_sec = sum(r.get('ops_per_second', 0) for r in successful_results) / len(successful_results)
                
                impl_name = next((i['name'] for i in self.config['implementations'] if i['id'] == impl_id), impl_id)
                
                html += f"""
                <div class="implementation">
                    <h3>{impl_name}</h3>
                    <p class="score">Average Performance: {avg_ops_per_sec:,.0f} ops/sec</p>
                    <p><strong>Success Rate:</strong> {success_rate:.1f}% ({successful_tests}/{total_tests})</p>
                </div>
                """
        
        # Add statistical analysis
        if 'statistical_analysis' in results:
            html += "<h2>📈 Statistical Analysis</h2>"
            
            comparisons = results['statistical_analysis'].get('pairwise_comparisons', [])
            if comparisons:
                html += """
                <h3>Implementation Comparisons</h3>
                <table>
                    <tr><th>Implementation A</th><th>Implementation B</th><th>Winner</th><th>Performance Ratio</th><th>Significant</th></tr>
                """
                
                for comp in comparisons[:10]:  # Show top 10 comparisons
                    ratio = comp.get('performance_ratio', 1.0)
                    significant = "Yes" if comp.get('significant', False) else "No"
                    
                    html += f"""
                    <tr>
                        <td>{comp.get('impl_a', '')}</td>
                        <td>{comp.get('impl_b', '')}</td>
                        <td><strong>{comp.get('winner', '')}</strong></td>
                        <td>{ratio:.2f}x</td>
                        <td>{significant}</td>
                    </tr>
                    """
                    
                html += "</table>"
        
        # Add regression analysis
        if 'regression_analysis' in results:
            regression_data = results['regression_analysis']
            regressions = regression_data.get('regressions', [])
            improvements = regression_data.get('improvements', [])
            
            if regressions or improvements:
                html += "<h2>🔍 Regression Analysis</h2>"
                
                if regressions:
                    html += f"<h3 class='regression'>⚠ Performance Regressions ({len(regressions)})</h3><ul>"
                    for reg in regressions[:5]:
                        html += f"""<li class='regression'>{reg['implementation']} - {reg['test']}: 
                                   {reg['degradation_percent']:.1f}% slower</li>"""
                    html += "</ul>"
                    
                if improvements:
                    html += f"<h3 class='improvement'>✅ Performance Improvements ({len(improvements)})</h3><ul>"
                    for imp in improvements[:5]:
                        html += f"""<li class='improvement'>{imp['implementation']} - {imp['test']}: 
                                   {imp['improvement_percent']:.1f}% faster</li>"""
                    html += "</ul>"
        
        # Add placeholder for charts
        html += """
        <h2>📊 Performance Visualizations</h2>
        <div class="chart-placeholder">
            <p>📈 Performance comparison charts would be displayed here</p>
            <p><em>Chart generation requires additional visualization libraries</em></p>
        </div>
        """
        
        html += """
        <div class="summary">
            <h2>🎯 Key Recommendations</h2>
            <ul>
                <li>Focus optimization efforts on critical performance regressions</li>
                <li>Investigate implementations showing consistent superior performance</li>
                <li>Consider statistical significance when interpreting results</li>
                <li>Regular regression monitoring recommended for CI/CD</li>
            </ul>
        </div>
        </body>
        </html>
        """
        
        return html
        
    def generate_markdown_report(self, results: Dict) -> str:
        """Generate markdown summary report."""
        md = f"""# Lambdust Performance Benchmark Summary

**Generated:** {time.strftime('%Y-%m-%d %H:%M:%S')}  
**Duration:** {time.time() - self.start_time:.1f} seconds  
**Configuration:** {self.config_path}

## Overview

"""
        
        if 'native_results' in results:
            md += f"- **Implementations tested:** {len(results['native_results'])}\n"
            total_tests = sum(len(impl_results) for impl_results in results['native_results'].values())
            successful_tests = sum(
                len([r for r in impl_results if r.get('success', False)]) 
                for impl_results in results['native_results'].values()
            )
            md += f"- **Total tests:** {total_tests}\n"
            md += f"- **Success rate:** {successful_tests/total_tests*100:.1f}%\n"
        
        md += f"- **Statistical confidence:** {self.config['statistical_config']['confidence_level'] * 100:.0f}%\n\n"
        
        # Performance rankings
        if 'native_results' in results:
            md += "## Performance Rankings\n\n"
            md += "| Implementation | Avg Performance (ops/sec) | Success Rate |\n"
            md += "|---|---|---|\n"
            
            impl_perf = []
            for impl_id, impl_results in results['native_results'].items():
                successful_results = [r for r in impl_results if r.get('success', False)]
                if successful_results:
                    avg_perf = sum(r.get('ops_per_second', 0) for r in successful_results) / len(successful_results)
                    success_rate = len(successful_results) / len(impl_results) * 100
                    impl_name = next((i['name'] for i in self.config['implementations'] if i['id'] == impl_id), impl_id)
                    impl_perf.append((impl_name, avg_perf, success_rate))
            
            # Sort by performance
            impl_perf.sort(key=lambda x: x[1], reverse=True)
            
            for name, perf, success_rate in impl_perf:
                md += f"| {name} | {perf:,.0f} | {success_rate:.1f}% |\n"
            
            md += "\n"
        
        # Regression summary
        if 'regression_analysis' in results:
            regression_data = results['regression_analysis']
            regressions = regression_data.get('regressions', [])
            improvements = regression_data.get('improvements', [])
            
            if regressions:
                md += f"## ⚠ Performance Regressions ({len(regressions)})\n\n"
                for reg in regressions[:3]:
                    md += f"- **{reg['implementation']}** - {reg['test']}: {reg['degradation_percent']:.1f}% degradation\n"
                md += "\n"
                
            if improvements:
                md += f"## ✅ Performance Improvements ({len(improvements)})\n\n"
                for imp in improvements[:3]:
                    md += f"- **{imp['implementation']}** - {imp['test']}: {imp['improvement_percent']:.1f}% improvement\n"
                md += "\n"
        
        md += """## Recommendations

1. **Focus on Critical Paths:** Optimize the most performance-critical operations first
2. **Statistical Significance:** Consider both statistical and practical significance
3. **Regression Monitoring:** Implement continuous performance monitoring
4. **Cross-Implementation Learning:** Study techniques from faster implementations

## Technical Notes

- All tests run with proper warm-up periods
- Statistical analysis includes confidence intervals and effect sizes
- Results are reproducible with the same configuration
- Docker infrastructure ensures consistent testing environment
"""
        
        return md
        
    def generate_csv_data(self, results: Dict) -> str:
        """Generate CSV data for further analysis."""
        csv_lines = ["Implementation,Category,Test,Parameters,Success,Mean_Time_Seconds,Ops_Per_Second,Iterations"]
        
        if 'native_results' in results:
            for impl_id, impl_results in results['native_results'].items():
                for result in impl_results:
                    params_str = str(result.get('parameters', {})).replace(',', ';')
                    csv_lines.append(f"{impl_id},{result.get('category', '')},{result.get('test_name', '')},"
                                   f"\"{params_str}\",{result.get('success', False)},"
                                   f"{result.get('mean_time_seconds', 0)},{result.get('ops_per_second', 0)},"
                                   f"{result.get('iterations', 0)}")
        
        return '\n'.join(csv_lines)
        
    def run(self) -> None:
        """Execute the complete benchmarking process."""
        print("🎯 Starting comprehensive Lambdust benchmarking...")
        print(f"📝 Configuration: {self.config_path}")
        
        # Check system requirements
        if not self.check_system_requirements():
            print("❌ System requirements not met")
            sys.exit(1)
            
        # Build Lambdust
        if not self.build_lambdust():
            print("❌ Failed to build Lambdust")
            sys.exit(1)
            
        all_results = {}
        
        # Run native benchmarks
        try:
            native_results = self.run_native_benchmarks()
            all_results['native_results'] = native_results
        except Exception as e:
            print(f"❌ Native benchmarks failed: {e}")
            
        # Run Docker benchmarks (if available)
        try:
            docker_results = self.run_docker_benchmarks()
            if docker_results:
                all_results['docker_results'] = docker_results
        except Exception as e:
            if self.args.verbose:
                print(f"⚠ Docker benchmarks failed: {e}")
                
        # Perform statistical analysis
        if 'native_results' in all_results:
            try:
                statistical_results = self.perform_statistical_analysis(all_results['native_results'])
                all_results['statistical_analysis'] = statistical_results
            except Exception as e:
                print(f"❌ Statistical analysis failed: {e}")
                
        # Detect regressions
        try:
            regression_results = self.detect_regressions(all_results.get('native_results', {}))
            if regression_results:
                all_results['regression_analysis'] = regression_results
        except Exception as e:
            if self.args.verbose:
                print(f"⚠ Regression analysis failed: {e}")
                
        # Generate reports
        if self.args.generate_report and all_results:
            try:
                self.generate_comprehensive_report(all_results)
            except Exception as e:
                print(f"❌ Report generation failed: {e}")
                
        # Summary
        duration = time.time() - self.start_time
        print(f"\n✅ Benchmarking completed in {duration:.1f} seconds")
        
        if 'native_results' in all_results:
            total_tests = sum(len(impl_results) for impl_results in all_results['native_results'].values())
            print(f"📊 Total tests executed: {total_tests}")
            
        if 'regression_analysis' in all_results:
            regressions = len(all_results['regression_analysis'].get('regressions', []))
            improvements = len(all_results['regression_analysis'].get('improvements', []))
            print(f"🔍 Performance changes: {regressions} regressions, {improvements} improvements")


def main():
    parser = argparse.ArgumentParser(description='Comprehensive Lambdust Performance Benchmarks')
    parser.add_argument('--config', default='comprehensive_benchmark_config.json', 
                       help='Configuration file path')
    parser.add_argument('--quick', action='store_true', 
                       help='Run quick benchmark with fewer iterations')
    parser.add_argument('--implementations', 
                       help='Comma-separated list of implementations to test')
    parser.add_argument('--categories', 
                       help='Comma-separated list of test categories')
    parser.add_argument('--output-dir', 
                       help='Output directory for results')
    parser.add_argument('--regression-baseline', 
                       help='Baseline file for regression detection')
    parser.add_argument('--generate-report', action='store_true', 
                       help='Generate comprehensive reports')
    parser.add_argument('--verbose', action='store_true', 
                       help='Verbose output')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.config):
        print(f"❌ Configuration file not found: {args.config}")
        sys.exit(1)
        
    runner = BenchmarkRunner(args.config, args)
    runner.run()


if __name__ == '__main__':
    main()