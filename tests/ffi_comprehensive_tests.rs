//! Comprehensive FFI system tests.
//!
//! This test suite covers all aspects of the FFI system including:
//! - Dynamic library loading
//! - Type marshalling and conversion
//! - Callback functions
//! - Memory management
//! - Safety validation
//! - Performance profiling
//! - Scheme API integration

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lambdust::eval::{Value, Environment};
use lambdust::ffi::*;

/// Test dynamic library loading
#[cfg(test)]
mod library_tests {
    use super::*;

    #[test]
    fn test_library_manager_creation() {
        let manager = LibraryManager::new();
        assert_eq!(manager.list_libraries().len(), 0);
    }

    #[test]
    fn test_library_search_config() {
        let manager = LibraryManager::new();
        let config = LibrarySearchConfig {
            search_paths: vec!["/usr/local/lib".into()],
            use_system_paths: true,
            use_current_dir: false,
            prefixes: vec!["lib".to_string()],
        };
        
        manager.set_search_config(config);
        manager.add_search_path("/custom/path");
    }

    #[test]
    fn test_library_statistics() {
        let manager = LibraryManager::new();
        let stats = manager.stats();
        assert_eq!(stats.currently_loaded, 0);
        assert_eq!(stats.total_loaded, 0);
    }

    #[test] 
    fn test_dependency_management() {
        let manager = LibraryManager::new();
        manager.add_dependency("app", "libfoo");
        manager.add_dependency("app", "libbar");
        
        let deps = manager.get_dependencies("app");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"libfoo".to_string()));
        assert!(deps.contains(&"libbar".to_string()));
    }
}

/// Test C type system and data marshalling
#[cfg(test)]
mod type_system_tests {
    use super::*;

    #[test]
    fn test_c_type_properties() {
        assert_eq!(CType::Int32.size(), 4);
        assert_eq!(CType::Int64.size(), 8);
        assert_eq!(CType::Float.size(), 4);
        assert_eq!(CType::Double.size(), 8);
        
        assert!(CType::Int32.is_numeric());
        assert!(CType::Float.is_numeric());
        assert!(!CType::CString.is_numeric());
        
        assert!(CType::CString.is_pointer());
        assert!(CType::Pointer(Box::new(CType::Int32)).is_pointer());
        assert!(!CType::Int32.is_pointer());
    }

    #[test]
    fn test_struct_definition() {
        let fields = vec![
            CField {
                name: "x".to_string(),
                c_type: CType::Int32,
                offset: 0,
            },
            CField {
                name: "y".to_string(),
                c_type: CType::Int32,
                offset: 4,
            },
        ];

        let struct_type = CType::Struct {
            name: "Point".to_string(),
            fields,
            alignment: 4,
            size: 8,
        };

        assert_eq!(struct_type.size(), 8);
        assert_eq!(struct_type.alignment(), 4);
    }

    #[test]
    fn test_type_marshaller() {
        let mut marshaller = TypeMarshaller::new();
        
        // Test basic type resolution
        assert!(marshaller.resolve_type("int").is_some());
        assert!(marshaller.resolve_type("string").is_some());
        assert!(marshaller.resolve_type("nonexistent").is_none());
        
        // Test type registration
        marshaller.register_alias("my_int".to_string(), CType::Int32);
        assert!(marshaller.resolve_type("my_int").is_some());
    }

    #[test]
    fn test_value_conversion() {
        let mut marshaller = TypeMarshaller::new();
        
        // Test integer conversion
        let value = Value::Integer(42);
        let buffer = marshaller.to_c_data(&value, &CType::Int32).unwrap();
        assert_eq!(buffer.size(), 4);
        
        let converted_back = marshaller.from_c_data(&buffer).unwrap();
        if let Value::Integer(n) = converted_back {
            assert_eq!(n, 42);
        } else {
            panic!("Expected integer value");
        }
    }

