#![allow(unused_variables)]
//! Streaming I/O operations with lazy evaluation and compression support.
//!
//! This module provides:
//! - Lazy streaming I/O operations
//! - Compression and decompression streams
//! - Pipeline and stream transformation utilities
//! - Memory-efficient processing of large datasets
//! - Backpressure handling and flow control

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{
    Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment
};
use crate::effects::Effect;
use std::sync::Arc;
use std::collections::VecDeque;
// Most IO traits not currently used
// use std::io::{Read, Write, BufReader, BufWriter};

#[cfg(feature = "compression")]
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
#[cfg(feature = "compression")]
use zstd::{Decoder as ZstdDecoder, Encoder as ZstdEncoder};
#[cfg(feature = "compression")]
use lz4_flex::{compress, decompress};

use memmap2::MmapOptions;
use std::fs::File;
use std::path::Path;

/// Stream processing state
#[derive(Debug, Clone)]
pub enum StreamState {
    Ready,
    Processing,
    Finished,
    Error(String),
}

/// Stream chunk with metadata
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub data: Vec<u8>,
    pub position: u64,
    pub is_final: bool,
    pub compression: Option<CompressionType>,
}

/// Compression algorithms
#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    Gzip,
    Zstd,
    Lz4,
}

/// Streaming I/O processor
#[derive(Debug)]
pub struct StreamProcessor {
    pub state: StreamState,
    pub buffer_size: usize,
    pub chunks: VecDeque<StreamChunk>,
    pub total_processed: u64,
    pub compression: Option<CompressionType>,
}

impl StreamProcessor {
    pub fn new(buffer_size: usize) -> Self {
        StreamProcessor {
            state: StreamState::Ready,
            buffer_size,
            chunks: VecDeque::new(),
            total_processed: 0,
            compression: None,
        }
    }
    
    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = Some(compression);
        self
    }
    
    pub fn add_chunk(&mut self, chunk: StreamChunk) {
        self.chunks.push_back(chunk);
    }
    
    pub fn next_chunk(&mut self) -> Option<StreamChunk> {
        self.chunks.pop_front()
    }
    
    pub fn is_finished(&self) -> bool {
        matches!(self.state, StreamState::Finished)
    }
}

/// Memory-mapped file wrapper
#[derive(Debug)]
pub struct MemoryMappedFile {
    pub mmap: memmap2::Mmap,
    pub size: u64,
    pub position: u64,
}

impl MemoryMappedFile {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        
        Ok(MemoryMappedFile {
            mmap,
            size: metadata.len(),
            position: 0,
        })
    }
    
    pub fn read_chunk(&mut self, size: usize) -> Option<&[u8]> {
        if self.position >= self.size {
            return None;
        }
        
        let start = self.position as usize;
        let end = std::cmp::min(start + size, self.size as usize);
        let chunk = &self.mmap[start..end];
        
        self.position = end as u64;
        Some(chunk)
    }
    
    pub fn seek(&mut self, position: u64) -> bool {
        if position <= self.size {
            self.position = position;
            true
        } else {
            false
        }
    }
}

/// Creates streaming I/O operation bindings.
pub fn create_streaming_io_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Stream creation and management
    bind_stream_operations(env);
    
    // Compression and decompression
    bind_compression_operations(env);
    
    // Memory-mapped file operations
    bind_mmap_operations(env);
    
    // Stream transformations
    bind_transformation_operations(env);
    
    // Pipeline operations
    bind_pipeline_operations(env);
}

// ============= STREAM OPERATIONS =============

fn bind_stream_operations(env: &Arc<ThreadSafeEnvironment>) {
    // create-stream
    env.define("create-stream".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "create-stream".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_create_stream),
        effects: vec![Effect::IO],
    })));
    
    // stream-read-chunk
    env.define("stream-read-chunk".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "stream-read-chunk".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_stream_read_chunk),
        effects: vec![Effect::IO],
    })));
    
    // stream-write-chunk
    env.define("stream-write-chunk".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "stream-write-chunk".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_stream_write_chunk),
        effects: vec![Effect::IO],
    })));
    
    // stream-finished?
    env.define("stream-finished?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "stream-finished?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_stream_finished_p),
        effects: vec![Effect::IO],
    })));
    
    // stream-close
    env.define("stream-close".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "stream-close".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_stream_close),
        effects: vec![Effect::IO],
    })));
}

