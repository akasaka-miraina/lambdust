# 型システムガイド

Lambdustは、動的型付けから完全な静的型付けへのシームレスな移行を可能にする洗練された4レベル漸進的型付けシステムを実装しています。このガイドでは、完全な型システムの実装と使用パターンについて説明します。

## 概要

Lambdust型システムは以下を提供します：

- **漸進的型付け**: 動的型付けと静的型付けの間のスムーズな移行
- **型推論**: 拡張を含む高度なHindley-Milner型推論
- **代数的データ型**: パターンマッチングを伴う直和型と直積型
- **型クラス**: Haskellスタイルの型制約とポリモーフィズム
- **依存型**: 上級ユーザー向けの限定的な依存型機能
- **エフェクト型**: 代数エフェクトシステムとの統合

## 型システムレベル

### レベル1: 動的型付け

ランタイム型チェックを伴う純粋な動的型付け：

```scheme
;; 型注釈なし - 完全に動的
(define (add x y)
  (+ x y))

(add 1 2)        ;; => 3
(add 1.5 2.3)    ;; => 3.8
(add "hello" " world")  ;; 実行時エラー: + は数値を期待
```

### レベル2: オプション型付け

文書化と基本チェックのためのオプション型注釈：

```scheme
;; オプション型ヒント
(define (add x : Number y : Number) : Number
  (+ x y))

(define (greet name : String) : String
  (string-append "こんにちは、" name "！"))

;; 型ヒントはチェックされるが強制されない
(add 1 2)        ;; => 3
(add 1 "2")      ;; 警告: 型の不一致、ただし実行は継続
```

### レベル3: 漸進的型付け

漸進的強制を伴う静的・動的型付けの混在：

```scheme
;; 静的型付け関数
(define (safe-add x : Number y : Number) : Number
  (+ x y))

;; 動的型付け関数
(define (flexible-add x y)
  (+ x y))

;; 漸進的相互作用
(define (mixed-computation data)
  (let ([typed-result : Number (safe-add 1 2)]
        [dynamic-result (flexible-add 3 4)])
    (+ typed-result dynamic-result)))

;; 型境界が強制される
(safe-add 1 "2")  ;; 型エラー: 引数2はNumberでなければならない
```

### レベル4: 静的型付け

コンパイル時検証を伴う完全な静的型付け：

```scheme
#:type-level static

;; すべての関数は完全に型付けされている必要がある
(define (factorial n : Natural) : Natural
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

;; 型チェックがランタイムエラーを防ぐ
(define (process-data data : (List Number)) : Number
  (fold + 0 data))

;; コンパイル時エラー防止
(process-data '(1 2 "3"))  ;; コンパイルエラー: "3"はNumberではない
```

## 型構文

### 基本型

```scheme
;; プリミティブ型
Number          ;; 浮動小数点数
Integer         ;; 整数
String          ;; テキスト文字列
Boolean         ;; #t または #f
Character       ;; 単一文字
Symbol          ;; Schemeシンボル

;; コンテナ型
(List Number)           ;; 数値のリスト
(Vector String)         ;; 文字列のベクタ
(Pair Number String)    ;; 型付きコンポーネントを持つペア

;; 関数型
(Number Number -> Number)       ;; 2つの数値から数値へ
(String -> Boolean)             ;; 文字列からブール値へ
((List a) -> Number)            ;; ジェネリックリストから数値へ
```

### ジェネリック型

```scheme
;; 型変数
(define (identity x : a) : a
  x)

(define (first lst : (List a)) : a
  (car lst))

;; 複数の型変数
(define (map f : (a -> b) lst : (List a)) : (List b)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;; 型制約
(define (sort lst : (List a)) : (List a)
  (where (Ord a)
    (quick-sort lst)))
```

## 代数的データ型

### 直和型（バリアント）

```scheme
;; 直和型の定義
(define-type Color
  (Red)
  (Green)
  (Blue)
  (RGB Number Number Number)
  (HSV Number Number Number))

;; コンストラクタ関数は自動的に作成される
(define red-color (Red))
(define custom-color (RGB 255 128 0))

;; パターンマッチング
(define (color-to-hex color : Color) : String
  (match color
    [(Red) "#FF0000"]
    [(Green) "#00FF00"] 
    [(Blue) "#0000FF"]
    [(RGB r g b) (format "#~2,'0X~2,'0X~2,'0X" 
                        (inexact->exact r)
                        (inexact->exact g)
                        (inexact->exact b))]
    [(HSV h s v) (rgb-to-hex (hsv->rgb h s v))]))
```

### 直積型（レコード）

```scheme
;; 直積型の定義
(define-type Person
  (make-person name : String
               age : Number
               email : String))

;; 使用法
(define john (make-person "田中太郎" 30 "tanaka@example.com"))

;; フィールドアクセサ（自動生成される）
(person-name john)    ;; => "田中太郎"
(person-age john)     ;; => 30
(person-email john)   ;; => "tanaka@example.com"

;; レコードを使ったパターンマッチング
(define (adult? person : Person) : Boolean
  (match person
    [(make-person _ age _) (>= age 18)]))
```

