# Lambdust (λust) - Rust Scheme Interpreter

## 重要

コードコメントやCLAUDE.md以外のmarkdownドキュメントは英語で，CLAUDE.mdやチャットは日本語で行います．

## 概要

Lambdust（λust）は、Rustで実装されたR7RS準拠のSchemeインタプリタです。アプリケーションへのマクロ組み込みメカニズムを提供することを目的としています。

## プロジェクト概要

- **言語**: Rust
- **対象仕様**: R7RS Scheme
- **主目的**: 外部アプリケーションへの組み込み可能なSchemeインタプリタ
- **特徴**: 軽量、高速、安全性重視

## 開発計画

### Phase 1: コア実装 (高優先度)
1. **R7RS仕様調査と実装範囲決定**
   - R7RS Small言語仕様の詳細調査
   - 最小実装セットの定義
   - 拡張実装の優先度決定

2. **プロジェクト構造設計**
   - Rustプロジェクト構造の決定
   - 依存関係の選定
   - モジュール設計

3. **字句解析器（Lexer）**
   - トークナイザーの実装
   - 数値、文字列、識別子の認識
   - コメント処理

4. **構文解析器（Parser）**
   - S式パーサーの実装
   - エラー処理機能
   - 位置情報の保持

5. **抽象構文木（AST）定義**
   - Scheme式の表現
   - 型安全なAST設計
   - パターンマッチング対応

6. **評価器（Evaluator）コア**
   - 基本的な式評価
   - 特殊形式の処理
   - 末尾再帰最適化

7. **環境管理**
   - スコープチェーンの実装
   - 変数束縛管理
   - クロージャーサポート

### Phase 2: 機能拡張 (中優先度)
8. **組み込み関数実装**
   - 算術演算
   - リスト操作
   - 条件分岐
   - I/O関数

9. **マクロシステム**
   - syntax-rules実装
   - マクロ展開エンジン
   - 衛生的マクロ

10. **組み込みAPI設計**
    - C FFI互換インターフェース
    - Rust API
    - エラーハンドリング

11. **テストスイート**
    - 単体テスト
    - 統合テスト
    - R7RS適合性テスト

## アーキテクチャ

```
lambdust/
├── src/
│   ├── lexer.rs      # 字句解析
│   ├── parser.rs     # 構文解析
│   ├── ast.rs        # AST定義
│   ├── evaluator.rs  # 評価器
│   ├── environment.rs # 環境管理
│   ├── builtins.rs   # 組み込み関数
│   ├── macros.rs     # マクロシステム
│   ├── api.rs        # 外部API
│   └── lib.rs        # ライブラリエントリーポイント
├── tests/            # テスト
├── examples/         # 使用例
└── Cargo.toml
```

## 実装方針

- **安全性**: Rustの型システムを活用したメモリ安全性
- **パフォーマンス**: ゼロコスト抽象化の活用
- **組み込み性**: 軽量で依存関係最小限
- **拡張性**: プラグイン機能とモジュール化

## ビルド・テストコマンド

```bash
# ビルド
cargo build

# テスト実行
cargo test

# リリースビルド
cargo build --release

# ドキュメント生成
cargo doc --open

# フォーマット
cargo fmt

# リント
cargo clippy
```

## 開発フロー

プロジェクトではpre-commitフックを使用してコード品質を自動チェックしています：

- **Clippy**: コードの静的解析とリント
- **Tests**: 全テストの実行とパス確認  
- **Documentation**: ドキュメントビルドの成功確認
- **Formatting**: コードフォーマットの確認（警告のみ）

コミット前に自動的にこれらのチェックが実行され、すべてグリーンシグナルであることが確認されます。

## 開発ステータス

- [x] 基本設計完了
- [x] 字句解析器実装
- [x] 構文解析器実装
- [x] 評価器実装（従来型 + R7RS形式的意味論準拠CPS評価器）
- [x] 組み込み関数実装（99%完了：103個の標準関数）
- [x] マクロシステム実装（SRFI 46拡張対応）
- [x] 外部API実装（ホスト連携・マーシャリング）
- [x] テスト完備（91テスト全パス）
- [x] ドキュメント整備

### R7RS Small実装完了ステータス（99%達成）

