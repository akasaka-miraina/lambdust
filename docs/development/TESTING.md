# Lambdust Testing Structure

Lambdustプロジェクトでは、テストを明確に分離して保守性を向上させています。

## テスト構造

```
tests/
├── unit/                 # 単体テスト
│   ├── mod.rs           # 単体テストモジュール
│   ├── lexer_tests.rs   # Lexer単体テスト (7テスト)
│   ├── parser_tests.rs  # Parser単体テスト (9テスト)
│   ├── lib_tests.rs     # Library API単体テスト (2テスト)
│   └── higher_order_tests.rs # 高階関数単体テスト (3テスト)
└── integration/          # 統合テスト
    ├── mod.rs           # 統合テストモジュール
    ├── bridge_tests.rs  # Bridge API統合テスト
    ├── error_handling_tests.rs # エラーハンドリング統合テスト
    ├── evaluator_tests.rs # Evaluator統合テスト
    ├── exception_handling_tests.rs # 例外処理統合テスト
    ├── integration_tests.rs # 基本統合テスト
    ├── r7rs_compliance_tests.rs # R7RS準拠性テスト
    ├── syntax_rules_tests.rs # マクロシステムテスト
    ├── srfi_1_tests.rs  # SRFI 1統合テスト
    ├── srfi_13_tests.rs # SRFI 13統合テスト
    ├── srfi_69_tests.rs # SRFI 69統合テスト
    ├── srfi_97_tests.rs # SRFI 97統合テスト
    └── srfi_tests.rs    # 一般SRFI統合テスト
```

## テストタイプ

### 単体テスト (Unit Tests)

**場所**: `tests/unit/`

**目的**: 個別のコンポーネントや関数を分離してテストします。

**特徴**:
- 高速実行
- 依存関係最小
- 関数レベルの詳細テスト
- モックやスタブを使用

**実行方法**:
```bash
# 全単体テスト実行
cargo test unit

# 特定モジュールの単体テスト実行
cargo test unit::lexer_tests
cargo test unit::parser_tests
cargo test unit::higher_order_tests
cargo test unit::lib_tests
```

#### 現在の単体テスト

1. **Lexer Tests (7テスト)**
   - 基本トークン認識
   - 数値（整数・実数・有理数）
   - 文字列リテラル
   - シンボル
   - ブール値
   - クォート関連トークン
   - コメント処理

2. **Parser Tests (9テスト)**
   - アトム構文解析
   - 単純リスト
   - ネストリスト
   - クォート式
   - ドット記法リスト
   - 空リスト
   - クォートシンタックス
   - エラーハンドリング

3. **Higher-Order Tests (3テスト)**
   - map関数（builtin関数）
   - apply関数（builtin関数）
   - fold関数（builtin関数）

4. **Library Tests (2テスト)**
   - 基本算術評価
   - 変数定義・参照

### 統合テスト (Integration Tests)

**場所**: `tests/integration/`

**目的**: 複数のコンポーネントが連携して動作することを確認します。

**特徴**:
- 実際のAPI使用
- エンドツーエンドテスト
- 実データでのテスト
- システム全体の動作確認

**実行方法**:
```bash
# 全統合テスト実行
cargo test integration

# 特定の統合テスト実行
cargo test integration::evaluator_tests
cargo test integration::srfi_1_tests
```

#### 現在の統合テスト

1. **Core System Tests**
   - Bridge API: ホスト・ゲスト間連携
   - Error Handling: エラー検出・報告
   - Evaluator: 式評価エンジン
   - Exception Handling: 例外システム
   - Integration: 基本統合機能

2. **Compliance Tests**
   - R7RS Compliance: R7RS仕様準拠性
   - Syntax Rules: マクロシステム

3. **SRFI Tests**
   - SRFI 1: List Library
   - SRFI 13: String Libraries
   - SRFI 69: Basic Hash Tables
   - SRFI 97: SRFI Feature Availability
   - General SRFI: 汎用SRFI機能

## テスト実行

### 全テスト実行

```bash
# 全テスト（単体＋統合）
cargo test

# 並列実行制御
cargo test -- --test-threads=1

# 詳細出力
cargo test -- --nocapture
```

### カテゴリ別実行

```bash
# 単体テストのみ
cargo test unit

# 統合テストのみ  
cargo test integration

# 特定機能テスト
cargo test lexer
cargo test parser
cargo test srfi
```

### フィルタリング

```bash
# 特定パターンにマッチするテスト
cargo test test_basic

# 特定ファイルのテスト
cargo test --test integration

# 無視されたテストも実行
cargo test -- --ignored
```

## CI/CD統合

### GitHub Actions

プロジェクトはGitHub ActionsでのCI/CDに対応しています：

```yaml
- name: Run Unit Tests
  run: cargo test unit

- name: Run Integration Tests  
  run: cargo test integration

- name: Run All Tests
  run: cargo test
```

### Pre-commit Hooks

コミット前に自動的にテストが実行されます：

```bash
# すべてのテストをパス
cargo test

# リンティング
cargo clippy

# フォーマット確認
cargo fmt --check
```

## テストの追加

### 新しい単体テストの追加

1. `tests/unit/`に新しいファイルを作成
2. `tests/unit/mod.rs`にモジュール追加
3. テスト関数を実装

```rust
// tests/unit/new_module_tests.rs
use lambdust::your_module::YourFunction;

#[test]
fn test_your_function() {
    let result = YourFunction::new();
    assert!(result.is_ok());
}
```

### 新しい統合テストの追加

1. `tests/integration/`に新しいファイルを作成
2. `tests/integration/mod.rs`にモジュール追加
3. エンドツーエンドテストを実装

```rust
// tests/integration/new_feature_tests.rs
use lambdust::{Interpreter, Value};

#[test]
fn test_new_feature_integration() {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval("(new-feature)").unwrap();
    assert_eq!(result, Value::from(expected_value));
}
```

## ベストプラクティス

### 単体テスト

- 一つの関数・メソッドに焦点
- 高速実行を重視
- モックやスタブを活用
- エッジケースを網羅
- 明確なテスト名

### 統合テスト

- 実際のユースケースをテスト
- 公開APIのみ使用
- データの流れを確認
- エラーシナリオも含める
- 環境依存性を最小化

### テスト品質

- **可読性**: テストの意図が明確
- **信頼性**: 一貫した結果
- **保守性**: 変更に対する耐性
- **高速性**: 開発サイクルを妨げない
- **網羅性**: 重要な機能を確実にカバー

## トラブルシューティング

### よくある問題

1. **テストの失敗**
   ```bash
   # 詳細情報表示
   cargo test failing_test -- --nocapture
   
   # 単体実行でデバッグ
   cargo test specific_test_name
   ```

2. **並行実行の問題**
   ```bash
   # シーケンシャル実行
   cargo test -- --test-threads=1
   ```

3. **環境依存の問題**
   ```bash
   # クリーンビルド
   cargo clean && cargo test
   ```

### パフォーマンス最適化

- 重いテストは`#[ignore]`でマーク
- 単体テストを優先的に実行
- 並列実行可能な設計
- 不要な初期化処理を削減

## 将来の拡張

- **マルチプラットフォームテスト**: Windows/macOS/Linux
- **パフォーマンステスト**: ベンチマーク統合
- **プロパティベーステスト**: QuickCheck風テスト
- **ファズテスト**: セキュリティ強化
- **カバレッジ測定**: テスト網羅率分析