### 再帰型

```scheme
;; 二分木
(define-type (Tree a)
  (Empty)
  (Node (Tree a) a (Tree a)))

;; リスト定義
(define-type (MyList a)
  (Nil)
  (Cons a (MyList a)))

;; 使用法
(define int-tree : (Tree Number)
  (Node (Node (Empty) 1 (Empty))
        2
        (Node (Empty) 3 (Empty))))

(define (tree-sum tree : (Tree Number)) : Number
  (match tree
    [(Empty) 0]
    [(Node left value right)
     (+ value (tree-sum left) (tree-sum right))]))
```

## 型クラス

型クラスは、Haskellの型クラスに類似した構造化ポリモーフィズムを提供します：

### 型クラスの定義

```scheme
;; 基本型クラス
(define-type-class (Eq a)
  (equal? : a a -> Boolean)
  (not-equal? : a a -> Boolean))

;; デフォルト実装
(define-type-class (Eq a) 
  (equal? : a a -> Boolean)
  (not-equal? : a a -> Boolean)
  
  ;; not-equal?のデフォルト実装
  (default not-equal? (lambda (x y) (not (equal? x y)))))

;; 依存関係を持つ型クラス
(define-type-class (Ord a)
  (super (Eq a))  ;; OrdはEqを必要とする
  (compare : a a -> Ordering)
  (< : a a -> Boolean)
  (<= : a a -> Boolean)
  (> : a a -> Boolean)
  (>= : a a -> Boolean))
```

### 型クラスインスタンス

```scheme
;; NumberのEqを実装
(define-instance (Eq Number)
  (define (equal? x y) (= x y)))

;; NumberのOrdを実装
(define-instance (Ord Number)
  (define (compare x y)
    (cond [(< x y) 'LT]
          [(> x y) 'GT]
          [else 'EQ]))
  (define (< x y) (< x y))
  (define (<= x y) (<= x y))
  (define (> x y) (> x y))
  (define (>= x y) (>= x y)))

;; 型クラスを使ったジェネリック関数
(define (sort lst : (List a)) : (List a)
  (where (Ord a)
    (merge-sort lst)))

;; 使用法
(sort '(3 1 4 1 5 9))  ;; NumberがOrdを実装しているので動作する
```

### 高度な型クラス

```scheme
;; Functor型クラス
(define-type-class (Functor f)
  (fmap : (a -> b) (f a) -> (f b)))

;; Monad型クラス
(define-type-class (Monad m)
  (super (Functor m))
  (return : a -> (m a))
  (bind : (m a) (a -> (m b)) -> (m b)))

;; FunctorとMonadインスタンスを持つMaybe型
(define-type (Maybe a)
  (Nothing)
  (Just a))

(define-instance (Functor Maybe)
  (define (fmap f maybe)
    (match maybe
      [(Nothing) (Nothing)]
      [(Just x) (Just (f x))])))

(define-instance (Monad Maybe)
  (define (return x) (Just x))
  (define (bind maybe f)
    (match maybe
      [(Nothing) (Nothing)]
      [(Just x) (f x)])))
```

## 型推論

### Hindley-Milner推論

型システムには洗練された型推論が含まれています：

```scheme
;; 型注釈不要 - 型が推論される
(define (compose f g)
  (lambda (x) (f (g x))))
;; 推論された型: (b -> c) (a -> b) -> (a -> c)

(define (map f lst)
  (if (null? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))
;; 推論された型: (a -> b) (List a) -> (List b)

;; 複雑な推論
(define (fold f init lst)
  (if (null? lst)
      init
      (fold f (f init (car lst)) (cdr lst))))
;; 推論された型: (b a -> b) b (List a) -> b
```

### 型制約

```scheme
;; 制約付き型推論
(define (sort-by f lst)
  (sort (map f lst)))
;; 推論された型: (a -> b) (List a) -> (List b) where (Ord b)

(define (unique lst)
  (remove-duplicates lst))
;; 推論された型: (List a) -> (List a) where (Eq a)
```

## エフェクトシステムとの統合

型はエフェクトと組み合わせて正確な追跡が可能です：

```scheme
;; エフェクト型付き関数
(define (read-file filename : String) : (IO String)
  (with-file-input filename
    (lambda (port)
      (read-string #f port))))

(define (write-log message : String) : (IO Unit)
  (with-file-output "app.log"
    (lambda (port)
      (write-line message port))))

;; エフェクトと型の組み合わせ
(define (process-data filename : String) : (IO (Either Error (List Number)))
  (do [content (read-file filename)]
      [parsed (parse-numbers content)]
      [processed (map square parsed)]
      [_ (write-log (format "~a個の数値を処理しました" (length processed)))]
      (return (Right processed))))
```

## 型を使ったエラーハンドリング

### Result型

