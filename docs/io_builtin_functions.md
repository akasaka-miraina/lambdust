# I/O Module Builtin Function Specifications

This document specifies the Rust builtin functions required to support the R7RS-compliant I/O module (`stdlib/modules/io.scm`). These functions provide the low-level implementation for Lambdust's port-based I/O system.

## Port Predicates

### `builtin:input-port?`
- **Signature**: `(input-port? obj) -> boolean`
- **Description**: Returns `#t` if `obj` is an input port, `#f` otherwise
- **Rust Implementation**: Check if `Value` is `Value::Port(_)` with input capability

### `builtin:output-port?`
- **Signature**: `(output-port? obj) -> boolean`
- **Description**: Returns `#t` if `obj` is an output port, `#f` otherwise
- **Rust Implementation**: Check if `Value` is `Value::Port(_)` with output capability

### `builtin:textual-port?`
- **Signature**: `(textual-port? obj) -> boolean`
- **Description**: Returns `#t` if `obj` is a textual port, `#f` otherwise
- **Rust Implementation**: Check if port supports text operations (UTF-8 encoding/decoding)

### `builtin:binary-port?`
- **Signature**: `(binary-port? obj) -> boolean`
- **Description**: Returns `#t` if `obj` is a binary port, `#f` otherwise
- **Rust Implementation**: Check if port supports raw byte operations

### `builtin:port-open?`
- **Signature**: `(port-open? port) -> boolean`
- **Description**: Returns `#t` if port is still open and capable of I/O
- **Rust Implementation**: Check port's internal state flag

## Current Ports

### `builtin:current-input-port`
- **Signature**: `(current-input-port) -> input-port`
- **Description**: Returns the current default input port (typically stdin)
- **Rust Implementation**: Return global/thread-local current input port

### `builtin:current-output-port`
- **Signature**: `(current-output-port) -> output-port`
- **Description**: Returns the current default output port (typically stdout)
- **Rust Implementation**: Return global/thread-local current output port

### `builtin:current-error-port`
- **Signature**: `(current-error-port) -> output-port`
- **Description**: Returns the current default error port (typically stderr)
- **Rust Implementation**: Return global/thread-local current error port

### `builtin:set-current-input-port!`
- **Signature**: `(set-current-input-port! port) -> unspecified`
- **Description**: Sets the current default input port
- **Rust Implementation**: Update global/thread-local current input port
- **Note**: Not exported - used internally by `with-input-from-file`

### `builtin:set-current-output-port!`
- **Signature**: `(set-current-output-port! port) -> unspecified`
- **Description**: Sets the current default output port
- **Rust Implementation**: Update global/thread-local current output port
- **Note**: Not exported - used internally by `with-output-to-file`

## File Operations

### `builtin:open-input-file`
- **Signature**: `(open-input-file filename) -> textual-input-port`
- **Description**: Opens the named file for textual input
- **Rust Implementation**: Open file with `std::fs::File::open()`, wrap in textual input port
- **Error Handling**: Signal file-error if file doesn't exist or can't be opened

### `builtin:open-output-file`
- **Signature**: `(open-output-file filename) -> textual-output-port`
- **Description**: Opens the named file for textual output
- **Rust Implementation**: Create file with `std::fs::File::create()`, wrap in textual output port
- **Error Handling**: Signal file-error if file can't be created

### `builtin:open-binary-input-file`
- **Signature**: `(open-binary-input-file filename) -> binary-input-port`
- **Description**: Opens the named file for binary input
- **Rust Implementation**: Open file in binary mode, wrap in binary input port

### `builtin:open-binary-output-file`
- **Signature**: `(open-binary-output-file filename) -> binary-output-port`
- **Description**: Opens the named file for binary output
- **Rust Implementation**: Create file in binary mode, wrap in binary output port

### `builtin:close-port`
- **Signature**: `(close-port port) -> unspecified`
- **Description**: Closes the port and frees associated resources
- **Rust Implementation**: Close underlying file/stream, mark port as closed

### `builtin:close-input-port`
- **Signature**: `(close-input-port port) -> unspecified`
- **Description**: Closes the input port
- **Rust Implementation**: Same as `close-port` but verify it's an input port

### `builtin:close-output-port`
- **Signature**: `(close-output-port port) -> unspecified`
- **Description**: Closes the output port
- **Rust Implementation**: Same as `close-port` but verify it's an output port

## String Ports

### `builtin:open-input-string`
- **Signature**: `(open-input-string string) -> textual-input-port`
- **Description**: Creates a textual input port that reads from the string
- **Rust Implementation**: Wrap string in a cursor-based reader port

### `builtin:open-output-string`
- **Signature**: `(open-output-string) -> textual-output-port`
- **Description**: Creates a textual output port that accumulates to a string
- **Rust Implementation**: Use internal buffer (e.g., `Vec<u8>` or `String`)

