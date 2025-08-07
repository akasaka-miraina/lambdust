//! Comprehensive Performance Benchmarks for Lambdust
//!
//! This benchmark suite provides detailed performance analysis for all core components
//! of the Lambdust Scheme interpreter. It focuses on identifying bottlenecks and 
//! measuring the impact of optimizations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput, black_box};
use lambdust::eval::{Value, Evaluator, Environment};
use lambdust::numeric::{NumericValue, NumericType};
use lambdust::eval::fast_path::{FastPathOp, execute_fast_path, is_fast_path_operation, get_fast_path_stats};
use lambdust::ast::Literal;
use lambdust::utils::{intern_symbol, profiler::{ProfileCategory, profile}};
use lambdust::containers::ThreadSafeHashTable;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

// ============================================================================
// MICRO-BENCHMARKS FOR PRIMITIVE OPERATIONS
// ============================================================================

fn bench_numeric_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric_operations");
    group.measurement_time(Duration::from_secs(5));
    
    // Integer arithmetic benchmarks
    let integers = vec![
        NumericValue::integer(42),
        NumericValue::integer(17),
        NumericValue::integer(100),
        NumericValue::integer(-25),
    ];
    
    group.bench_function("integer_addition", |b| {
        b.iter(|| {
            let mut sum = NumericValue::integer(0);
            for num in &integers {
                sum = sum.add(black_box(num)).unwrap();
            }
            black_box(sum)
        });
    });
    
    group.bench_function("integer_multiplication", |b| {
        b.iter(|| {
            let mut product = NumericValue::integer(1);
            for num in &integers {
                product = product.multiply(black_box(num)).unwrap();
            }
            black_box(product)
        });
    });
    
    // Floating point arithmetic
    let floats = vec![
        NumericValue::real(3.14159),
        NumericValue::real(2.71828),
        NumericValue::real(1.41421),
        NumericValue::real(0.57721),
    ];
    
    group.bench_function("float_addition", |b| {
        b.iter(|| {
            let mut sum = NumericValue::real(0.0);
            for num in &floats {
                sum = sum.add(black_box(num)).unwrap();
            }
            black_box(sum)
        });
    });
    
    group.bench_function("float_division", |b| {
        b.iter(|| {
            let mut result = NumericValue::real(1000.0);
            for num in &floats {
                result = result.divide(black_box(num)).unwrap();
            }
            black_box(result)
        });
    });
    
    // Complex number operations
    let complex_nums = vec![
        NumericValue::complex(1.0, 2.0),
        NumericValue::complex(3.0, -1.0),
        NumericValue::complex(-2.0, 4.0),
        NumericValue::complex(0.5, -0.5),
    ];
    
    group.bench_function("complex_multiplication", |b| {
        b.iter(|| {
            let mut product = NumericValue::complex(1.0, 0.0);
            for num in &complex_nums {
                product = product.multiply(black_box(num)).unwrap();
            }
            black_box(product)
        });
    });
    
    // Rational number operations
    let rationals = vec![
        NumericValue::rational(1, 2),
        NumericValue::rational(3, 4),
        NumericValue::rational(5, 6),
        NumericValue::rational(7, 8),
    ];
    
    group.bench_function("rational_addition", |b| {
        b.iter(|| {
            let mut sum = NumericValue::rational(0, 1);
            for num in &rationals {
                sum = sum.add(black_box(num)).unwrap();
            }
            black_box(sum)
        });
    });
    
    // Numeric tower conversion overhead
    group.bench_function("numeric_tower_promotion", |b| {
        b.iter(|| {
            let int_val = black_box(NumericValue::integer(42));
            let float_val = black_box(NumericValue::real(3.14));
            let complex_val = black_box(NumericValue::complex(1.0, 2.0));
            
            // Force promotions
            let _result1 = int_val.add(&float_val).unwrap();
            let _result2 = float_val.multiply(&complex_val).unwrap();
            let _result3 = int_val.divide(&complex_val).unwrap();
        });
    });
    
    group.finish();
}

