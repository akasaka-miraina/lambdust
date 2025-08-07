//! C type system integration and data marshalling for FFI operations.
//!
//! This module provides comprehensive support for converting between Rust/Lambdust
//! values and C types, including structs, arrays, pointers, and complex nested data.

use std::collections::HashMap;
use std::ffi::{CStr, CString, c_void};
use std::fmt;
use std::mem;
use std::slice;

use crate::eval::Value;
use crate::ast::Literal;
use crate::diagnostics::Error;

/// C type definitions
#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    /// Void type (for function returns)
    Void,
    /// Boolean (mapped to c_int)
    Bool,
    /// Signed integers
    Int8,
    Int16,
    Int32,
    Int64,
    /// Unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    /// Platform-dependent integer types
    CInt,
    CUInt,
    CSizeT,
    /// Floating point
    Float,
    Double,
    /// Character types
    Char,
    WChar,
    /// Pointer types
    Pointer(Box<CType>),
    /// Array types
    Array(Box<CType>, usize),
    /// String types
    CString,
    WString,
    /// Structure types
    Struct {
        name: String,
        fields: Vec<CField>,
        alignment: usize,
        size: usize,
    },
    /// Union types
    Union {
        name: String,
        fields: Vec<CField>,
        size: usize,
    },
    /// Function pointer types
    Function {
        return_type: Box<CType>,
        parameters: Vec<CType>,
        variadic: bool,
    },
    /// Opaque handle (void pointer with type name)
    Handle(String),
}

/// C structure field
#[derive(Debug, Clone, PartialEq)]
pub struct CField {
    pub name: String,
    pub c_type: CType,
    pub offset: usize,
}

impl CType {
    /// Get the size of this type in bytes
    pub fn size(&self) -> usize {
        match self {
            CType::Void => 0,
            CType::Bool => mem::size_of::<i32>(),
            CType::Int8 => 1,
            CType::Int16 => 2,
            CType::Int32 => 4,
            CType::Int64 => 8,
            CType::UInt8 => 1,
            CType::UInt16 => 2,
            CType::UInt32 => 4,
            CType::UInt64 => 8,
            CType::CInt => mem::size_of::<libc::c_int>(),
            CType::CUInt => mem::size_of::<libc::c_uint>(),
            CType::CSizeT => mem::size_of::<libc::size_t>(),
            CType::Float => 4,
            CType::Double => 8,
            CType::Char => 1,
            CType::WChar => mem::size_of::<libc::wchar_t>(),
            CType::Pointer(_) => mem::size_of::<*const c_void>(),
            CType::Array(element_type, count) => element_type.size() * count,
            CType::CString => mem::size_of::<*const libc::c_char>(),
            CType::WString => mem::size_of::<*const libc::wchar_t>(),
            CType::Struct { size, .. } => *size,
            CType::Union { size, .. } => *size,
            CType::Function { .. } => mem::size_of::<*const c_void>(),
            CType::Handle(_) => mem::size_of::<*const c_void>(),
        }
    }

    /// Get the alignment of this type
    pub fn alignment(&self) -> usize {
        match self {
            CType::Void => 1,
            CType::Bool => mem::align_of::<i32>(),
            CType::Int8 => 1,
            CType::Int16 => 2,
            CType::Int32 => 4,
            CType::Int64 => 8,
            CType::UInt8 => 1,
            CType::UInt16 => 2,
            CType::UInt32 => 4,
            CType::UInt64 => 8,
            CType::CInt => mem::align_of::<libc::c_int>(),
            CType::CUInt => mem::align_of::<libc::c_uint>(),
            CType::CSizeT => mem::align_of::<libc::size_t>(),
            CType::Float => 4,
            CType::Double => 8,
            CType::Char => 1,
            CType::WChar => mem::align_of::<libc::wchar_t>(),
            CType::Pointer(_) => mem::align_of::<*const c_void>(),
            CType::Array(element_type, _) => element_type.alignment(),
            CType::CString => mem::align_of::<*const libc::c_char>(),
            CType::WString => mem::align_of::<*const libc::wchar_t>(),
            CType::Struct { alignment, .. } => *alignment,
            CType::Union { fields, .. } => {
                fields.iter()
                    .map(|f| f.c_type.alignment())
                    .max()
                    .unwrap_or(1)
            }
            CType::Function { .. } => mem::align_of::<*const c_void>(),
            CType::Handle(_) => mem::align_of::<*const c_void>(),
        }
    }

