//! Comprehensive benchmark suite for Lambdust containers.
//!
//! This benchmark compares the performance of Lambdust's container implementations
//! against standard Rust collections and evaluates R7RS-large compliance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::containers::*;
use lambdust::eval::value::Value;
use std::collections::{HashMap, VecDeque, BinaryHeap, BTreeSet};

fn bench_hash_tables(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_tables");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Benchmark Lambdust ThreadSafeHashTable insertion
        group.bench_with_input(
            BenchmarkId::new("lambdust_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let table = ThreadSafeHashTable::new();
                    for i in 0..size {
                        table.insert(
                            black_box(Value::number(i as f64)),
                            black_box(Value::string(format!("value{}", i))),
                        );
                    }
                    table
                });
            },
        );
        
        // Benchmark std::HashMap insertion
        group.bench_with_input(
            BenchmarkId::new("std_hashmap_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut map = HashMap::new();
                    for i in 0..size {
                        map.insert(black_box(i), black_box(format!("value{}", i)));
                    }
                    map
                });
            },
        );
        
        // Benchmark Lambdust ThreadSafeHashTable lookup
        let table = ThreadSafeHashTable::new();
        for i in 0..*size {
            table.insert(Value::number(i as f64), Value::string(format!("value{}", i)));
        }
        
        group.bench_with_input(
            BenchmarkId::new("lambdust_lookup", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(table.get(&Value::number(i as f64)));
                    }
                });
            },
        );
        
        // Benchmark std::HashMap lookup
        let mut std_map = HashMap::new();
        for i in 0..*size {
            std_map.insert(i, format!("value{}", i));
        }
        
        group.bench_with_input(
            BenchmarkId::new("std_hashmap_lookup", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(std_map.get(&i));
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_ideques(c: &mut Criterion) {
    let mut group = c.benchmark_group("ideques");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Benchmark PersistentIdeque cons (front insertion)
        group.bench_with_input(
            BenchmarkId::new("lambdust_cons", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut ideque = PersistentIdeque::new();
                    for i in 0..size {
                        ideque = ideque.cons(black_box(Value::number(i as f64)));
                    }
                    ideque
                });
            },
        );
        
        // Benchmark PersistentIdeque snoc (back insertion)
        group.bench_with_input(
            BenchmarkId::new("lambdust_snoc", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut ideque = PersistentIdeque::new();
                    for i in 0..size {
                        ideque = ideque.snoc(black_box(Value::number(i as f64)));
                    }
                    ideque
                });
            },
        );
        
        // Compare with std::VecDeque
        group.bench_with_input(
            BenchmarkId::new("std_vecdeque_front", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut deque = VecDeque::new();
                    for i in 0..size {
                        deque.push_front(black_box(i));
                    }
                    deque
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("std_vecdeque_back", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut deque = VecDeque::new();
                    for i in 0..size {
                        deque.push_back(black_box(i));
                    }
                    deque
                });
            },
        );
    }
    
    group.finish();
}

