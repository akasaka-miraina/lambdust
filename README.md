# Lambdust (λust) - Advanced R7RS Scheme Interpreter

[![Crates.io](https://img.shields.io/crates/v/lambdust)](https://crates.io/crates/lambdust)
[![Documentation](https://docs.rs/lambdust/badge.svg)](https://docs.rs/lambdust)
[![CI](https://github.com/akasaka-miraina/lambdust/workflows/Continuous%20Integration/badge.svg)](https://github.com/akasaka-miraina/lambdust/actions)
[![Coverage](https://codecov.io/gh/akasaka-miraina/lambdust/branch/main/graph/badge.svg)](https://codecov.io/gh/akasaka-miraina/lambdust)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)

**Lambdust** is a production-ready R7RS Scheme interpreter implemented in Rust, designed for embedding in applications as a macro and scripting system. The name combines "lambda" (λ) with "Rust," reflecting Scheme's functional nature and the ability to add expressive power to existing applications.

**日本語**: LambdustはRustで実装されたR7RS Scheme インタープリターです。アプリケーションへの組み込みとマクロ・スクリプティングシステムとして設計されています。

## Features / 機能

- **R7RS Compliance**: Complete R7RS Small specification implementation with 546+ tests
- **SRFI Support**: Extensive SRFI library support (SRFI 1, 13, 69, 111, 113, 125, 132-141)
- **Advanced Optimizations**: JIT loop optimization, tail-call optimization, continuation pooling
- **Embedded Design**: Designed for seamless integration into Rust applications
- **Formal Verification**: Agda-based formal verification system for correctness guarantees
- **Memory Safety**: RAII-based memory management with zero-cost abstractions
- **Bridge API**: Type-safe interoperability between Rust and Scheme
- **Production Ready**: Comprehensive error handling, stack overflow protection, robust evaluator

**日本語機能**: R7RS完全準拠・高度SRFI対応・JIT最適化・形式的検証・メモリ安全性・プロダクション品質

## Quick Start / クイックスタート

Add Lambdust to your `Cargo.toml`:

```toml
[dependencies]
lambdust = "0.3.0"
```

### Basic Usage / 基本的な使用方法

```rust
use lambdust::Interpreter;

fn main() {
    let mut interpreter = Interpreter::new();
    
    // Basic arithmetic / 基本的な算術演算
    let result = interpreter.eval("(+ 1 2 3)").unwrap();
    println!("Result: {}", result); // Prints: Result: 6
    
    // Advanced R7RS features / 高度なR7RS機能
    let fibonacci = interpreter.eval(r#"
        (define (fib n)
          (if (<= n 1)
              n
              (+ (fib (- n 1)) (fib (- n 2)))))
        (fib 10)
    "#).unwrap();
    println!("Fibonacci(10): {}", fibonacci); // Prints: Fibonacci(10): 55
    
    // SRFI support / SRFI対応
    interpreter.eval("(import (srfi 1))").unwrap();
    let result = interpreter.eval("(fold + 0 '(1 2 3 4 5))").unwrap();
    println!("Sum: {}", result); // Prints: Sum: 15
}
```

### Advanced Integration with Bridge API / Bridge APIを使った高度統合

```rust
use lambdust::{LambdustBridge, FromScheme, ToScheme, Value};

fn main() {
    let mut bridge = LambdustBridge::new();

    // Register external function / 外部関数の登録
    bridge.register_function("square", Some(1), |args| {
        let n = f64::from_scheme(&args[0])?;
        (n * n).to_scheme()
    });

    // Define variables / 変数の定義
    bridge.define("pi", Value::from(3.14159));

    // Execute Scheme code / Schemeコードの実行
    let result = bridge.eval("(* pi 2)").unwrap();
    println!("2π = {}", result);
    
    // Advanced features with SRFI support / SRFI対応高度機能
    bridge.eval("(import (srfi 69))").unwrap(); // Hash tables / ハッシュテーブル
    bridge.eval("(import (srfi 135))").unwrap(); // Immutable texts / 不変テキスト
    
    let hash_result = bridge.eval(r#"
        (let ((ht (make-hash-table)))
          (hash-table-set! ht "key" 42)
          (hash-table-ref ht "key"))
    "#).unwrap();
    println!("Hash table result: {}", hash_result);
}
```

## Architecture / アーキテクチャ

The interpreter consists of several key components:

- **Lexer**: Tokenizes Scheme source code / Schemeソースコードのトークン化
- **Parser**: Builds Abstract Syntax Trees (AST) from tokens / トークンからASTを構築
- **CPS Evaluator**: R7RS-compliant continuation-passing style evaluator / R7RS準拠継続渡しスタイル評価器
- **Environment**: Manages variable bindings and lexical scoping / 変数バインディングとレキシカルスコープ管理
- **Optimization Engine**: JIT loop optimization, tail-call optimization / JITループ最適化、末尾呼び出し最適化
- **SRFI Modules**: Extensive SRFI library implementations / 幅幅いSRFIライブラリ実装
- **Bridge API**: Type-safe interoperability with external Rust code / 外部Rustコードとの型安全な相互運用性
- **Memory Management**: RAII-based memory management with continuation pooling / RAIIベースメモリ管理と継続プーリング
- **Formal Verification**: Agda-based theorem proving for correctness / Agdaベースの定理証明で正しさを保証

## Supported Scheme Features / 対応するScheme機能

### R7RS Small Complete Implementation / R7RS Small完全実装
- All basic data types (numbers, strings, symbols, lists, vectors, etc.) / 全基本データ型
- Special forms (`define`, `lambda`, `if`, `cond`, `let`, `letrec`, `begin`, etc.) / 特殊形式
- First-class procedures and closures / 第一級手続きとクロージャ
- Continuation system (`call/cc`, `dynamic-wind`) / 継続システム
- Exception handling (`raise`, `guard`, `with-exception-handler`) / 例外処理
- Macro system with `syntax-rules` / マクロシステム
- Proper lexical scoping and environments / 適切なレキシカルスコープ
- Multiple values (`values`, `call-with-values`) / 多値システム

### SRFI Support / SRFI対応
- **SRFI 1**: List Library (enhanced list operations) / リストライブラリ
- **SRFI 13**: String Libraries (comprehensive string operations) / 文字列ライブラリ
- **SRFI 69**: Basic Hash Tables (hash table operations) / ハッシュテーブル
- **SRFI 111**: Boxes (mutable containers) / ボックス
- **SRFI 113**: Sets and Bags (collection data structures) / セットとバッグ
- **SRFI 125**: Intermediate Hash Tables (advanced hash tables) / 中級ハッシュテーブル
- **SRFI 132-141**: R7RS Large Red Edition features / R7RS Large機能

### Advanced Optimizations / 高度最適化
- **JIT Loop Optimization**: Compile-time loop optimization / コンパイル時ループ最適化
- **Tail-Call Optimization**: Stack-safe recursive calls / スタック安全な再帰呼び出し
- **Continuation Pooling**: Memory-efficient continuation reuse / メモリ効率的継続再利用
- **Expression Analysis**: Static analysis for optimization hints / 静的解析で最適化ヒント
- **Stack Overflow Protection**: Robust handling of deep recursion / 深い再帰の堅牢な処理

### Built-in Procedures / 組み込み関数 (103+ functions)

#### Arithmetic Operations / 算術演算 (28 functions)
- `+`, `-`, `*`, `/`: Basic arithmetic / 基本算術
- `=`, `<`, `>`, `<=`, `>=`: Numeric comparisons / 数値比較
- `abs`, `floor`, `ceiling`, `round`, `sqrt`, `expt`: Math functions / 数学関数
- `quotient`, `remainder`, `modulo`: Division operations / 除算演算
- `min`, `max`: Aggregation functions / 集約関数
- `exact?`, `inexact?`, `exact->inexact`, `inexact->exact`: Exactness / 正確性

#### List Operations / リスト演算 (15+ functions)
- `car`, `cdr`, `cons`: Basic list operations / 基本リスト演算
- `list`, `length`, `append`, `reverse`: List construction / リスト構築
- `map`, `filter`, `fold-left`, `fold-right`: Higher-order functions / 高階関数
- `take`, `drop`, `fold`, `any`, `every`: SRFI 1 extensions / SRFI 1拡張

#### String Operations / 文字列演算 (33+ functions)
- `string-length`, `string-append`, `substring`: Basic operations / 基本演算
- `string=?`, `string<?`, `string>?`: String comparisons / 文字列比較
- `string-contains`, `string-prefix?`, `string-suffix?`: SRFI 13 search / SRFI 13検索
- `string-take`, `string-drop`, `string-concatenate`: SRFI 13 manipulation / SRFI 13操作

#### Hash Tables / ハッシュテーブル (19+ functions)
- `make-hash-table`, `hash-table?`: Construction and predicates / 構築と述語
- `hash-table-set!`, `hash-table-ref`, `hash-table-delete!`: Basic operations / 基本演算
- `hash-table-keys`, `hash-table-values`, `hash-table-size`: Information / 情報取得
- `hash-table-fold`, `hash-table-map`: Higher-order operations / 高階演算

#### Type Predicates / 型述語 (15+ functions)
- `number?`, `string?`, `symbol?`, `list?`, `pair?`, `null?`: Basic types / 基本型
- `boolean?`, `procedure?`, `vector?`, `port?`: Advanced types / 高度型
- `integer?`, `real?`, `complex?`, `rational?`: Numeric types / 数値型

#### Control Flow / 制御フロー
- `call/cc`, `call-with-current-continuation`: Continuations / 継続
- `dynamic-wind`: Dynamic extent control / 動的範囲制御
- `raise`, `guard`, `with-exception-handler`: Exception handling / 例外処理
- `values`, `call-with-values`: Multiple values / 多値システム

#### I/O Operations / I/O演算
- `display`, `write`, `newline`, `read`: Basic I/O / 基本I/O
- `read-char`, `write-char`, `peek-char`: Character I/O / 文字I/O
- `eof-object?`, `char-ready?`: I/O predicates / I/O述語

## Bridge API / ブリッジAPI

The Bridge API enables seamless integration between Rust and Scheme code:

**日本語**: Bridge APIはRustとSchemeコード間のシームレスな統合を実現します。

### Type Conversion

Implement `ToScheme` and `FromScheme` traits for automatic type conversion:

```rust
use lambdust::{ToScheme, FromScheme, Value, Result};

// Custom type conversion
impl ToScheme for MyStruct {
    fn to_scheme(&self) -> Result<Value> {
        // Convert to Scheme value
        Ok(Value::from(self.value))
    }
}

impl FromScheme for MyStruct {
    fn from_scheme(value: &Value) -> Result<Self> {
        // Convert from Scheme value
        let val = i64::from_scheme(value)?;
        Ok(MyStruct { value: val })
    }
}
```

### External Functions

Register Rust functions to be callable from Scheme:

```rust
bridge.register_function("my-func", Some(2), |args| {
    let a = i64::from_scheme(&args[0])?;
    let b = i64::from_scheme(&args[1])?;
    (a + b).to_scheme()
});
```

### Object Management

Register and manipulate Rust objects from Scheme:

```rust
#[derive(Debug)]
struct Counter { value: i32 }

let counter = Counter { value: 0 };
let counter_id = bridge.register_object(counter, "Counter");
bridge.define("my-counter", Value::from(counter_id));
```

## Examples / 例

See the `examples/` directory for complete examples:

- `bridge_example.rs`: Demonstrates Bridge API usage / Bridge APIの使用方法
- `r7rs_standard_functions_demo.rs`: R7RS standard functions / R7RS標準関数
- `control_flow_demo.rs`: Advanced control flow (`call/cc`, exceptions) / 高度制御フロー
- `srfi_9_demo.rs`: SRFI 9 record types / SRFI 9レコード型
- `error_reporting_demo.rs`: Error handling and reporting / エラーハンドリング
- `advanced_repl_demo.scm`: Interactive REPL examples / インタラクティブREPL例
- `c_integration/`: C FFI integration examples / C FFI統合例
- `cpp_integration/`: C++ wrapper examples / C++ラッパー例

## Documentation

Complete API documentation is available at [docs.rs/lambdust](https://docs.rs/lambdust).

## Building from Source / ソースからのビルド

```bash
git clone https://github.com/akasaka-miraina/lambdust.git
cd lambdust
cargo build --release
```

### Feature Flags / 機能フラグ

```bash
# Minimal embedded build / 最小組み込みビルド
cargo build --release --no-default-features --features embedded

# Standard build with SRFI support / SRFI対応標準ビルド
cargo build --release --features standard

# Development build with all features / 全機能開発ビルド
cargo build --release --features development

# REPL executable / REPL実行ファイル
cargo build --release --features repl
```

### Running Tests / テスト実行

```bash
# Run all tests / 全テスト実行
cargo test

# Run specific test suites / 特定テストスイート実行
cargo test integration::r7rs_compliance_tests
cargo test unit::evaluator::
cargo test unit::srfi::

# Run tests with coverage report / カバレッジレポート付きテスト
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --open

# Performance benchmarks / パフォーマンスベンチマーク
cargo bench
```

### Test Coverage / テストカバレッジ

The project maintains high test coverage across all modules:

- **Unit Tests**: 150+ test cases covering core functionality / コア機能カバー
- **Integration Tests**: 50+ end-to-end tests / エンドツーエンドテスト
- **R7RS Compliance**: 546+ R7RS specification tests / R7RS仕様テスト
- **SRFI Tests**: 100+ SRFI library tests / SRFIライブラリテスト
- **Performance Tests**: Benchmark suites for optimization verification / 最適化検証ベンチマーク
- **Coverage Reports**: Automatically generated and uploaded to Codecov / 自動カバレッジレポート

Test coverage is automatically tracked and reported via Codecov. The project maintains comprehensive test coverage across all modules with targets of ≥85% overall and ≥80% for patches.

**日本語**: プロジェクトは全モジュールにわたって高いテストカバレッジを維持しています。

### Generating Documentation

```bash
cargo doc --open --no-deps
```

## Development Status / 開発状態

The development follows a systematic approach as documented in `CLAUDE.md`:

1. ✅ **Phase 1**: Core interpreter (lexer, parser, evaluator) / コアインタープリター
2. ✅ **Phase 2**: Built-in functions and macro system / 組み込み関数とマクロシステム
3. ✅ **Phase 3**: Bridge API and external integration / ブリッジAPIと外部統合
4. ✅ **Phase 4**: Advanced optimization (continuation pooling, RAII) / 高度最適化
5. ✅ **Phase 5**: Expression analysis and memory management / 式解析とメモリ管理
6. ✅ **Phase 6**: JIT optimization and stack safety / JIT最適化とスタック安全性
7. ✅ **SRFI Implementation**: Complete SRFI 1, 13, 69, 111, 113, 125, 132-141 / SRFI実装完了
8. ✅ **Production Ready**: Comprehensive error handling, formal verification / プロダクション品質

### Current Status (v0.3.0) / 現在の状態
- **R7RS Compliance**: 546/546 tests passing / R7RS準拠テスト全通過
- **SRFI Support**: 9 major SRFIs implemented / 9個の主要SRFI実装完了
- **Optimization**: JIT loop optimization, tail-call optimization / JITループ最適化、末尾呼び出し最適化
- **Memory Safety**: Stack overflow protection, RAII memory management / スタックオーバーフロー保護、RAIIメモリ管理
- **Quality**: Comprehensive testing, formal verification framework / 包括的テスト、形式的検証フレームワーク

## Contributing / コントリビューション

Contributions are welcome! Please feel free to submit a Pull Request.

**日本語**: コントリビューションを歓迎します！プルリクエストをお気軽に提出してください。

### Development Guidelines / 開発ガイドライン
- Follow the coding standards in `CLAUDE.md` / `CLAUDE.md`のコーディング規約を遵守
- Ensure all tests pass before submitting / 提出前に全テストが通ることを確認
- Add tests for new features / 新機能にはテストを追加
- Update documentation as needed / 必要に応じてドキュメントを更新

### Pre-commit Hooks / プリコミットフック
The project uses automated quality checks:
- **Clippy**: Static analysis and linting / 静的解析とリント
- **Tests**: All tests must pass / 全テスト合格必須
- **Documentation**: Doc build verification / ドキュメントビルド検証
- **Formatting**: Code formatting check / コードフォーマットチェック

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Performance / パフォーマンス

Lambdust is optimized for production use:

- **Memory Management**: RAII-based memory management with zero-cost abstractions / ゼロコスト抽象化でRAIIメモリ管理
- **JIT Optimization**: Loop optimization with native code generation / ネイティブコード生成でループ最適化
- **Stack Safety**: Robust stack overflow protection for deep recursion / 深い再帰のための強固なスタックオーバーフロー保護
- **Continuation Pooling**: Memory-efficient continuation reuse / メモリ効率的な継続再利用
- **Expression Analysis**: Static analysis for runtime optimization / ランタイム最適化のための静的解析

## Formal Verification / 形式的検証

Lambdust includes a formal verification framework:

- **Agda Integration**: Mathematical proofs for evaluator correctness / 評価器の正しさの数学的証明
- **Theorem Derivation**: Automated proof generation for optimizations / 最適化の自動証明生成
- **Runtime Verification**: Property-based testing with formal guarantees / 形式的保証付きプロパティベーステスト
- **R7RS Compliance**: Formal verification of R7RS specification adherence / R7RS仕様遵守の形式的検証

## Acknowledgments / 謝辞

- Built with R7RS Scheme specification compliance in mind / R7RS Scheme仕様準拠を念頭に構築
- Inspired by the elegance of Scheme and the power of Rust / SchemeのエレガンスとRustのパワーにインスパイア
- Special thanks to the Rust and Scheme communities / RustとSchemeコミュニティへの特別な感謝
- Formal verification support from Agda community / Agdaコミュニティからの形式的検証サポート

---

**Lambdust** - Adding λ-powered expressiveness to Rust applications.
**Lambdust** - Rustアプリケーションにλパワーの表現力を追加。