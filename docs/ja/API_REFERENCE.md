# Lambdust APIリファレンス

このドキュメントでは、構造リファクタリングの成功とクリーンアーキテクチャの実装後の現在の状態を反映した、Lambdustのコア機能の包括的なAPIリファレンスを提供します。

## コアAPI概要

Lambdustは、明確な関心の分離を持つ論理モジュールに整理された豊富なAPIを提供します。すべてのAPIは一貫したパターンに従い、先進機能を提供しながらR7RS準拠を維持します。

## 値システム

### コア値型

```rust
use lambdust::eval::Value;

// 値の作成
let number = Value::number(42.0);
let string = Value::string("こんにちは");
let boolean = Value::boolean(true);
let symbol = Value::symbol("my-symbol");
let nil = Value::Nil;

// リストとペア
let list = Value::list(vec![
    Value::number(1.0),
    Value::number(2.0),
    Value::number(3.0)
]);
let pair = Value::pair(Value::number(1.0), Value::number(2.0));

// ベクタ
let vector = Value::vector(vec![
    Value::string("a"),
    Value::string("b"),
    Value::string("c")
]);
```

### 値操作

```rust
// 型チェック
assert!(value.is_number());
assert!(value.is_string());
assert!(value.is_list());

// 変換
let as_number: Option<f64> = value.as_number();
let as_string: Option<&str> = value.as_string();
let as_list: Option<Vec<Value>> = value.as_list();

// 表示とフォーマット
println!("{}", value);              // 表示表現
println!("{:?}", value);            // デバッグ表現
```

## 評価エンジン

### 基本評価

```rust
use lambdust::eval::{Evaluator, Environment};

// 評価器の作成
let mut evaluator = Evaluator::new();

// 簡単な式
let result = evaluator.eval("(+ 1 2 3)")?;
assert_eq!(result, Value::number(6.0));

// 変数
evaluator.eval("(define x 42)")?;
let result = evaluator.eval("x")?;
assert_eq!(result, Value::number(42.0));

// 関数
evaluator.eval("(define (square x) (* x x))")?;
let result = evaluator.eval("(square 5)")?;
assert_eq!(result, Value::number(25.0));
```

### 高度な評価

```rust
use lambdust::eval::monadic_architecture::MonadicEvaluationOrchestrator;

// エフェクトを含むモナディック評価
let orchestrator = MonadicEvaluationOrchestrator::new(config);
let input = MonadicEvaluationInput::new(expression, environment);
let result = orchestrator.evaluate_expression(input).await?;

// エフェクト対応評価
let result = orchestrator.evaluate_with_effects(
    expression,
    vec![Effect::IO, Effect::State],
    handlers
).await?;
```

## 型システム

### 漸進的型付け

```rust
use lambdust::types::{TypeSystem, TypeLevel, TypeInference};

// 型システム設定
let mut type_system = TypeSystem::new();
type_system.set_level(TypeLevel::Gradual);

// 型推論
let inferred_type = type_system.infer_type(expression)?;
println!("推論された型: {}", inferred_type);

// 型チェック
let is_valid = type_system.check_type(value, expected_type)?;

// 漸進的型注釈
evaluator.eval("(define (add x : Number y : Number) : Number (+ x y))")?;
```

### 型定義

```scheme
;; 代数的データ型
(define-type Color
  (Red)
  (Green) 
  (Blue)
  (RGB Number Number Number))

;; パターンマッチング
(define (color-to-string color)
  (match color
    [(Red) "red"]
    [(Green) "green"]
    [(Blue) "blue"]
    [(RGB r g b) (format "rgb(~a,~a,~a)" r g b)]))

;; 型クラス
(define-type-class (Eq a)
  (equal? : a a -> Boolean))

(define-instance (Eq Number)
  (define (equal? x y) (= x y)))
```

## エフェクトシステム

### 基本エフェクト

```rust
use lambdust::effects::{Effect, EffectHandler, EffectSystem};

// エフェクト型の定義
#[derive(Debug, Clone)]
pub enum ConsoleEffect {
    Print(String),
    ReadLine,
}

// エフェクトハンドラーの実装
struct ConsoleHandler;

impl EffectHandler<ConsoleEffect> for ConsoleHandler {
    type Result = Value;
    
    fn handle(&self, effect: ConsoleEffect) -> Result<Self::Result> {
        match effect {
            ConsoleEffect::Print(msg) => {
                println!("{}", msg);
                Ok(Value::Unspecified)
            }
            ConsoleEffect::ReadLine => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                Ok(Value::string(input.trim()))
            }
        }
    }
}
```