    #[test]
    fn test_string_conversion() {
        let mut marshaller = TypeMarshaller::new();
        
        let value = Value::String("hello world".to_string());
        let buffer = marshaller.to_c_data(&value, &CType::CString).unwrap();
        
        // Verify null termination
        unsafe {
            let c_str_ptr = *(buffer.as_ptr() as *const *const libc::c_char);
            let c_str = CStr::from_ptr(c_str_ptr);
            assert_eq!(c_str.to_str().unwrap(), "hello world");
        }
    }

    #[test]
    fn test_array_handling() {
        let array_type = CType::Array(Box::new(CType::Int32), 5);
        assert_eq!(array_type.size(), 20); // 5 * 4 bytes
        assert_eq!(array_type.alignment(), 4);
        
        if let Some(element_type) = array_type.element_type() {
            assert_eq!(*element_type, CType::Int32);
        } else {
            panic!("Expected element type");
        }
    }
}

/// Test callback function system
#[cfg(test)]
mod callback_tests {
    use super::*;

    #[test]
    fn test_callback_registry() {
        let registry = CallbackRegistry::new();
        let stats = registry.stats();
        assert_eq!(stats.currently_active, 0);
        assert_eq!(stats.total_registered, 0);
    }

    #[test]
    fn test_callback_signature() {
        let signature = CallbackSignature {
            name: "test_callback".to_string(),
            parameters: vec![CType::Int32, CType::CString],
            return_type: CType::Int32,
            variadic: false,
            calling_convention: CallingConvention::C,
        };

        assert_eq!(signature.name, "test_callback");
        assert_eq!(signature.parameters.len(), 2);
        assert!(!signature.variadic);
        assert_eq!(signature.calling_convention, CallingConvention::C);
    }

    #[test]
    fn test_callback_registration() {
        let registry = CallbackRegistry::new();
        let signature = CallbackSignature {
            name: "test_callback".to_string(),
            parameters: vec![CType::Int32],
            return_type: CType::Int32,
            variadic: false,
            calling_convention: CallingConvention::C,
        };

        let function = Value::Integer(42); // Placeholder function
        let environment = Arc::new(Mutex::new(Environment::new()));
        
        let result = registry.register_callback(
            signature,
            function,
            environment,
            Some(Duration::from_secs(60)),
        );
        
        assert!(result.is_ok());
        let callbacks = registry.list_callbacks();
        assert!(callbacks.contains(&"test_callback".to_string()));
    }

    #[test]
    fn test_callback_cleanup() {
        let registry = CallbackRegistry::new();
        
        // Initially no expired callbacks
        let cleaned = registry.cleanup_expired();
        assert_eq!(cleaned, 0);
    }

    #[test]
    fn test_callback_unregistration() {
        let registry = CallbackRegistry::new();
        
        // Try to unregister non-existent callback
        let result = registry.unregister_callback("nonexistent");
        assert!(matches!(result, Err(CallbackError::NotFound(_))));
    }
}

