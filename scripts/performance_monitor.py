#!/usr/bin/env python3

"""
Performance Monitoring Utilities

Advanced performance monitoring and profiling for Scheme implementations.
"""

import os
import sys
import json
import psutil
import subprocess
import time
import threading
from pathlib import Path
from typing import Dict, List, Any, Optional, Callable
from collections import defaultdict
from dataclasses import dataclass, asdict
from contextlib import contextmanager


@dataclass
class SystemMetrics:
    """System performance metrics snapshot."""
    timestamp: float
    cpu_percent: float
    memory_percent: float
    memory_rss_mb: float
    memory_vms_mb: float
    io_read_bytes: int
    io_write_bytes: int
    ctx_switches_voluntary: int
    ctx_switches_involuntary: int
    open_files: int


@dataclass
class ProcessMetrics:
    """Process-specific performance metrics."""
    pid: int
    name: str
    cpu_percent: float
    memory_rss_mb: float
    memory_vms_mb: float
    num_threads: int
    status: str
    create_time: float


class PerformanceMonitor:
    """Real-time performance monitoring for benchmark processes."""
    
    def __init__(self, sampling_interval: float = 0.1):
        """Initialize the performance monitor.
        
        Args:
            sampling_interval: Interval between samples in seconds
        """
        self.sampling_interval = sampling_interval
        self.monitoring = False
        self.metrics_history = []
        self.process_history = defaultdict(list)
        self._monitor_thread = None
        self._target_processes = set()
    
    def add_target_process(self, pid: int):
        """Add a process to monitor."""
        self._target_processes.add(pid)
    
    def remove_target_process(self, pid: int):
        """Remove a process from monitoring."""
        self._target_processes.discard(pid)
    
    def start_monitoring(self):
        """Start performance monitoring in background thread."""
        if self.monitoring:
            return
        
        self.monitoring = True
        self.metrics_history.clear()
        self.process_history.clear()
        
        self._monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self._monitor_thread.start()
    
    def stop_monitoring(self) -> Dict[str, Any]:
        """Stop monitoring and return collected metrics."""
        self.monitoring = False
        
        if self._monitor_thread:
            self._monitor_thread.join(timeout=1.0)
        
        return {
            'system_metrics': [asdict(m) for m in self.metrics_history],
            'process_metrics': {
                str(pid): [asdict(m) for m in metrics]
                for pid, metrics in self.process_history.items()
            }
        }
    
    def _monitor_loop(self):
        """Main monitoring loop."""
        while self.monitoring:
            try:
                # Collect system metrics
                system_metrics = self._collect_system_metrics()
                self.metrics_history.append(system_metrics)
                
                # Collect per-process metrics
                for pid in list(self._target_processes):
                    try:
                        process_metrics = self._collect_process_metrics(pid)
                        if process_metrics:
                            self.process_history[pid].append(process_metrics)
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        # Process may have terminated
                        self._target_processes.discard(pid)
                
                time.sleep(self.sampling_interval)
                
            except Exception as e:
                print(f"Error in monitoring loop: {e}")
                continue
    
    def _collect_system_metrics(self) -> SystemMetrics:
        """Collect system-wide performance metrics."""
        cpu_percent = psutil.cpu_percent(interval=None)
        memory = psutil.virtual_memory()
        disk_io = psutil.disk_io_counters()
        
        return SystemMetrics(
            timestamp=time.time(),
            cpu_percent=cpu_percent,
            memory_percent=memory.percent,
            memory_rss_mb=memory.used / (1024 * 1024),
            memory_vms_mb=memory.total / (1024 * 1024),
            io_read_bytes=disk_io.read_bytes if disk_io else 0,
            io_write_bytes=disk_io.write_bytes if disk_io else 0,
            ctx_switches_voluntary=0,  # System-wide context switches not easily available
            ctx_switches_involuntary=0,
            open_files=len(psutil.pids())  # Approximation
        )
    
    def _collect_process_metrics(self, pid: int) -> Optional[ProcessMetrics]:
        """Collect metrics for a specific process."""
        try:
            process = psutil.Process(pid)
            
            # Get memory info
            memory_info = process.memory_info()
            
            return ProcessMetrics(
                pid=pid,
                name=process.name(),
                cpu_percent=process.cpu_percent(),
                memory_rss_mb=memory_info.rss / (1024 * 1024),
                memory_vms_mb=memory_info.vms / (1024 * 1024),
                num_threads=process.num_threads(),
                status=process.status(),
                create_time=process.create_time()
            )
            
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return None
    
    @contextmanager
    def monitor_process(self, process):
        """Context manager to monitor a subprocess."""
        self.add_target_process(process.pid)
        self.start_monitoring()
        
        try:
            yield
        finally:
            metrics = self.stop_monitoring()
            self.remove_target_process(process.pid)


