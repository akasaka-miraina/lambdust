//! Module for registering all built-in FFI functions.
//!
//! This module provides the BuiltinFfiModule struct which handles the registration
//! of all built-in FFI functions from their respective specialized modules.

#![allow(missing_docs)]

use super::*;
use crate::ffi::arithmetic_functions::*;
use crate::ffi::string_functions::*;
use crate::ffi::list_functions::*;
use crate::ffi::type_checking_functions::*;
use crate::ffi::io_functions::*;

/// Module for registering all built-in FFI functions.
pub struct BuiltinFfiModule;

impl FfiModule for BuiltinFfiModule {
    fn register_functions(registry: &FfiRegistry) -> std::result::Result<(), FfiError> {
        // Arithmetic functions
        registry.register(AddFunction)?;
        registry.register(SubtractFunction)?;
        registry.register(MultiplyFunction)?;
        registry.register(DivideFunction)?;
        
        // String functions
        registry.register(StringLengthFunction)?;
        registry.register(StringConcatFunction)?;
        registry.register(StringUpperFunction)?;
        registry.register(StringLowerFunction)?;
        
        // List functions
        registry.register(ListLengthFunction)?;
        registry.register(ListMapFunction)?;
        registry.register(ListFilterFunction)?;
        
        // Type checking functions
        registry.register(IsNumberFunction)?;
        registry.register(IsStringFunction)?;
        registry.register(IsListFunction)?;
        registry.register(IsBooleanFunction)?;
        
        // I/O functions
        registry.register(PrintFunction)?;
        registry.register(PrintlnFunction)?;
        
        Ok(())
    }
}