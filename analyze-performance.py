#!/usr/bin/env python3
"""
Performance Analysis Tool for Lambdust Benchmarks

This tool provides comprehensive analysis of benchmark results from both native
and Docker-based benchmark runs. It can compare performance across different
runs, detect regressions, and generate detailed reports.

Usage:
    python analyze-performance.py --file benchmark_results.json
    python analyze-performance.py --compare baseline.json current.json
    python analyze-performance.py --directory benchmark-results/
"""

import json
import argparse
import os
import sys
from datetime import datetime
from typing import Dict, List, Optional, Tuple
import statistics
from pathlib import Path

class PerformanceAnalyzer:
    """Comprehensive performance analysis for Lambdust benchmarks."""
    
    def __init__(self):
        self.results_cache = {}
        self.baseline_data = None
        
    def load_benchmark_results(self, file_path: str) -> Dict:
        """Load benchmark results from JSON file."""
        try:
            with open(file_path, 'r') as f:
                data = json.load(f)
            print(f"✅ Loaded benchmark results from {file_path}")
            return data
        except Exception as e:
            print(f"❌ Error loading {file_path}: {e}")
            return {}
    
    def analyze_single_run(self, results_file: str) -> Dict:
        """Analyze a single benchmark run."""
        data = self.load_benchmark_results(results_file)
        if not data:
            return {}
        
        analysis = {
            'file_path': results_file,
            'timestamp': data.get('timestamp', 'unknown'),
            'overall_score': data.get('overall_summary', {}).get('overall_performance_score', 0),
            'total_tests': data.get('overall_summary', {}).get('total_tests', 0),
            'execution_time': data.get('overall_summary', {}).get('total_execution_time_seconds', 0),
            'categories': {},
            'performance_characteristics': data.get('overall_summary', {}).get('performance_characteristics', {}),
            'strengths': data.get('overall_summary', {}).get('lambdust_strengths', []),
            'recommendations': data.get('performance_recommendations', [])
        }
        
        # Analyze each category
        for category in data.get('categories', []):
            cat_analysis = {
                'name': category.get('name', 'Unknown'),
                'avg_ops_per_sec': category.get('summary', {}).get('avg_ops_per_second', 0),
                'performance_grade': category.get('summary', {}).get('performance_grade', 'Unknown'),
                'test_count': len(category.get('results', [])),
                'detailed_results': []
            }
            
            # Detailed test analysis
            for result in category.get('results', []):
                test_analysis = {
                    'name': result.get('test_name', 'Unknown'),
                    'ops_per_second': result.get('ops_per_second', 0),
                    'total_time_ms': result.get('total_time_ms', 0),
                    'memory_usage_mb': result.get('memory_usage_mb', 0),
                    'iterations': result.get('iterations', 0),
                    'statistical_metrics': result.get('statistical_metrics', {})
                }
                cat_analysis['detailed_results'].append(test_analysis)
            
            analysis['categories'][category.get('name', 'Unknown')] = cat_analysis
        
        return analysis
    
    def compare_benchmark_runs(self, baseline_file: str, current_file: str) -> Dict:
        """Compare two benchmark runs to identify changes and regressions."""
        baseline = self.analyze_single_run(baseline_file)
        current = self.analyze_single_run(current_file)
        
        if not baseline or not current:
            print("❌ Could not load benchmark files for comparison")
            return {}
        
        comparison = {
            'baseline_file': baseline_file,
            'current_file': current_file,
            'baseline_timestamp': baseline.get('timestamp'),
            'current_timestamp': current.get('timestamp'),
            'overall_score_change': current.get('overall_score', 0) - baseline.get('overall_score', 0),
            'execution_time_change': current.get('execution_time', 0) - baseline.get('execution_time', 0),
            'category_comparisons': {},
            'regressions': [],
            'improvements': [],
            'summary': {}
        }
        
        # Compare categories
        for cat_name in set(baseline.get('categories', {}).keys()) | set(current.get('categories', {}).keys()):
            baseline_cat = baseline.get('categories', {}).get(cat_name, {})
            current_cat = current.get('categories', {}).get(cat_name, {})
            
            if not baseline_cat or not current_cat:
                continue
                
            baseline_ops = baseline_cat.get('avg_ops_per_sec', 0)
            current_ops = current_cat.get('avg_ops_per_sec', 0)
            
            if baseline_ops > 0:
                percentage_change = ((current_ops - baseline_ops) / baseline_ops) * 100
            else:
                percentage_change = 0
            
            cat_comparison = {
                'baseline_ops_per_sec': baseline_ops,
                'current_ops_per_sec': current_ops,
                'percentage_change': percentage_change,
                'baseline_grade': baseline_cat.get('performance_grade', 'Unknown'),
                'current_grade': current_cat.get('performance_grade', 'Unknown'),
                'test_comparisons': []
            }
            
            # Compare individual tests
            baseline_tests = {t['name']: t for t in baseline_cat.get('detailed_results', [])}
            current_tests = {t['name']: t for t in current_cat.get('detailed_results', [])}
            
            for test_name in set(baseline_tests.keys()) | set(current_tests.keys()):
                if test_name in baseline_tests and test_name in current_tests:
                    baseline_test = baseline_tests[test_name]
                    current_test = current_tests[test_name]
                    
                    baseline_ops = baseline_test.get('ops_per_second', 0)
                    current_ops = current_test.get('ops_per_second', 0)
                    
                    if baseline_ops > 0:
                        test_change = ((current_ops - baseline_ops) / baseline_ops) * 100
                    else:
                        test_change = 0
                    
                    cat_comparison['test_comparisons'].append({
                        'test_name': test_name,
                        'baseline_ops_per_sec': baseline_ops,
                        'current_ops_per_sec': current_ops,
                        'percentage_change': test_change
                    })
                    
                    # Identify significant changes
                    if test_change < -10:  # More than 10% degradation
                        comparison['regressions'].append({
                            'category': cat_name,
                            'test': test_name,
                            'change': test_change
                        })
                    elif test_change > 10:  # More than 10% improvement
                        comparison['improvements'].append({
                            'category': cat_name,
                            'test': test_name,
                            'change': test_change
                        })
            
            comparison['category_comparisons'][cat_name] = cat_comparison
        
        # Generate summary
        comparison['summary'] = {
            'total_regressions': len(comparison['regressions']),
            'total_improvements': len(comparison['improvements']),
            'overall_trend': 'improvement' if comparison['overall_score_change'] > 0 else 'regression' if comparison['overall_score_change'] < 0 else 'stable',
            'significant_changes': len(comparison['regressions']) + len(comparison['improvements'])
        }
        
        return comparison
    
    def analyze_directory(self, directory: str) -> Dict:
        """Analyze all benchmark results in a directory."""
        results_files = []
        dir_path = Path(directory)
        
        if not dir_path.exists():
            print(f"❌ Directory {directory} does not exist")
            return {}
        
        # Find all JSON files
        for file_path in dir_path.glob("*.json"):
            if "benchmark" in file_path.name.lower():
                results_files.append(str(file_path))
        
        results_files.sort()  # Sort by filename (which often includes timestamp)
        
        if not results_files:
            print(f"❌ No benchmark result files found in {directory}")
            return {}
        
        print(f"📁 Found {len(results_files)} benchmark result files")
        
        analysis = {
            'directory': directory,
            'total_files': len(results_files),
            'files': results_files,
            'trend_analysis': {},
            'performance_evolution': []
        }
        
        # Load all results
        all_results = []
        for file_path in results_files:
            result = self.analyze_single_run(file_path)
            if result:
                all_results.append(result)
        
        # Analyze trends
        if len(all_results) >= 2:
            analysis['trend_analysis'] = self.analyze_performance_trends(all_results)
        
        analysis['performance_evolution'] = all_results
        return analysis
    
    def analyze_performance_trends(self, results_list: List[Dict]) -> Dict:
        """Analyze performance trends across multiple benchmark runs."""
        trends = {
            'overall_score_trend': [],
            'execution_time_trend': [],
            'category_trends': {},
            'regression_periods': [],
            'improvement_periods': []
        }
        
        # Extract overall metrics
        for result in results_list:
            trends['overall_score_trend'].append(result.get('overall_score', 0))
            trends['execution_time_trend'].append(result.get('execution_time', 0))
        
        # Analyze category trends
        all_categories = set()
        for result in results_list:
            all_categories.update(result.get('categories', {}).keys())
        
        for category in all_categories:
            cat_trend = []
            for result in results_list:
                cat_data = result.get('categories', {}).get(category, {})
                cat_trend.append(cat_data.get('avg_ops_per_sec', 0))
            trends['category_trends'][category] = cat_trend
        
        # Detect regression/improvement periods
        for i in range(1, len(results_list)):
            prev_score = results_list[i-1].get('overall_score', 0)
            curr_score = results_list[i].get('overall_score', 0)
            
            if prev_score > 0:
                change = ((curr_score - prev_score) / prev_score) * 100
                if change < -5:  # 5% or more degradation
                    trends['regression_periods'].append({
                        'from_index': i-1,
                        'to_index': i,
                        'change_percentage': change,
                        'from_timestamp': results_list[i-1].get('timestamp'),
                        'to_timestamp': results_list[i].get('timestamp')
                    })
                elif change > 5:  # 5% or more improvement
                    trends['improvement_periods'].append({
                        'from_index': i-1,
                        'to_index': i,
                        'change_percentage': change,
                        'from_timestamp': results_list[i-1].get('timestamp'),
                        'to_timestamp': results_list[i].get('timestamp')
                    })
        
        return trends
    
    def generate_performance_report(self, analysis_data: Dict, report_type: str = "single") -> str:
        """Generate a comprehensive performance report."""
        report = []
        report.append("=" * 60)
        report.append("LAMBDUST PERFORMANCE ANALYSIS REPORT")
        report.append("=" * 60)
        report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")
        
        if report_type == "single":
            return self._generate_single_run_report(analysis_data, report)
        elif report_type == "comparison":
            return self._generate_comparison_report(analysis_data, report)
        elif report_type == "directory":
            return self._generate_directory_report(analysis_data, report)
        else:
            report.append("❌ Unknown report type")
        
        return "\n".join(report)
    
    def _generate_single_run_report(self, analysis: Dict, report: List[str]) -> str:
        """Generate report for single benchmark run."""
        report.append("📊 SINGLE RUN ANALYSIS")
        report.append("-" * 30)
        report.append(f"File: {analysis.get('file_path', 'Unknown')}")
        report.append(f"Timestamp: {analysis.get('timestamp', 'Unknown')}")
        report.append(f"Overall Performance Score: {analysis.get('overall_score', 0):.1f}/100")
        report.append(f"Total Tests: {analysis.get('total_tests', 0)}")
        report.append(f"Execution Time: {analysis.get('execution_time', 0):.2f}s")
        report.append("")
        
        # Category performance
        report.append("📋 CATEGORY PERFORMANCE:")
        for cat_name, cat_data in analysis.get('categories', {}).items():
            report.append(f"  • {cat_name}: {cat_data.get('avg_ops_per_sec', 0):.0f} ops/sec ({cat_data.get('performance_grade', 'Unknown')})")
        report.append("")
        
        # Top performing tests
        report.append("🏆 TOP PERFORMING TESTS:")
        all_tests = []
        for cat_data in analysis.get('categories', {}).values():
            all_tests.extend(cat_data.get('detailed_results', []))
        
        all_tests.sort(key=lambda x: x.get('ops_per_second', 0), reverse=True)
        for test in all_tests[:5]:
            report.append(f"  • {test.get('name', 'Unknown')}: {test.get('ops_per_second', 0):.0f} ops/sec")
        report.append("")
        
        # Recommendations
        recommendations = analysis.get('recommendations', [])
        if recommendations:
            report.append("🔧 OPTIMIZATION RECOMMENDATIONS:")
            for rec in recommendations[:5]:
                report.append(f"  • {rec}")
        
        return "\n".join(report)
    
    def _generate_comparison_report(self, comparison: Dict, report: List[str]) -> str:
        """Generate report comparing two benchmark runs."""
        report.append("⚖️  BENCHMARK COMPARISON ANALYSIS")
        report.append("-" * 40)
        report.append(f"Baseline: {comparison.get('baseline_file', 'Unknown')}")
        report.append(f"Current:  {comparison.get('current_file', 'Unknown')}")
        report.append(f"Overall Score Change: {comparison.get('overall_score_change', 0):+.1f}")
        report.append(f"Execution Time Change: {comparison.get('execution_time_change', 0):+.2f}s")
        report.append("")
        
        # Summary
        summary = comparison.get('summary', {})
        report.append("📈 CHANGE SUMMARY:")
        report.append(f"  • Total Regressions: {summary.get('total_regressions', 0)}")
        report.append(f"  • Total Improvements: {summary.get('total_improvements', 0)}")
        report.append(f"  • Overall Trend: {summary.get('overall_trend', 'unknown').title()}")
        report.append("")
        
        # Regressions
        regressions = comparison.get('regressions', [])
        if regressions:
            report.append("🚨 PERFORMANCE REGRESSIONS:")
            for reg in regressions[:10]:  # Top 10 regressions
                report.append(f"  • {reg.get('category', 'Unknown')}/{reg.get('test', 'Unknown')}: {reg.get('change', 0):.1f}%")
            report.append("")
        
        # Improvements
        improvements = comparison.get('improvements', [])
        if improvements:
            report.append("📈 PERFORMANCE IMPROVEMENTS:")
            for imp in improvements[:10]:  # Top 10 improvements
                report.append(f"  • {imp.get('category', 'Unknown')}/{imp.get('test', 'Unknown')}: {imp.get('change', 0):+.1f}%")
            report.append("")
        
        # Category changes
        report.append("📊 CATEGORY CHANGES:")
        for cat_name, cat_comp in comparison.get('category_comparisons', {}).items():
            change = cat_comp.get('percentage_change', 0)
            trend_symbol = "📈" if change > 0 else "📉" if change < 0 else "➡️"
            report.append(f"  {trend_symbol} {cat_name}: {change:+.1f}% ({cat_comp.get('current_ops_per_sec', 0):.0f} ops/sec)")
        
        return "\n".join(report)
    
    def _generate_directory_report(self, analysis: Dict, report: List[str]) -> str:
        """Generate report for directory analysis."""
        report.append("📁 DIRECTORY ANALYSIS")
        report.append("-" * 25)
        report.append(f"Directory: {analysis.get('directory', 'Unknown')}")
        report.append(f"Total Files: {analysis.get('total_files', 0)}")
        report.append("")
        
        # Trend analysis
        trends = analysis.get('trend_analysis', {})
        if trends:
            report.append("📈 PERFORMANCE TRENDS:")
            
            overall_scores = trends.get('overall_score_trend', [])
            if overall_scores:
                if len(overall_scores) >= 2:
                    trend_direction = "improving" if overall_scores[-1] > overall_scores[0] else "declining"
                    report.append(f"  • Overall Score: {trend_direction} ({overall_scores[0]:.1f} → {overall_scores[-1]:.1f})")
                
            exec_times = trends.get('execution_time_trend', [])
            if exec_times and len(exec_times) >= 2:
                trend_direction = "increasing" if exec_times[-1] > exec_times[0] else "decreasing"
                report.append(f"  • Execution Time: {trend_direction} ({exec_times[0]:.2f}s → {exec_times[-1]:.2f}s)")
            
            report.append("")
            
            # Category trends
            cat_trends = trends.get('category_trends', {})
            if cat_trends:
                report.append("📊 CATEGORY TRENDS:")
                for cat_name, cat_values in cat_trends.items():
                    if len(cat_values) >= 2:
                        trend_direction = "↗️" if cat_values[-1] > cat_values[0] else "↘️"
                        report.append(f"  {trend_direction} {cat_name}: {cat_values[0]:.0f} → {cat_values[-1]:.0f} ops/sec")
                report.append("")
            
            # Regression/improvement periods
            regressions = trends.get('regression_periods', [])
            improvements = trends.get('improvement_periods', [])
            
            if regressions or improvements:
                report.append("🔍 SIGNIFICANT CHANGES:")
                for reg in regressions:
                    report.append(f"  📉 Regression: {reg.get('change_percentage', 0):.1f}% decline")
                for imp in improvements:
                    report.append(f"  📈 Improvement: {imp.get('change_percentage', 0):.1f}% gain")
        
        return "\n".join(report)
    
    def export_to_csv(self, analysis_data: Dict, output_file: str) -> bool:
        """Export analysis data to CSV format for spreadsheet analysis."""
        try:
            import csv
            
            with open(output_file, 'w', newline='') as csvfile:
                if 'categories' in analysis_data:
                    # Single run export
                    writer = csv.writer(csvfile)
                    writer.writerow(['Category', 'Test Name', 'Ops/Second', 'Total Time (ms)', 'Memory (MB)', 'Iterations'])
                    
                    for cat_name, cat_data in analysis_data.get('categories', {}).items():
                        for test in cat_data.get('detailed_results', []):
                            writer.writerow([
                                cat_name,
                                test.get('name', ''),
                                test.get('ops_per_second', 0),
                                test.get('total_time_ms', 0),
                                test.get('memory_usage_mb', 0),
                                test.get('iterations', 0)
                            ])
                
                elif 'category_comparisons' in analysis_data:
                    # Comparison export
                    writer = csv.writer(csvfile)
                    writer.writerow(['Category', 'Test Name', 'Baseline Ops/Sec', 'Current Ops/Sec', 'Change %'])
                    
                    for cat_name, cat_comp in analysis_data.get('category_comparisons', {}).items():
                        for test_comp in cat_comp.get('test_comparisons', []):
                            writer.writerow([
                                cat_name,
                                test_comp.get('test_name', ''),
                                test_comp.get('baseline_ops_per_sec', 0),
                                test_comp.get('current_ops_per_sec', 0),
                                test_comp.get('percentage_change', 0)
                            ])
            
            print(f"✅ Exported analysis to {output_file}")
            return True
            
        except Exception as e:
            print(f"❌ Failed to export to CSV: {e}")
            return False