fn bind_compression_operations(env: &Arc<ThreadSafeEnvironment>) {
    // compress-stream
    env.define("compress-stream".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "compress-stream".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_compress_stream),
        effects: vec![Effect::Pure],
    })));
    
    // decompress-stream
    env.define("decompress-stream".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "decompress-stream".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_decompress_stream),
        effects: vec![Effect::Pure],
    })));
    
    // gzip-compress
    env.define("gzip-compress".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "gzip-compress".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_gzip_compress),
        effects: vec![Effect::Pure],
    })));
    
    // gzip-decompress
    env.define("gzip-decompress".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "gzip-decompress".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_gzip_decompress),
        effects: vec![Effect::Pure],
    })));
    
    // zstd-compress
    env.define("zstd-compress".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "zstd-compress".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_zstd_compress),
        effects: vec![Effect::Pure],
    })));
    
    // zstd-decompress
    env.define("zstd-decompress".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "zstd-decompress".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_zstd_decompress),
        effects: vec![Effect::Pure],
    })));
    
    // lz4-compress
    env.define("lz4-compress".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "lz4-compress".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_lz4_compress),
        effects: vec![Effect::Pure],
    })));
    
    // lz4-decompress
    env.define("lz4-decompress".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "lz4-decompress".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_lz4_decompress),
        effects: vec![Effect::Pure],
    })));
}

fn bind_mmap_operations(env: &Arc<ThreadSafeEnvironment>) {
    // memory-map-file
    env.define("memory-map-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "memory-map-file".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_memory_map_file),
        effects: vec![Effect::IO],
    })));
    
    // mmap-read-chunk
    env.define("mmap-read-chunk".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "mmap-read-chunk".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_mmap_read_chunk),
        effects: vec![Effect::IO],
    })));
    
    // mmap-seek
    env.define("mmap-seek".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "mmap-seek".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_mmap_seek),
        effects: vec![Effect::IO],
    })));
    
    // mmap-size
    env.define("mmap-size".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "mmap-size".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_mmap_size),
        effects: vec![Effect::IO],
    })));
}

fn bind_transformation_operations(env: &Arc<ThreadSafeEnvironment>) {
    // stream-map
    env.define("stream-map".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "stream-map".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_stream_map),
        effects: vec![Effect::IO],
    })));
    
    // stream-filter
    env.define("stream-filter".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "stream-filter".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_stream_filter),
        effects: vec![Effect::IO],
    })));
    
    // stream-fold
    env.define("stream-fold".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "stream-fold".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_stream_fold),
        effects: vec![Effect::IO],
    })));
}

fn bind_pipeline_operations(env: &Arc<ThreadSafeEnvironment>) {
    // create-pipeline
    env.define("create-pipeline".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "create-pipeline".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_create_pipeline),
        effects: vec![Effect::IO],
    })));
    
    // pipeline-add-stage
    env.define("pipeline-add-stage".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "pipeline-add-stage".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_pipeline_add_stage),
        effects: vec![Effect::IO],
    })));
    
    // pipeline-execute
    env.define("pipeline-execute".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "pipeline-execute".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_pipeline_execute),
        effects: vec![Effect::IO],
    })));
}

// ============= IMPLEMENTATION FUNCTIONS =============

// === Stream Operations ===

pub fn primitive_create_stream(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("create-stream expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let source = extract_string(&args[0], "create-stream")?;
    let buffer_size = if args.len() > 1 {
        extract_integer(&args[1], "create-stream")? as usize
    } else {
        8192 // 8KB default buffer
    };
    
    let compression = if args.len() > 2 {
        let comp_str = extract_string(&args[2], "create-stream")?;
        match comp_str.as_str() {
            "gzip" => Some(CompressionType::Gzip),
            "zstd" => Some(CompressionType::Zstd),
            "lz4" => Some(CompressionType::Lz4),
            "none" => None,
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Unsupported compression type: {comp_str}"),
                    None,
                )));
            }
        }
    } else {
        None
    };
    
    let mut processor = StreamProcessor::new(buffer_size);
    if let Some(comp) = compression {
        processor = processor.with_compression(comp);
    }
    
    // For now, we'll create a simple stream processor
    // In a full implementation, this would handle various source types
    Ok(Value::opaque(Box::new(processor)))
}

