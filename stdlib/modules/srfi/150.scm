;; SRFI-150: Minimal Test

(define-library (srfi 150)
  (import (scheme base))
  (export test-fn)
  (begin
    (define (test-fn) 42)))