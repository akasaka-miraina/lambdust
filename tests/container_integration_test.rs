//! Integration test for container implementations

use lambdust::containers::*;
use lambdust::eval::value::Value;

#[test]
fn test_hash_table_basic_operations() {
    let table = ThreadSafeHashTable::new();
    table.insert(Value::number(1.0), Value::string("one"));
    table.insert(Value::number(2.0), Value::string("two"));
    
    assert_eq!(table.len(), 2);
    assert!(table.get(&Value::number(1.0)).is_some());
    assert!(table.get(&Value::number(3.0)).is_none());
}

#[test]
fn test_ideque_operations() {
    let mut ideque = Ideque::new();
    ideque.push_back(Value::number(1.0));
    ideque.push_back(Value::number(2.0));
    ideque.push_front(Value::number(0.0));
    
    assert_eq!(ideque.len(), 3);
    assert_eq!(ideque.front(), Some(&Value::number(0.0)));
    assert_eq!(ideque.back(), Some(&Value::number(2.0)));
    
    assert_eq!(ideque.pop_front(), Some(Value::number(0.0)));
    assert_eq!(ideque.len(), 2);
}

#[test]
fn test_persistent_ideque() {
    let ideque = PersistentIdeque::new();
    let ideque1 = ideque.cons(Value::number(1.0));
    let ideque2 = ideque1.snoc(Value::number(2.0));
    
    // Original should be unchanged
    assert_eq!(ideque.len(), 0);
    assert_eq!(ideque1.len(), 1);
    assert_eq!(ideque2.len(), 2);
    
    assert_eq!(ideque2.front(), Some(Value::number(1.0)));
    assert_eq!(ideque2.back(), Some(Value::number(2.0)));
}

#[test]
fn test_priority_queue() {
    let pq = ThreadSafePriorityQueue::new();
    pq.insert(Value::string("low"), Value::number(1.0));
    pq.insert(Value::string("high"), Value::number(10.0));
    pq.insert(Value::string("medium"), Value::number(5.0));
    
    assert_eq!(pq.len(), 3);
    
    // Should extract highest priority first
    if let Some((_, priority)) = pq.extract() {
        // Due to max-heap behavior, should get 10.0
        assert_eq!(priority, Value::number(10.0));
    }
    assert_eq!(pq.len(), 2);
}

#[test]
fn test_ordered_set() {
    let set = ThreadSafeOrderedSet::new();
    set.insert(Value::number(3.0));
    set.insert(Value::number(1.0));
    set.insert(Value::number(2.0));
    set.insert(Value::number(2.0)); // Duplicate should not increase size
    
    assert_eq!(set.len(), 3);
    assert!(set.contains(&Value::number(2.0)));
    assert!(!set.contains(&Value::number(4.0)));
}

#[test]
fn test_list_queue() {
    let queue = ThreadSafeListQueue::new();
    queue.enqueue(Value::string("first"));
    queue.enqueue(Value::string("second"));
    queue.enqueue(Value::string("third"));
    
    assert_eq!(queue.len(), 3);
    
    // FIFO behavior
    assert_eq!(queue.dequeue(), Some(Value::string("first")));
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.front(), Some(Value::string("second")));
}

#[test]
fn test_random_access_list() {
    let list = ThreadSafeRandomAccessList::new();
    list.push_front(Value::number(3.0));  // [3]
    list.push_front(Value::number(2.0));  // [2, 3]
    list.push_front(Value::number(1.0));  // [1, 2, 3]
    
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0), Some(Value::number(1.0)));
    assert_eq!(list.get(1), Some(Value::number(2.0)));
    assert_eq!(list.get(2), Some(Value::number(3.0)));
    assert_eq!(list.get(3), None);
}