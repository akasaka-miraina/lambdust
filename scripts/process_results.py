#!/usr/bin/env python3

"""
Benchmark Results Processor

Processes raw benchmark results and generates statistical analysis
and comparison data for Scheme implementations.
"""

import os
import json
import csv
import yaml
import argparse
import statistics
from pathlib import Path
from typing import Dict, List, Any, Optional
from collections import defaultdict


class BenchmarkProcessor:
    """Processes benchmark results and generates analysis."""
    
    def __init__(self, config_file: str):
        """Initialize with configuration."""
        with open(config_file, 'r') as f:
            self.config = yaml.safe_load(f)
        
        self.implementations = self.config['implementations']
        self.benchmark_suites = self.config['benchmark_suites']
        self.output_config = self.config.get('output', {})
    
    def process_all_results(self, input_dir: Path, output_dir: Path):
        """Process all benchmark results."""
        
        # Create output directories
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Load and organize raw results
        raw_results = self.load_raw_results(input_dir)
        
        if not raw_results:
            print("No raw results found to process")
            return
        
        print(f"Processing {len(raw_results)} raw result files...")
        
        # Organize results by test and implementation
        organized_results = self.organize_results(raw_results)
        
        # Generate statistical analysis
        stats_results = self.generate_statistics(organized_results)
        
        # Generate comparison analysis
        comparison_results = self.generate_comparisons(stats_results)
        
        # Save processed results
        self.save_results(output_dir, {
            'organized': organized_results,
            'statistics': stats_results,
            'comparisons': comparison_results
        })
        
        print("Results processing completed!")
    
    def load_raw_results(self, input_dir: Path) -> List[Dict[str, Any]]:
        """Load all raw result files."""
        
        raw_results = []
        
        for result_file in input_dir.glob('*.json'):
            try:
                with open(result_file, 'r') as f:
                    result = json.load(f)
                    result['source_file'] = str(result_file)
                    raw_results.append(result)
            except (json.JSONDecodeError, IOError) as e:
                print(f"Warning: Could not load {result_file}: {e}")
        
        return raw_results
    
    def organize_results(self, raw_results: List[Dict[str, Any]]) -> Dict[str, Dict[str, List[Dict[str, Any]]]]:
        """Organize results by test and implementation."""
        
        organized = defaultdict(lambda: defaultdict(list))
        
        for result in raw_results:
            if not result.get('success', False):
                continue
                
            test_name = result.get('test', 'unknown')
            impl_name = result.get('implementation', 'unknown')
            
            organized[test_name][impl_name].append(result)
        
        return dict(organized)
    
    def generate_statistics(self, organized_results: Dict[str, Dict[str, List[Dict[str, Any]]]]) -> Dict[str, Dict[str, Dict[str, float]]]:
        """Generate statistical analysis of results."""
        
        stats = {}
        
        for test_name, test_results in organized_results.items():
            stats[test_name] = {}
            
            for impl_name, impl_results in test_results.items():
                if not impl_results:
                    continue
                
                # Extract timing data
                durations = [r.get('duration', 0) for r in impl_results if r.get('duration') is not None]
                user_times = [r.get('user_time', 0) for r in impl_results if r.get('user_time') is not None]
                memory_usage = [r.get('max_memory_kb', 0) for r in impl_results if r.get('max_memory_kb') is not None]
                
                if not durations:
                    continue
                
                # Calculate statistics
                impl_stats = {
                    'count': len(durations),
                    'duration_mean': statistics.mean(durations),
                    'duration_median': statistics.median(durations),
                    'duration_stdev': statistics.stdev(durations) if len(durations) > 1 else 0.0,
                    'duration_min': min(durations),
                    'duration_max': max(durations),
                }
                
                # Add percentiles if we have enough data
                if len(durations) >= 4:
                    impl_stats.update({
                        'duration_p25': statistics.quantiles(durations, n=4)[0],
                        'duration_p75': statistics.quantiles(durations, n=4)[2],
                    })
                
                if len(durations) >= 10:
                    impl_stats.update({
                        'duration_p90': statistics.quantiles(durations, n=10)[8],
                        'duration_p95': statistics.quantiles(durations, n=20)[18],
                        'duration_p99': statistics.quantiles(durations, n=100)[98],
                    })
                
                # User time statistics
                if user_times:
                    impl_stats.update({
                        'user_time_mean': statistics.mean(user_times),
                        'user_time_median': statistics.median(user_times),
                        'user_time_stdev': statistics.stdev(user_times) if len(user_times) > 1 else 0.0,
                    })
                
                # Memory statistics  
                if memory_usage:
                    impl_stats.update({
                        'memory_mean_kb': statistics.mean(memory_usage),
                        'memory_median_kb': statistics.median(memory_usage),
                        'memory_max_kb': max(memory_usage),
                        'memory_min_kb': min(memory_usage),
                    })
                
                stats[test_name][impl_name] = impl_stats
        
        return stats
    
    def generate_comparisons(self, stats_results: Dict[str, Dict[str, Dict[str, float]]]) -> Dict[str, Any]:
        """Generate comparative analysis between implementations."""
        
        comparisons = {
            'relative_performance': {},
            'rankings': {},
            'summary': {}
        }
        
        baseline_impl = self.output_config.get('comparisons', {}).get('baseline', 'chez')
        
        for test_name, test_stats in stats_results.items():
            if baseline_impl not in test_stats:
                # Use first available implementation as baseline
                baseline_impl = next(iter(test_stats.keys())) if test_stats else None
            
            if not baseline_impl or baseline_impl not in test_stats:
                continue
            
            baseline_time = test_stats[baseline_impl]['duration_mean']
            
            # Calculate relative performance
            test_comparisons = {}
            rankings = []
            
            for impl_name, impl_stats in test_stats.items():
                impl_time = impl_stats['duration_mean']
                relative_performance = impl_time / baseline_time if baseline_time > 0 else float('inf')
                
                test_comparisons[impl_name] = {
                    'relative_performance': relative_performance,
                    'speedup_factor': baseline_time / impl_time if impl_time > 0 else 0,
                    'absolute_time': impl_time,
                    'memory_usage_kb': impl_stats.get('memory_mean_kb', 0),
                }
                
                rankings.append({
                    'implementation': impl_name,
                    'time': impl_time,
                    'relative_performance': relative_performance
                })
            
            # Sort rankings by performance (lower time = better)
            rankings.sort(key=lambda x: x['time'])
            for i, ranking in enumerate(rankings):
                ranking['rank'] = i + 1
            
            comparisons['relative_performance'][test_name] = test_comparisons
            comparisons['rankings'][test_name] = rankings
        
        # Generate overall summary
        comparisons['summary'] = self.generate_overall_summary(comparisons['rankings'])
        
        return comparisons
    
    def generate_overall_summary(self, rankings: Dict[str, List[Dict[str, Any]]]) -> Dict[str, Any]:
        """Generate overall performance summary across all tests."""
        
        impl_scores = defaultdict(list)
        impl_wins = defaultdict(int)
        
        # Collect scores and wins for each implementation
        for test_name, test_rankings in rankings.items():
            for ranking in test_rankings:
                impl_name = ranking['implementation']
                rank = ranking['rank']
                impl_scores[impl_name].append(rank)
                
                if rank == 1:
                    impl_wins[impl_name] += 1
        
        # Calculate overall statistics
        summary = {}
        
        for impl_name, scores in impl_scores.items():
            if not scores:
                continue
            
            summary[impl_name] = {
                'average_rank': statistics.mean(scores),
                'median_rank': statistics.median(scores),
                'total_wins': impl_wins[impl_name],
                'total_tests': len(scores),
                'win_percentage': (impl_wins[impl_name] / len(scores)) * 100 if scores else 0
            }
        
        # Overall rankings by average rank
        overall_rankings = sorted(
            summary.items(),
            key=lambda x: x[1]['average_rank']
        )
        
        return {
            'individual_stats': summary,
            'overall_rankings': [
                {
                    'implementation': impl_name,
                    'average_rank': stats['average_rank'],
                    'win_percentage': stats['win_percentage']
                }
                for impl_name, stats in overall_rankings
            ]
        }
    
    def save_results(self, output_dir: Path, processed_data: Dict[str, Any]):
        """Save processed results in multiple formats."""
        
        formats = self.output_config.get('formats', ['json'])
        
        for format_type in formats:
            if format_type == 'json':
                self.save_json_results(output_dir, processed_data)
            elif format_type == 'csv':
                self.save_csv_results(output_dir, processed_data)
    
    def save_json_results(self, output_dir: Path, processed_data: Dict[str, Any]):
        """Save results in JSON format."""
        
        # Save complete processed data
        with open(output_dir / 'complete_results.json', 'w') as f:
            json.dump(processed_data, f, indent=2)
        
        # Save individual components
        for component_name, component_data in processed_data.items():
            with open(output_dir / f'{component_name}.json', 'w') as f:
                json.dump(component_data, f, indent=2)
        
        print(f"JSON results saved to {output_dir}")
    
    def save_csv_results(self, output_dir: Path, processed_data: Dict[str, Any]):
        """Save results in CSV format."""
        
        stats_data = processed_data.get('statistics', {})
        comparisons_data = processed_data.get('comparisons', {})
        
        # Save statistics CSV
        stats_file = output_dir / 'statistics.csv'
        with open(stats_file, 'w', newline='') as f:
            writer = csv.writer(f)
            
            # Header
            header = [
                'test', 'implementation', 'count', 'duration_mean', 'duration_median',
                'duration_stdev', 'duration_min', 'duration_max', 'memory_mean_kb'
            ]
            writer.writerow(header)
            
            # Data rows
            for test_name, test_stats in stats_data.items():
                for impl_name, impl_stats in test_stats.items():
                    row = [
                        test_name, impl_name,
                        impl_stats.get('count', 0),
                        impl_stats.get('duration_mean', 0),
                        impl_stats.get('duration_median', 0),
                        impl_stats.get('duration_stdev', 0),
                        impl_stats.get('duration_min', 0),
                        impl_stats.get('duration_max', 0),
                        impl_stats.get('memory_mean_kb', 0)
                    ]
                    writer.writerow(row)
        
        # Save comparisons CSV
        comparisons_file = output_dir / 'comparisons.csv'
        with open(comparisons_file, 'w', newline='') as f:
            writer = csv.writer(f)
            
            # Header
            header = [
                'test', 'implementation', 'rank', 'absolute_time',
                'relative_performance', 'speedup_factor', 'memory_usage_kb'
            ]
            writer.writerow(header)
            
            # Data rows
            rankings = comparisons_data.get('rankings', {})
            relative_perf = comparisons_data.get('relative_performance', {})
            
            for test_name, test_rankings in rankings.items():
                test_relative = relative_perf.get(test_name, {})
                
                for ranking in test_rankings:
                    impl_name = ranking['implementation']
                    impl_relative = test_relative.get(impl_name, {})
                    
                    row = [
                        test_name, impl_name,
                        ranking['rank'],
                        ranking['time'],
                        ranking['relative_performance'],
                        impl_relative.get('speedup_factor', 0),
                        impl_relative.get('memory_usage_kb', 0)
                    ]
                    writer.writerow(row)
        
        print(f"CSV results saved to {output_dir}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Process Scheme benchmark results')
    parser.add_argument('--config', required=True, help='Configuration file')
    parser.add_argument('--input-dir', required=True, help='Input directory with raw results')
    parser.add_argument('--output-dir', required=True, help='Output directory for processed results')
    
    args = parser.parse_args()
    
    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Process results
    processor = BenchmarkProcessor(args.config)
    processor.process_all_results(Path(args.input_dir), output_dir)
    
    print("Result processing completed!")


if __name__ == '__main__':
    main()