/// Test FFI memory management
#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn test_memory_manager() {
        let manager = FfiMemoryManager::new();
        let stats = manager.stats();
        assert_eq!(stats.current_usage, 0);
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_memory_allocation() {
        let manager = FfiMemoryManager::new();
        
        let ptr = manager.allocate(64, Some(CType::Int32)).unwrap();
        assert!(!ptr.as_ptr().is_null());
        
        let stats = manager.stats();
        assert_eq!(stats.active_allocations, 1);
        assert!(stats.current_usage >= 64);
        
        manager.deallocate(ptr).unwrap();
        
        let stats = manager.stats();
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_double_free_protection() {
        let manager = FfiMemoryManager::new();
        let ptr = manager.allocate(64, None).unwrap();
        
        // First free should succeed
        manager.deallocate(ptr).unwrap();
        
        // Second free should fail with double free error
        let result = manager.deallocate(ptr);
        assert!(matches!(result, Err(MemoryError::DoubleFree(_))));
        
        let stats = manager.stats();
        assert_eq!(stats.double_frees_prevented, 1);
    }

    #[test]
    fn test_memory_leak_detection() {
        let manager = FfiMemoryManager::new();
        
        // Allocate some memory without freeing
        let _ptr1 = manager.allocate(128, None).unwrap();
        let _ptr2 = manager.allocate(256, None).unwrap();
        
        // Check for leaks (won't detect immediately in test)
        let leaks = manager.check_leaks();
        // Leaks won't be detected immediately due to time threshold
        assert_eq!(leaks.len(), 0);
    }

    #[test]
    fn test_memory_pools() {
        let pool = MemoryPool::new("test_pool".to_string(), 64, 4);
        
        let ptr1 = pool.allocate().unwrap();
        let ptr2 = pool.allocate().unwrap();
        
        let stats = pool.stats();
        assert_eq!(stats.used_blocks, 2);
        assert_eq!(stats.free_blocks, 2);
        
        assert!(pool.deallocate(ptr1));
        assert!(pool.deallocate(ptr2));
        
        let stats = pool.stats();
        assert_eq!(stats.used_blocks, 0);
        assert_eq!(stats.free_blocks, 4);
    }

    #[test]
    fn test_memory_configuration() {
        let manager = FfiMemoryManager::new();
        let config = MemoryConfig {
            max_memory_usage: 1024,
            use_memory_pools: false,
            leak_detection: true,
            ..Default::default()
        };
        
        manager.configure(config);
        
        // Should be able to allocate within limit
        let ptr1 = manager.allocate(512, None).unwrap();
        
        // Should fail to allocate beyond limit
        let result = manager.allocate(600, None);
        assert!(matches!(result, Err(MemoryError::AllocationFailed { .. })));
        
        manager.deallocate(ptr1).unwrap();
    }
}

/// Test type safety and validation
#[cfg(test)]
mod safety_tests {
    use super::*;

    #[test]
    fn test_safety_validator() {
        let validator = TypeSafetyValidator::new();
        let stats = validator.stats();
        assert_eq!(stats.total_validations, 0);
    }

    #[test]
    fn test_function_signature_registration() {
        let validator = TypeSafetyValidator::new();
        let signature = FunctionSignature {
            name: "test_function".to_string(),
            parameters: vec![CType::Int32, CType::CString],
            return_type: CType::Int32,
            variadic: false,
            safe: true,
            constraints: vec![TypeConstraint::NonNull(1)],
        };

        validator.register_function_signature(signature.clone()).unwrap();
        
        let retrieved = validator.get_function_signature("test_function").unwrap();
        assert_eq!(retrieved.name, "test_function");
        assert_eq!(retrieved.parameters.len(), 2);
        assert_eq!(retrieved.constraints.len(), 1);
    }

    #[test]
    fn test_parameter_validation() {
        let validator = TypeSafetyValidator::new();
        let signature = FunctionSignature {
            name: "test_function".to_string(),
            parameters: vec![CType::Int32],
            return_type: CType::Int32,
            variadic: false,
            safe: true,
            constraints: vec![],
        };

        validator.register_function_signature(signature).unwrap();

        // Valid call
        let args = vec![Value::Integer(42)];
        let result = validator.validate_function_call("test_function", &args, ptr::null());
        assert!(result.is_ok());

        // Invalid call - wrong type
        let args = vec![Value::String("hello".to_string())];
        let result = validator.validate_function_call("test_function", &args, ptr::null());
        assert!(matches!(result, Err(SafetyError::RuntimeTypeCheck { .. })));
    }

    #[test]
    fn test_null_pointer_validation() {
        let validator = TypeSafetyValidator::new();
        let signature = FunctionSignature {
            name: "test_function".to_string(),
            parameters: vec![CType::CString],
            return_type: CType::Int32,
            variadic: false,
            safe: true,
            constraints: vec![TypeConstraint::NonNull(0)],
        };

        validator.register_function_signature(signature).unwrap();

        // Valid call
        let args = vec![Value::String("hello".to_string())];
        let result = validator.validate_function_call("test_function", &args, ptr::null());
        assert!(result.is_ok());

        // Invalid call - null pointer
        let args = vec![Value::Nil];
        let result = validator.validate_function_call("test_function", &args, ptr::null());
        assert!(matches!(result, Err(SafetyError::NullPointerDereference { .. })));
    }