```scheme
(define-type (Result a e)
  (Ok a)
  (Error e))

(define (safe-divide x : Number y : Number) : (Result Number String)
  (if (= y 0)
      (Error "ゼロで割ろうとしました")
      (Ok (/ x y))))

;; モナディックエラーハンドリング
(define-instance (Monad (Result e))
  (define (return x) (Ok x))
  (define (bind result f)
    (match result
      [(Error e) (Error e)]
      [(Ok x) (f x)])))

;; 使用法
(define computation
  (do [x (safe-divide 10 2)]    ;; Ok 5
      [y (safe-divide x 0)]     ;; Error "ゼロで割ろうとしました"
      [z (safe-divide y 3)]     ;; エラーのためスキップ
      (return z)))
```

## パフォーマンスに関する考慮事項

### 型特殊化

型システムはパフォーマンス最適化を可能にします：

```scheme
;; ジェネリック関数
(define (sum lst : (List a)) : a
  (where (Num a)
    (fold + (zero) lst)))

;; 特殊化バージョンが自動生成される
;; sum-number : (List Number) -> Number    ; 数値用に最適化
;; sum-complex : (List Complex) -> Complex ; 複素数用に最適化
```

### コンパイル時最適化

```scheme
#:optimize-types #t

;; 型情報により以下が可能になる:
;; - 型特定操作のインライン化
;; - ランタイム型チェックの除去
;; - 数値型のSIMD最適化
;; - メモリレイアウト最適化

(define (vector-add v1 : (Vector Number) v2 : (Vector Number)) : (Vector Number)
  ;; 最適化されたSIMD操作にコンパイルされる
  (vector-map + v1 v2))
```

## 高度な機能

### 依存型（限定的）

```scheme
;; 長さインデックス付きベクタ
(define-type (Vec n a)
  (make-vec (vector a) (= (vector-length vector) n)))

(define (safe-head vec : (Vec (> n 0) a)) : a
  (vector-ref (vec-data vec) 0))

;; 制限型
(define-type Positive (and Number (> x 0)))
(define-type NonEmptyString (and String (> (string-length x) 0)))

(define (sqrt-positive x : Positive) : Positive
  (sqrt x))  ;; 型システムがx > 0かつ結果 > 0を保証
```

### 型レベルプログラミング

```scheme
;; 型レベル計算
(define-type-function (Replicate n a)
  (if (= n 0)
      '()
      (cons a (Replicate (- n 1) a))))

;; 使用法
(define tuple : (Replicate 3 Number)
  '(1 2 3))  ;; 型: (Number Number Number)
```

## 設定

### 型システム設定

```scheme
;; グローバル型システム設定
(set-type-level! 'gradual)          ;; デフォルト型レベルを設定
(set-type-inference! #t)            ;; 型推論を有効化
(set-type-optimization! #t)         ;; 型ベース最適化を有効化
(set-type-warnings! 'strict)        ;; 型不一致の警告レベル

;; モジュール固有設定
#:type-level static                 ;; このモジュールは静的型付けを使用
#:type-inference aggressive         ;; 積極的推論を使用
#:type-checking strict             ;; 厳密な型チェック
```

## 例

### 完全な型システム使用例

```scheme
#!/usr/bin/env lambdust
#:type-level gradual

(import (scheme base)
        (lambdust types)
        (lambdust effects))

;; 型を使った完全なデータ処理パイプラインを定義

;; カスタムデータ型
(define-type (Employee)
  (make-employee name : String
                 id : Integer
                 salary : Number
                 department : String))

(define-type Department
  (Engineering)
  (Sales)  
  (Marketing)
  (HR))

;; ドメイン用型クラス
(define-type-class (Payroll a)
  (calculate-pay : a -> Number)
  (tax-rate : a -> Number))

(define-instance (Payroll Employee)
  (define (calculate-pay emp)
    (* (employee-salary emp) 0.8))  ;; 控除後
  (define (tax-rate emp)
    (cond [(> (employee-salary emp) 10000000) 0.3]  ;; 1000万円以上
          [(> (employee-salary emp) 5000000) 0.25]   ;; 500万円以上
          [else 0.2])))

;; 型を持つエフェクト計算
(define (process-payroll employees : (List Employee)) : (IO (List Number))
  (do [_ (log-info "給与処理を開始します")]
      [payments (map calculate-pay employees)]
      [total (sum payments)]
      [_ (log-info (format "総給与額: ￥~a" total))]
      [_ (write-payroll-report employees payments)]
      (return payments)))

;; エラーハンドリングを伴う安全な計算
(define (load-employee-data filename : String) : (IO (Result (List Employee) String))
  (guard (condition
          [(file-not-found? condition)
           (return (Error "従業員データファイルが見つかりません"))]
          [(parse-error? condition) 
           (return (Error "従業員データの形式が無効です"))])
    (do [content (read-file filename)]
        [employees (parse-employees content)]
        (return (Ok employees)))))

;; メインプログラム
(define (main)
  (do [result (load-employee-data "employees.json")]
      (match result
        [(Error msg) 
         (log-error msg)
         (exit 1)]
        [(Ok employees)
         (do [payments (process-payroll employees)]
             [_ (log-info "給与処理が完了しました")]
             (return payments))])))

(when (script-file?)
  (main))
```

この型システムは、Lispの言語の力を保持しながら、信頼性が高く効率的で保守しやすいSchemeプログラムを構築するための堅固な基盤を提供します。