fn bench_fast_path_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_path_operations");
    group.measurement_time(Duration::from_secs(5));
    
    // Test arithmetic fast paths
    let add_symbol = intern_symbol("+");
    let sub_symbol = intern_symbol("-");
    let mul_symbol = intern_symbol("*");
    let div_symbol = intern_symbol("/");
    
    let numeric_args = vec![
        Value::integer(42),
        Value::integer(17),
        Value::integer(25),
        Value::number(3.14),
        Value::number(2.71),
    ];
    
    group.bench_function("fast_path_addition", |b| {
        b.iter(|| {
            if let Some(op) = is_fast_path_operation(add_symbol) {
                let result = execute_fast_path(op, black_box(&numeric_args));
                black_box(result)
            }
        });
    });
    
    group.bench_function("fast_path_multiplication", |b| {
        b.iter(|| {
            if let Some(op) = is_fast_path_operation(mul_symbol) {
                let result = execute_fast_path(op, black_box(&numeric_args));
                black_box(result)
            }
        });
    });
    
    // Test list operations
    let cons_symbol = intern_symbol("cons");
    let car_symbol = intern_symbol("car");
    let cdr_symbol = intern_symbol("cdr");
    
    let list = Value::pair(
        Value::integer(1),
        Value::pair(
            Value::integer(2),
            Value::pair(Value::integer(3), Value::Nil)
        )
    );
    
    group.bench_function("fast_path_cons", |b| {
        b.iter(|| {
            if let Some(op) = is_fast_path_operation(cons_symbol) {
                let args = vec![black_box(Value::integer(42)), black_box(Value::Nil)];
                let result = execute_fast_path(op, &args);
                black_box(result)
            }
        });
    });
    
    group.bench_function("fast_path_car", |b| {
        b.iter(|| {
            if let Some(op) = is_fast_path_operation(car_symbol) {
                let args = vec![black_box(list.clone())];
                let result = execute_fast_path(op, &args);
                black_box(result)
            }
        });
    });
    
    // Test type predicates
    let is_number_symbol = intern_symbol("number?");
    let is_pair_symbol = intern_symbol("pair?");
    let is_null_symbol = intern_symbol("null?");
    
    let test_values = vec![
        Value::integer(42),
        Value::number(3.14),
        Value::string("hello".to_string()),
        Value::Nil,
        list.clone(),
    ];
    
    group.bench_function("fast_path_type_predicates", |b| {
        b.iter(|| {
            for value in &test_values {
                if let Some(op) = is_fast_path_operation(is_number_symbol) {
                    let args = vec![black_box(value.clone())];
                    let _result = execute_fast_path(op, &args);
                }
                if let Some(op) = is_fast_path_operation(is_pair_symbol) {
                    let args = vec![black_box(value.clone())];
                    let _result = execute_fast_path(op, &args);
                }
            }
        });
    });
    
    group.finish();
}

fn bench_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_operations");
    group.measurement_time(Duration::from_secs(5));
    
    // Generate lists of varying sizes
    let sizes = vec![10, 100, 1000, 5000];
    
    for &size in &sizes {
        // Create a proper list of the given size
        let mut list = Value::Nil;
        for i in (0..size).rev() {
            list = Value::pair(Value::integer(i), list);
        }
        
        group.throughput(Throughput::Elements(size as u64));
        
        // Benchmark list length calculation
        group.bench_with_input(BenchmarkId::new("list_length", size), &list, |b, list| {
            b.iter(|| {
                let mut current = black_box(list);
                let mut length = 0;
                loop {
                    match current {
                        Value::Nil => break,
                        Value::Pair(_, cdr) => {
                            length += 1;
                            current = cdr.as_ref();
                        }
                        _ => break,
                    }
                }
                black_box(length)
            });
        });
        
        // Benchmark list traversal
        group.bench_with_input(BenchmarkId::new("list_traversal", size), &list, |b, list| {
            b.iter(|| {
                let mut current = black_box(list);
                let mut sum = 0;
                loop {
                    match current {
                        Value::Nil => break,
                        Value::Pair(car, cdr) => {
                            if let Some(n) = car.as_integer() {
                                sum += n;
                            }
                            current = cdr.as_ref();
                        }
                        _ => break,
                    }
                }
                black_box(sum)
            });
        });
        
        // Benchmark list append (create new list)
        let small_list = Value::pair(Value::integer(999), Value::Nil);
        group.bench_with_input(BenchmarkId::new("list_append", size), &(list.clone(), small_list.clone()), |b, (list1, list2)| {
            b.iter(|| {
                let mut result = black_box(list2.clone());
                let mut current = black_box(list1);
                let mut elements = Vec::new();
                
                // Collect elements
                loop {
                    match current {
                        Value::Nil => break,
                        Value::Pair(car, cdr) => {
                            elements.push(car.as_ref().clone());
                            current = cdr.as_ref();
                        }
                        _ => break,
                    }
                }
                
                // Build new list
                for element in elements.into_iter().rev() {
                    result = Value::pair(element, result);
                }
                black_box(result)
            });
        });
    }
    
    group.finish();
}

