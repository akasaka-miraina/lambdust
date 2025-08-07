#!/usr/bin/env python3

"""
Benchmark Report Generator

Generates comprehensive HTML reports from processed benchmark results.
"""

import os
import json
import argparse
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime


class ReportGenerator:
    """Generates benchmark reports in various formats."""
    
    def __init__(self):
        """Initialize the report generator."""
        self.template = self._get_html_template()
    
    def generate_html_report(self, input_dir: Path, output_dir: Path):
        """Generate comprehensive HTML report."""
        
        # Load processed data
        processed_data = self.load_processed_data(input_dir)
        
        if not processed_data:
            print("No processed data found")
            return
        
        # Generate HTML content
        html_content = self.generate_html_content(processed_data)
        
        # Save report
        report_file = output_dir / 'benchmark_report.html'
        with open(report_file, 'w') as f:
            f.write(html_content)
        
        # Generate static assets
        self.generate_static_assets(output_dir)
        
        print(f"HTML report generated: {report_file}")
    
    def load_processed_data(self, input_dir: Path) -> Dict[str, Any]:
        """Load all processed benchmark data."""
        
        data = {}
        
        # Try to load complete results first
        complete_file = input_dir / 'complete_results.json'
        if complete_file.exists():
            with open(complete_file, 'r') as f:
                return json.load(f)
        
        # Otherwise load individual components
        for json_file in input_dir.glob('*.json'):
            try:
                with open(json_file, 'r') as f:
                    component_name = json_file.stem
                    data[component_name] = json.load(f)
            except (json.JSONDecodeError, IOError) as e:
                print(f"Warning: Could not load {json_file}: {e}")
        
        return data
    
    def generate_html_content(self, processed_data: Dict[str, Any]) -> str:
        """Generate the complete HTML report content."""
        
        statistics = processed_data.get('statistics', {})
        comparisons = processed_data.get('comparisons', {})
        
        # Generate sections
        summary_section = self.generate_summary_section(comparisons)
        performance_section = self.generate_performance_section(statistics, comparisons)
        detailed_section = self.generate_detailed_section(statistics)
        charts_section = self.generate_charts_section(processed_data)
        
        # Generate timestamp
        timestamp = datetime.now().isoformat()
        
        # Replace placeholders in template
        html_content = self.template.format(
            timestamp=timestamp,
            summary_section=summary_section,
            performance_section=performance_section,
            detailed_section=detailed_section,
            charts_section=charts_section
        )
        
        return html_content
    
    def generate_summary_section(self, comparisons: Dict[str, Any]) -> str:
        """Generate executive summary section."""
        
        summary_data = comparisons.get('summary', {})
        overall_rankings = summary_data.get('overall_rankings', [])
        
        if not overall_rankings:
            return "<p>No summary data available.</p>"
        
        html = """
        <div class="summary-grid">
            <div class="summary-card">
                <h3>Top Performers</h3>
                <ol class="top-performers">
        """
        
        # Show top 5 performers
        for i, ranking in enumerate(overall_rankings[:5]):
            impl_name = ranking['implementation']
            avg_rank = ranking['average_rank']
            win_pct = ranking['win_percentage']
            
            html += f"""
                    <li class="performer-item">
                        <span class="impl-name">{impl_name}</span>
                        <span class="metrics">
                            Avg Rank: {avg_rank:.2f} | Wins: {win_pct:.1f}%
                        </span>
                    </li>
            """
        
        html += """
                </ol>
            </div>
            <div class="summary-card">
                <h3>Key Insights</h3>
                <ul class="insights">
        """
        
        # Generate key insights
        if overall_rankings:
            best_performer = overall_rankings[0]
            html += f"""
                    <li>Best overall: <strong>{best_performer['implementation']}</strong> 
                        (avg rank {best_performer['average_rank']:.2f})</li>
            """
            
            if len(overall_rankings) > 1:
                lambdust_ranking = next(
                    (r for r in overall_rankings if r['implementation'] == 'lambdust'),
                    None
                )
                if lambdust_ranking:
                    position = next(i for i, r in enumerate(overall_rankings) if r['implementation'] == 'lambdust') + 1
                    html += f"""
                        <li>Lambdust ranks #{position} overall with {lambdust_ranking['win_percentage']:.1f}% wins</li>
                    """
        
        html += """
                </ul>
            </div>
        </div>
        """
        
        return html
    
    def generate_performance_section(self, statistics: Dict[str, Any], comparisons: Dict[str, Any]) -> str:
        """Generate performance comparison section."""
        
        rankings = comparisons.get('rankings', {})
        
        if not rankings:
            return "<p>No performance data available.</p>"
        
        html = """
        <div class="performance-overview">
            <h3>Performance by Test Category</h3>
            <div class="test-results">
        """
        
        # Group tests by category
        test_categories = {
            'Micro-benchmarks': [],
            'Algorithm Performance': [],
            'Data Structures': [],
            'Other': []
        }
        
        for test_name, test_rankings in rankings.items():
            # Categorize test based on name patterns
            if any(keyword in test_name for keyword in ['arithmetic', 'function_calls', 'closure']):
                category = 'Micro-benchmarks'
            elif any(keyword in test_name for keyword in ['fibonacci', 'factorial', 'quicksort']):
                category = 'Algorithm Performance'
            elif any(keyword in test_name for keyword in ['list', 'vector', 'string']):
                category = 'Data Structures'
            else:
                category = 'Other'
            
            test_categories[category].append((test_name, test_rankings))
        
        # Generate tables for each category
        for category, tests in test_categories.items():
            if not tests:
                continue
                
            html += f"""
                <div class="category-section">
                    <h4>{category}</h4>
                    <table class="results-table">
                        <thead>
                            <tr>
                                <th>Test</th>
                                <th>Winner</th>
                                <th>Time (s)</th>
                                <th>Top 3</th>
                            </tr>
                        </thead>
                        <tbody>
            """
            
            for test_name, test_rankings in tests:
                winner = test_rankings[0] if test_rankings else None
                top_3 = test_rankings[:3] if len(test_rankings) >= 3 else test_rankings
                
                if winner:
                    html += f"""
                            <tr>
                                <td class="test-name">{test_name}</td>
                                <td class="winner">{winner['implementation']}</td>
                                <td class="time">{winner['time']:.4f}</td>
                                <td class="top-three">
                    """
                    
                    for i, ranking in enumerate(top_3):
                        html += f"{ranking['implementation']}"
                        if i < len(top_3) - 1:
                            html += ", "
                    
                    html += """
                                </td>
                            </tr>
                    """
            
            html += """
                        </tbody>
                    </table>
                </div>
            """
        
        html += """
            </div>
        </div>
        """
        
        return html
    
    def generate_detailed_section(self, statistics: Dict[str, Any]) -> str:
        """Generate detailed statistics section."""
        
        if not statistics:
            return "<p>No detailed statistics available.</p>"
        
        html = """
        <div class="detailed-stats">
            <h3>Detailed Performance Statistics</h3>
        """
        
        for test_name, test_stats in statistics.items():
            html += f"""
            <div class="test-detail">
                <h4>{test_name}</h4>
                <table class="stats-table">
                    <thead>
                        <tr>
                            <th>Implementation</th>
                            <th>Mean (s)</th>
                            <th>Median (s)</th>
                            <th>Std Dev</th>
                            <th>Min (s)</th>
                            <th>Max (s)</th>
                            <th>Memory (KB)</th>
                        </tr>
                    </thead>
                    <tbody>
            """
            
            # Sort by mean time
            sorted_impls = sorted(
                test_stats.items(),
                key=lambda x: x[1].get('duration_mean', float('inf'))
            )
            
            for impl_name, impl_stats in sorted_impls:
                html += f"""
                        <tr>
                            <td class="impl-name">{impl_name}</td>
                            <td>{impl_stats.get('duration_mean', 0):.4f}</td>
                            <td>{impl_stats.get('duration_median', 0):.4f}</td>
                            <td>{impl_stats.get('duration_stdev', 0):.4f}</td>
                            <td>{impl_stats.get('duration_min', 0):.4f}</td>
                            <td>{impl_stats.get('duration_max', 0):.4f}</td>
                            <td>{impl_stats.get('memory_mean_kb', 0):.0f}</td>
                        </tr>
                """
            
            html += """
                    </tbody>
                </table>
            </div>
            """
        
        html += """
        </div>
        """
        
        return html
    
    def generate_charts_section(self, processed_data: Dict[str, Any]) -> str:
        """Generate charts and visualizations section."""
        
        # For now, generate placeholder for charts
        # In a full implementation, this would generate JavaScript charts
        
        html = """
        <div class="charts-section">
            <h3>Performance Visualizations</h3>
            <div class="chart-grid">
                <div class="chart-container">
                    <h4>Performance Comparison</h4>
                    <div id="performance-chart" class="chart-placeholder">
                        Chart would be rendered here with Chart.js or D3.js
                    </div>
                </div>
                <div class="chart-container">
                    <h4>Memory Usage</h4>
                    <div id="memory-chart" class="chart-placeholder">
                        Memory usage chart would be rendered here
                    </div>
                </div>
            </div>
        </div>
        """
        
        return html
    
    def generate_static_assets(self, output_dir: Path):
        """Generate CSS and JavaScript assets."""
        
        css_content = self._get_css_styles()
        css_file = output_dir / 'styles.css'
        
        with open(css_file, 'w') as f:
            f.write(css_content)
        
        print(f"Static assets generated in {output_dir}")
    
    def _get_html_template(self) -> str:
        """Get the HTML template."""
        
        return """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Scheme Implementation Benchmark Report</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <header class="report-header">
        <h1>Scheme Implementation Benchmark Report</h1>
        <p class="subtitle">Comprehensive performance comparison across major Scheme implementations</p>
        <p class="timestamp">Generated: {timestamp}</p>
    </header>
    
    <nav class="nav-menu">
        <ul>
            <li><a href="#summary">Executive Summary</a></li>
            <li><a href="#performance">Performance Overview</a></li>
            <li><a href="#detailed">Detailed Statistics</a></li>
            <li><a href="#charts">Visualizations</a></li>
        </ul>
    </nav>
    
    <main class="report-content">
        <section id="summary" class="report-section">
            <h2>Executive Summary</h2>
            {summary_section}
        </section>
        
        <section id="performance" class="report-section">
            <h2>Performance Overview</h2>
            {performance_section}
        </section>
        
        <section id="detailed" class="report-section">
            <h2>Detailed Statistics</h2>
            {detailed_section}
        </section>
        
        <section id="charts" class="report-section">
            <h2>Visualizations</h2>
            {charts_section}
        </section>
    </main>
    
    <footer class="report-footer">
        <p>Generated by Lambdust Benchmarking Suite</p>
    </footer>
</body>
</html>"""
    
    def _get_css_styles(self) -> str:
        """Get CSS styles for the report."""
        
        return """/* Scheme Benchmark Report Styles */

:root {
    --primary-color: #2c5aa0;
    --secondary-color: #5d7db3;
    --accent-color: #f39c12;
    --background-color: #f8f9fa;
    --text-color: #333;
    --border-color: #dee2e6;
    --success-color: #28a745;
    --warning-color: #ffc107;
    --error-color: #dc3545;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--background-color);
}

.report-header {
    background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
    color: white;
    padding: 2rem 1rem;
    text-align: center;
    margin-bottom: 2rem;
}

.report-header h1 {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
    font-weight: 700;
}

.report-header .subtitle {
    font-size: 1.2rem;
    opacity: 0.9;
    margin-bottom: 1rem;
}

.report-header .timestamp {
    font-size: 0.9rem;
    opacity: 0.8;
}

.nav-menu {
    background: white;
    border-bottom: 2px solid var(--border-color);
    padding: 0;
    margin-bottom: 2rem;
    position: sticky;
    top: 0;
    z-index: 100;
}

.nav-menu ul {
    list-style: none;
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
}

.nav-menu li {
    margin: 0;
}

.nav-menu a {
    display: block;
    padding: 1rem 2rem;
    text-decoration: none;
    color: var(--text-color);
    font-weight: 500;
    border-bottom: 3px solid transparent;
    transition: all 0.3s ease;
}

.nav-menu a:hover {
    background-color: var(--background-color);
    border-bottom-color: var(--primary-color);
}

.report-content {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 1rem;
}

.report-section {
    background: white;
    margin-bottom: 2rem;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.report-section h2 {
    color: var(--primary-color);
    margin-bottom: 1.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 2px solid var(--border-color);
    font-size: 2rem;
}

/* Summary Section */
.summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
    margin-bottom: 2rem;
}

.summary-card {
    background: var(--background-color);
    padding: 1.5rem;
    border-radius: 6px;
    border-left: 4px solid var(--primary-color);
}

.summary-card h3 {
    color: var(--primary-color);
    margin-bottom: 1rem;
    font-size: 1.3rem;
}

.top-performers {
    list-style: none;
    counter-reset: performer-counter;
}

.performer-item {
    counter-increment: performer-counter;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    background: white;
    border-radius: 4px;
    position: relative;
    padding-left: 3rem;
}

.performer-item::before {
    content: counter(performer-counter);
    position: absolute;
    left: 0.75rem;
    top: 50%;
    transform: translateY(-50%);
    background: var(--primary-color);
    color: white;
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 0.9rem;
}

.impl-name {
    font-weight: 600;
    color: var(--primary-color);
}

.metrics {
    font-size: 0.9rem;
    color: #666;
}

.insights {
    list-style: none;
}

.insights li {
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-color);
}

.insights li:last-child {
    border-bottom: none;
}

/* Performance Section */
.performance-overview {
    margin-bottom: 2rem;
}

.category-section {
    margin-bottom: 2rem;
}

.category-section h4 {
    color: var(--secondary-color);
    margin-bottom: 1rem;
    font-size: 1.4rem;
}

/* Tables */
.results-table, .stats-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 1.5rem;
    background: white;
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.results-table th, .stats-table th {
    background: var(--primary-color);
    color: white;
    padding: 1rem;
    text-align: left;
    font-weight: 600;
}

.results-table td, .stats-table td {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-color);
}

.results-table tr:hover, .stats-table tr:hover {
    background-color: var(--background-color);
}

.test-name {
    font-weight: 500;
    color: var(--primary-color);
}

.winner {
    font-weight: 600;
    color: var(--success-color);
}

.time {
    font-family: 'Monaco', 'Menlo', monospace;
    color: #666;
}

.top-three {
    font-size: 0.9rem;
    color: #666;
}

/* Charts Section */
.charts-section {
    margin-top: 2rem;
}

.chart-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 2rem;
}

.chart-container {
    background: var(--background-color);
    padding: 1.5rem;
    border-radius: 6px;
    border: 1px solid var(--border-color);
}

.chart-container h4 {
    color: var(--primary-color);
    margin-bottom: 1rem;
    text-align: center;
}

.chart-placeholder {
    height: 300px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: white;
    border: 2px dashed var(--border-color);
    border-radius: 4px;
    color: #666;
    font-style: italic;
}

/* Detailed Stats */
.test-detail {
    margin-bottom: 2rem;
}

.test-detail h4 {
    color: var(--secondary-color);
    margin-bottom: 1rem;
    font-size: 1.3rem;
}

/* Footer */
.report-footer {
    text-align: center;
    padding: 2rem;
    color: #666;
    font-size: 0.9rem;
    margin-top: 3rem;
    border-top: 1px solid var(--border-color);
}

/* Responsive Design */
@media (max-width: 768px) {
    .report-header {
        padding: 1.5rem 0.5rem;
    }
    
    .report-header h1 {
        font-size: 2rem;
    }
    
    .nav-menu ul {
        flex-direction: column;
    }
    
    .nav-menu a {
        padding: 0.75rem 1rem;
    }
    
    .report-content {
        padding: 0 0.5rem;
    }
    
    .report-section {
        padding: 1rem;
    }
    
    .summary-grid {
        grid-template-columns: 1fr;
    }
    
    .chart-grid {
        grid-template-columns: 1fr;
    }
    
    .results-table, .stats-table {
        font-size: 0.9rem;
    }
    
    .results-table th, .stats-table th,
    .results-table td, .stats-table td {
        padding: 0.5rem;
    }
}

/* Print Styles */
@media print {
    .nav-menu {
        display: none;
    }
    
    .report-section {
        page-break-inside: avoid;
        box-shadow: none;
        border: 1px solid var(--border-color);
    }
    
    .chart-placeholder {
        display: none;
    }
}"""


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Generate Scheme benchmark reports')
    parser.add_argument('--input-dir', required=True, help='Input directory with processed results')
    parser.add_argument('--output-dir', required=True, help='Output directory for reports')
    parser.add_argument('--format', default='html', choices=['html', 'pdf'], 
                       help='Report format')
    
    args = parser.parse_args()
    
    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate report
    generator = ReportGenerator()
    
    if args.format == 'html':
        generator.generate_html_report(Path(args.input_dir), output_dir)
    else:
        print(f"Format {args.format} not yet implemented")
    
    print("Report generation completed!")


if __name__ == '__main__':
    main()