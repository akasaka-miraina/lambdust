# Lambdust C/C++ Integration Examples / C/C++統合サンプル

This directory contains comprehensive examples demonstrating how to integrate Lambdust Scheme interpreter with C and C++ applications.

**日本語**: このディレクトリには、Lambdust Schemeインタープリターをを C/C++アプリケーションと統合する方法を示す包括的なサンプルが含まれています。

## Overview / 概要

Lambdust provides a complete C FFI (Foreign Function Interface) that allows:
- Embedding the Scheme interpreter in C/C++ applications / C/C++アプリケーションへのSchemeインタープリター埋め込み
- Registering C/C++ functions callable from Scheme / SchemeからC/C++関数の呼び出し
- Type-safe data exchange between C/C++ and Scheme / C/C++とScheme間の型安全データ交換
- Advanced template-based integration for modern C++ / モダンC++のためのテンプレートベース統合
- Production-ready error handling and memory management / プロダクション品質のエラーハンドリングとメモリ管理

## Prerequisites / 前提条件

- CMake 3.15 or later / CMake 3.15以降
- C compiler with C11 support / C11対応Cコンパイラ
- C++ compiler with C++14 support (C++17/20 for advanced examples) / C++14対応コンパイラ（高度なサンプルにはC++17/20）
- Rust toolchain (for building the core library) / Rustツールチェーン（コアライブラリのビルドに必要）

## Building the Examples / サンプルのビルド

### Using CMake / CMakeを使用

```bash
# Configure and build / 設定とビルド
mkdir build && cd build
cmake .. -DBUILD_EXAMPLES=ON
make

# Run tests / テスト実行
ctest
```

### Manual Build (if CMake is not available) / 手動ビルド（CMakeが利用できない場合）

```bash
# Build the Rust library first / 最初にRustライブラリをビルド
cargo build --release

# Compile C examples / Cサンプルのコンパイル
gcc -std=c11 -I../include examples/c_integration/basic_usage.c \
    -L../target/release -llambdust -ldl -lm -o basic_usage

# Compile C++ examples / C++サンプルのコンパイル
g++ -std=c++17 -I../include examples/cpp_integration/cpp_wrapper.cpp \
    -L../target/release -llambdust -ldl -lm -o cpp_wrapper
```

### Using Cargo Features / Cargoフィーチャーを使用

```bash
# Build with specific features / 特定機能でビルド
cargo build --release --features "standard srfi-support"

# Minimal embedded build for smaller binaries / 小さなバイナリ用の最小埋め込みビルド
cargo build --release --features "embedded"
```

## C Integration Examples

### 1. Basic Usage (`c_integration/basic_usage.c`)

Demonstrates fundamental Lambdust operations:
- Context creation and destruction
- Basic expression evaluation
- Variable and function definitions
- Error handling

```c
#include "lambdust.h"

int main() {
    LambdustContext* ctx = lambdust_create_context();
    
    char* result = NULL;
    lambdust_eval(ctx, "(+ 1 2 3)", &result);
    printf("Result: %s\n", result);
    
    lambdust_free_string(result);
    lambdust_destroy_context(ctx);
    return 0;
}
```

### 2. Host Functions (`c_integration/host_functions.c`)

Shows how to register C functions callable from Scheme:
- Function registration with proper signatures
- Argument validation and conversion
- Memory management for C strings
- Stateful host functions

### 3. Calculator (`c_integration/calculator.c`)

A practical calculator application using Scheme as the expression engine:
- Interactive and command-line modes
- Mathematical functions (sin, cos, log, etc.)
- Memory operations
- Error handling and user feedback

### 4. Plugin System (`c_integration/plugin_system.c`)

Demonstrates building a plugin architecture with Scheme:
- Dynamic plugin loading from `.scm` files
- Plugin information and metadata
- Inter-plugin communication
- Plugin lifecycle management

### 5. Configuration Management (`c_integration/config_example.c`)

Shows using Scheme for flexible application configuration:
- Programmable configuration files
- Environment variable integration
- Conditional configuration
- Runtime configuration updates

## C++ Integration Examples

### 1. C++ Wrapper (`cpp_integration/cpp_wrapper.cpp`)

Modern C++ wrapper providing:
- RAII resource management
- Exception safety
- Type-safe function registration
- Move semantics and smart pointers

```cpp
#include "lambdust.h"

lambdust::Interpreter interp;
std::string result = interp.eval("(+ 1 2 3)");

interp.register_function<int, int>("add", 
    [](int a, int b) { return std::to_string(a + b); });
```