#### 🆕 R7RS形式的意味論準拠機能

1. **継続渡しスタイル評価器**
   - R7RS仕様書の形式文法に完全準拠
   - 継続ベースの評価モデル実装
   - 動的ポイント・環境変換サポート

2. **未指定評価順序サポート**
   - 左から右・右から左・非決定的順序
   - R7RSの"unspecified order"セマンティクス実装
   - 準拠性テスト対応

3. **拡張継続システム**
   - call/ccとescape procedures完全実装
   - dynamic-wind実装
   - 例外処理システム（guard, raise, with-exception-handler）

4. **完全多値システム**
   - values、call-with-values（evaluator統合完了）
   - 継続ベースの多値処理
   - R7RS準拠の戻り値処理

#### ✅ 完全実装済み

1. **基本データ型とリテラル**
   - 数値（整数・実数）、文字列、文字、シンボル、真偽値
   - ペア（cons cell）、リスト、ベクタ、レコード型

2. **算術・数値関数** (28関数)
   - 基本演算: +, -, *, /, quotient, remainder, modulo
   - 数学関数: abs, floor, ceiling, sqrt, expt
   - 集約関数: min, max
   - 述語: number?, integer?, real?, rational?, complex?, exact?, inexact?
   - 変換: exact->inexact, inexact->exact, number->string, string->number

3. **比較・等価関数** (12関数)
   - 数値比較: =, <, >, <=, >=
   - オブジェクト等価: eq?, eqv?, equal?
   - 型述語: boolean?, symbol?, char?, string?, pair?, null?, procedure?

4. **リスト操作関数** (11関数)
   - 基本操作: car, cdr, cons, list, append, reverse, length
   - 破壊的操作: set-car!, set-cdr!（クローンベース実装）
   - 変換: list->vector, list->string

5. **文字列・文字関数** (23関数)
   - 文字述語・比較: char=?, char<?, char>?, char-alphabetic?, char-numeric?等
   - 文字変換: char-upcase, char-downcase, char->integer, integer->char
   - 文字列操作: string=?, string<?, make-string, string-length, string-ref等
   - 変換: string->list, string->number, char->string, number->string

6. **ベクタ操作関数** (6関数)
   - 基本操作: vector, make-vector, vector-length, vector-ref, vector-set!
   - 変換: vector->list, list->vector

7. **I/O関数** (7関数)
   - 基本I/O: read, write, read-char, write-char, peek-char
   - 述語: eof-object?, char-ready?

8. **高階関数**
   - apply, map, for-each（完全実装）

9. **継続・例外処理** (5関数)
   - 継続: call/cc, call-with-current-continuation
   - 例外: raise, with-exception-handler
   - 制御: dynamic-wind

10. **多値システム**
    - values, call-with-values（基盤実装完了）

11. **レコード型（SRFI 9）** (4関数)
    - make-record, record-of-type?, record-field, record-set-field!
    - 完全なdefine-record-type実装

12. **エラーハンドリング**
    - error関数（irritant対応）

#### 🔄 部分実装・統合待ち

- **call-with-values**: 多値システム基盤は完成、evaluator統合待ち
- **継続・例外処理**: 基盤構造完成、完全動作にはevaluator統合必要
- **マクロシステム**: 基本syntax-rules実装済み、SRFI 46拡張待ち

#### ⏳ 今後の拡張予定

- **SRFI 45**: 遅延評価プリミティブ（delay, force, promise?）
- **SRFI 46**: syntax-rules拡張（楕円記法強化）
- **完全なevaluator統合**: 継続・例外・多値の完全動作化

### アーキテクチャ改善完了

- **モジュール化**: 2663行の巨大builtins.rsを8つの機能別モジュールに分割
  - arithmetic.rs（算術）、list_ops.rs（リスト）、string_char.rs（文字列・文字）
  - vector.rs（ベクタ）、predicates.rs（述語）、io.rs（I/O）
  - control_flow.rs（継続・例外）、misc.rs（多値・レコード）、error_handling.rs（エラー）
- **保守性向上**: 機能別の独立テスト可能性と新機能追加の容易性確保
- **テスト完備**: 76テスト全パス、デモプログラム5個で動作確認

## R7RS Small仕様とSRFI実装計画

