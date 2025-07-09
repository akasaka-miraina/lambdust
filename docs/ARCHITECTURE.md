# アーキテクチャ

## 🏗️ アーキテクチャ理解のポイント

### 評価器システム（src/evaluator/）
- **CPS評価器**: 継続渡しスタイルでR7RS準拠の理論的正確性を実現
- **トランポリン実装**: スタックオーバーフロー防止のためのevaluator/trampoline.rs
- **JIT最適化**: 反復処理をネイティブコードに変換するjit_loop_optimization.rs
- **継続管理**: continuation.rs・continuation_pooling.rs・doloop_continuation.rs
- **式解析**: expression_analyzer.rsによる静的解析・最適化ヒント生成

### 値システム（src/value/）
- **統合Value型**: 全Scheme値の統一表現・型安全性確保
- **手続き**: procedure.rs・continuation.rs・promise.rs
- **データ構造**: list.rs・pair.rs・record.rs・port.rs
- **変換**: conversions.rs・equality.rs・predicates.rs

### 組み込み関数（src/builtins/）
- **モジュール化**: 機能別分割・重複排除・utils.rs共通化
- **算術**: arithmetic.rs・文字列: string_char.rs・リスト: list_ops.rs
- **制御**: control_flow.rs・I/O: io.rs・述語: predicates.rs
- **高階**: higher_order.rs・例外: error_handling.rs・遅延: lazy.rs

### SRFI実装（src/srfi/）
- **モジュール統合**: SrfiModule trait・registry.rs登録システム
- **完全実装**: SRFI 1・13・69・111・113・125・132・133・141
- **型安全**: 統一インターフェース・エラーハンドリング統合

### 埋め込みAPI（src/）
- **bridge.rs**: Rust-Scheme間の型安全な値交換・関数登録・オブジェクト管理
- **interpreter.rs**: 高レベルAPI・実行環境・評価インターフェース
- **marshal.rs**: 自動型変換・ToScheme/FromScheme traits・エラーハンドリング

## アーキテクチャ

```
lambdust/
├── src/
│   ├── lexer.rs         # 字句解析
│   ├── parser.rs        # 構文解析
│   ├── ast.rs           # AST定義
│   ├── evaluator/       # R7RS準拠CPS評価器（モジュール化完了）
│   │   ├── mod.rs       # コア評価ロジック
│   │   ├── continuation.rs # 継続データ構造
│   │   ├── types.rs     # 基本型定義
│   │   ├── special_forms.rs # 特殊形式評価
│   │   ├── control_flow.rs # 制御フロー
│   │   ├── higher_order.rs # 高階関数
│   │   └── imports.rs   # SRFIインポート
│   ├── environment.rs   # 環境管理
│   ├── builtins/        # 組み込み関数モジュール群
│   │   ├── mod.rs       # 統合モジュール
│   │   ├── utils.rs     # 共通ユーティリティ（重複削減）
│   │   ├── arithmetic.rs # 算術関数
│   │   ├── list_ops.rs  # リスト操作
│   │   ├── string_char.rs # 文字列・文字
│   │   ├── vector.rs    # ベクタ操作
│   │   ├── predicates.rs # 述語関数
│   │   ├── io.rs        # I/O関数
│   │   ├── control_flow.rs # 継続・例外処理
│   │   ├── misc.rs      # 多値・レコード
│   │   ├── error_handling.rs # エラー処理
│   │   └── lazy.rs      # 遅延評価（SRFI 45）
│   ├── macros.rs        # マクロシステム
│   ├── bridge.rs        # アプリケーション統合API
│   ├── interpreter.rs   # ホスト連携インターフェース
│   ├── host.rs          # ホスト関数管理
│   ├── marshal.rs       # 型安全マーシャリング
│   ├── value.rs         # Scheme値システム
│   ├── error.rs         # エラーハンドリング
│   └── lib.rs           # ライブラリエントリーポイント
├── tests/               # テスト
├── examples/            # 使用例
├── .github/             # GitHub統合
│   ├── workflows/       # CI/CD Actions
│   └── ISSUE_TEMPLATE/  # テンプレート
└── Cargo.toml
```