# Feature Flag最適化計画

## 現状分析

### 現在のdefault features
```toml
default = ["repl", "async", "advanced-io"]
```

これにより以下の重い依存関係が強制的に含まれている：
- **tokio** (1.47) - 最大級の依存関係
- **rustls** (0.23) - TLS実装
- **tokio-rustls** (0.26) 
- **webpki-roots** (0.26)
- **nix** (0.28) - Unix system calls
- **winapi** (0.3) - Windows APIs
- **rustyline** (13.0) - REPL support

## 最適化戦略

### 1. Minimal Default設定
**目標**: 最小限の機能でデフォルト構成

```toml
default = ["minimal-repl"]
minimal-repl = ["dep:colored"]  # coloredのみでシンプルREPL
```

**効果**: tokio, rustls, nix, winapiなど重い依存関係を除外

### 2. Optional Heavy Features
重い機能をオプション化：

```toml
full-repl = ["repl"]
async-runtime = ["async", "tokio", "tokio-util"] 
network-io = ["advanced-io"]
platform-extensions = ["unix-extensions", "windows-extensions"]
```

### 3. 使用頻度別feature分類

#### 🟢 Core Features (デフォルト維持)
- `minimal-repl` - 基本REPL (colored only)

#### 🟡 Common Features (明示的opt-in)
- `full-repl` - フル機能REPL
- `async-runtime` - 非同期処理
- `text-processing` - テキスト処理拡張

#### 🔴 Specialized Features (専用用途)
- `network-io` - ネットワーク機能
- `ffi` - FFI interop
- `benchmarks` - ベンチマーク
- `tls` - TLS/暗号化

## 実装計画

### Phase 1: Default Features軽量化
1. `default`から重い依存関係を削除
2. `minimal-repl`実装
3. 既存ヘビー機能のオプション化

### Phase 2: 条件付きコンパイル最適化
1. 未使用機能の`#[cfg(feature)]`追加
2. 重い依存関係の条件付きimport
3. ダミー実装で機能無効化

### Phase 3: Binary Size測定
1. feature組み合わせ別サイズ測定
2. 削減効果の定量評価

## 予想効果

### Binary Size削減
- **minimal config**: 60-70%削減 (tokio除外効果大)
- **selective features**: ユーザー選択による柔軟な最適化
- **development vs production**: 用途別最適化

### Development Experience
- 高速コンパイル (tokio除外)
- 明確な機能選択
- 段階的feature追加