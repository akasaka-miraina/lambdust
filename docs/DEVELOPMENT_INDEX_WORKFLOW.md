# Lambdust Development Index Workflow

コードベースインデックス化システムにより、構造体・関数シグネチャの一貫性を保ち、保守性を大幅に向上させます。

## 🎯 インデックス化システムの目的

1. **構造体・関数の完全カタログ化**: 全ての public API の詳細な追跡
2. **シグネチャ一貫性の保証**: 実装時のエラーを事前防止
3. **変更影響の可視化**: 修正時の波及効果を明確化
4. **開発効率の向上**: 必要な情報への高速アクセス

## 📋 インデックス化ルール

### 1. 必須インデックス項目

**構造体定義**:
- 全てのpublic構造体
- フィールド名と型
- ファイル位置（パス + 行番号）
- 最終更新日時

**関数シグネチャ**:
- 全てのpublic関数・メソッド
- 完全なシグネチャ（引数・戻り値型）
- ファイル位置とライン番号
- 依存関係情報

**Enum定義**:
- 全てのpublic enum
- バリアント詳細
- フィールド付きバリアント

### 2. 更新タイミング

**必須更新**:
- 新しい public struct/enum/function 追加時
- 既存 API シグネチャ変更時
- ファイル移動・リネーム時
- release 準備時

**推奨更新**:
- 大きな機能実装後
- PR 作成前
- daily development workflow

## 🔧 コマンド使用方法

### 基本コマンド

```bash
# インデックス生成・更新
make index

# インデックス検証（CI用）
make index-check

# 詳細なコードベース統計
make stats
```

### 高度なワークフロー

```bash
# 開発時フルチェック（フォーマット・lint・テスト・インデックス）
make dev-full

# CI完全チェック（インデックス検証含む）
make ci-check-full

# R7RS-pico特定テスト
make test-pico
make demo-pico
```

## 📝 開発ワークフロー統合

### プレコミットチェック

```bash
# 推奨プレコミット手順
make pre-commit  # または以下を個別実行：
make fmt         # コードフォーマット
make lint        # Clippy linting
make test        # テスト実行
make index       # インデックス更新
```

### CI/CD 統合

```yaml
# GitHub Actions example
- name: Validate Code Index
  run: make index-check

- name: Run Full CI Pipeline
  run: make ci-check-full
```

## 🔍 インデックス利用方法

### 1. API 検索

インデックスファイルで構造体・関数を高速検索：

```bash
# 特定の構造体を検索
grep -n "struct.*PicoEvaluator" docs/CODE_INDEX_GENERATED.md

# 特定のモジュールの関数を確認
grep -A 5 "src/evaluator/pico_evaluator.rs" docs/CODE_INDEX_GENERATED.md
```

### 2. 実装時の参考

新機能実装前に必ず関連する既存 API を確認：

```bash
# Error 型の現在の構造を確認
grep -A 10 "LambdustError enum" docs/CODE_INDEX.md

# Environment API の確認
grep -A 20 "Environment struct" docs/CODE_INDEX.md
```

### 3. 影響分析

API 変更時の影響範囲を事前確認：

```bash
# 特定の型を使用している箇所を確認
grep -r "ExecutionContext" src/ --include="*.rs"

# 依存関係チェーン確認
cargo tree --depth 2
```

## 📊 生成されるインデックス情報

### 統計情報サマリー

- **Total Files**: 解析対象Rustファイル数
- **Total Structs**: Public構造体数
- **Total Enums**: Public enum数  
- **Total Functions**: Public関数数
- **Total Lines**: 総コード行数

### カテゴリ別整理

1. **🏗️ Core Infrastructure**: error.rs, value/, environment.rs
2. **🚀 Evaluator System**: evaluator/
3. **🎯 AST and Parsing**: ast.rs, parser.rs, lexer.rs
4. **🧮 Memory Management**: memory_pool.rs
5. **📝 Macro System**: macros/
6. **🔧 Built-in Functions**: builtins/
7. **🧪 Testing**: tests/
8. **📊 Type System**: type_system/
9. **🎨 SRFI Implementation**: srfi/

### 詳細 API 索引

重要な構造体・enum の完全定義とファイル位置

## ⚠️ 重要な注意事項

### インデックス更新の必須ケース

1. **新しい public API 追加**
   ```rust
   // この追加後は必ず make index
   pub struct NewEvaluator {
       // ...
   }
   ```

2. **既存シグネチャ変更**
   ```rust
   // 変更前: fn evaluate(&self, expr: Expr) -> Result<Value>
   // 変更後: fn evaluate(&mut self, expr: &Expr, env: Rc<Environment>) -> Result<Value>
   // 変更後は必ず make index
   ```

3. **ファイル移動・リネーム**
   ```bash
   # ファイル移動後
   git mv src/old_module.rs src/new_module.rs
   make index  # 必須
   ```

### CI での検証

```bash
# CI パイプライン内での検証
make index-check || exit 1  # インデックスが最新でない場合は失敗
```

## 🎯 最適な開発プラクティス

### 新機能開発時

```bash
# 1. 既存 API の確認
grep -n "相関する構造体名" docs/CODE_INDEX_GENERATED.md

# 2. 実装
# ... 開発作業 ...

# 3. インデックス更新
make index

# 4. 総合チェック
make dev-full
```

### バグ修正時

```bash
# 1. 影響範囲確認
grep -r "修正対象の関数名" src/ --include="*.rs"

# 2. 修正実装
# ... バグ修正 ...

# 3. API 変更があった場合のみインデックス更新
make index

# 4. テスト実行
make test
```

### R7RS-pico 開発時

```bash
# R7RS-pico 特定の開発ワークフロー
make check-pico     # pico feature でのコンパイル確認
make test-pico      # pico 関連テスト実行
make demo-pico      # demo 実行
make index          # インデックス更新
```

## 📈 パフォーマンス最適化

### インデックス生成の最適化

- **初回生成**: ~30秒 (398ファイル解析)
- **差分更新**: 検討中（将来の改善項目）
- **並列処理**: Python multiprocessing 活用可能

### CI 時間短縮

```bash
# 高速検証（生成なし）
make index-check  # 1-2秒で完了

# vs フル生成
make index        # 30秒程度
```

## 🔄 継続的改善

### Phase 1 (完了)
- ✅ 基本インデックス生成システム
- ✅ Makefile 統合
- ✅ CI 検証システム

### Phase 2 (将来)
- 🔄 差分更新システム
- 🔄 LSP インテグレーション
- 🔄 Visual Studio Code 拡張

### Phase 3 (構想)
- 🔄 依存関係グラフ可視化
- 🔄 自動ドキュメント生成
- 🔄 破壊的変更検出

## 📚 関連ドキュメント

- [CODE_INDEX.md](CODE_INDEX.md) - マニュアルインデックス
- [CODE_INDEX_GENERATED.md](CODE_INDEX_GENERATED.md) - 自動生成インデックス
- [DEVELOPMENT_FLOW.md](development/DEVELOPMENT_FLOW.md) - 開発フロー全般
- [BUILD_COMMANDS.md](user/BUILD_COMMANDS.md) - ビルドコマンド

---

**最終更新**: 2025-01-12  
**メンテナ**: Claude Code Assistant  
**システムバージョン**: v1.0