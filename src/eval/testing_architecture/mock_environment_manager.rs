use crate::eval::Environment;
use super::{EnvironmentCall, MockEnvironmentBehavior};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Mock environment manager for testing
#[derive(Debug)]
pub struct MockEnvironmentManager {
    /// Environment storage
    environments: Arc<Mutex<HashMap<u64, Rc<Environment>>>>,
    
    /// Environment ID counter
    next_id: Arc<Mutex<u64>>,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<EnvironmentCall>>>,
    
    /// Behavior configuration
    behavior: MockEnvironmentBehavior,
}