### モナディックプログラミング

```scheme
;; IOモナド例
(define-monad IO
  (return : a -> (IO a))
  (bind : (IO a) (a -> (IO b)) -> (IO b)))

;; 状態付き計算
(define-monad (State s)
  (return : a -> (State s a))
  (bind : (State s a) (a -> (State s b)) -> (State s b)))

;; エフェクトの組み合わせ
(define (interactive-program)
  (do [name (read-line)]
      [_ (print (string-append "こんにちは、" name "さん！"))]
      [count (get-state)]
      [_ (put-state (+ count 1))]
      (return count)))
```

### エフェクト協調

```rust
use lambdust::runtime::effect_coordination::EffectCoordinator;

// エフェクト協調の設定
let coordinator = EffectCoordinator::new()
    .with_isolation_level(EffectIsolationLevel::Strict)
    .with_concurrent_effects(true)
    .with_resource_limits(limits);

// 協調を用いた実行
let result = coordinator.execute_with_coordination(
    computation,
    dependencies,
    handlers
).await?;
```

## 並行性

### アクターモデル

```rust
use lambdust::concurrency::actors::{Actor, ActorSystem, Message};

// アクターの定義
struct CounterActor {
    count: i32,
}

#[derive(Debug)]
enum CounterMessage {
    Increment,
    Decrement,
    GetCount(tokio::sync::oneshot::Sender<i32>),
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    async fn handle(&mut self, message: Self::Message) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            CounterMessage::Increment => self.count += 1,
            CounterMessage::Decrement => self.count -= 1,
            CounterMessage::GetCount(sender) => {
                let _ = sender.send(self.count);
            }
        }
        Ok(())
    }
}

// アクターシステムの使用
let system = ActorSystem::new();
let counter = system.spawn(CounterActor { count: 0 }).await?;

counter.send(CounterMessage::Increment).await?;
let (tx, rx) = tokio::sync::oneshot::channel();
counter.send(CounterMessage::GetCount(tx)).await?;
let count = rx.await?;
```

### 並列評価

```scheme
;; 並列マップ
(define (parallel-map f lst)
  (parallel-eval
    (map (lambda (x) (spawn (lambda () (f x)))) lst)))

;; フューチャとプロミス
(define future-result
  (future (expensive-computation input)))

;; 後で...
(define result (force future-result))

;; 同期化を伴う並行評価
(define (concurrent-example)
  (let ([barrier (make-barrier 3)])
    (parallel-eval
      (spawn (lambda () (task-1) (barrier-wait barrier)))
      (spawn (lambda () (task-2) (barrier-wait barrier)))
      (spawn (lambda () (task-3) (barrier-wait barrier))))
    (final-task)))
```

### 同期化プリミティブ

```rust
use lambdust::concurrency::sync::{Mutex, RwLock, Semaphore, Barrier};

// 排他アクセス用ミューテックス
let mutex = Mutex::new(shared_data);
{
    let guard = mutex.lock().await;
    // クリティカルセクション
    guard.modify();
}

// 複数リーダー用RwLock
let rwlock = RwLock::new(shared_data);
let read_guard = rwlock.read().await;
let value = read_guard.get();

// リソース制御用セマフォ
let semaphore = Semaphore::new(3); // 3つの並行操作を許可
let permit = semaphore.acquire().await?;
// リソースを使用
drop(permit); // リソースを解放

// 同期化用バリア
let barrier = Barrier::new(4); // 4つのタスクを待機
barrier.wait().await;
```

## 外部関数インターフェース（FFI）

### C統合

```rust
use lambdust::ffi::{FfiRegistry, CFunction, safe_call};

// C関数の登録
let registry = FfiRegistry::global();
registry.register_function(
    "c_sqrt",
    CFunction::new("libm.so", "sqrt")
        .with_signature("double sqrt(double)")
        .with_safety_checks(true)
)?;

// Schemeから呼び出し
evaluator.eval("(define sqrt (ffi-import \"c_sqrt\"))")?;
let result = evaluator.eval("(sqrt 16.0)")?;
assert_eq!(result, Value::number(4.0));
```

