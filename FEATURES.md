# Lambdust 機能フラグドキュメント

## 📋 概要

Lambdustは条件コンパイルを使用して、様々な機能を選択的に有効にできます。これにより、必要な機能のみを含む軽量なバイナリを作成できます。

## 🎯 デフォルト設定

```toml
default = ["minimal-repl"]
```

**デフォルトでは軽量な最小設定のREPLのみが有効になっています。**

## 🔧 機能カテゴリ

### 1. REPL設定

| 機能フラグ | 説明 | 依存関係 | デフォルト |
|-----------|-----|---------|----------|
| `minimal-repl` | 基本色付きREPL | `colored` | ✅ |
| `repl` | フル機能REPL | `minimal-repl`, `rustyline`, `dirs` | ❌ |
| `enhanced-repl` | 高機能REPL | `repl`, `reedline`, `nu-ansi-term`, `crossterm`, `syntect` | ❌ |

### 2. ランタイム設定

| 機能フラグ | 説明 | 依存関係 | デフォルト |
|-----------|-----|---------|----------|
| `async-runtime` | 非同期ランタイム | `tokio`, `tokio-util` | ❌ |
| `network-io` | ネットワークI/O | `async-runtime`, TLS関連 | ❌ |
| `platform-extensions` | プラットフォーム拡張 | Unix/Windows API | ❌ |

### 3. 開発・テスト機能

| 機能フラグ | 説明 | 依存関係 | デフォルト |
|-----------|-----|---------|----------|
| `benchmarks` | パフォーマンス計測 | `criterion`, `flame` | ❌ |
| `property-testing` | プロパティベーステスト | `proptest` | ❌ |
| `multithreaded-tests` | マルチスレッドテスト | `async`, `tokio/test-util` | ❌ |

### 4. 高度な機能

| 機能フラグ | 説明 | 依存関係 | デフォルト |
|-----------|-----|---------|----------|
| `ffi` | 外部関数インターフェース | `libffi`, `cc` | ❌ |
| `jit` | JITコンパイル（将来予定） | なし | ❌ |
| `formal-methods` | 形式手法統合 | なし | ❌ |

### 5. I/O・テキスト処理

| 機能フラグ | 説明 | 依存関係 | デフォルト |
|-----------|-----|---------|----------|
| `compression` | 圧縮サポート | `flate2`, `zstd`, `lz4_flex` | ❌ |
| `text-processing` | Unicode高度処理 | ICU関連 | ❌ |
| `tls` | TLS/SSL暗号化 | `rustls`, `tokio-rustls`, `webpki-roots` | ❌ |

## ✅ 条件コンパイル修正完了

### 修正された機能フラグ

1. **JIT関連** - 正しい条件コンパイルに修正完了
   - ✅ `#[cfg(feature = "jit")]` - JIT機能が有効な場合
   - ✅ `#[cfg(not(feature = "jit"))]` - JIT機能が無効な場合（デフォルト）

2. **型システム関連** - 新しい型システムフラグを追加
   - ✅ `hindley-milner` - Hindley-Milner型推論システム
   - ✅ `monad-aware` - モナド対応型システム  
   - ✅ `experimental-type-system` - 実験的型システム機能全般

### 追加された型システムフラグ

```toml
# Type System Features  
hindley-milner = []                                 # Hindley-Milner type inference
monad-aware = ["hindley-milner"]                    # Monad-aware type system  
experimental-type-system = ["hindley-milner", "monad-aware"] # Advanced type system features
```

### 使用状況

- **デフォルト**: 型システム機能とJIT機能は**無効**
- **実験的型システム有効化**: `--features experimental-type-system`
- **JIT機能有効化**: `--features jit`
- **完全な開発環境**: `--features "experimental-type-system,jit,enhanced-repl,benchmarks"`

## 💡 推奨使用例

### 最小限のインストール
```bash
cargo build --no-default-features
```

### REPL開発環境
```bash 
cargo build --features "repl,benchmarks"
```

### フル機能開発環境
```bash
cargo build --features "enhanced-repl,async-runtime,benchmarks,property-testing"
```

### プロダクション配布
```bash
cargo build --release --features "minimal-repl,compression"
```

## 📝 注意事項

1. **デフォルトは最小構成** - 必要な機能のみ明示的に有効化
2. **後方互換性** - 一部のフラグは既存コードとの互換性のために残されている
3. **実験的機能** - `formal-methods`, `jit` などは将来の機能
4. **条件コンパイル警告** - 現在の `*-disabled` フラグは修正が必要

このドキュメントは機能フラグの使用状況に応じて更新される予定です。