    #[test]
    fn test_bounds_validation() {
        let validator = TypeSafetyValidator::new();
        let signature = FunctionSignature {
            name: "test_function".to_string(),
            parameters: vec![CType::Int32],
            return_type: CType::Int32,
            variadic: false,
            safe: true,
            constraints: vec![TypeConstraint::Bounds {
                parameter: 0,
                min: 0,
                max: 100,
            }],
        };

        validator.register_function_signature(signature).unwrap();

        // Valid call
        let args = vec![Value::Integer(50)];
        let result = validator.validate_function_call("test_function", &args, ptr::null());
        assert!(result.is_ok());

        // Invalid call - out of bounds
        let args = vec![Value::Integer(150)];
        let result = validator.validate_function_call("test_function", &args, ptr::null());
        assert!(matches!(result, Err(SafetyError::BoundaryViolation { .. })));
    }

    #[test]
    fn test_validation_rules() {
        let validator = TypeSafetyValidator::new();
        
        let rule = ValidationRule {
            name: "null_check".to_string(),
            trigger: ValidationTrigger::PreCall,
            validator: ValidationFunction::NullPointerCheck,
            enabled: true,
        };

        validator.add_validation_rule("test_function".to_string(), rule);

        // This would fail with null check
        let args = vec![Value::Nil];
        let result = validator.apply_validation_rules("test_function", &args, ValidationTrigger::PreCall);
        assert!(matches!(result, Err(SafetyError::NullPointerDereference { .. })));
    }
}

/// Test libffi integration
#[cfg(test)]
mod libffi_tests {
    use super::*;

    #[test]
    fn test_libffi_engine() {
        let engine = LibffiEngine::new();
        let stats = engine.stats();
        assert_eq!(stats.total_calls, 0);
        assert_eq!(stats.prepared_functions, 0);
    }

    #[test]
    fn test_type_conversion() {
        let engine = LibffiEngine::new();
        
        // Test basic type conversions
        let int_type = engine.convert_c_type_to_ffi_type(&CType::Int32).unwrap();
        // Would check libffi::Type equivalence in real implementation
        
        let float_type = engine.convert_c_type_to_ffi_type(&CType::Float).unwrap();
        let pointer_type = engine.convert_c_type_to_ffi_type(&CType::CString).unwrap();
    }

    #[test]
    fn test_ffi_interface() {
        let interface = FfiInterface::new();
        let builtins = interface.list_builtin_functions();
        assert!(!builtins.is_empty());
        assert!(builtins.contains(&"strlen".to_string()));
        assert!(builtins.contains(&"malloc".to_string()));
    }

    #[test]
    fn test_builtin_signatures() {
        let interface = FfiInterface::new();
        
        let strlen_sig = interface.get_builtin_signature("strlen").unwrap();
        assert_eq!(strlen_sig.name, "strlen");
        assert_eq!(strlen_sig.parameters.len(), 1);
        assert_eq!(strlen_sig.parameters[0], CType::CString);
        assert_eq!(strlen_sig.return_type, CType::CSizeT);
    }
}

/// Test Scheme API
#[cfg(test)]
mod scheme_api_tests {
    use super::*;

    #[test]
    fn test_scheme_ffi_api() {
        let api = SchemeFfiApi::new();
        let libraries = api.list_libraries();
        assert!(libraries.is_empty());
    }

    #[test]
    fn test_library_definition() {
        let api = SchemeFfiApi::new();
        
        let lib_def = LibraryDefinition {
            name: "test_lib".to_string(),
            path: None,
            functions: HashMap::new(),
            types: HashMap::new(),
            metadata: LibraryMetadata::default(),
        };

        let result = api.define_library(lib_def);
        assert!(result.is_ok());
        
        let libraries = api.list_libraries();
        assert!(libraries.contains(&"test_lib".to_string()));
    }