    /// Check if this is a pointer type
    pub fn is_pointer(&self) -> bool {
        matches!(self, CType::Pointer(_) | CType::CString | CType::WString | CType::Function { .. } | CType::Handle(_))
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(self, 
            CType::Int8 | CType::Int16 | CType::Int32 | CType::Int64 |
            CType::UInt8 | CType::UInt16 | CType::UInt32 | CType::UInt64 |
            CType::CInt | CType::CUInt | CType::CSizeT |
            CType::Float | CType::Double
        )
    }

    /// Get the element type for arrays and pointers
    pub fn element_type(&self) -> Option<&CType> {
        match self {
            CType::Pointer(elem) => Some(elem),
            CType::Array(elem, _) => Some(elem),
            _ => None,
        }
    }
}

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CType::Void => write!(f, "void"),
            CType::Bool => write!(f, "bool"),
            CType::Int8 => write!(f, "int8_t"),
            CType::Int16 => write!(f, "int16_t"),
            CType::Int32 => write!(f, "int32_t"),
            CType::Int64 => write!(f, "int64_t"),
            CType::UInt8 => write!(f, "uint8_t"),
            CType::UInt16 => write!(f, "uint16_t"),
            CType::UInt32 => write!(f, "uint32_t"),
            CType::UInt64 => write!(f, "uint64_t"),
            CType::CInt => write!(f, "int"),
            CType::CUInt => write!(f, "unsigned int"),
            CType::CSizeT => write!(f, "size_t"),
            CType::Float => write!(f, "float"),
            CType::Double => write!(f, "double"),
            CType::Char => write!(f, "char"),
            CType::WChar => write!(f, "wchar_t"),
            CType::Pointer(inner) => write!(f, "{}*", inner),
            CType::Array(inner, size) => write!(f, "{}[{}]", inner, size),
            CType::CString => write!(f, "char*"),
            CType::WString => write!(f, "wchar_t*"),
            CType::Struct { name, .. } => write!(f, "struct {}", name),
            CType::Union { name, .. } => write!(f, "union {}", name),
            CType::Function { return_type, parameters, variadic }  => {
                write!(f, "{} (", return_type)?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                if *variadic {
                    if !parameters.is_empty() { write!(f, ", ")?; }
                    write!(f, "...")?;
                }
                write!(f, ")")
            }
            CType::Handle(name) => write!(f, "{}*", name),
        }
    }
}

/// Errors that can occur during type conversion
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Type mismatch
    TypeMismatch {
        expected: CType,
        actual: String,
    },
    /// Invalid pointer or null pointer dereference
    InvalidPointer,
    /// Buffer overflow or underflow
    BufferOverflow {
        buffer_size: usize,
        requested_size: usize,
    },
    /// String conversion error
    StringConversion(String),
    /// Struct field not found
    FieldNotFound {
        struct_name: String,
        field_name: String,
    },
    /// Array index out of bounds
    IndexOutOfBounds {
        index: usize,
        length: usize,
    },
    /// Memory allocation failure
    AllocationFailed(usize),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::TypeMismatch { expected, actual }  => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            ConversionError::InvalidPointer  => {
                write!(f, "Invalid or null pointer")
            }
            ConversionError::BufferOverflow { buffer_size, requested_size }  => {
                write!(f, "Buffer overflow: buffer size {}, requested {}", buffer_size, requested_size)
            }
            ConversionError::StringConversion(msg)  => {
                write!(f, "String conversion error: {}", msg)
            }
            ConversionError::FieldNotFound { struct_name, field_name }  => {
                write!(f, "Field '{}' not found in struct '{}'", field_name, struct_name)
            }
            ConversionError::IndexOutOfBounds { index, length }  => {
                write!(f, "Array index {} out of bounds (length {})", index, length)
            }
            ConversionError::AllocationFailed(size)  => {
                write!(f, "Memory allocation failed for {} bytes", size)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<ConversionError> for Error {
    fn from(conv_error: ConversionError) -> Self {
        Error::runtime_error(conv_error.to_string(), None)
    }
}

/// C data buffer for holding converted values
#[derive(Debug)]
pub struct CDataBuffer {
    /// Raw data buffer
    data: Vec<u8>,
    /// Type information
    c_type: CType,
    /// Whether this buffer owns the data
    owns_data: bool,
}

impl CDataBuffer {
    /// Create a new buffer for the given type
    pub fn new(c_type: CType) -> Self {
        let size = c_type.size();
        Self {
            data: vec![0u8; size],
            c_type,
            owns_data: true,
        }
    }

    /// Create a buffer from existing data (non-owning)
    pub unsafe fn from_raw(data: *const u8, c_type: CType) -> Self {
        let size = c_type.size();
        let data_slice = slice::from_raw_parts(data, size);
        Self {
            data: data_slice.to_vec(),
            c_type,
            owns_data: false,
        }
    }

    /// Get the type of this buffer
    pub fn c_type(&self) -> &CType {
        &self.c_type
    }

    /// Get raw data pointer
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get mutable raw data pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Get size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Convert to a specific type
    pub unsafe fn as_type<T>(&self) -> &T {
        &*(self.as_ptr() as *const T)
    }

    /// Convert to a mutable specific type
    pub unsafe fn as_type_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *(self.as_mut_ptr() as *mut T) }
    }
}

