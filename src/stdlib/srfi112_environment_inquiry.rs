//! SRFI-112: Environment Inquiry
//!
//! This module provides procedures to inquire about the Scheme implementation
//! and runtime environment according to SRFI-112 specification.
//!
//! ## Procedures
//!
//! - `implementation-name` - Returns the name of the Scheme implementation
//! - `implementation-version` - Returns the version of the implementation  
//! - `cpu-architecture` - Returns the CPU architecture
//! - `machine-name` - Returns the machine name
//! - `os-name` - Returns the operating system name
//! - `os-version` - Returns the operating system version

use crate::diagnostics::Result;
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::sync::Arc;

/// Initialize SRFI-112 environment inquiry procedures in the environment.
pub fn init_srfi112_environment_inquiry(env: &Arc<ThreadSafeEnvironment>) {
    // Implementation name
    env.define(
        "implementation-name".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "implementation-name".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_implementation_name),
            effects: vec![Effect::Pure],
        })),
    );

    // Implementation version
    env.define(
        "implementation-version".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "implementation-version".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_implementation_version),
            effects: vec![Effect::Pure],
        })),
    );

    // CPU architecture  
    env.define(
        "cpu-architecture".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "cpu-architecture".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_cpu_architecture),
            effects: vec![Effect::Pure],
        })),
    );

    // Machine name
    env.define(
        "machine-name".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "machine-name".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_machine_name),
            effects: vec![Effect::Pure],
        })),
    );

    // OS name
    env.define(
        "os-name".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "os-name".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_os_name),
            effects: vec![Effect::Pure],
        })),
    );

    // OS version
    env.define(
        "os-version".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "os-version".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_os_version),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Returns the name of the Scheme implementation
fn primitive_implementation_name(_args: &[Value]) -> Result<Value> {
    Ok(Value::string("lambdust"))
}

/// Returns the version of the implementation
fn primitive_implementation_version(_args: &[Value]) -> Result<Value> {
    Ok(Value::string(env!("CARGO_PKG_VERSION")))
}

/// Returns the CPU architecture
fn primitive_cpu_architecture(_args: &[Value]) -> Result<Value> {
    Ok(Value::string(std::env::consts::ARCH))
}

/// Returns the machine name (hostname)
fn primitive_machine_name(_args: &[Value]) -> Result<Value> {
    match hostname::get() {
        Ok(hostname) => Ok(Value::string(hostname.to_string_lossy())),
        Err(_) => Ok(Value::string("unknown")),
    }
}

/// Returns the operating system name
fn primitive_os_name(_args: &[Value]) -> Result<Value> {
    Ok(Value::string(std::env::consts::OS))
}

/// Returns the operating system version
fn primitive_os_version(_args: &[Value]) -> Result<Value> {
    match os_info::get().version() {
        os_info::Version::Unknown => Ok(Value::string("unknown")),
        version => Ok(Value::string(version.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::evaluator::Evaluator;
    use std::rc::Rc;

    fn test_environment() -> Arc<ThreadSafeEnvironment> {
        let env = Arc::new(ThreadSafeEnvironment::new_global());
        init_srfi112_environment_inquiry(&env);
        env
    }

    #[test]
    fn test_implementation_name() {
        let result = primitive_implementation_name(&[]).unwrap();
        assert_eq!(result.as_string(), Some("lambdust"));
    }

    #[test]
    fn test_implementation_version() {
        let result = primitive_implementation_version(&[]).unwrap();
        let version = result.as_string().unwrap();
        // Should be a valid version string (contains digits and dots)
        assert!(version.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_cpu_architecture() {
        let result = primitive_cpu_architecture(&[]).unwrap();
        let arch = result.as_string().unwrap();
        // Should be a valid architecture string
        assert!(!arch.is_empty());
        println!("CPU Architecture: {}", arch);
    }

    #[test]
    fn test_machine_name() {
        let result = primitive_machine_name(&[]).unwrap();
        let name = result.as_string().unwrap();
        // Should have a machine name (could be "unknown")
        assert!(!name.is_empty());
        println!("Machine Name: {}", name);
    }

    #[test]
    fn test_os_name() {
        let result = primitive_os_name(&[]).unwrap();
        let os = result.as_string().unwrap();
        // Should be a valid OS name
        assert!(!os.is_empty());
        println!("OS Name: {}", os);
    }

    #[test]
    fn test_os_version() {
        let result = primitive_os_version(&[]).unwrap();
        let version = result.as_string().unwrap();
        // Should have an OS version (could be "unknown")
        assert!(!version.is_empty());
        println!("OS Version: {}", version);
    }

    #[test]
    fn test_environment_integration() {
        let env = test_environment();
        
        // Test that all procedures are bound
        assert!(env.lookup("implementation-name").is_some());
        assert!(env.lookup("implementation-version").is_some());
        assert!(env.lookup("cpu-architecture").is_some());
        assert!(env.lookup("machine-name").is_some());
        assert!(env.lookup("os-name").is_some());
        assert!(env.lookup("os-version").is_some());
    }
}