    #[test]
    fn test_function_wrapper_generation() {
        let api = SchemeFfiApi::new();
        
        let func_def = FunctionDefinition {
            name: "test_func".to_string(),
            c_name: None,
            signature: FunctionSignature {
                name: "test_func".to_string(),
                parameters: vec![CType::Int32, CType::CString],
                return_type: CType::Int32,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            wrapper_code: None,
            documentation: Some("A test function".to_string()),
        };

        let wrapper = api.generate_function_wrapper(&func_def).unwrap();
        assert!(wrapper.contains("ffi-test_func"));
        assert!(wrapper.contains("A test function"));
        assert!(wrapper.contains("param0"));
        assert!(wrapper.contains("param1"));
    }

    #[test]
    fn test_c_header_generation() {
        let api = SchemeFfiApi::new();
        
        let mut functions = HashMap::new();
        functions.insert("test_func".to_string(), FunctionDefinition {
            name: "test_func".to_string(),
            c_name: None,
            signature: FunctionSignature {
                name: "test_func".to_string(),
                parameters: vec![CType::Int32],
                return_type: CType::Int32,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            wrapper_code: None,
            documentation: None,
        });

        let lib_def = LibraryDefinition {
            name: "test_lib".to_string(),
            path: None,
            functions,
            types: HashMap::new(),
            metadata: LibraryMetadata::default(),
        };

        api.define_library(lib_def).unwrap();
        
        let header = api.generate_c_header("test_lib").unwrap();
        assert!(header.contains("#ifndef TEST_LIB_H"));
        assert!(header.contains("int test_func(int param0);"));
    }

    #[test]
    fn test_scheme_module_export() {
        let api = SchemeFfiApi::new();
        
        let mut functions = HashMap::new();
        functions.insert("test_func".to_string(), FunctionDefinition {
            name: "test_func".to_string(),
            c_name: None,
            signature: FunctionSignature {
                name: "test_func".to_string(),
                parameters: vec![],
                return_type: CType::Int32,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            wrapper_code: None,
            documentation: None,
        });

        let lib_def = LibraryDefinition {
            name: "test_lib".to_string(),
            path: None,
            functions,
            types: HashMap::new(),
            metadata: LibraryMetadata {
                description: Some("A test library".to_string()),
                ..Default::default()
            },
        };

        api.define_library(lib_def).unwrap();
        
        let module = api.export_as_scheme_module("test_lib").unwrap();
        assert!(module.contains("(define-library (ffi test_lib)"));
        assert!(module.contains("A test library"));
        assert!(module.contains("ffi-test_func"));
    }
}

/// Test profiling system
#[cfg(test)]
mod profiling_tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = FfiProfiler::new();
        assert!(!profiler.is_active());
    }

    #[test]
    fn test_profiler_start_stop() {
        let profiler = FfiProfiler::new();
        
        profiler.start().unwrap();
        assert!(profiler.is_active());
        
        profiler.stop().unwrap();
        assert!(!profiler.is_active());
    }

    #[test]
    fn test_call_recording() {
        let profiler = FfiProfiler::new();
        profiler.start().unwrap();
        
        let args = vec![Value::Integer(42)];
        let event_id = profiler.record_call_start("test_func", "test_lib", &args);
        assert!(event_id.is_some());
        
        let result = Ok(Value::Integer(84));
        profiler.record_call_end(event_id.unwrap(), &result);
        
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
    }

    #[test]
    fn test_memory_recording() {
        let profiler = FfiProfiler::new();
        profiler.start().unwrap();
        
        profiler.record_memory_allocation(0x1000, 64, AllocationType::FfiAllocation);
        
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.memory_stats.allocation_count, 1);
        assert_eq!(metrics.memory_stats.total_allocated, 64);
    }

