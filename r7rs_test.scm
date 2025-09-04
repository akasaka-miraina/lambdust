;; R7RS compliance test for SafeOptimizedValue system

;; Test basic values and predicates
(define test1 (eq? #f #f))
(define test2 (eqv? 42 42))  
(define test3 (equal? "hello" "hello"))
(define test4 (null? '()))
(define test5 (pair? '(1 . 2)))

;; Test numeric operations
(define test6 (+ 1 2 3))
(define test7 (* 4 5))
(define test8 (= 42 42.0))

;; Test list operations
(define test9 (cons 1 '()))
(define test10 (car '(a b c)))
(define test11 (cdr '(a b c)))
(define test12 (length '(1 2 3 4 5)))

;; Test boolean semantics (only #f is falsy)
(define test13 (if 0 'truthy 'falsy))    ; 0 should be truthy
(define test14 (if '() 'truthy 'falsy))  ; '() should be truthy
(define test15 (if "" 'truthy 'falsy))   ; "" should be truthy
(define test16 (if #f 'truthy 'falsy))   ; #f should be falsy

;; Test symbol identity
(define test17 (eq? 'symbol 'symbol))
(define test18 (symbol? 'hello))

;; Display results
(display "R7RS SafeOptimizedValue Compliance Test Results:")
(newline)
(display "test1 (eq? #f #f): ") (display test1) (newline)
(display "test2 (eqv? 42 42): ") (display test2) (newline) 
(display "test3 (equal? strings): ") (display test3) (newline)
(display "test4 (null? '()): ") (display test4) (newline)
(display "test5 (pair? '(1 . 2)): ") (display test5) (newline)
(display "test6 (+ 1 2 3): ") (display test6) (newline)
(display "test7 (* 4 5): ") (display test7) (newline)
(display "test8 (= 42 42.0): ") (display test8) (newline)
(display "test9 (cons 1 '()): ") (display test9) (newline)
(display "test10 (car '(a b c)): ") (display test10) (newline)
(display "test11 (cdr '(a b c)): ") (display test11) (newline)
(display "test12 (length '(1 2 3 4 5)): ") (display test12) (newline)
(display "test13 (if 0): ") (display test13) (newline)
(display "test14 (if '()): ") (display test14) (newline)
(display "test15 (if \"\"): ") (display test15) (newline)
(display "test16 (if #f): ") (display test16) (newline)
(display "test17 (eq? 'symbol 'symbol): ") (display test17) (newline)
(display "test18 (symbol? 'hello): ") (display test18) (newline)