/// Type marshaller for converting between Lambdust values and C types
#[derive(Debug)]
pub struct TypeMarshaller {
    /// Registered struct definitions
    structs: HashMap<String, CType>,
    /// Type aliases
    aliases: HashMap<String, CType>,
    /// String cache for C strings
    string_cache: Vec<CString>,
}

impl Default for TypeMarshaller {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeMarshaller {
    /// Create a new type marshaller
    pub fn new() -> Self {
        let mut marshaller = Self {
            structs: HashMap::new(),
            aliases: HashMap::new(),
            string_cache: Vec::new(),
        };
        
        // Register common type aliases
        marshaller.register_alias("int".to_string(), CType::CInt);
        marshaller.register_alias("uint".to_string(), CType::CUInt);
        marshaller.register_alias("size_t".to_string(), CType::CSizeT);
        marshaller.register_alias("char*".to_string(), CType::CString);
        marshaller.register_alias("string".to_string(), CType::CString);
        
        marshaller
    }

    /// Register a struct definition
    pub fn register_struct(&mut self, name: String, fields: Vec<CField>) -> std::result::Result<(), ConversionError> {
        // Calculate struct layout
        let mut offset = 0;
        let mut max_alignment = 1;
        let mut calculated_fields = Vec::new();
        
        for field in fields {
            let field_alignment = field.c_type.alignment();
            max_alignment = max_alignment.max(field_alignment);
            
            // Align offset to field alignment
            offset = (offset + field_alignment - 1) & !(field_alignment - 1);
            
            calculated_fields.push(CField {
                name: field.name,
                c_type: field.c_type,
                offset,
            });
            
            offset += calculated_fields.last().unwrap().c_type.size();
        }
        
        // Align total size to struct alignment
        let size = (offset + max_alignment - 1) & !(max_alignment - 1);
        
        let struct_type = CType::Struct {
            name: name.clone()),
            fields: calculated_fields,
            alignment: max_alignment,
            size,
        };
        
        self.structs.insert(name, struct_type);
        Ok(())
    }

    /// Register a type alias
    pub fn register_alias(&mut self, alias: String, c_type: CType) {
        self.aliases.insert(alias, c_type);
    }

    /// Resolve a type name to a CType
    pub fn resolve_type(&self, name: &str) -> Option<&CType> {
        self.aliases.get(name).or_else(|| self.structs.get(name))
    }

    /// Convert a Lambdust value to C data
    pub fn to_c_data(&mut self, value: &Value, c_type: &CType) -> std::result::Result<CDataBuffer, ConversionError> {
        let mut buffer = CDataBuffer::new(c_type.clone());
        self.write_value_to_buffer(value, c_type, &mut buffer, 0)?;
        Ok(buffer)
    }