    #[test]
    fn test_report_generation() {
        let profiler = FfiProfiler::new();
        profiler.start().unwrap();
        
        // Add some test data
        let args = vec![Value::Integer(42)];
        let event_id = profiler.record_call_start("test_func", "test_lib", &args).unwrap();
        profiler.record_call_end(event_id, &Ok(Value::Integer(84)));
        
        let text_report = profiler.generate_report("text").unwrap();
        assert!(text_report.contains("FFI Profiling Report"));
        assert!(text_report.contains("Total calls: 1"));
        
        let json_report = profiler.generate_report("json").unwrap();
        assert!(json_report.contains("total_calls"));
        
        let html_report = profiler.generate_report("html").unwrap();
        assert!(html_report.contains("<html>"));
    }

    #[test]
    fn test_configuration_validation() {
        let profiler = FfiProfiler::new();
        
        let invalid_config = ProfilingConfig {
            sampling_rate: -0.5, // Invalid
            ..Default::default()
        };
        
        let result = profiler.configure(invalid_config);
        assert!(matches!(result, Err(ProfilingError::InvalidConfig { .. })));
    }
}

/// Integration tests combining multiple subsystems
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_ffi_workflow() {
        // Create all components
        let library_manager = Arc::new(LibraryManager::new());
        let marshaller = Arc::new(RwLock::new(TypeMarshaller::new()));
        let validator = Arc::new(TypeSafetyValidator::new());
        let engine = LibffiEngine::with_components(
            Arc::clone(&marshaller),
            Arc::clone(&validator),
            Arc::clone(&library_manager),
        );

        // Configure the system
        let memory_config = MemoryConfig {
            max_memory_usage: 1024 * 1024, // 1MB limit
            leak_detection: true,
            double_free_protection: true,
            ..Default::default()
        };
        
        let memory_manager = FfiMemoryManager::new();
        memory_manager.configure(memory_config);

        // Register a function signature
        let signature = FunctionSignature {
            name: "test_function".to_string(),
            parameters: vec![CType::Int32, CType::CString],
            return_type: CType::Int32,
            variadic: false,
            safe: true,
            constraints: vec![
                TypeConstraint::NonNull(1),
                TypeConstraint::Bounds { parameter: 0, min: 0, max: 1000 },
            ],
        };

        validator.register_function_signature(signature).unwrap();

        // Test validation
        let args = vec![Value::Integer(42), Value::String("test".to_string())];
        let validation_result = validator.validate_function_call(
            "test_function",
            &args,
            ptr::null(),
        );
        assert!(validation_result.is_ok());

        // Test memory allocation
        let ptr = memory_manager.allocate(128, Some(CType::Int32)).unwrap();
        let stats = memory_manager.stats();
        assert_eq!(stats.active_allocations, 1);

        // Clean up
        memory_manager.deallocate(ptr).unwrap();
        let stats = memory_manager.stats();
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_profiling_integration() {
        let profiler = FfiProfiler::new();
        let memory_manager = FfiMemoryManager::new();
        
        // Start profiling
        profiler.start().unwrap();
        
        // Simulate FFI operations
        let args = vec![Value::Integer(42)];
        let event_id = profiler.record_call_start("strlen", "libc", &args).unwrap();
        
        // Simulate memory allocation
        let ptr = memory_manager.allocate(64, Some(CType::CString)).unwrap();
        profiler.record_memory_allocation(
            ptr.as_ptr() as usize,
            64,
            AllocationType::FfiAllocation,
        );
        
        // Simulate call completion
        profiler.record_call_end(event_id, &Ok(Value::Integer(5)));
        
        // Free memory
        memory_manager.deallocate(ptr).unwrap();
        profiler.record_memory_allocation(
            ptr.as_ptr() as usize,
            64,
            AllocationType::Deallocation,
        );
        
        // Check metrics
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.memory_stats.allocation_count, 1);
        assert_eq!(metrics.memory_stats.deallocation_count, 1);
        
        // Generate report
        let report = profiler.generate_report("text").unwrap();
        assert!(report.contains("Total calls: 1"));
        assert!(report.contains("Successful calls: 1"));
    }

    #[test]
    fn test_scheme_api_integration() {
        let api = SchemeFfiApi::new();
        
        // Define a library with functions
        let mut functions = HashMap::new();
        functions.insert("add".to_string(), FunctionDefinition {
            name: "add".to_string(),
            c_name: Some("add_integers".to_string()),
            signature: FunctionSignature {
                name: "add".to_string(),
                parameters: vec![CType::Int32, CType::Int32],
                return_type: CType::Int32,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            wrapper_code: None,
            documentation: Some("Add two integers".to_string()),
        });

        let lib_def = LibraryDefinition {
            name: "math_lib".to_string(),
            path: Some("/usr/lib/libmath.so".to_string()),
            functions,
            types: HashMap::new(),
            metadata: LibraryMetadata {
                version: Some("1.0.0".to_string()),
                description: Some("Math library".to_string()),
                ..Default::default()
            },
        };

        // Define the library
        api.define_library(lib_def).unwrap();
        
        // Generate Scheme module
        let module = api.export_as_scheme_module("math_lib").unwrap();
        assert!(module.contains("(define-library (ffi math_lib)"));
        assert!(module.contains("Math library"));
        assert!(module.contains("ffi-add"));
        
        // Generate C header
        let header = api.generate_c_header("math_lib").unwrap();
        assert!(header.contains("#ifndef MATH_LIB_H"));
        assert!(header.contains("int add(int param0, int param1);"));
        
        // Test wrapper generation
        let wrapper = api.get_wrapper("add").unwrap();
        assert!(wrapper.contains("Add two integers"));
        assert!(wrapper.contains("(define (ffi-add param0 param1)"));
    }

    #[test]
    fn test_error_handling_chain() {
        // Test error propagation through the system
        let validator = TypeSafetyValidator::new();
        
        // Invalid function call should propagate errors correctly
        let args = vec![Value::String("invalid".to_string())];
        let result = validator.validate_function_call("nonexistent", &args, ptr::null());
        
        // Should return appropriate error without panicking
        // The exact error type depends on implementation details
        assert!(result.is_err());
    }
}

