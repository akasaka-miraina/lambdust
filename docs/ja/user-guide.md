# Lambdust 依存型ユーザーガイド

## はじめに

Lambdustの依存型システムについて説明する。このガイドは、Schemeプログラムで依存型を理解し使用するための情報を提供する。Lambdustは漸進的型付けシステムを提供し、動的型付けから完全な依存型まで、自分のペースで移行できる。

## 4段階の型付けレベル

Lambdustは4段階の型付けレベルをサポートし、コードに適した型安全性レベルを選択できる：

1. **動的型付け** - 実行時型チェック付きの従来のScheme
2. **契約型付け** - 型仕様付きの実行時契約
3. **静的型付け** - 依存型機能を持たないコンパイル時型チェック
4. **依存型付け** - コンパイル時検証付きの完全な依存型

### レベル1: 動的型付け

これは従来のSchemeである - 型注釈は不要：

```scheme
;; 動的型付け - 従来のSchemeと同じ
(define (add-numbers x y)
  (+ x y))

(define (process-list lst)
  (map (lambda (x) (* x 2)) lst))

;; 実行時に任意の型で動作
(add-numbers 5 10)        ; => 15
(add-numbers 3.5 2.1)     ; => 5.6
```

### レベル2: 契約型付け

実行時検証付きの型仕様：

```scheme
;; 契約による型仕様
(define/contract (factorial n)
  (-> natural-number/c natural-number/c)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

;; リスト処理の契約
(define/contract (process-numbers lst)
  (-> (listof number?) (listof number?))
  (map (lambda (x) (* x x)) lst))
```

### レベル3: 静的型付け

コンパイル時の型チェック：

```scheme
;; 静的型注釈
(define (add-integers (x : Integer) (y : Integer)) : Integer
  (+ x y))

;; 型付きリスト操作
(define (sum-list (numbers : (List Integer))) : Integer
  (fold + 0 numbers))

;; ジェネリック関数
(define (identity [T] (x : T)) : T
  x)
```

### レベル4: 依存型付け

完全な依存型システム：

```scheme
;; 長さ依存型
(define (safe-head [n : Nat] (lst : (Vec n Integer))) : Integer
  (if (> n 0)
      (vector-ref lst 0)
      (error "Empty vector")))

;; 証明付き関数
(define (bounded-add (x : (Range 0 100)) (y : (Range 0 100))) 
  : (Range 0 200)
  (+ x y))

;; インデックス安全配列アクセス
(define (safe-nth [n : Nat] [i : (Range 0 n)] (vec : (Vec n Integer))) 
  : Integer
  (vector-ref vec i))
```

## 基本的な依存型

### 自然数型

```scheme
;; 自然数（0以上の整数）
(define (repeat [n : Nat] [x : Integer]) : (Vec n Integer)
  (make-vector n x))

;; 正の数
(define (positive-sqrt (x : Pos)) : Pos
  (sqrt x))
```

### 範囲型

```scheme
;; 範囲制限
(define (percentage (x : (Range 0 100))) : String
  (string-append (number->string x) "%"))

;; 年齢型
(define Age (Range 0 150))
(define (can-vote? (age : Age)) : Boolean
  (>= age 18))
```

### 長さ依存型

```scheme
;; 固定長ベクトル
(define (dot-product [n : Nat] 
                    (v1 : (Vec n Real)) 
                    (v2 : (Vec n Real))) 
  : Real
  (fold + 0 (vector-map * v1 v2)))

;; 非空リスト
(define (head (lst : (NonEmpty (List Integer)))) : Integer
  (car lst))
```

## 型クラス

### Equatable

```scheme
;; 等価性判定可能な型
(define (remove-duplicates [T : Equatable] (lst : (List T))) : (List T)
  (remove-duplicates-helper lst '()))
```

### Ordered

```scheme
;; 順序付け可能な型
(define (quicksort [T : Ordered] (lst : (List T))) : (List T)
  (cond
    [(null? lst) '()]
    [(null? (cdr lst)) lst]
    [else (let ([pivot (car lst)]
              [rest (cdr lst)])
            (append (quicksort (filter (lambda (x) (< x pivot)) rest))
                   (list pivot)
                   (quicksort (filter (lambda (x) (>= x pivot)) rest))))]))
```

## 副作用システム

### 基本的な副作用

```scheme
;; IO副作用
(define (read-file (filename : String)) : (IO String)
  (call-with-input-file filename get-string-all))

;; State副作用
(define-effect (State s)
  (get () -> s)
  (put (new-state s) -> Unit))

;; 副作用ハンドラ
(with-handler state-handler
  (perform (put 42))
  (perform (get)))
```

