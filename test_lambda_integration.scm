;; Test lambda function integration with higher-order functions

;; Test map with lambda
(define test1 (map (lambda (x) (* x 2)) '(1 2 3 4)))
(display "map with lambda: ")
(display test1)
(newline)

;; Test apply with lambda
(define test2 (apply (lambda (x y) (+ x y)) '(10 20)))
(display "apply with lambda: ")
(display test2)
(newline)

;; Test fold with lambda
(define test3 (fold (lambda (acc x) (+ acc x)) 0 '(1 2 3 4)))
(display "fold with lambda: ")
(display test3)
(newline)

;; Test nested lambda functions
(define test4 (map (lambda (x) (+ x 1)) (map (lambda (x) (* x 2)) '(1 2 3))))
(display "nested lambda with map: ")
(display test4)
(newline)