/// Performance and stress tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_memory_allocation_performance() {
        let manager = FfiMemoryManager::new();
        let start = Instant::now();
        
        let mut ptrs = Vec::new();
        for _ in 0..1000 {
            let ptr = manager.allocate(64, None).unwrap();
            ptrs.push(ptr);
        }
        
        let alloc_time = start.elapsed();
        println!("1000 allocations took: {:?}", alloc_time);
        
        let start = Instant::now();
        for ptr in ptrs {
            manager.deallocate(ptr).unwrap();
        }
        let dealloc_time = start.elapsed();
        println!("1000 deallocations took: {:?}", dealloc_time);
        
        // Performance assertions (adjust based on acceptable performance)
        assert!(alloc_time < Duration::from_millis(100));
        assert!(dealloc_time < Duration::from_millis(100));
    }

    #[test]
    fn test_type_conversion_performance() {
        let mut marshaller = TypeMarshaller::new();
        let start = Instant::now();
        
        for i in 0..1000 {
            let value = Value::Integer(i);
            let buffer = marshaller.to_c_data(&value, &CType::Int32).unwrap();
            let _converted_back = marshaller.from_c_data(&buffer).unwrap();
        }
        
        let conversion_time = start.elapsed();
        println!("1000 type conversions took: {:?}", conversion_time);
        
        // Should be reasonably fast
        assert!(conversion_time < Duration::from_millis(50));
    }

    #[test]
    fn test_validation_performance() {
        let validator = TypeSafetyValidator::new();
        let signature = FunctionSignature {
            name: "test_function".to_string(),
            parameters: vec![CType::Int32, CType::CString],
            return_type: CType::Int32,
            variadic: false,
            safe: true,
            constraints: vec![TypeConstraint::NonNull(1)],
        };

        validator.register_function_signature(signature).unwrap();
        
        let args = vec![Value::Integer(42), Value::String("test".to_string())];
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _result = validator.validate_function_call("test_function", &args, ptr::null());
        }
        
        let validation_time = start.elapsed();
        println!("1000 validations took: {:?}", validation_time);
        
        // Validation should be fast
        assert!(validation_time < Duration::from_millis(20));
    }
}