pub fn primitive_stream_read_chunk(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("stream-read-chunk expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let max_size = if args.len() > 1 {
        Some(extract_integer(&args[1], "stream-read-chunk")? as usize)
    } else {
        None
    };
    
    // Extract stream processor from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(processor) = opaque_data.downcast_ref::<StreamProcessor>() {
                // This is a simplified implementation
                // In reality, we'd need mutable access to the processor
                Ok(Value::Nil) // Placeholder - would return next chunk
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "stream-read-chunk requires stream processor".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "stream-read-chunk requires stream processor".to_string(),
            None,
        ))),
    }
}

pub fn primitive_stream_write_chunk(_args: &[Value]) -> Result<Value> {
    // TODO: Implement stream chunk writing
    Err(Box::new(DiagnosticError::runtime_error(
        "stream-write-chunk not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_stream_finished_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("stream-finished? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // Extract stream processor from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(processor) = opaque_data.downcast_ref::<StreamProcessor>() {
                Ok(Value::boolean(processor.is_finished()))
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "stream-finished? requires stream processor".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "stream-finished? requires stream processor".to_string(),
            None,
        ))),
    }
}

pub fn primitive_stream_close(_args: &[Value]) -> Result<Value> {
    // TODO: Implement stream closing
    Ok(Value::Unspecified)
}

// === Compression Operations ===

pub fn primitive_compress_stream(_args: &[Value]) -> Result<Value> {
    // TODO: Implement stream compression
    Err(Box::new(DiagnosticError::runtime_error(
        "compress-stream not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_decompress_stream(_args: &[Value]) -> Result<Value> {
    // TODO: Implement stream decompression
    Err(Box::new(DiagnosticError::runtime_error(
        "decompress-stream not yet implemented".to_string(),
        None,
    )))
}

#[cfg(feature = "compression")]
pub fn primitive_gzip_compress(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("gzip-compress expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let data = extract_bytevector(&args[0], "gzip-compress")?;
    let level = if args.len() > 1 {
        extract_integer(&args[1], "gzip-compress")? as u32
    } else {
        6 // Default compression level
    };
    
    let compression = Compression::new(level);
    let mut encoder = GzEncoder::new(Vec::new(), compression);
    
    match encoder.write_all(&data) {
        Ok(()) => {
            match encoder.finish() {
                Ok(compressed) => Ok(Value::bytevector(compressed)),
                Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                    format!("Gzip compression failed: {}", e),
                    None,
                ))),
            }
        }
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Gzip compression failed: {}", e),
            None,
        ))),
    }
}

#[cfg(not(feature = "compression"))]
pub fn primitive_gzip_compress(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "gzip-compress requires compression feature".to_string(),
        None,
    )))
}

#[cfg(feature = "compression")]
pub fn primitive_gzip_decompress(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("gzip-decompress expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let compressed = extract_bytevector(&args[0], "gzip-decompress")?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(Value::bytevector(decompressed)),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Gzip decompression failed: {}", e),
            None,
        ))),
    }
}

#[cfg(not(feature = "compression"))]
pub fn primitive_gzip_decompress(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "gzip-decompress requires compression feature".to_string(),
        None,
    )))
}

#[cfg(feature = "compression")]
pub fn primitive_zstd_compress(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("zstd-compress expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let data = extract_bytevector(&args[0], "zstd-compress")?;
    let level = if args.len() > 1 {
        extract_integer(&args[1], "zstd-compress")? as i32
    } else {
        3 // Default compression level
    };
    
    match zstd::encode_all(&data[..], level) {
        Ok(compressed) => Ok(Value::bytevector(compressed)),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Zstd compression failed: {}", e),
            None,
        ))),
    }
}