    /// Convert C data to a Lambdust value
    pub fn from_c_data(&self, buffer: &CDataBuffer) -> std::result::Result<Value, ConversionError> {
        self.read_value_from_buffer(buffer.c_type(), buffer, 0)
    }

    /// Write a value to a buffer at the given offset
    fn write_value_to_buffer(&mut self, value: &Value, c_type: &CType, buffer: &mut CDataBuffer, offset: usize) 
        -> std::result::Result<(), ConversionError> {
        
        if offset + c_type.size() > buffer.size() {
            return Err(ConversionError::BufferOverflow {
                buffer_size: buffer.size(),
                requested_size: offset + c_type.size(),
            });
        }

        unsafe {
            let ptr = buffer.as_mut_ptr().add(offset);
            
            match (value, c_type) {
                (Value::Literal(Literal::Number(i)), CType::Int8) if i.fract() == 0.0  => {
                    *(ptr as *mut i8) = *i as i8;
                }
                (Value::Literal(Literal::Number(i)), CType::Int16) if i.fract() == 0.0  => {
                    *(ptr as *mut i16) = *i as i16;
                }
                (Value::Literal(Literal::Number(i)), CType::Int32) => {
                    *(ptr as *mut i32) = *i as i32;
                }
                (Value::Literal(Literal::Number(i)), CType::Int64) => {
                    *(ptr as *mut i64) = *i as i64;
                }
                (Value::Literal(Literal::Number(i)), CType::UInt8) => {
                    *(ptr as *mut u8) = *i as u8;
                }
                (Value::Literal(Literal::Number(i)), CType::UInt16) => {
                    *(ptr as *mut u16) = *i as u16;
                }
                (Value::Literal(Literal::Number(i)), CType::UInt32) => {
                    *(ptr as *mut u32) = *i as u32;
                }
                (Value::Literal(Literal::Number(i)), CType::UInt64) => {
                    *(ptr as *mut u64) = *i as u64;
                }
                (Value::Literal(Literal::Number(i)), CType::CInt) => {
                    *(ptr as *mut libc::c_int) = *i as libc::c_int;
                }
                (Value::Literal(Literal::Number(i)), CType::CUInt) => {
                    *(ptr as *mut libc::c_uint) = *i as libc::c_uint;
                }
                (Value::Literal(Literal::Number(i)), CType::CSizeT) => {
                    *(ptr as *mut libc::size_t) = *i as libc::size_t;
                }
                (Value::Literal(Literal::Number(f)), CType::Float)  => {
                    *(ptr as *mut f32) = *f as f32;
                }
                (Value::Literal(Literal::Number(f)), CType::Double)  => {
                    *(ptr as *mut f64) = *f;
                }
                (Value::Literal(Literal::Boolean(b)), CType::Bool)  => {
                    *(ptr as *mut i32) = if *b { 1 } else { 0 };
                }
                (Value::Literal(Literal::String(s)), CType::CString)  => {
                    let c_string = CString::new(s.as_str())
                        .map_err(|e| ConversionError::StringConversion(e.to_string()))?;
                    *(ptr as *mut *const libc::c_char) = c_string.as_ptr();
                    self.string_cache.push(c_string); // Keep alive
                }
                (Value::Literal(Literal::Character(c)), CType::Char)  => {
                    *(ptr as *mut libc::c_char) = *c as libc::c_char;
                }
                _  => {
                    return Err(ConversionError::TypeMismatch {
                        expected: c_type.clone()),
                        actual: format!("{:?}", value),
                    });
                }
            }
        }

