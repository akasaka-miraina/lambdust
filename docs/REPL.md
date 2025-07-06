# Lambdust REPL - Interactive Scheme Environment

Lambdust REPL（Read-Eval-Print Loop）は、R7RS準拠のSchemeインタプリタであるLambdustの対話型実行環境です。

## インストールとビルド

REPLを使用するには、`repl`フィーチャーを有効にしてビルドする必要があります：

```bash
# REPLバイナリをビルド
cargo build --features repl --bin lambdust

# またはリリースビルド
cargo build --release --features repl --bin lambdust
```

## 基本的な使用方法

### 対話型セッションの開始

```bash
# REPLを起動
./target/debug/lambdust

# バナーを表示せずに起動
./target/debug/lambdust --no-banner

# カスタムプロンプトで起動
./target/debug/lambdust --prompt "scheme> "
```

### ファイルの読み込み

```bash
# ファイルを読み込んでからREPLを開始
./target/debug/lambdust examples/demo.scm
```

## REPL機能

### 基本的な評価

```scheme
λust> (+ 1 2 3)
6

λust> (define square (lambda (x) (* x x)))

λust> (square 5)
25

λust> (map square '(1 2 3 4))
(1 4 9 16)
```

### 複数行入力

REPLは括弧の対応を自動的に検出し、複数行入力をサポートします：

```scheme
λust> (define factorial
   ...   (lambda (n)
   ...     (if (= n 0)
   ...         1
   ...         (* n (factorial (- n 1))))))

λust> (factorial 5)
120
```

### 特別なコマンド

- `(exit)` または `(quit)` - REPLを終了
- `(help)` - ヘルプメッセージを表示
- `(clear)` - 画面をクリア
- `(env)` - 現在の環境（関数・変数）を表示
- `(reset)` - インタプリタの状態をリセット
- `(load "filename")` - ファイルを読み込んで評価
- `(debug on)` - デバッグモードを有効化
- `(debug off)` - デバッグモードを無効化
- `(break)` - 次の評価でブレークポイントを設定
- `(step)` - ステップ実行モードを有効化
- `(continue)` - ブレークポイントから実行を継続
- `(backtrace)` - 呼び出しスタックを表示

### キーボードショートカット

- `Ctrl+C` - 現在の入力をキャンセル
- `Ctrl+D` - REPLを終了（EOF）
- `↑/↓` - コマンド履歴の操作
- `Tab` - 関数名・特殊フォーム・ファイル名の補完
- `Alt+Tab` - 詳細な補完候補表示

## コマンドライン オプション

```
USAGE:
    lambdust [OPTIONS] [FILE]

ARGUMENTS:
    <FILE>    読み込み実行するSchemeファイル

OPTIONS:
    --no-banner           ウェルカムバナーを表示しない
    --no-history          コマンド履歴を無効にする
    --prompt <PROMPT>     カスタムプロンプト文字列 [default: "λust> "]
    -h, --help            ヘルプ情報を表示
    -V, --version         バージョン情報を表示
```

## 実装されている機能

### R7RS Small準拠機能

- **基本データ型**: 数値、文字列、文字、シンボル、リスト、ベクタ
- **算術演算**: `+`, `-`, `*`, `/`, `modulo`, `remainder`, `abs`, `sqrt`等
- **比較演算**: `=`, `<`, `>`, `<=`, `>=`, `eq?`, `eqv?`, `equal?`
- **リスト操作**: `car`, `cdr`, `cons`, `list`, `append`, `reverse`, `length`
- **文字列操作**: `string=?`, `string-length`, `substring`, `string-append`
- **制御構造**: `if`, `cond`, `case`, `and`, `or`, `when`, `unless`
- **変数定義**: `define`, `set!`
- **関数定義**: `lambda`, `define`
- **高階関数**: `map`, `for-each`, `apply`

### SRFI実装

- **SRFI 1**: List Library（基本機能）
- **SRFI 9**: Define-record-type
- **SRFI 13**: String Libraries（基本機能）
- **SRFI 45**: Lazy evaluation（`delay`, `force`, `lazy`）
- **SRFI 46**: Basic syntax-rules extensions
- **SRFI 69**: Basic Hash Tables（基本機能）

### 拡張機能

- **例外処理**: `raise`, `with-exception-handler`, `guard`
- **継続**: `call/cc`, `call-with-current-continuation`
- **多値**: `values`, `call-with-values`
- **遅延評価**: `promise?`, プロミス操作
- **マクロシステム**: `syntax-rules`, `define-syntax`

## 使用例

### 基本的な計算

```scheme
λust> (+ (* 2 3) (/ 8 2))
10

λust> (sqrt 16)
4

λust> (expt 2 10)
1024
```

### リスト操作

```scheme
λust> (define lst '(1 2 3 4 5))

λust> (car lst)
1

λust> (cdr lst)
(2 3 4 5)

λust> (append lst '(6 7 8))
(1 2 3 4 5 6 7 8)

λust> (map (lambda (x) (* x 2)) lst)
(2 4 6 8 10)
```

### 関数定義

```scheme
λust> (define (fibonacci n)
   ...   (cond
   ...     ((= n 0) 0)
   ...     ((= n 1) 1)
   ...     (else (+ (fibonacci (- n 1))
   ...              (fibonacci (- n 2))))))

λust> (fibonacci 10)
55
```