class ValgrindProfiler:
    """Valgrind-based memory profiling."""
    
    def __init__(self, tool: str = 'massif'):
        """Initialize Valgrind profiler.
        
        Args:
            tool: Valgrind tool to use (massif, callgrind, memcheck)
        """
        self.tool = tool
        self.output_dir = Path('/tmp/valgrind_output')
        self.output_dir.mkdir(exist_ok=True)
    
    def profile_command(self, command: List[str], output_prefix: str) -> Dict[str, Any]:
        """Profile a command with Valgrind."""
        
        output_file = self.output_dir / f"{output_prefix}_{self.tool}.out"
        
        valgrind_cmd = [
            'valgrind',
            f'--tool={self.tool}',
            f'--{self.tool}-out-file={output_file}',
            '--time-stamp=yes'
        ]
        
        if self.tool == 'massif':
            valgrind_cmd.extend([
                '--pages-as-heap=yes',
                '--heap-admin=0',
                '--stacks=yes'
            ])
        elif self.tool == 'callgrind':
            valgrind_cmd.extend([
                '--collect-jumps=yes',
                '--collect-systime=yes'
            ])
        
        valgrind_cmd.extend(command)
        
        start_time = time.time()
        
        try:
            result = subprocess.run(
                valgrind_cmd,
                capture_output=True,
                text=True,
                timeout=600  # 10 minute timeout
            )
            
            end_time = time.time()
            
            # Parse Valgrind output
            profile_data = self._parse_valgrind_output(output_file)
            
            return {
                'success': result.returncode == 0,
                'duration': end_time - start_time,
                'output_file': str(output_file),
                'profile_data': profile_data,
                'stderr': result.stderr
            }
            
        except subprocess.TimeoutExpired:
            return {
                'success': False,
                'error': 'Valgrind profiling timed out',
                'duration': time.time() - start_time
            }
    
    def _parse_valgrind_output(self, output_file: Path) -> Dict[str, Any]:
        """Parse Valgrind output file."""
        
        if not output_file.exists():
            return {}
        
        try:
            with open(output_file, 'r') as f:
                content = f.read()
            
            if self.tool == 'massif':
                return self._parse_massif_output(content)
            elif self.tool == 'callgrind':
                return self._parse_callgrind_output(content)
            else:
                return {'raw_output': content}
                
        except Exception as e:
            return {'error': f'Failed to parse output: {e}'}
    
    def _parse_massif_output(self, content: str) -> Dict[str, Any]:
        """Parse Massif output."""
        lines = content.strip().split('\n')
        
        peak_memory = 0
        snapshots = []
        
        for line in lines:
            if line.startswith('mem_heap_B='):
                heap_bytes = int(line.split('=')[1])
                peak_memory = max(peak_memory, heap_bytes)
            elif line.startswith('snapshot='):
                # Parse snapshot data
                snapshot_data = {'line': line}
                snapshots.append(snapshot_data)
        
        return {
            'peak_memory_bytes': peak_memory,
            'peak_memory_mb': peak_memory / (1024 * 1024),
            'num_snapshots': len(snapshots),
            'snapshots': snapshots[:10]  # First 10 snapshots
        }
    
    def _parse_callgrind_output(self, content: str) -> Dict[str, Any]:
        """Parse Callgrind output."""
        lines = content.strip().split('\n')
        
        total_instructions = 0
        functions = []
        
        for line in lines:
            if line.startswith('summary:'):
                parts = line.split()
                if len(parts) >= 2:
                    total_instructions = int(parts[1])
            elif line.startswith('fn='):
                functions.append(line[3:])  # Function name
        
        return {
            'total_instructions': total_instructions,
            'num_functions': len(functions),
            'functions': functions[:20]  # Top 20 functions
        }