### R7RS Smallで標準組み込み済みSRFI

以下のSRFIはR7RS Small仕様に標準として組み込まれており、必須実装項目です：

1. **SRFI 9: Define-record-type**
   - レコード型定義（define-record-type）
   - 構造体的なデータ型の定義機能
   - 優先度: 必須

2. **SRFI 45: Primitives for Expressing Iterative Lazy Algorithms** ✅
   - プロミス（promise）とディレイ（delay）の拡張
   - 遅延評価機能の強化（delay, force, lazy, promise?）
   - 優先度: 必須 → **完全実装済み**

3. **SRFI 46: Basic Syntax-rules Extensions** ✅
   - syntax-rulesマクロシステムの拡張
   - 楕円記法の強化（nested ellipsis対応）
   - 優先度: 必須 → **完全実装済み**

### 実装推奨SRFI（高優先度）

R7RS Small実装で広く使用される基本機能：

4. **SRFI 1: List Library**
   - リスト処理の基本ライブラリ
   - fold, map, filter等の高階関数
   - 優先度: 高

5. **SRFI 13: String Libraries**
   - 文字列操作の基本ライブラリ
   - インデックスベースの文字列処理
   - 優先度: 高

6. **SRFI 69: Basic Hash Tables**
   - ハッシュテーブルの基本実装
   - 辞書型データ構造
   - 優先度: 高

### 実装推奨SRFI（中優先度）

データ構造と操作の拡張：

7. **SRFI 111: Boxes**
   - 単一スロットレコード（box）
   - 可変参照型
   - 優先度: 中

8. **SRFI 125: Intermediate Hash Tables**
   - SRFI 69の上位互換拡張
   - より高度なハッシュテーブル機能
   - 優先度: 中

9. **SRFI 128: Comparators**
   - 比較子ライブラリ
   - ソートや検索で使用
   - 優先度: 中

10. **SRFI 133: Vector Library**
    - ベクタ操作の拡張ライブラリ
    - SRFI 43のR7RS互換版
    - 優先度: 中

### 実装予定SRFI（低優先度）

高度な機能拡張：

11. **SRFI 113: Sets and Bags**
    - 集合と多重集合のデータ構造
    - 線形更新対応
    - 優先度: 低

12. **SRFI 130: Cursor-based String Library**
    - カーソルベースの文字列処理
    - SRFI 13の拡張版
    - 優先度: 低

### 実装方針

- **Phase 1**: 必須SRFI（9, 45, 46）の完全実装
- **Phase 2**: 高優先度SRFI（1, 13, 69）の実装
- **Phase 3**: 中優先度SRFI（111, 125, 128, 133）の実装
- **Phase 4**: 低優先度SRFI（113, 130）の実装

各SRFIは独立したモジュールとして実装し、必要に応じて組み込み可能な設計とします。

## ホストアプリケーション連携機能設計

### 設計思想

LambdustはGIMPのScript-Fuのように、ホストアプリケーションとの双方向連携を可能にします。安全性を重視し、unsafeな操作はマーシャリング層に封じ込めることで、将来的なC/C++埋め込みにも対応します。

### 1. ホスト関数の公開機能

ホストアプリケーションからlambdust環境への関数公開：

```rust
// ホスト側でのlambdust関数公開例
let mut interpreter = LambdustInterpreter::new();

// 型安全な関数登録
interpreter.register_host_function(
    "host-print",           // Scheme関数名
    |args: &[Value]| -> Result<Value, Error> {
        // ホスト側の実装
        println!("{}", args[0].to_string());
        Ok(Value::Void)
    }
);

// 複雑な型の自動変換
interpreter.register_host_function_with_signature(
    "host-calculate",
    vec![ValueType::Number, ValueType::Number], // 引数型
    ValueType::Number,                          // 戻り値型
    |args| {
        let a = args[0].as_number()?;
        let b = args[1].as_number()?;
        Ok(Value::Number(a + b))
    }
);
```

### 2. lambdust関数の呼び出し機能

ホストアプリケーションからlambdust環境の関数呼び出し：

```rust
// lambdust環境で定義された関数の呼び出し
let result = interpreter.call_scheme_function(
    "user-defined-function",
    &[Value::Number(42.0), Value::String("hello".to_string())]
)?;

// 型安全な結果の取得
match result {
    Value::Number(n) => println!("Result: {}", n),
    Value::String(s) => println!("Result: {}", s),
    _ => println!("Unexpected result type"),
}
```