fn bench_hash_table_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_table_operations");
    group.measurement_time(Duration::from_secs(5));
    
    // Test with different hash table sizes
    let sizes = vec![10, 100, 1000, 10000];
    
    for &size in &sizes {
        group.throughput(Throughput::Elements(size as u64));
        
        // Create a hash table with the given number of elements
        let mut table = ThreadSafeHashTable::new();
        for i in 0..size {
            let key = Value::integer(i);
            let value = Value::string(format!("value_{}", i));
            table.insert(key, value).unwrap();
        }
        let table = Arc::new(table);
        
        // Benchmark hash table lookup
        group.bench_with_input(BenchmarkId::new("hash_lookup", size), &table, |b, table| {
            b.iter(|| {
                let key = Value::integer(black_box(size / 2));
                let result = table.get(&key).unwrap();
                black_box(result)
            });
        });
        
        // Benchmark hash table insertion
        group.bench_with_input(BenchmarkId::new("hash_insert", size), &table, |b, table| {
            b.iter(|| {
                let key = Value::integer(black_box(size + 1000));
                let value = Value::string("new_value".to_string());
                let result = table.insert(key, value);
                black_box(result)
            });
        });
        
        // Benchmark hash table iteration
        group.bench_with_input(BenchmarkId::new("hash_iterate", size), &table, |b, table| {
            b.iter(|| {
                let mut count = 0;
                for (key, value) in table.iter() {
                    black_box((key, value));
                    count += 1;
                }
                black_box(count)
            });
        });
    }
    
    group.finish();
}

fn bench_symbol_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbol_interning");
    group.measurement_time(Duration::from_secs(5));
    
    // Common symbols that should be interned
    let common_symbols = vec![
        "lambda", "define", "if", "cond", "case", "let", "let*", "letrec",
        "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
        "cons", "car", "cdr", "list", "append", "map", "fold",
        "number?", "string?", "pair?", "null?", "procedure?",
    ];
    
    group.bench_function("intern_common_symbols", |b| {
        b.iter(|| {
            for symbol in &common_symbols {
                let id = intern_symbol(black_box(symbol));
                black_box(id);
            }
        });
    });
    
    // Unique symbols that haven't been interned
    group.bench_function("intern_unique_symbols", |b| {
        let mut counter = 0;
        b.iter(|| {
            let unique_symbol = format!("unique_symbol_{}", counter);
            counter += 1;
            let id = intern_symbol(black_box(&unique_symbol));
            black_box(id);
        });
    });
    
    // Very long symbols
    let long_symbol = "a".repeat(1000);
    group.bench_function("intern_long_symbol", |b| {
        b.iter(|| {
            let id = intern_symbol(black_box(&long_symbol));
            black_box(id);
        });
    });
    
    group.finish();
}

fn bench_environment_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("environment_operations");
    group.measurement_time(Duration::from_secs(5));
    
    // Test environment lookups at different depths
    let depths = vec![1, 5, 10, 20, 50];
    
    for &depth in &depths {
        // Create nested environments
        let mut env = Environment::new();
        let test_symbol = intern_symbol("test_var");
        let test_value = Value::integer(42);
        
        // Create nested scopes
        for i in 0..depth {
            let symbol = intern_symbol(&format!("var_{}", i));
            let value = Value::integer(i as i64);
            env.bind(symbol, value);
            
            if i < depth - 1 {
                env = Environment::new_child(Arc::new(env));
            }
        }
        
        // Bind the test variable in the innermost scope
        env.bind(test_symbol, test_value.clone());
        
        group.throughput(Throughput::Elements(depth as u64));
        
        // Benchmark variable lookup
        group.bench_with_input(BenchmarkId::new("env_lookup", depth), &env, |b, env| {
            b.iter(|| {
                let result = env.lookup(black_box(test_symbol));
                black_box(result)
            });
        });
        
        // Benchmark variable binding
        group.bench_with_input(BenchmarkId::new("env_bind", depth), &env, |b, env| {
            b.iter(|| {
                let new_symbol = intern_symbol("new_binding");
                let new_value = Value::string("test".to_string());
                env.bind(black_box(new_symbol), black_box(new_value));
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// MACRO-BENCHMARKS FOR REALISTIC SCHEME PROGRAMS
// ============================================================================

fn bench_factorial_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("factorial_computation");
    group.measurement_time(Duration::from_secs(8));
    
    let factorial_program = r#"
        (define (factorial n)
          (if (= n 0)
              1
              (* n (factorial (- n 1)))))
    "#;
    
    let sizes = vec![5, 10, 15, 20];
    
    for &n in &sizes {
        group.throughput(Throughput::Elements(n as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let evaluator = Evaluator::new();
                let factorial_call = format!("(factorial {})", n);
                
                // In a real implementation, we would:
                // 1. Parse the program
                // 2. Evaluate the define
                // 3. Evaluate the factorial call
                // For now, we simulate the computational work
                
                fn factorial(n: u64) -> u64 {
                    if n == 0 { 1 } else { n * factorial(n - 1) }
                }
                
                let result = factorial(black_box(n));
                black_box(result)
            });
        });
    }
    
    group.finish();
}

fn bench_fibonacci_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci_computation");
    group.measurement_time(Duration::from_secs(10));
    
    let sizes = vec![10, 15, 20, 25];
    
    for &n in &sizes {
        group.throughput(Throughput::Elements(n as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                // Simulate Scheme fibonacci computation overhead
                fn fib(n: u64) -> u64 {
                    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
                }
                
                let result = fib(black_box(n));
                black_box(result)
            });
        });
    }
    
    group.finish();
}