#[cfg(not(feature = "compression"))]
pub fn primitive_zstd_compress(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "zstd-compress requires compression feature".to_string(),
        None,
    )))
}

#[cfg(feature = "compression")]
pub fn primitive_zstd_decompress(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("zstd-decompress expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let compressed = extract_bytevector(&args[0], "zstd-decompress")?;
    
    match zstd::decode_all(&compressed[..]) {
        Ok(decompressed) => Ok(Value::bytevector(decompressed)),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Zstd decompression failed: {}", e),
            None,
        ))),
    }
}

#[cfg(not(feature = "compression"))]
pub fn primitive_zstd_decompress(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "zstd-decompress requires compression feature".to_string(),
        None,
    )))
}

#[cfg(feature = "compression")]
pub fn primitive_lz4_compress(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("lz4-compress expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let data = extract_bytevector(&args[0], "lz4-compress")?;
    let compressed = compress(&data);
    Ok(Value::bytevector(compressed))
}

#[cfg(not(feature = "compression"))]
pub fn primitive_lz4_compress(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "lz4-compress requires compression feature".to_string(),
        None,
    )))
}

#[cfg(feature = "compression")]
pub fn primitive_lz4_decompress(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("lz4-decompress expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let compressed = extract_bytevector(&args[0], "lz4-decompress")?;
    let expected_size = extract_integer(&args[1], "lz4-decompress")? as usize;
    
    match decompress(&compressed, expected_size) {
        Ok(decompressed) => Ok(Value::bytevector(decompressed)),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("LZ4 decompression failed: {}", e),
            None,
        ))),
    }
}

#[cfg(not(feature = "compression"))]
pub fn primitive_lz4_decompress(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "lz4-decompress requires compression feature".to_string(),
        None,
    )))
}

// === Memory-mapped File Operations ===

pub fn primitive_memory_map_file(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("memory-map-file expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "memory-map-file")?;
    let read_only = if args.len() > 1 {
        extract_boolean(&args[1], "memory-map-file")?
    } else {
        true
    };
    
    if !read_only {
        return Err(Box::new(DiagnosticError::runtime_error(
            "Writable memory mapping not yet implemented".to_string(),
            None,
        )));
    }
    
    match MemoryMappedFile::new(&path) {
        Ok(mmap_file) => Ok(Value::opaque(Box::new(mmap_file))),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot memory-map file '{path}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_mmap_read_chunk(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("mmap-read-chunk expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let chunk_size = extract_integer(&args[1], "mmap-read-chunk")? as usize;
    
    // Extract memory-mapped file from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(mmap_file) = opaque_data.downcast_ref::<MemoryMappedFile>() {
                // This is a simplified implementation - we'd need mutable access
                // In reality, we'd need a different approach for mutability
                Ok(Value::Nil) // Placeholder
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "mmap-read-chunk requires memory-mapped file".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "mmap-read-chunk requires memory-mapped file".to_string(),
            None,
        ))),
    }
}

pub fn primitive_mmap_seek(_args: &[Value]) -> Result<Value> {
    // TODO: Implement memory-mapped file seeking
    Err(Box::new(DiagnosticError::runtime_error(
        "mmap-seek not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_mmap_size(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("mmap-size expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // Extract memory-mapped file from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(mmap_file) = opaque_data.downcast_ref::<MemoryMappedFile>() {
                Ok(Value::integer(mmap_file.size as i64))
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "mmap-size requires memory-mapped file".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "mmap-size requires memory-mapped file".to_string(),
            None,
        ))),
    }
}

// === Transformation Operations ===