class BenchmarkProfiler:
    """High-level benchmark profiling coordinator."""
    
    def __init__(self):
        """Initialize the benchmark profiler."""
        self.performance_monitor = PerformanceMonitor()
        self.valgrind_profiler = ValgrindProfiler()
        self.results = {}
    
    def profile_benchmark(self, 
                         command: List[str], 
                         test_name: str,
                         implementation: str,
                         enable_valgrind: bool = False) -> Dict[str, Any]:
        """Profile a benchmark execution comprehensively."""
        
        profile_results = {
            'test_name': test_name,
            'implementation': implementation,
            'command': command,
            'start_time': time.time()
        }
        
        try:
            if enable_valgrind and self._valgrind_available():
                # Use Valgrind profiling
                valgrind_results = self.valgrind_profiler.profile_command(
                    command, f"{implementation}_{test_name}"
                )
                profile_results['valgrind'] = valgrind_results
                profile_results['success'] = valgrind_results['success']
                profile_results['duration'] = valgrind_results['duration']
                
            else:
                # Use psutil-based monitoring
                start_time = time.time()
                
                process = subprocess.Popen(
                    command,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True
                )
                
                with self.performance_monitor.monitor_process(process):
                    stdout, stderr = process.communicate(timeout=300)
                
                end_time = time.time()
                
                # Get monitoring results
                monitoring_results = self.performance_monitor.stop_monitoring()
                
                profile_results.update({
                    'success': process.returncode == 0,
                    'duration': end_time - start_time,
                    'stdout': stdout,
                    'stderr': stderr,
                    'returncode': process.returncode,
                    'monitoring': monitoring_results
                })
        
        except subprocess.TimeoutExpired:
            profile_results.update({
                'success': False,
                'error': 'Benchmark timed out',
                'duration': time.time() - profile_results['start_time']
            })
        
        except Exception as e:
            profile_results.update({
                'success': False,
                'error': str(e),
                'duration': time.time() - profile_results['start_time']
            })
        
        profile_results['end_time'] = time.time()
        return profile_results
    
    def _valgrind_available(self) -> bool:
        """Check if Valgrind is available."""
        try:
            subprocess.run(['valgrind', '--version'], 
                          capture_output=True, check=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False
    
    def save_profile_results(self, results: Dict[str, Any], output_file: Path):
        """Save profiling results to file."""
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2, default=str)


def main():
    """Main entry point for standalone profiling."""
    import argparse
    
    parser = argparse.ArgumentParser(description='Profile Scheme benchmark execution')
    parser.add_argument('--command', required=True, nargs='+', help='Command to profile')
    parser.add_argument('--test-name', required=True, help='Test name')
    parser.add_argument('--implementation', required=True, help='Implementation name')
    parser.add_argument('--output', required=True, help='Output file for results')
    parser.add_argument('--valgrind', action='store_true', help='Enable Valgrind profiling')
    
    args = parser.parse_args()
    
    profiler = BenchmarkProfiler()
    
    results = profiler.profile_benchmark(
        args.command,
        args.test_name,
        args.implementation,
        args.valgrind
    )
    
    profiler.save_profile_results(results, Path(args.output))
    
    if results['success']:
        print(f"Profiling completed successfully in {results['duration']:.3f}s")
    else:
        print(f"Profiling failed: {results.get('error', 'Unknown error')}")
        sys.exit(1)


if __name__ == '__main__':
    main()