fn bench_list_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_processing");
    group.measurement_time(Duration::from_secs(8));
    
    let sizes = vec![100, 500, 1000, 2000];
    
    for &size in &sizes {
        // Create a list to process
        let mut list = Value::Nil;
        for i in (1..=size).rev() {
            list = Value::pair(Value::integer(i), list);
        }
        
        group.throughput(Throughput::Elements(size as u64));
        
        // Benchmark map-like operation
        group.bench_with_input(BenchmarkId::new("map_square", size), &list, |b, list| {
            b.iter(|| {
                let mut result = Value::Nil;
                let mut current = black_box(list);
                let mut elements = Vec::new();
                
                // Collect elements
                loop {
                    match current {
                        Value::Nil => break,
                        Value::Pair(car, cdr) => {
                            if let Some(n) = car.as_integer() {
                                elements.push(Value::integer(n * n));
                            }
                            current = cdr.as_ref();
                        }
                        _ => break,
                    }
                }
                
                // Build result list
                for element in elements.into_iter().rev() {
                    result = Value::pair(element, result);
                }
                black_box(result)
            });
        });
        
        // Benchmark fold-like operation
        group.bench_with_input(BenchmarkId::new("fold_sum", size), &list, |b, list| {
            b.iter(|| {
                let mut sum = 0;
                let mut current = black_box(list);
                
                loop {
                    match current {
                        Value::Nil => break,
                        Value::Pair(car, cdr) => {
                            if let Some(n) = car.as_integer() {
                                sum += n;
                            }
                            current = cdr.as_ref();
                        }
                        _ => break,
                    }
                }
                black_box(sum)
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// MEMORY ALLOCATION AND GC BENCHMARKS
// ============================================================================

fn bench_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    group.measurement_time(Duration::from_secs(8));
    
    // Benchmark small object allocation
    group.bench_function("small_object_allocation", |b| {
        b.iter(|| {
            let mut objects = Vec::new();
            for i in 0..1000 {
                let obj = Value::integer(black_box(i));
                objects.push(obj);
            }
            black_box(objects)
        });
    });
    
    // Benchmark large object allocation
    group.bench_function("large_object_allocation", |b| {
        b.iter(|| {
            let mut objects = Vec::new();
            for i in 0..100 {
                let large_string = "x".repeat(black_box(1000 + i));
                let obj = Value::string(large_string);
                objects.push(obj);
            }
            black_box(objects)
        });
    });
    
    // Benchmark mixed allocation patterns
    group.bench_function("mixed_allocation_pattern", |b| {
        b.iter(|| {
            let mut objects = Vec::new();
            for i in 0..500 {
                match black_box(i % 4) {
                    0 => objects.push(Value::integer(i)),
                    1 => objects.push(Value::number(i as f64 * 3.14)),
                    2 => objects.push(Value::string(format!("str_{}", i))),
                    3 => objects.push(Value::pair(Value::integer(i), Value::Nil)),
                    _ => unreachable!(),
                }
            }
            black_box(objects)
        });
    });
    
    // Benchmark nested structure allocation
    group.bench_function("nested_structure_allocation", |b| {
        b.iter(|| {
            let mut list = Value::Nil;
            for i in 0..black_box(100) {
                let pair = Value::pair(
                    Value::integer(i),
                    Value::pair(Value::string(format!("item_{}", i)), Value::Nil)
                );
                list = Value::pair(pair, list);
            }
            black_box(list)
        });
    });
    
    group.finish();
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    micro_benchmarks,
    bench_numeric_operations,
    bench_fast_path_operations,
    bench_list_operations,
    bench_hash_table_operations,
    bench_symbol_interning,
    bench_environment_operations
);

criterion_group!(
    macro_benchmarks,
    bench_factorial_computation,
    bench_fibonacci_computation,
    bench_list_processing
);

criterion_group!(
    memory_benchmarks,
    bench_memory_allocation_patterns
);

criterion_main!(
    micro_benchmarks,
    macro_benchmarks,
    memory_benchmarks
);