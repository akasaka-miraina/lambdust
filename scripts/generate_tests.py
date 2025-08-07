#!/usr/bin/env python3

"""
Test File Generator for Scheme Benchmarking

Generates standardized test files for different Scheme implementations,
handling syntax differences and ensuring fair comparison.
"""

import os
import sys
import yaml
import argparse
from pathlib import Path
from typing import Dict, List, Any


class TestGenerator:
    """Generates test files for different Scheme implementations."""
    
    def __init__(self, config_file: str):
        """Initialize with configuration."""
        with open(config_file, 'r') as f:
            self.config = yaml.safe_load(f)
        
        self.implementations = self.config['implementations']
        self.benchmark_suites = self.config['benchmark_suites']
    
    def generate_all_tests(self, output_dir: Path, target_implementations: List[str]):
        """Generate all test files for specified implementations."""
        
        for suite_name, suite_config in self.benchmark_suites.items():
            suite_dir = output_dir / suite_name
            suite_dir.mkdir(parents=True, exist_ok=True)
            
            print(f"Generating {suite_name} tests...")
            
            for test_name in suite_config['tests']:
                self.generate_test_variants(test_name, suite_dir, target_implementations)
    
    def generate_test_variants(self, test_name: str, output_dir: Path, target_implementations: List[str]):
        """Generate test variants for different implementations."""
        
        # Get the base test definition
        test_definition = self.get_test_definition(test_name)
        
        for impl_name in target_implementations:
            if impl_name not in self.implementations:
                continue
                
            impl_config = self.implementations[impl_name]
            file_extension = impl_config['file_extension']
            
            test_file = output_dir / f"{test_name}{file_extension}"
            
            # Generate implementation-specific test
            test_code = self.generate_test_code(test_name, impl_name, test_definition)
            
            with open(test_file, 'w') as f:
                f.write(test_code)
            
            print(f"  Generated {test_file}")
    
    def get_test_definition(self, test_name: str) -> Dict[str, Any]:
        """Get test definition with implementation-agnostic logic."""
        
        # Test definitions - these contain the core logic for each test
        test_definitions = {
            # Arithmetic benchmarks
            'arithmetic_ops': {
                'description': 'Basic arithmetic operations performance',
                'setup': '',
                'main_code': '''
(define (arithmetic-test n)
  (let loop ((i 0) (sum 0))
    (if (< i n)
        (loop (+ i 1) 
              (+ sum (* i i) (- i 1) (/ (+ i 1) 2)))
        sum)))

(define (run-test)
  (arithmetic-test 100000))
''',
                'expected_type': 'number'
            },
            
            'list_operations': {
                'description': 'List creation and traversal performance',
                'setup': '',
                'main_code': '''
(define (make-test-list n)
  (let loop ((i 0) (lst '()))
    (if (< i n)
        (loop (+ i 1) (cons i lst))
        lst)))

(define (sum-list lst)
  (let loop ((lst lst) (sum 0))
    (if (null? lst)
        sum
        (loop (cdr lst) (+ sum (car lst))))))

(define (run-test)
  (let ((test-list (make-test-list 10000)))
    (sum-list test-list)))
''',
                'expected_type': 'number'
            },
            
            'vector_operations': {
                'description': 'Vector creation and access performance',
                'setup': '',
                'main_code': '''
(define (vector-test n)
  (let ((vec (make-vector n 0)))
    (let loop ((i 0))
      (if (< i n)
          (begin
            (vector-set! vec i (* i i))
            (loop (+ i 1)))))
    (let loop ((i 0) (sum 0))
      (if (< i n)
          (loop (+ i 1) (+ sum (vector-ref vec i)))
          sum))))

(define (run-test)
  (vector-test 10000))
''',
                'expected_type': 'number'
            },
            
            'string_operations': {
                'description': 'String manipulation performance',
                'setup': '',
                'main_code': '''
(define (string-test n)
  (let ((base-str "benchmark"))
    (let loop ((i 0) (result ""))
      (if (< i n)
          (loop (+ i 1) (string-append result base-str))
          (string-length result)))))

(define (run-test)
  (string-test 1000))
''',
                'expected_type': 'number'
            },
            
            'function_calls': {
                'description': 'Function call overhead performance',
                'setup': '',
                'main_code': '''
(define (identity x) x)

(define (chain-calls n x)
  (if (<= n 0)
      x
      (identity (chain-calls (- n 1) x))))

(define (run-test)
  (chain-calls 10000 42))
''',
                'expected_type': 'number'
            },
            
            'closure_creation': {
                'description': 'Closure creation and invocation performance',
                'setup': '',
                'main_code': '''
(define (make-adder n)
  (lambda (x) (+ x n)))

(define (closure-test iterations)
  (let loop ((i 0) (sum 0))
    (if (< i iterations)
        (let ((adder (make-adder i)))
          (loop (+ i 1) (+ sum (adder i))))
        sum)))

(define (run-test)
  (closure-test 5000))
''',
                'expected_type': 'number'
            },
            
            'recursion_depth': {
                'description': 'Deep recursion performance',
                'setup': '',
                'main_code': '''
(define (deep-recursion n acc)
  (if (<= n 0)
      acc
      (deep-recursion (- n 1) (+ acc 1))))

(define (run-test)
  (deep-recursion 10000 0))
''',
                'expected_type': 'number'
            },
            
            'tail_call_optimization': {
                'description': 'Tail call optimization performance',
                'setup': '',
                'main_code': '''
(define (tail-factorial n acc)
  (if (<= n 1)
      acc
      (tail-factorial (- n 1) (* acc n))))

(define (run-test)
  (tail-factorial 1000 1))
''',
                'expected_type': 'number'
            },
            
            # Algorithm benchmarks
            'fibonacci': {
                'description': 'Fibonacci sequence calculation',
                'setup': '',
                'main_code': '''
(define (fib n)
  (if (< n 2)
      n
      (+ (fib (- n 1)) (fib (- n 2)))))

(define (run-test)
  (fib 30))
''',
                'expected_type': 'number'
            },
            
            'factorial': {
                'description': 'Factorial calculation',
                'setup': '',
                'main_code': '''
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(define (run-test)
  (factorial 100))
''',
                'expected_type': 'number'
            },
            
            'quicksort': {
                'description': 'Quicksort algorithm performance',
                'setup': '',
                'main_code': '''
(define (quicksort lst)
  (if (or (null? lst) (null? (cdr lst)))
      lst
      (let ((pivot (car lst))
            (rest (cdr lst)))
        (append
          (quicksort (filter (lambda (x) (< x pivot)) rest))
          (list pivot)
          (quicksort (filter (lambda (x) (>= x pivot)) rest))))))

(define (filter pred lst)
  (cond ((null? lst) '())
        ((pred (car lst)) (cons (car lst) (filter pred (cdr lst))))
        (else (filter pred (cdr lst)))))

(define (make-random-list n)
  (let loop ((i 0) (lst '()))
    (if (< i n)
        (loop (+ i 1) (cons (modulo (* i 7) 1000) lst))
        lst)))

(define (run-test)
  (length (quicksort (make-random-list 1000))))
''',
                'expected_type': 'number'
            },
            
            # Data structure benchmarks  
            'list_creation_access': {
                'description': 'List creation and random access',
                'setup': '',
                'main_code': '''
(define (list-ref-safe lst n)
  (if (or (null? lst) (<= n 0))
      #f
      (if (= n 0)
          (car lst)
          (list-ref-safe (cdr lst) (- n 1)))))

(define (test-list-access n)
  (let ((test-list (let loop ((i 0) (lst '()))
                     (if (< i n)
                         (loop (+ i 1) (cons i lst))
                         lst))))
    (let loop ((i 0) (sum 0))
      (if (< i (/ n 2))
          (let ((val (list-ref-safe test-list i)))
            (loop (+ i 1) (if val (+ sum val) sum)))
          sum))))

(define (run-test)
  (test-list-access 1000))
''',
                'expected_type': 'number'
            }
        }
        
        return test_definitions.get(test_name, {
            'description': f'Test for {test_name}',
            'setup': '',
            'main_code': f'(define (run-test) "placeholder-{test_name}")',
            'expected_type': 'any'
        })
    
    def generate_test_code(self, test_name: str, impl_name: str, test_definition: Dict[str, Any]) -> str:
        """Generate implementation-specific test code."""
        
        # Common header
        header = f'''; Benchmark: {test_name}
; Implementation: {impl_name}
; Description: {test_definition.get('description', 'Performance test')}
; Generated automatically - do not edit

'''
        
        # Implementation-specific setup
        setup = self.get_implementation_setup(impl_name)
        
        # Test-specific setup
        test_setup = test_definition.get('setup', '')
        
        # Main test code
        main_code = test_definition.get('main_code', '')
        
        # Implementation-specific timing and execution
        execution_code = self.get_execution_wrapper(impl_name, test_name)
        
        return f"{header}{setup}\n{test_setup}\n{main_code}\n{execution_code}"
    
    def get_implementation_setup(self, impl_name: str) -> str:
        """Get implementation-specific setup code."""
        
        setup_code = {
            'lambdust': '''
; Lambdust-specific setup
''',
            'chez': '''
; Chez Scheme setup
(import (chezscheme))
''',
            'racket': '''
#lang racket
; Racket setup
''',
            'gambit': '''
; Gambit setup
''',
            'gauche': '''
; Gauche setup
(use gauche.time)
''',
            'chicken': '''
; Chicken setup
''',
            'mit-scheme': '''
; MIT Scheme setup
''',
            'guile': '''
; Guile setup
''',
            'cyclone': '''
; Cyclone setup
'''
        }
        
        return setup_code.get(impl_name, '; Generic setup')
    
    def get_execution_wrapper(self, impl_name: str, test_name: str) -> str:
        """Get implementation-specific execution wrapper."""
        
        # Standard execution for most implementations
        standard_execution = '''
; Execute the test
(define start-time (current-jiffy))
(define result (run-test))
(define end-time (current-jiffy))
(define elapsed (/ (- end-time start-time) (jiffies-per-second)))

; Output results in standard format
(display "BENCHMARK_RESULT:")
(display result)
(newline)
(display "BENCHMARK_TIME:")
(display elapsed)
(newline)
'''
        
        execution_wrappers = {
            'chez': '''
; Execute the test with Chez timing
(import (only (chezscheme) time))
(define start-time (current-time 'time-monotonic))
(define result (run-test))
(define end-time (current-time 'time-monotonic))
(define elapsed (- end-time start-time))

(display "BENCHMARK_RESULT:")
(display result)
(newline)
(display "BENCHMARK_TIME:")
(display elapsed)
(newline)
''',
            
            'racket': '''
; Execute the test with Racket timing
(define start-time (current-inexact-milliseconds))
(define result (run-test))
(define end-time (current-inexact-milliseconds))
(define elapsed (/ (- end-time start-time) 1000.0))

(display "BENCHMARK_RESULT:")
(display result)
(newline)
(display "BENCHMARK_TIME:")
(display elapsed)
(newline)
''',
            
            'gauche': '''
; Execute the test with Gauche timing
(define start-time (sys-time))
(define result (run-test))
(define end-time (sys-time))
(define elapsed (- end-time start-time))

(display "BENCHMARK_RESULT:")
(display result)
(newline)
(display "BENCHMARK_TIME:")
(display elapsed)
(newline)
''',
            
            'guile': '''
; Execute the test with Guile timing
(use-modules (srfi srfi-19))
(define start-time (current-time))
(define result (run-test))
(define end-time (current-time))
(define elapsed (time-difference end-time start-time))

(display "BENCHMARK_RESULT:")
(display result)
(newline)
(display "BENCHMARK_TIME:")
(display (time-second elapsed))
(newline)
''',
        }
        
        return execution_wrappers.get(impl_name, standard_execution)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Generate Scheme benchmark tests')
    parser.add_argument('--config', required=True, help='Configuration file')
    parser.add_argument('--output-dir', required=True, help='Output directory')
    parser.add_argument('--implementations', default='all', 
                       help='Comma-separated implementations or "all"')
    
    args = parser.parse_args()
    
    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Initialize generator
    generator = TestGenerator(args.config)
    
    # Determine target implementations
    if args.implementations == 'all':
        target_implementations = list(generator.implementations.keys())
    else:
        target_implementations = [impl.strip() for impl in args.implementations.split(',')]
    
    print(f"Generating tests for implementations: {target_implementations}")
    
    # Generate all tests
    generator.generate_all_tests(output_dir, target_implementations)
    
    print("Test generation completed!")


if __name__ == '__main__':
    main()