### `builtin:get-output-string`
- **Signature**: `(get-output-string port) -> string`
- **Description**: Returns accumulated string from output string port
- **Rust Implementation**: Convert internal buffer to String, reset buffer

### `builtin:open-input-bytevector`
- **Signature**: `(open-input-bytevector bytevector) -> binary-input-port`
- **Description**: Creates a binary input port that reads from the bytevector
- **Rust Implementation**: Wrap bytevector in cursor-based reader

### `builtin:open-output-bytevector`
- **Signature**: `(open-output-bytevector) -> binary-output-port`
- **Description**: Creates a binary output port that accumulates to a bytevector
- **Rust Implementation**: Use internal `Vec<u8>` buffer

### `builtin:get-output-bytevector`
- **Signature**: `(get-output-bytevector port) -> bytevector`
- **Description**: Returns accumulated bytevector from output port
- **Rust Implementation**: Return `Vec<u8>` as bytevector, reset buffer

## Input Operations

### `builtin:read`
- **Signature**: `(read port) -> object`
- **Description**: Reads and parses a Scheme object from the textual input port
- **Rust Implementation**: Use existing parser to read S-expression from port
- **Error Handling**: Return EOF object if at end of input

### `builtin:read-char`
- **Signature**: `(read-char port) -> character | eof-object`
- **Description**: Reads next character from textual input port
- **Rust Implementation**: Read UTF-8 character, advance position
- **Error Handling**: Return EOF object if no more characters

### `builtin:peek-char`
- **Signature**: `(peek-char port) -> character | eof-object`
- **Description**: Returns next character without advancing position
- **Rust Implementation**: Read character but don't advance position
- **Error Handling**: Return EOF object if no more characters

### `builtin:read-line`
- **Signature**: `(read-line port) -> string | eof-object`
- **Description**: Reads line of text up to newline
- **Rust Implementation**: Read until '\n' or EOF, return as string
- **Error Handling**: Return EOF object if no characters before EOF

### `builtin:read-string`
- **Signature**: `(read-string k port) -> string | eof-object`
- **Description**: Reads at most k characters into a string
- **Rust Implementation**: Read up to k characters, return as string
- **Error Handling**: Return EOF object if no characters available

### `builtin:read-u8`
- **Signature**: `(read-u8 port) -> byte | eof-object`
- **Description**: Reads next byte from binary input port
- **Rust Implementation**: Read single byte (0-255)
- **Error Handling**: Return EOF object if no more bytes

### `builtin:peek-u8`
- **Signature**: `(peek-u8 port) -> byte | eof-object`
- **Description**: Returns next byte without advancing position
- **Rust Implementation**: Peek at next byte without consuming
- **Error Handling**: Return EOF object if no more bytes

### `builtin:char-ready?`
- **Signature**: `(char-ready? port) -> boolean`
- **Description**: Returns `#t` if character is ready for reading
- **Rust Implementation**: Check if data is available without blocking
- **Note**: For file/string ports, typically always `#t`

### `builtin:u8-ready?`
- **Signature**: `(u8-ready? port) -> boolean`
- **Description**: Returns `#t` if byte is ready for reading
- **Rust Implementation**: Check if binary data is available without blocking

### `builtin:eof-object`
- **Signature**: `(eof-object) -> eof-object`
- **Description**: Returns the end-of-file object
- **Rust Implementation**: Return special EOF value constant

### `builtin:eof-object?`
- **Signature**: `(eof-object? obj) -> boolean`
- **Description**: Returns `#t` if obj is an end-of-file object
- **Rust Implementation**: Compare with EOF object singleton

## Output Operations

### `builtin:write`
- **Signature**: `(write obj port) -> unspecified`
- **Description**: Writes object's external representation to textual output port
- **Rust Implementation**: Use existing printer with proper escaping

### `builtin:write-char`
- **Signature**: `(write-char char port) -> unspecified`
- **Description**: Writes character to textual output port
- **Rust Implementation**: Write UTF-8 encoded character

### `builtin:write-string`
- **Signature**: `(write-string string port start end) -> unspecified`
- **Description**: Writes substring to textual output port
- **Rust Implementation**: Write UTF-8 encoded string slice

### `builtin:write-u8`
- **Signature**: `(write-u8 byte port) -> unspecified`
- **Description**: Writes byte to binary output port
- **Rust Implementation**: Write single byte value

### `builtin:newline`
- **Signature**: `(newline port) -> unspecified`
- **Description**: Writes platform-appropriate newline to textual output port
- **Rust Implementation**: Write '\n' (or platform-specific line ending)

### `builtin:display`
- **Signature**: `(display obj port) -> unspecified`
- **Description**: Writes human-readable representation to textual output port
- **Rust Implementation**: Like write but without string quotes/escaping

