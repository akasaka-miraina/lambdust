;; Debug call/cc nested escape behavior
;; Expected behavior:
;; (k 42) should completely escape and return 42 directly
;; Not: (* 2 42) = 84

(* 2 (+ 1 (call/cc (lambda (k) (* 3 (k 42))))))