        Ok(())
    }

    /// Read a value from a buffer at the given offset
    fn read_value_from_buffer(&self, c_type: &CType, buffer: &CDataBuffer, offset: usize) 
        -> std::result::Result<Value, ConversionError> {
        
        if offset + c_type.size() > buffer.size() {
            return Err(ConversionError::BufferOverflow {
                buffer_size: buffer.size(),
                requested_size: offset + c_type.size(),
            });
        }

        unsafe {
            let ptr = buffer.as_ptr().add(offset);
            
            let value = match c_type {
                CType::Int8 => Value::Literal(Literal::Number(*(ptr as *const i8) as f64)),
                CType::Int16 => Value::Literal(Literal::Number(*(ptr as *const i16) as f64)),
                CType::Int32 => Value::Literal(Literal::Number(*(ptr as *const i32) as f64)),
                CType::Int64 => Value::Literal(Literal::Number(*(ptr as *const i64) as f64)),
                CType::UInt8 => Value::Literal(Literal::Number(*(ptr as *const u8) as f64)),
                CType::UInt16 => Value::Literal(Literal::Number(*(ptr as *const u16) as f64)),
                CType::UInt32 => Value::Literal(Literal::Number(*(ptr as *const u32) as f64)),
                CType::UInt64 => Value::Literal(Literal::Number(*(ptr as *const u64) as f64)),
                CType::CInt => Value::Literal(Literal::Number(*(ptr as *const libc::c_int) as f64)),
                CType::CUInt => Value::Literal(Literal::Number(*(ptr as *const libc::c_uint) as f64)),
                CType::CSizeT => Value::Literal(Literal::Number(*(ptr as *const libc::size_t) as f64)),
                CType::Float => Value::Literal(Literal::Number(*(ptr as *const f32) as f64)),
                CType::Double => Value::Literal(Literal::Number(*(ptr as *const f64))),
                CType::Bool  => {
                    let int_val = *(ptr as *const i32);
                    Value::Literal(Literal::Boolean(int_val != 0))
                }
                CType::Char  => {
                    let char_val = *(ptr as *const libc::c_char);
                    Value::Literal(Literal::Character(char_val as u8 as char))
                }
                CType::CString  => {
                    let c_str_ptr = *(ptr as *const *const libc::c_char);
                    if c_str_ptr.is_null() {
                        Value::Literal(Literal::String("".to_string()))
                    } else {
                        let c_str = CStr::from_ptr(c_str_ptr);
                        let rust_str = c_str.to_str()
                            .map_err(|e| ConversionError::StringConversion(e.to_string()))?;
                        Value::Literal(Literal::String(rust_str.to_string()))
                    }
                }
                _  => {
                    return Err(ConversionError::TypeMismatch {
                        expected: c_type.clone()),
                        actual: "unsupported type".to_string(),
                    });
                }
            };

            Ok(value)
        }
    }

    /// Clear the string cache
    pub fn clear_cache(&mut self) {
        self.string_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_type_sizes() {
        assert_eq!(CType::Int32.size(), 4);
        assert_eq!(CType::Int64.size(), 8);
        assert_eq!(CType::Float.size(), 4);
        assert_eq!(CType::Double.size(), 8);
        assert!(CType::Pointer(Box::new(CType::Int32)).size() >= mem::size_of::<*const c_void>());
    }

    #[test]
    fn test_c_type_display() {
        assert_eq!(CType::Int32.to_string(), "int32_t");
        assert_eq!(CType::Pointer(Box::new(CType::Int32)).to_string(), "int32_t*");
        assert_eq!(CType::Array(Box::new(CType::Int32), 10).to_string(), "int32_t[10]");
    }

    #[test]
    fn test_type_marshaller_creation() {
        let marshaller = TypeMarshaller::new();
        assert!(marshaller.resolve_type("int").is_some());
        assert!(marshaller.resolve_type("string").is_some());
    }

    #[test]
    fn test_basic_conversion() {
        let mut marshaller = TypeMarshaller::new();
        let value = Value::Literal(Literal::Number(42.0));
        let buffer = marshaller.to_c_data(&value, &CType::Int32).unwrap();
        
        unsafe {
            let int_val = *(buffer.as_ptr() as *const i32);
            assert_eq!(int_val, 42);
        }
    }

    #[test] 
    fn test_string_conversion() {
        let mut marshaller = TypeMarshaller::new();
        let value = Value::Literal(Literal::String("hello".to_string()));
        let buffer = marshaller.to_c_data(&value, &CType::CString).unwrap();
        
        unsafe {
            let c_str_ptr = *(buffer.as_ptr() as *const *const libc::c_char);
            assert!(!c_str_ptr.is_null());
            
            let c_str = CStr::from_ptr(c_str_ptr);
            assert_eq!(c_str.to_str().unwrap(), "hello");
        }
    }
}