### 2. Modern Features (`cpp_integration/cpp_modern_features.cpp`)

Advanced C++ features integration:
- Concepts and constraints (C++20)
- Ranges and algorithms (C++20)
- Coroutines (C++20)
- Smart pointers and RAII
- Async evaluation

### 3. Template Integration (`cpp_integration/cpp_template_integration.cpp`)

Template-based type-safe integration:
- Compile-time type checking
- Template metaprogramming
- Type-safe function wrappers
- Compile-time DSL generation
- Automatic arity validation

## Using pkg-config

Once installed, you can use pkg-config to get compilation flags:

```bash
# Get compile flags
pkg-config --cflags lambdust

# Get link flags  
pkg-config --libs lambdust

# Example compilation
gcc $(pkg-config --cflags lambdust) myapp.c $(pkg-config --libs lambdust) -o myapp
```

## API Reference

### Core Functions

- `lambdust_create_context()` - Create interpreter context
- `lambdust_destroy_context()` - Destroy context and free memory
- `lambdust_eval()` - Evaluate Scheme expression
- `lambdust_register_function()` - Register host function
- `lambdust_call_function()` - Call Scheme function from C
- `lambdust_get_last_error()` - Get error message
- `lambdust_free_string()` - Free Lambdust-allocated strings

### Error Codes

- `LAMBDUST_SUCCESS` - Operation successful
- `LAMBDUST_ERROR` - General error
- `LAMBDUST_INVALID_ARGUMENT` - Invalid argument
- `LAMBDUST_NULL_POINTER` - Null pointer error
- `LAMBDUST_MEMORY_ERROR` - Memory allocation failure
- `LAMBDUST_EVALUATION_ERROR` - Scheme evaluation error
- `LAMBDUST_TYPE_ERROR` - Type error
- `LAMBDUST_ARITY_ERROR` - Wrong number of arguments
- `LAMBDUST_RUNTIME_ERROR` - Runtime error

### Host Function Signature

```c
typedef int (*LambdustHostFunction)(
    int argc,                    // Number of arguments
    const char* const* argv,     // Argument strings
    char** result               // Output result string
);
```

## Best Practices

### Memory Management

- Always free strings returned by Lambdust using `lambdust_free_string()`
- Check for null pointers before using context or results
- Properly handle error conditions

### Error Handling

```c
char* result = NULL;
int error = lambdust_eval(ctx, expression, &result);
if (error != LAMBDUST_SUCCESS) {
    const char* error_msg = lambdust_get_last_error(ctx);
    // Handle error
} else {
    // Use result
    lambdust_free_string(result);
}
```

### Host Functions

- Validate argument count (argc)
- Check for null arguments
- Return appropriate error codes
- Allocate result strings with malloc()
- Use thread-safe functions when needed

### C++ Integration

- Use RAII for automatic resource management
- Leverage templates for type safety
- Handle exceptions appropriately
- Use smart pointers for memory management

## Performance Considerations

- Reuse contexts when possible
- Minimize string allocations
- Cache frequently called functions
- Use batch operations for multiple evaluations

## Thread Safety

The C API is not thread-safe by default. For multi-threaded applications:
- Use separate contexts per thread
- Implement external synchronization if sharing contexts
- Be careful with static variables in host functions

## Troubleshooting

### Common Issues

1. **Linking errors**: Ensure proper library paths and dependencies
2. **Memory leaks**: Always free strings with `lambdust_free_string()`
3. **Segmentation faults**: Check for null pointers and proper initialization
4. **Evaluation errors**: Check Scheme syntax and function availability

### Debug Tips

- Use `lambdust_check_library()` to verify installation
- Check `lambdust_get_last_error()` for detailed error messages
- Enable debug logging in your application
- Use memory debugging tools (valgrind, AddressSanitizer)

## Examples Output

When you run the examples, you should see output similar to:

```
=== Lambdust Basic Usage Example ===

Library version: Lambdust 0.1.1
Context created successfully

=== Basic Arithmetic ===
Evaluating: (+ 1 2 3)
Result: 6
Evaluating: (* 6 7)
Result: 42

=== List Operations ===
Evaluating: (list 1 2 3 4 5)
Result: (1 2 3 4 5)
```

## Further Reading

- [Lambdust Documentation](../docs/)
- [R7RS Scheme Specification](https://small.r7rs.org/)
- [C FFI Best Practices](../docs/FFI_GUIDE.md)
- [API Reference](../docs/API_REFERENCE.md)