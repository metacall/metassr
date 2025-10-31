#!/usr/bin/env python3
"""
MetaSSR Benchmark Results Analyzer
Analyzes benchmark results and generates comprehensive reports
"""

import json
import csv
import argparse
import os
import sys
from datetime import datetime
from pathlib import Path
import statistics
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

class BenchmarkAnalyzer:
    def __init__(self, results_dir="benchmark-results"):
        self.results_dir = Path(results_dir)
        self.results_dir.mkdir(exist_ok=True)
        
    def load_results(self, result_file):
        """Load benchmark results from JSON file"""
        with open(result_file, 'r') as f:
            return json.load(f)
    
    def analyze_performance(self, results):
        """Analyze performance metrics"""
        analysis = {
            'summary': {},
            'trends': {},
            'recommendations': []
        }
        
        tests = results['tests']
        
        # Calculate overall statistics
        rps_values = [float(test['results']['requests_per_sec'].replace(',', '')) 
                     for test in tests if test['results']['requests_per_sec']]
        
        analysis['summary'] = {
            'total_tests': len(tests),
            'max_rps': max(rps_values) if rps_values else 0,
            'avg_rps': statistics.mean(rps_values) if rps_values else 0,
            'min_rps': min(rps_values) if rps_values else 0
        }
        
        # Analyze each test
        for test in tests:
            test_name = test['name']
            results_data = test['results']
            
            # Convert latency to milliseconds
            avg_latency = self.parse_latency(results_data.get('avg_latency', ''))
            
            # Safely get P99 latency
            latency_percentiles = results_data.get('latency_percentiles', {})
            p99_latency = self.parse_latency(latency_percentiles.get('p99', '')) if latency_percentiles else 0
            
            # Safely parse numeric values
            try:
                rps = float(str(results_data.get('requests_per_sec', '0')).replace(',', '') or 0)
            except (ValueError, TypeError):
                rps = 0
                
            try:
                total_requests = int(str(results_data.get('total_requests', '0')).replace(',', '') or 0)
            except (ValueError, TypeError):
                total_requests = 0
                
            try:
                errors = int(str(results_data.get('total_errors', '0')) or 0)
            except (ValueError, TypeError):
                errors = 0
            
            analysis['trends'][test_name] = {
                'rps': rps,
                'avg_latency_ms': avg_latency,
                'p99_latency_ms': p99_latency,
                'errors': errors,
                'total_requests': total_requests
            }
        
        # Generate recommendations
        analysis['recommendations'] = self.generate_recommendations(analysis)
        
        return analysis
    
    def parse_latency(self, latency_str):
        """Parse latency string and convert to milliseconds"""
        if not latency_str:
            return 0
        
        try:
            # Clean the string - handle multi-line strings and extra data
            latency_str = str(latency_str).strip()
            
            # Split by newlines and take the first non-empty line
            lines = [line.strip() for line in latency_str.split('\n') if line.strip()]
            if not lines:
                return 0
                
            # Take the first line and get the first word (the actual latency value)
            first_value = lines[0].split()[0].lower()
            
            # Parse based on unit
            if 'ms' in first_value:
                return float(first_value.replace('ms', ''))
            elif 'us' in first_value:
                return float(first_value.replace('us', '')) / 1000
            elif first_value.endswith('s') and 'ms' not in first_value and 'us' not in first_value:
                return float(first_value.replace('s', '')) * 1000
            else:
                # Try to parse as plain number (assume milliseconds)
                return float(first_value)
                
        except (ValueError, TypeError, IndexError, AttributeError):
            return 0
    
    def generate_recommendations(self, analysis):
        """Generate performance recommendations"""
        recommendations = []
        trends = analysis['trends']
        
        # Check for high latency
        high_latency_tests = [name for name, data in trends.items() 
                            if data['avg_latency_ms'] > 100]
        if high_latency_tests:
            recommendations.append({
                'type': 'warning',
                'message': f"High average latency detected in: {', '.join(high_latency_tests)}",
                'suggestion': "Consider optimizing server response time or reducing load"
            })
        
        # Check for errors
        error_tests = [name for name, data in trends.items() if data['errors'] > 0]
        if error_tests:
            recommendations.append({
                'type': 'critical',
                'message': f"Errors detected in: {', '.join(error_tests)}",
                'suggestion': "Investigate error causes and improve error handling"
            })
        
        # Check performance scaling
        rps_values = [(name, data['rps']) for name, data in trends.items()]
        rps_values.sort(key=lambda x: x[1], reverse=True)
        
        if len(rps_values) > 1:
            best_test = rps_values[0]
            recommendations.append({
                'type': 'info',
                'message': f"Best performance: {best_test[0]} with {best_test[1]:.0f} RPS",
                'suggestion': "Use this configuration as baseline for optimization"
            })
        
        return recommendations
    
    def generate_plots(self, analysis, output_dir):
        """Generate performance visualization plots"""
        plt.style.use('seaborn-v0_8')
        output_dir = Path(output_dir)
        output_dir.mkdir(exist_ok=True)
        
        trends = analysis['trends']
        test_names = list(trends.keys())
        
        # RPS vs Test scenario
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 12))
        
        # Requests per second
        rps_values = [trends[name]['rps'] for name in test_names]
        ax1.bar(test_names, rps_values, color='skyblue')
        ax1.set_title('Requests per Second by Test Scenario')
        ax1.set_ylabel('Requests/sec')
        ax1.tick_params(axis='x', rotation=45)
        
        # Average latency
        latency_values = [trends[name]['avg_latency_ms'] for name in test_names]
        ax2.bar(test_names, latency_values, color='lightcoral')
        ax2.set_title('Average Latency by Test Scenario')
        ax2.set_ylabel('Latency (ms)')
        ax2.tick_params(axis='x', rotation=45)
        
        # P99 latency
        p99_values = [trends[name]['p99_latency_ms'] for name in test_names]
        ax3.bar(test_names, p99_values, color='lightgreen')
        ax3.set_title('P99 Latency by Test Scenario')
        ax3.set_ylabel('P99 Latency (ms)')
        ax3.tick_params(axis='x', rotation=45)
        
        # Error count
        error_values = [trends[name]['errors'] for name in test_names]
        ax4.bar(test_names, error_values, color='orange')
        ax4.set_title('Errors by Test Scenario')
        ax4.set_ylabel('Error Count')
        ax4.tick_params(axis='x', rotation=45)
        
        plt.tight_layout()
        plt.savefig(output_dir / 'performance_overview.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        # Performance trend line
        plt.figure(figsize=(12, 6))
        plt.plot(test_names, rps_values, marker='o', linewidth=2, markersize=8)
        plt.title('Performance Trend Across Test Scenarios')
        plt.xlabel('Test Scenario')
        plt.ylabel('Requests per Second')
        plt.xticks(rotation=45)
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        plt.savefig(output_dir / 'performance_trend.png', dpi=300, bbox_inches='tight')
        plt.close()
    
    def generate_report(self, results, analysis, output_file):
        """Generate comprehensive markdown report"""
        with open(output_file, 'w') as f:
            f.write("# MetaSSR Benchmark Report\n\n")
            
            # Metadata
            metadata = results['metadata']
            f.write("## System Information\n\n")
            f.write(f"- **Timestamp:** {metadata['timestamp']}\n")
            f.write(f"- **Hostname:** {metadata['hostname']}\n")
            f.write(f"- **OS:** {metadata['os']}\n")
            f.write(f"- **Architecture:** {metadata['arch']}\n")
            f.write(f"- **CPU Cores:** {metadata['cpu_cores']}\n")
            f.write(f"- **Memory:** {metadata['memory_gb']} GB\n\n")
            
            # Summary
            summary = analysis['summary']
            f.write("## Performance Summary\n\n")
            f.write(f"- **Total Tests:** {summary['total_tests']}\n")
            f.write(f"- **Maximum RPS:** {summary['max_rps']:,.0f}\n")
            f.write(f"- **Average RPS:** {summary['avg_rps']:,.0f}\n")
            f.write(f"- **Minimum RPS:** {summary['min_rps']:,.0f}\n\n")
            
            # Detailed results
            f.write("## Detailed Results\n\n")
            f.write("| Test Scenario | RPS | Avg Latency | P99 Latency | Errors | Total Requests |\n")
            f.write("|---------------|-----|-------------|-------------|--------|-----------------|\n")
            
            for name, data in analysis['trends'].items():
                f.write(f"| {name} | {data['rps']:,.0f} | {data['avg_latency_ms']:.2f}ms | "
                       f"{data['p99_latency_ms']:.2f}ms | {data['errors']} | {data['total_requests']:,} |\n")
            
            # Recommendations
            if analysis['recommendations']:
                f.write("\n## Recommendations\n\n")
                for rec in analysis['recommendations']:
                    icon = "🔴" if rec['type'] == 'critical' else "⚠️" if rec['type'] == 'warning' else "ℹ️"
                    f.write(f"{icon} **{rec['message']}**\n")
                    f.write(f"   {rec['suggestion']}\n\n")
    
    def export_csv(self, analysis, output_file):
        """Export results to CSV format"""
        with open(output_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['Test', 'RPS', 'Avg_Latency_ms', 'P99_Latency_ms', 'Errors', 'Total_Requests'])
            
            for name, data in analysis['trends'].items():
                writer.writerow([
                    name,
                    data['rps'],
                    data['avg_latency_ms'],
                    data['p99_latency_ms'],
                    data['errors'],
                    data['total_requests']
                ])

def main():
    parser = argparse.ArgumentParser(description='Analyze MetaSSR benchmark results')
    parser.add_argument('result_file', help='Path to benchmark results JSON file')
    parser.add_argument('-o', '--output', default='analysis_report', 
                       help='Output directory for analysis results')
    parser.add_argument('--plots', action='store_true', 
                       help='Generate performance plots')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.result_file):
        print(f"Error: Result file {args.result_file} not found")
        sys.exit(1)
    
    # Create analyzer
    analyzer = BenchmarkAnalyzer()
    
    # Load and analyze results
    print("Loading benchmark results...")
    results = analyzer.load_results(args.result_file)
    
    print("Analyzing performance...")
    analysis = analyzer.analyze_performance(results)
    
    # Create output directory
    output_dir = Path(args.output)
    output_dir.mkdir(exist_ok=True)
    
    # Generate reports
    print("Generating reports...")
    
    # Markdown report
    report_file = output_dir / 'benchmark_report.md'
    analyzer.generate_report(results, analysis, report_file)
    print(f"Generated report: {report_file}")
    
    # CSV export
    csv_file = output_dir / 'benchmark_results.csv'
    analyzer.export_csv(analysis, csv_file)
    print(f"Generated CSV: {csv_file}")
    
    # Generate plots if requested
    if args.plots:
        try:
            print("Generating performance plots...")
            analyzer.generate_plots(analysis, output_dir)
            print(f"Generated plots in: {output_dir}")
        except ImportError:
            print("Warning: matplotlib/seaborn not available, skipping plots")
    
    # Print summary
    print("\n=== Performance Summary ===")
    summary = analysis['summary']
    print(f"Maximum RPS: {summary['max_rps']:,.0f}")
    print(f"Average RPS: {summary['avg_rps']:,.0f}")
    print(f"Total Tests: {summary['total_tests']}")
    
    if analysis['recommendations']:
        print("\n=== Recommendations ===")
        for rec in analysis['recommendations']:
            print(f"- {rec['message']}")

if __name__ == '__main__':
    main()