### 安全なFFIパターン

```rust
use lambdust::ffi::{FfiCallback, MemoryManager};

// CからSchemeへのコールバック
let callback = FfiCallback::new(|args: &[Value]| -> Result<Value> {
    // 安全なコールバック実装
    let result = scheme_function(args)?;
    Ok(result)
});

// メモリ管理
let memory_manager = MemoryManager::new()
    .with_auto_cleanup(true)
    .with_leak_detection(cfg!(debug_assertions));

// 安全な外部メモリアクセス
let foreign_ptr = memory_manager.allocate_tracked(size)?;
// ドロップ時に自動クリーンアップ
```

## モジュールシステム

### モジュール定義

```scheme
;; モジュールを定義
(define-library (my-library math)
  (import (scheme base)
          (scheme inexact))
  (export square cube factorial)
  
  (begin
    (define (square x) (* x x))
    (define (cube x) (* x x x))
    (define (factorial n)
      (if (<= n 1)
          1
          (* n (factorial (- n 1)))))))
```

### モジュール読み込み

```rust
use lambdust::module_system::{ModuleSystem, LibraryId};

// モジュール読み込み
let module_system = ModuleSystem::new();
let library_id = LibraryId::new(vec!["my-library".to_string(), "math".to_string()]);
let library = module_system.load_library(&library_id)?;

// エクスポートにアクセス
let square_fn = library.get_export("square")?;
let result = evaluator.call_function(square_fn, vec![Value::number(5.0)])?;
```

### 動的モジュール読み込み

```scheme
;; ランタイムモジュール読み込み
(import-dynamically '(srfi 1) 
  (lambda (success?)
    (if success?
        (begin
          (display "SRFI-1が正常に読み込まれました")
          (use-list-functions))
        (error "SRFI-1の読み込みに失敗しました"))))

;; 条件付きインポート
(cond-expand
  (srfi-1 (import (srfi 1)))
  (else   (import (my-library list-utils))))
```

## パフォーマンスとベンチマーク

### 組み込みベンチマーク

```rust
use lambdust::benchmarks::{BenchmarkSuite, BenchmarkConfig};

// ベンチマークスイートの作成
let suite = BenchmarkSuite::new()
    .with_config(BenchmarkConfig {
        iterations: 1000,
        warmup_iterations: 100,
        timeout: Duration::from_secs(30),
        statistical_analysis: true,
    });

// ベンチマークの追加
suite.add_benchmark("fibonacci", || {
    evaluator.eval("(fibonacci 30)")
})?;

suite.add_benchmark("sort", || {
    evaluator.eval("(sort (generate-random-list 1000) <)")
})?;

// ベンチマーク実行
let results = suite.run().await?;
println!("結果: {}", results.summary());
```

### パフォーマンス監視

```scheme
;; 組み込みプロファイリング
(with-profiler
  (complex-computation input))

;; メモリ使用量監視
(define memory-before (gc-stats))
(run-memory-intensive-task)
(define memory-after (gc-stats))
(display-memory-diff memory-before memory-after)

;; パフォーマンスアサーション
(define-benchmark "fast-sort"
  (lambda () (sort random-data <))
  #:max-time 100ms
  #:min-ops-per-sec 1000)
```

## エラーハンドリング

### 診断エラー

```rust
use lambdust::diagnostics::{Error, DiagnosticError, SourceSpan};

// 診断エラーの作成
let error = DiagnosticError::new(
    "型の不一致",
    SourceSpan::new(line, column, length),
    "Number が期待されましたが、String が見つかりました"
);

// エラーヘルパー
use lambdust::diagnostics::error::helpers;

let runtime_error = helpers::runtime_error_simple("不正な操作");
let type_error = helpers::type_error("関数が期待されました", Some(span));
let syntax_error = helpers::syntax_error("予期しないトークン", span);
```

### 例外処理

```scheme
;; 例外処理
(define (safe-division x y)
  (guard (condition
          [(division-by-zero? condition) 
           (display "ゼロで割ることはできません")
           #f])
    (/ x y)))

;; カスタム例外型
(define-exception-type &custom-error
  &error
  make-custom-error
  custom-error?)

(define (raise-custom-error message)
  (raise (make-custom-error message)))

;; エラー回復
(with-exception-handler
  (lambda (condition)
    (log-error condition)
    (fallback-value))
  (lambda ()
    (risky-operation)))
```