pub fn primitive_stream_map(_args: &[Value]) -> Result<Value> {
    // TODO: Implement stream mapping
    Err(Box::new(DiagnosticError::runtime_error(
        "stream-map not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_stream_filter(_args: &[Value]) -> Result<Value> {
    // TODO: Implement stream filtering
    Err(Box::new(DiagnosticError::runtime_error(
        "stream-filter not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_stream_fold(_args: &[Value]) -> Result<Value> {
    // TODO: Implement stream folding
    Err(Box::new(DiagnosticError::runtime_error(
        "stream-fold not yet implemented".to_string(),
        None,
    )))
}

// === Pipeline Operations ===

pub fn primitive_create_pipeline(_args: &[Value]) -> Result<Value> {
    // TODO: Implement pipeline creation
    Err(Box::new(DiagnosticError::runtime_error(
        "create-pipeline not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_pipeline_add_stage(_args: &[Value]) -> Result<Value> {
    // TODO: Implement pipeline stage addition
    Err(Box::new(DiagnosticError::runtime_error(
        "pipeline-add-stage not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_pipeline_execute(_args: &[Value]) -> Result<Value> {
    // TODO: Implement pipeline execution
    Err(Box::new(DiagnosticError::runtime_error(
        "pipeline-execute not yet implemented".to_string(),
        None,
    )))
}

// ============= HELPER FUNCTIONS =============

/// Extracts a string from a Value.
fn extract_string(value: &Value, operation: &str) -> Result<String> {
    match value {
        Value::Literal(crate::ast::Literal::String(s)) => Ok(s.clone()),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires string arguments"),
            None,
        ))),
    }
}

/// Extracts a boolean from a Value.
fn extract_boolean(value: &Value, operation: &str) -> Result<bool> {
    match value {
        Value::Literal(crate::ast::Literal::Boolean(b)) => Ok(*b),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires boolean arguments"),
            None,
        ))),
    }
}

/// Extracts an integer from a Value.
fn extract_integer(value: &Value, operation: &str) -> Result<i64> {
    match value {
        Value::Literal(lit) => {
            if let Some(i) = lit.to_i64() {
                Ok(i)
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    format!("{operation} requires integer arguments"),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires integer arguments"),
            None,
        ))),
    }
}

/// Extracts a bytevector from a Value.
fn extract_bytevector(value: &Value, operation: &str) -> Result<Vec<u8>> {
    match value {
        Value::Literal(crate::ast::Literal::Bytevector(bv)) => Ok(bv.clone()),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires bytevector arguments"),
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_stream_creation() {
        let args = vec![
            Value::string("test-source".to_string()),
            Value::integer(4096),
            Value::string("none".to_string()),
        ];
        
        let result = primitive_create_stream(&args);
        assert!(result.is_ok());
    }
    
    #[cfg(feature = "compression")]
    #[test]
    fn test_gzip_compression() {
        let test_data = b"Hello, world! This is a test string for compression.";
        let args = vec![Value::bytevector(test_data.to_vec())];
        
        // Test compression
        let compressed_result = primitive_gzip_compress(&args);
        assert!(compressed_result.is_ok());
        
        if let Ok(Value::Literal(crate::ast::Literal::Bytevector(compressed))) = compressed_result {
            // Test decompression
            let decompress_args = vec![Value::bytevector(compressed)];
            let decompressed_result = primitive_gzip_decompress(&decompress_args);
            assert!(decompressed_result.is_ok());
            
            if let Ok(Value::Literal(crate::ast::Literal::Bytevector(decompressed))) = decompressed_result {
                assert_eq!(decompressed, test_data.to_vec());
            } else {
                panic!("Expected bytevector result from decompression");
            }
        } else {
            panic!("Expected bytevector result from compression");
        }
    }
    
    #[test]
    fn test_memory_mapping() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"This is test data for memory mapping.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        let path = temp_file.path().to_string_lossy().to_string();
        let args = vec![Value::string(path)];
        
        let result = primitive_memory_map_file(&args);
        assert!(result.is_ok());
        
        // Test getting size
        if let Ok(mmap_file) = result {
            let size_args = vec![mmap_file];
            let size_result = primitive_mmap_size(&size_args);
            assert!(size_result.is_ok());
            
            if let Ok(Value::Literal(crate::ast::Literal::Number(size))) = size_result {
                assert_eq!(size, test_data.len() as f64);
            } else {
                panic!("Expected integer size result");
            }
        }
    }
}