def main():
    parser = argparse.ArgumentParser(
        description="Comprehensive performance analysis for Lambdust benchmarks",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python analyze-performance.py --file results.json
  python analyze-performance.py --compare baseline.json current.json
  python analyze-performance.py --directory benchmark-results/
  python analyze-performance.py --file results.json --export results.csv
        """
    )
    
    parser.add_argument('--file', '-f', help='Single benchmark result file to analyze')
    parser.add_argument('--compare', '-c', nargs=2, metavar=('BASELINE', 'CURRENT'), 
                       help='Compare two benchmark result files')
    parser.add_argument('--directory', '-d', help='Directory containing benchmark result files')
    parser.add_argument('--export', '-e', help='Export analysis to CSV file')
    parser.add_argument('--output', '-o', help='Save report to file instead of printing')
    
    args = parser.parse_args()
    
    if not any([args.file, args.compare, args.directory]):
        parser.print_help()
        sys.exit(1)
    
    analyzer = PerformanceAnalyzer()
    
    # Perform analysis based on arguments
    if args.file:
        print(f"🔍 Analyzing single benchmark run: {args.file}")
        analysis = analyzer.analyze_single_run(args.file)
        report = analyzer.generate_performance_report(analysis, "single")
        
        if args.export:
            analyzer.export_to_csv(analysis, args.export)
    
    elif args.compare:
        baseline, current = args.compare
        print(f"⚖️  Comparing benchmarks: {baseline} vs {current}")
        analysis = analyzer.compare_benchmark_runs(baseline, current)
        report = analyzer.generate_performance_report(analysis, "comparison")
        
        if args.export:
            analyzer.export_to_csv(analysis, args.export)
    
    elif args.directory:
        print(f"📁 Analyzing directory: {args.directory}")
        analysis = analyzer.analyze_directory(args.directory)
        report = analyzer.generate_performance_report(analysis, "directory")
    
    # Output report
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
        print(f"📄 Report saved to: {args.output}")
    else:
        print("\n" + report)

if __name__ == "__main__":
    main()