## 高度な機能

### メタプログラミング

```scheme
;; コンパイル時評価
(define-syntax compile-time-factorial
  (syntax-rules ()
    [(_ n) (quote ,(factorial n))]))

;; コード生成
(define-syntax define-getter-setter
  (syntax-rules ()
    [(_ field)
     (begin
       (define (,(symbol-append 'get- field) obj)
         (,(symbol-append field '-ref) obj))
       (define (,(symbol-append 'set- field '!) obj value)
         (,(symbol-append field '-set!) obj value)))]))

;; プログラム解析
(define (analyze-performance expr)
  (let ([ast (parse expr)])
    (analyze-complexity ast)))
```

### リフレクション

```rust
use lambdust::metaprogramming::reflection::{Reflector, ObjectMetadata};

// ランタイムリフレクション
let reflector = Reflector::new();
let metadata = reflector.inspect_object(&value)?;

println!("型: {}", metadata.type_info());
println!("メソッド: {:?}", metadata.available_methods());
println!("フィールド: {:?}", metadata.field_names());

// 動的メソッド呼び出し
let result = reflector.invoke_method(&object, "method_name", args)?;
```

## 設定

### ランタイム設定

```rust
use lambdust::runtime::{LambdustRuntime, RuntimeConfig};

let config = RuntimeConfig {
    stack_size: 8 * 1024 * 1024,     // 8MBスタック
    heap_size: 256 * 1024 * 1024,   // 256MBヒープ
    gc_threshold: 0.8,                // ヒープ使用率80%でGC
    thread_pool_size: num_cpus::get(),
    enable_jit: true,
    enable_profiling: cfg!(debug_assertions),
    r7rs_strict_mode: false,
    allow_redefinition: true,
};

let runtime = LambdustRuntime::new(config)?;
```

### 機能フラグ

```toml
[dependencies.lambdust]
version = "0.1.1"
features = [
    "r7rs-large",      # R7RS-large標準ライブラリ
    "gradual-typing",  # 漸進的型システム
    "effect-system",   # 代数的エフェクト
    "simd",           # SIMD最適化
    "profiling",      # パフォーマンスプロファイリング
    "actors",         # アクターモデル
    "parallel",       # 並列評価
]
```

## 例

### 完全なプログラム

```scheme
#!/usr/bin/env lambdust

;; ファイル: fibonacci-server.scm
;; 型注釈とエフェクトを持つ並行フィボナッチサーバー

(import (scheme base)
        (scheme write)
        (lambdust actors)
        (lambdust effects)
        (lambdust types))

;; メモ化付き型付きフィボナッチ
(define (fibonacci n : Number) : Number
  (with-memoization
    (cond
      [(<= n 1) n]
      [else (+ (fibonacci (- n 1))
               (fibonacci (- n 2)))])))

;; フィボナッチ要求を処理するアクター
(define-actor fibonacci-actor
  (state (requests-handled : Number 0))
  
  (handle (compute n : Number)
    (do [result (fibonacci n)]
        [_ (set! requests-handled (+ requests-handled 1))]
        [_ (log-info (format "フィボナッチ(~a) = ~aを計算しました" n result))]
        (return result)))
  
  (handle (stats)
    (return requests-handled)))

;; エフェクト処理を伴うサーバー
(define (run-server port : Number)
  (with-effects [Console IO Network]
    (do [server (start-tcp-server port)]
        [actor (spawn-actor fibonacci-actor)]
        [_ (log-info (format "フィボナッチサーバーをポート~aで開始しました" port))]
        (server-loop server actor))))

(define (server-loop server actor)
  (do [request (accept-connection server)]
      [n (parse-request request)]
      [result (send-message actor (compute n))]
      [_ (send-response request result)]
      (server-loop server actor)))

;; メインエントリーポイント
(when (script-file?)
  (run-server 8080))
```

このAPIリファレンスは、体系的リファクタリングを通じて達成されたクリーンアーキテクチャ、包括的機能セット、プロフェッショナル実装品質を持つLambdustの現在の状態を反映しています。