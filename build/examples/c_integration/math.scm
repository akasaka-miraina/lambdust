;; Mathematical functions library for calculator example

;; Basic mathematical constants
(define pi 3.141592653589793)
(define e 2.718281828459045)

;; Extended mathematical functions
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(define (fibonacci n)
  (cond ((<= n 0) 0)
        ((= n 1) 1)
        (else (+ (fibonacci (- n 1))
                 (fibonacci (- n 2))))))

(define (gcd a b)
  (if (= b 0)
      a
      (gcd b (remainder a b))))

(define (lcm a b)
  (/ (* a b) (gcd a b)))

(define (power-of-2? n)
  (and (> n 0)
       (= (remainder n 2) 0)
       (or (= n 2)
           (power-of-2? (/ n 2)))))

(define (sum-of-squares a b)
  (+ (* a a) (* b b)))

(define (distance x1 y1 x2 y2)
  (sqrt (+ (expt (- x2 x1) 2)
           (expt (- y2 y1) 2))))

;; Statistical functions
(define (average . numbers)
  (/ (apply + numbers) (length numbers)))

(define (variance . numbers)
  (let ((avg (apply average numbers)))
    (/ (apply + (map (lambda (x) (expt (- x avg) 2)) numbers))
       (length numbers))))

(define (standard-deviation . numbers)
  (sqrt (apply variance numbers)))

;; Number theory functions
(define (prime? n)
  (cond ((<= n 1) #f)
        ((= n 2) #t)
        ((= (remainder n 2) 0) #f)
        (else (prime-helper n 3))))

(define (prime-helper n divisor)
  (cond ((> (* divisor divisor) n) #t)
        ((= (remainder n divisor) 0) #f)
        (else (prime-helper n (+ divisor 2)))))

(define (next-prime n)
  (if (prime? (+ n 1))
      (+ n 1)
      (next-prime (+ n 1))))

;; Utility message
(display "Mathematical functions library loaded")
(newline)