### `builtin:flush-output-port`
- **Signature**: `(flush-output-port port) -> unspecified`
- **Description**: Forces pending output to be delivered
- **Rust Implementation**: Call `flush()` on underlying writer

## Binary I/O Operations

### `builtin:read-bytevector`
- **Signature**: `(read-bytevector k port) -> bytevector | eof-object`
- **Description**: Reads at most k bytes into new bytevector
- **Rust Implementation**: Read up to k bytes into `Vec<u8>`
- **Error Handling**: Return EOF object if no bytes available

### `builtin:read-bytevector!`
- **Signature**: `(read-bytevector! bytevector port start end) -> exact-integer | eof-object`
- **Description**: Reads bytes into existing bytevector
- **Rust Implementation**: Read into bytevector slice, return count
- **Error Handling**: Return EOF object if no bytes available

### `builtin:write-bytevector`
- **Signature**: `(write-bytevector bytevector port start end) -> unspecified`
- **Description**: Writes bytevector slice to binary output port
- **Rust Implementation**: Write bytevector slice to port

## Format Operations (Extensions)

### `builtin:sprintf`
- **Signature**: `(sprintf format-string args) -> string`
- **Description**: Returns formatted string using printf-style format specifiers
- **Rust Implementation**: Printf-style formatting with Scheme object conversion
- **Format Specifiers**:
  - `~a` - Any object (display format)
  - `~s` - S-expression (write format)  
  - `~d` - Decimal integer
  - `~x` - Hexadecimal
  - `~o` - Octal
  - `~b` - Binary
  - `~f` - Floating point
  - `~%` - Newline

## File System Operations (Extensions)

### `builtin:file-exists?`
- **Signature**: `(file-exists? filename) -> boolean`
- **Description**: Returns `#t` if named file exists
- **Rust Implementation**: Use `std::path::Path::exists()`

### `builtin:delete-file`
- **Signature**: `(delete-file filename) -> unspecified`
- **Description**: Deletes the named file
- **Rust Implementation**: Use `std::fs::remove_file()`
- **Error Handling**: Signal error if file doesn't exist or can't be deleted

### `builtin:rename-file`
- **Signature**: `(rename-file old-filename new-filename) -> unspecified`
- **Description**: Renames/moves file from old to new name
- **Rust Implementation**: Use `std::fs::rename()`
- **Error Handling**: Signal error if operation fails

## Port Positioning (Extensions)

### `builtin:port-position`
- **Signature**: `(port-position port) -> exact-integer`
- **Description**: Returns current position in port
- **Rust Implementation**: Use `Seek::stream_position()` if supported
- **Error Handling**: Signal error if port doesn't support positioning

### `builtin:set-port-position!`
- **Signature**: `(set-port-position! port position) -> unspecified`
- **Description**: Sets position in port
- **Rust Implementation**: Use `Seek::seek()` if supported
- **Error Handling**: Signal error if port doesn't support seeking

### `builtin:port-has-port-position?`
- **Signature**: `(port-has-port-position? port) -> boolean`
- **Description**: Returns `#t` if port supports position queries
- **Rust Implementation**: Check if underlying resource implements `Seek`

### `builtin:port-has-set-port-position!?`
- **Signature**: `(port-has-set-port-position!? port) -> boolean`
- **Description**: Returns `#t` if port supports position setting
- **Rust Implementation**: Check if underlying resource implements `Seek` (writeable)

## Port Types Design

The Rust implementation should use an enum-based port system:

```rust
pub enum Port {
    Input(InputPort),
    Output(OutputPort),
    InputOutput(InputOutputPort), // For bidirectional ports
}

pub enum InputPort {
    File(BufReader<File>),
    String(Cursor<String>),
    Stdin(io::Stdin),
    Bytevector(Cursor<Vec<u8>>),
}

pub enum OutputPort {
    File(BufWriter<File>),
    String(Vec<u8>), // Internal buffer
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    Bytevector(Vec<u8>), // Internal buffer
}
```

## Error Handling

All I/O operations should follow R7RS error handling conventions:
- Use Lambdust's error system with appropriate error types
- File operations should signal `file-error?` conditions
- Port operations on closed ports should signal errors
- Type mismatches should signal type errors

## Thread Safety

Port operations should be thread-safe when accessed concurrently:
- Use appropriate locking mechanisms for shared resources
- Ensure atomic operations for port state changes
- Consider using `Arc<Mutex<Port>>` for shared ports

## Performance Considerations

- Use buffered I/O for file operations (`BufReader`/`BufWriter`)
- Implement efficient string/bytevector port operations
- Cache UTF-8 decoding state for textual ports
- Optimize common operations like character/byte reading

This specification provides the foundation for implementing a complete, R7RS-compliant I/O system in Lambdust.