### レコード型（SRFI 9）

```scheme
λust> (define-record-type person
   ...   (make-person name age)
   ...   person?
   ...   (name person-name)
   ...   (age person-age))

λust> (define alice (make-person "Alice" 30))

λust> (person-name alice)
"Alice"

λust> (person? alice)
#t
```

### ハッシュテーブル（SRFI 69）

```scheme
λust> (define ht (make-hash-table))

λust> (hash-table-set! ht "key1" 42)

λust> (hash-table-ref ht "key1")
42

λust> (hash-table-size ht)
1
```

## エラーハンドリング

REPLは詳細なエラー情報を提供します：

```scheme
λust> (+ 1 "hello")
Type error: Invalid argument types for arithmetic operation

λust> (car 42)
Type error: car: argument must be a pair

λust> (undefined-function)
Runtime error: Undefined variable: undefined-function
```

## 設定ファイル

REPLは以下の設定をサポートします：

- **履歴ファイル**: `.lambdust_history`（ホームディレクトリ）
- **設定ファイル**: `.lambdustrc`（将来実装予定）

## 開発者向け情報

### REPLの拡張

REPLは`src/bin/repl.rs`で実装されており、以下のコンポーネントで構成されています：

- `Repl` - メインREPLロジック
- `ReplConfig` - 設定管理
- 特別なコマンド処理
- 式の完全性チェック
- エラーハンドリング

### カスタムコマンドの追加

新しいREPLコマンドは`handle_special_command`メソッドに追加できます：

```rust
match trimmed {
    "(my-command)" => {
        // カスタムコマンドの実装
        println!("Custom command executed");
        Some(true)
    }
    // ...
}
```

## トラブルシューティング

### よくある問題

1. **REPLが起動しない**
   - `repl`フィーチャーが有効になっていることを確認
   - `cargo build --features repl --bin lambdust-repl`

2. **履歴が保存されない**
   - ホームディレクトリの書き込み権限を確認
   - `--no-history`オプションを使用していないことを確認

3. **式が評価されない**
   - 括弧の対応を確認
   - 複数行入力の場合、継続プロンプト`...`が表示されているか確認

### デバッグ情報の有効化

```bash
# デバッグビルドでより詳細な情報を取得
cargo build --features repl --bin lambdust

# ログレベルを設定（将来実装予定）
RUST_LOG=debug ./target/debug/lambdust
```

## 新機能

### タブ補完機能 ✅

REPLでTabキーを押すことで、以下が補完されます：

- **組み込み関数**: `+`, `-`, `*`, `/`, `car`, `cdr`, `map`, `filter` など
- **特殊フォーム**: `define`, `lambda`, `if`, `cond`, `let` など
- **ユーザー定義関数**: `(define (my-func ...) ...)` で定義した関数
- **ホスト関数**: 外部から登録された関数
- **ファイル名**: パスが含まれる場合のファイル補完

補完候補は色分けされて表示されます：
- 組み込み関数: 通常表示
- 特殊フォーム: "(special form)" 付きで表示

### シンタックスハイライト ✅

入力中の構文が自動的に色分けされます：

- **特殊フォーム**: 赤色 (`define`, `lambda`, `if` など)
- **組み込み関数**: 緑色 (`+`, `car`, `map` など)
- **数値**: 水色 (`42`, `3.14` など)
- **文字列**: 黄色 (`"hello"` など)
- **コメント**: 灰色 (`;; comment` など)
- **括弧**: 深度に応じて色分け (白→シアン→マゼンタ→青)

### デバッガー統合 ✅

基本的なデバッグ機能が利用できます：

#### デバッグモードの有効化
```scheme
λust> (debug on)
Debug mode enabled. Use (break) to set breakpoints, (step) to step through code.
```

#### ブレークポイントの設定
```scheme
λust[debug]> (break)
Breakpoint set for next evaluation.
λust[debug]> (factorial 5)
[DEBUG] Breaking at: (factorial 5)
[DEBUG] Use (continue) to proceed, (backtrace) to see stack
```

#### 呼び出しスタックの表示
```scheme
λust[debug]> (backtrace)
[BACKTRACE] Call stack:
  0: factorial(...)
  1: *(...)
[CURRENT] (factorial 5)
```

#### 実行の継続
```scheme
λust[debug]> (continue)
Continuing execution...
120
```

### 環境表示機能 ✅

現在の環境で利用可能な関数・変数を表示：

```scheme
λust> (env)
[ENVIRONMENT] Available functions and variables:
Builtin Functions:
+           -           *           /           abs         car        
cdr         cons        list        map         filter      ...

Special Forms:
define      lambda      if          cond        let         begin      
do          guard       when        unless      ...

User-defined Functions:
factorial   square      my-func     ...
```

## 今後の予定

- [x] タブ補完機能
- [x] シンタックスハイライト
- [x] デバッガー統合
- [ ] プロファイラー機能
- [ ] 設定ファイルサポート
- [ ] プラグインシステム
- [ ] ネットワークREPL（nREPL風）
- [ ] より高度なデバッグ機能（ステップイン・ステップアウト）
- [ ] ウォッチ変数機能