fn bench_priority_queues(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_queues");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Benchmark ThreadSafePriorityQueue insertion
        group.bench_with_input(
            BenchmarkId::new("lambdust_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let queue = ThreadSafePriorityQueue::new();
                    for i in 0..size {
                        queue.insert(
                            black_box(Value::string(format!("item{}", i))),
                            black_box(Value::number(i as f64)),
                        );
                    }
                    queue
                });
            },
        );
        
        // Compare with std::BinaryHeap
        group.bench_with_input(
            BenchmarkId::new("std_binaryheap_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut heap = BinaryHeap::new();
                    for i in 0..size {
                        heap.push(black_box(i));
                    }
                    heap
                });
            },
        );
        
        // Benchmark extraction
        let queue = ThreadSafePriorityQueue::new();
        for i in 0..*size {
            queue.insert(Value::string(format!("item{}", i)), Value::number(i as f64));
        }
        
        group.bench_with_input(
            BenchmarkId::new("lambdust_extract", size),
            size,
            |b, &size| {
                b.iter_batched(
                    || queue.clone(),
                    |queue| {
                        for _ in 0..size {
                            black_box(queue.extract());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

fn bench_ordered_sets(c: &mut Criterion) {
    let mut group = c.benchmark_group("ordered_sets");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Benchmark ThreadSafeOrderedSet insertion
        group.bench_with_input(
            BenchmarkId::new("lambdust_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let set = ThreadSafeOrderedSet::new();
                    for i in 0..size {
                        set.insert(black_box(Value::number(i as f64)));
                    }
                    set
                });
            },
        );
        
        // Compare with std::BTreeSet
        group.bench_with_input(
            BenchmarkId::new("std_btreeset_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut set = BTreeSet::new();
                    for i in 0..size {
                        set.insert(black_box(i));
                    }
                    set
                });
            },
        );
        
        // Benchmark contains operation
        let set = ThreadSafeOrderedSet::new();
        for i in 0..*size {
            set.insert(Value::number(i as f64));
        }
        
        group.bench_with_input(
            BenchmarkId::new("lambdust_contains", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(set.contains(&Value::number(i as f64)));
                    }
                });
            },
        );
        
        let std_set: BTreeSet<usize> = (0..*size).collect();
        group.bench_with_input(
            BenchmarkId::new("std_btreeset_contains", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(std_set.contains(&i));
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_list_queues(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_queues");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Benchmark ThreadSafeListQueue enqueue
        group.bench_with_input(
            BenchmarkId::new("lambdust_enqueue", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let queue = ThreadSafeListQueue::new();
                    for i in 0..size {
                        queue.enqueue(black_box(Value::number(i as f64)));
                    }
                    queue
                });
            },
        );
        
        // Benchmark dequeue
        let queue = ThreadSafeListQueue::new();
        for i in 0..*size {
            queue.enqueue(Value::number(i as f64));
        }
        
        group.bench_with_input(
            BenchmarkId::new("lambdust_dequeue", size),
            size,
            |b, &size| {
                b.iter_batched(
                    || queue.clone(),
                    |queue| {
                        for _ in 0..size {
                            black_box(queue.dequeue());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

fn bench_random_access_lists(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_access_lists");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Benchmark ThreadSafeRandomAccessList push_front
        group.bench_with_input(
            BenchmarkId::new("lambdust_push_front", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let list = ThreadSafeRandomAccessList::new();
                    for i in 0..size {
                        list.push_front(black_box(Value::number(i as f64)));
                    }
                    list
                });
            },
        );
        
        // Compare with Vec insert at front
        group.bench_with_input(
            BenchmarkId::new("std_vec_insert_front", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::new();
                    for i in 0..size {
                        vec.insert(0, black_box(i));
                    }
                    vec
                });
            },
        );
        
        // Benchmark random access
        let list = ThreadSafeRandomAccessList::new();
        for i in 0..*size {
            list.push_front(Value::number(i as f64));
        }
        
        group.bench_with_input(
            BenchmarkId::new("lambdust_random_access", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(list.get(i % list.len()));
                    }
                });
            },
        );
        
        let vec: Vec<usize> = (0..*size).collect();
        group.bench_with_input(
            BenchmarkId::new("std_vec_random_access", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        black_box(vec.get(i % vec.len()));
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // Test memory efficiency by measuring time to allocate and deallocate
    // large numbers of small containers
    
    let sizes = [1000, 5000, 10000];
    
    for size in sizes.iter() {
        group.bench_with_input(
            BenchmarkId::new("lambdust_small_hash_tables", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let tables: Vec<_> = (0..size)
                        .map(|i| {
                            let table = ThreadSafeHashTable::new();
                            table.insert(Value::number(i as f64), Value::string("test"));
                            table
                        })
                        .collect();
                    black_box(tables);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("std_small_hash_maps", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let maps: Vec<_> = (0..size)
                        .map(|i| {
                            let mut map = HashMap::new();
                            map.insert(i, "test");
                            map
                        })
                        .collect();
                    black_box(maps);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    
    // Benchmark concurrent operations (simulated by repeated access)
    let size = 1000;
    let iterations = 100;
    
    // Hash table concurrent reads
    let table = ThreadSafeHashTable::new();
    for i in 0..size {
        table.insert(Value::number(i as f64), Value::string(format!("value{}", i)));
    }
    
    group.bench_function("lambdust_concurrent_reads", |b| {
        b.iter(|| {
            for _ in 0..iterations {
                for i in 0..size {
                    black_box(table.get(&Value::number(i as f64)));
                }
            }
        });
    });
    
    // Priority queue concurrent operations
    let queue = ThreadSafePriorityQueue::new();
    for i in 0..size {
        queue.insert(Value::string(format!("item{}", i)), Value::number(i as f64));
    }
    
    group.bench_function("lambdust_priority_queue_mixed", |b| {
        b.iter(|| {
            let queue_clone = queue.clone();
            for i in 0..100 {
                if i % 2 == 0 {
                    queue_clone.insert(
                        Value::string(format!("new{}", i)),
                        Value::number(i as f64),
                    );
                } else {
                    black_box(queue_clone.extract());
                }
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_hash_tables,
    bench_ideques,
    bench_priority_queues,
    bench_ordered_sets,
    bench_list_queues,
    bench_random_access_lists,
    bench_memory_efficiency,
    bench_concurrent_access
);

criterion_main!(benches);