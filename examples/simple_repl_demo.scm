;; 簡単なREPLデモンストレーション用Schemeコード

;; 基本的な算術演算
(define demo-arithmetic
  (lambda ()
    (display "=== 算術演算デモ ===") (newline)
    (display "(+ 1 2 3 4 5) = ") (display (+ 1 2 3 4 5)) (newline)
    (display "(* 2 3 4) = ") (display (* 2 3 4)) (newline)
    (display "(sqrt 16) = ") (display (sqrt 16)) (newline)))

;; リスト操作
(define demo-lists
  (lambda ()
    (display "=== リスト操作デモ ===") (newline)
    (define lst '(1 2 3 4 5))
    (display "lst = ") (display lst) (newline)
    (display "(length lst) = ") (display (length lst)) (newline)
    (display "(reverse lst) = ") (display (reverse lst)) (newline)
    (display "(append lst '(6 7 8)) = ") (display (append lst '(6 7 8))) (newline)))

;; 高階関数
(define demo-higher-order
  (lambda ()
    (display "=== 高階関数デモ ===") (newline)
    (define square (lambda (x) (* x x)))
    (define numbers '(1 2 3 4 5))
    (display "numbers = ") (display numbers) (newline)
    (display "(map square numbers) = ") (display (map square numbers)) (newline)))

;; 再帰関数
(define factorial
  (lambda (n)
    (if (= n 0)
        1
        (* n (factorial (- n 1))))))

(define demo-recursion
  (lambda ()
    (display "=== 再帰関数デモ ===") (newline)
    (display "(factorial 5) = ") (display (factorial 5)) (newline)
    (display "(factorial 10) = ") (display (factorial 10)) (newline)))

;; 文字列操作
(define demo-strings
  (lambda ()
    (display "=== 文字列操作デモ ===") (newline)
    (define str "Hello, World!")
    (display "str = ") (display str) (newline)
    (display "(string-length str) = ") (display (string-length str)) (newline)))

;; 全デモ実行
(define run-all-demos
  (lambda ()
    (display "=== Lambdust REPL デモンストレーション ===") (newline) (newline)
    (demo-arithmetic) (newline)
    (demo-lists) (newline)
    (demo-higher-order) (newline)
    (demo-recursion) (newline)
    (demo-strings) (newline)
    (display "=== デモ完了 ===") (newline)))

;; 初期化メッセージ
(display "REPLデモファイルが読み込まれました!") (newline)
(display "(run-all-demos) を実行してデモを開始してください.") (newline)