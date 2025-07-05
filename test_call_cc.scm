;; Test call/cc deep nesting escape
(define (test-deep-escape)
  (+ 1 (* 2 (call/cc (lambda (escape)
                       (+ 10 (* 20 (escape 42)) 30)
                       100)))))

;; Test call/cc with multiple continuations
(define (test-multiple-continuations)
  (let ((cont1 #f)
        (cont2 #f))
    (+ (call/cc (lambda (k) (set! cont1 k) 1))
       (call/cc (lambda (k) (set! cont2 k) 2))
       (call/cc (lambda (k) (k 3))))))

;; Test call/cc with recursive function
(define (test-recursive-escape n)
  (if (= n 0)
      (call/cc (lambda (escape) 
                 (escape 999)))
      (+ n (test-recursive-escape (- n 1)))))

;; Test complex nested escape
(define (test-complex-nested)
  (call/cc (lambda (outer-escape)
             (+ 1 (call/cc (lambda (inner-escape)
                             (+ 10 (outer-escape 42) 20)))
                2))))