### 3. 型安全マーシャリング設計

安全性を確保するマーシャリング層：

```rust
/// 型安全なマーシャリング機能
pub struct TypeSafeMarshaller {
    type_registry: HashMap<TypeId, Box<dyn TypeConverter>>,
}

impl TypeSafeMarshaller {
    /// Rust型からScheme Valueへの変換
    pub fn rust_to_scheme<T: 'static>(&self, value: T) -> Result<Value, MarshalError> {
        // 型情報を使用した安全な変換
    }
    
    /// Scheme ValueからRust型への変換
    pub fn scheme_to_rust<T: 'static>(&self, value: &Value) -> Result<T, MarshalError> {
        // 型チェックを含む安全な変換
    }
}

/// 型変換エラー
#[derive(Debug)]
pub enum MarshalError {
    TypeMismatch { expected: String, found: String },
    ConversionFailed(String),
    UnsupportedType(String),
}
```

### 4. C/C++埋め込み対応設計

将来的なC/C++埋め込みを考慮したインターフェース：

```rust
/// C FFI互換インターフェース
#[repr(C)]
pub struct LambdustContext {
    interpreter: Box<LambdustInterpreter>,
    error_buffer: [c_char; 256],
}

/// C互換関数シグネチャ
pub type CHostFunction = unsafe extern "C" fn(
    argc: c_int,
    argv: *const *const c_char,
    result: *mut *mut c_char
) -> c_int;

#[no_mangle]
pub unsafe extern "C" fn lambdust_create_context() -> *mut LambdustContext {
    // C/C++から安全に呼び出し可能なコンテキスト作成
}

#[no_mangle]
pub unsafe extern "C" fn lambdust_register_function(
    ctx: *mut LambdustContext,
    name: *const c_char,
    func: CHostFunction
) -> c_int {
    // C関数の登録（unsafeな操作を内部で処理）
}
```

### 5. 安全性保証機能

- **型チェック**: 実行時型検証によるメモリ安全性確保
- **エラーハンドリング**: Panicを発生させない堅牢なエラー処理
- **メモリ管理**: 自動的なライフタイム管理とリソース解放
- **サンドボックス**: ホスト環境への不正アクセス防止

### 6. パフォーマンス考慮事項

- **ゼロコピー**: 可能な限りデータコピーを避ける設計
- **インライン展開**: 頻繁に呼ばれる関数の最適化
- **キャッシュ**: 型変換結果のキャッシュ機能
- **バッチ処理**: 複数の値を一括で変換する機能

### 実装段階

1. **Phase 1**: 基本マーシャリング機能とホスト関数登録
2. **Phase 2**: lambdust関数呼び出し機能と型安全性強化
3. **Phase 3**: C/C++ FFIインターフェースとパフォーマンス最適化
4. **Phase 4**: サンドボックス機能と高度なセキュリティ対策

## 開発フロー

### 基本的な作業手順

1. **Issue作成**: GitHubでIssueを作成し、作業内容を明確化
2. **ブランチ作成**: mainブランチからfeatureブランチをfork
3. **設計・実装**: 機能の設計と実装を行う
4. **Pull Request**: GitHub CopilotのレビューコメントあるPRを作成
5. **レビュー・マージ**: コードレビュー後、mainブランチにマージ

### Issue・PR作成のガイドライン

各作業では以下のテンプレートを使用してください：

- **Issue**: `.github/ISSUE_TEMPLATE/feature_request.md`
- **Pull Request**: `.github/pull_request_template.md`

これらのテンプレートはプロジェクトルートに配置されており、GitHub Copilotのレビューを効果的に活用できるよう設計されています。

### ブランチ命名規則

- 機能追加: `feature/description`
- バグ修正: `fix/description`
- ドキュメント: `docs/description`
- テスト: `test/description`

例: `feature/srfi-1-list-library`, `fix/memory-leak-in-parser`

## 今後の拡張予定

- REPL実装
- デバッガー機能
- プロファイラー
- コンパイラー機能（バイトコード生成）
- 並行処理サポート