### 副作用の組み合わせ

```scheme
;; 複数の副作用
(define (process-file (filename : String)) : (IO (State Int) String)
  (do [content <- (read-file filename)]
      [count <- (perform (get))]
      [(perform (put (+ count 1)))]
      (return (string-append content (number->string count)))))
```

## 証明システム

### 基本的な証明

```scheme
;; 等式証明
(define (add-zero-right [n : Nat]) : (= (+ n 0) n)
  (nat-induction n
    base: refl
    step: (lambda [k proof-k] (ap S proof-k))))

;; リスト長の証明
(define (append-length [A : Type] [xs : (List A)] [ys : (List A)]) 
  : (= (length (append xs ys)) (+ (length xs) (length ys)))
  (list-induction xs
    base: refl
    step: (lambda [x xs' ih] (ap S ih))))
```

### 安全な配列操作

```scheme
;; インデックス証明付き配列アクセス
(define (safe-array-ref [n : Nat] [i : Nat] 
                       (arr : (Array n Integer))
                       (proof : (< i n))) : Integer
  (array-ref arr i proof))

;; 境界チェック付き挿入
(define (bounded-insert [n : Nat] [i : Nat] (x : Integer)
                       (arr : (Array n Integer))
                       (proof : (<= i n))) : (Array (+ n 1) Integer)
  (array-insert arr i x proof))
```

## パフォーマンス最適化

### 特殊化

```scheme
;; 型特化による最適化
(define (sum-integers (lst : (List Integer))) : Integer
  ;; コンパイル時に整数特化コードを生成
  (fold + 0 lst))

(define (sum-reals (lst : (List Real))) : Real
  ;; コンパイル時に浮動小数点特化コードを生成
  (fold + 0.0 lst))
```

### SIMD最適化

```scheme
;; ベクトル演算の自動SIMD化
(define (vector-add [n : Nat] 
                   (v1 : (Vec n Real)) 
                   (v2 : (Vec n Real))) 
  : (Vec n Real)
  ;; 自動的にSIMD命令を使用
  (vector-map + v1 v2))
```

## 並行性

### Actor システム

```scheme
;; Actor定義
(define-actor counter-actor
  (state : Int)
  (messages:
    [(increment) (set-state (+ (get-state) 1))]
    [(get-count) (reply (get-state))]))

;; Actor使用
(define counter (spawn counter-actor 0))
(send counter 'increment)
(let ([count (call counter 'get-count)])
  (display count))
```

### Future/Promise

```scheme
;; 非同期計算
(define (parallel-computation (x : Integer) (y : Integer)) : (Future Integer)
  (future
    (+ (expensive-computation x)
       (expensive-computation y))))

;; 結果の待機
(let ([result (force (parallel-computation 10 20))])
  (display result))
```

## 外部関数インターフェース（FFI）

### C関数の呼び出し

```scheme
;; 安全なFFI
(foreign-call "libc" "strlen" 
  (-> CString -> Size)
  capability: read-only
  "Hello, World!")

;; 型安全なバインディング
(define-foreign strlen
  "libc" "strlen"
  (-> (Ptr Char) -> Size))
```

## マクロシステム

### 衛生的マクロ

```scheme
;; syntax-rulesマクロ
(define-syntax when
  (syntax-rules ()
    [(when condition body ...)
     (if condition
         (begin body ...)
         (void))]))

;; 型安全なマクロ
(define-typed-syntax (for/sum ([x : T] lst : (List T)) body : T) : T
  #'(fold + (cast 0 T) (map (lambda (x) body) lst)))
```

## デバッグとエラー処理

### 型エラーメッセージ

```
Error: Type mismatch
Expected: (Vec 5 Integer)
Got:      (Vec 3 Integer)
In expression: (safe-head 5 short-vector)
at line 42, column 15

Suggestion: Check that your vector has the correct length,
or use a dynamic length check with 'vector-length'.
```

### 実行時デバッグ

```scheme
;; デバッグ情報付きアサーション
(assert (> x 0) "x must be positive, got: ~a" x)

;; 型情報の実行時検査
(display-type-info x)  ; 型情報を表示
```

## まとめ

Lambdustの依存型システムは、従来のSchemeの柔軟性を保ちながら、強力な型安全性と性能を提供する。段階的移行により、既存のコードを破壊することなく、必要に応じて型安全性を向上させることができる。

詳細な情報については、以下を参照：
- [API リファレンス](api-reference.md)
- [型システム仕様](../architecture/dependent-type-theory.md)
- [パフォーマンスガイド](../development/performance-guide.md)