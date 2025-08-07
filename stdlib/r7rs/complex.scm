;; R7RS Standard Library - Complex Numbers Module
;; Provides complex number operations

(define-library (scheme complex)
  (export 
    ;; Complex number constructors
    make-rectangular make-polar
    
    ;; Complex number accessors
    real-part imag-part magnitude angle
    
    ;; Complex number predicates
    complex? real? rational? integer?
    
    ;; Arithmetic operations
    + - * / abs
    
    ;; Transcendental functions
    exp log sin cos tan asin acos atan
    sqrt expt
    
    ;; Additional complex operations
    conjugate phase
    
    ;; Conversion functions
    exact inexact exact->inexact inexact->exact)

  (begin

  ;; ============= Complex Number Constructors =============

  (define (make-rectangular real-part imag-part)
    "Constructs a complex number from real and imaginary parts."
    (builtin:make-rectangular real-part imag-part))

  (define (make-polar magnitude angle)
    "Constructs a complex number from magnitude and angle."
    (builtin:make-polar magnitude angle))

  ;; ============= Complex Number Accessors =============

  (define (real-part z)
    "Returns the real part of complex number z."
    (builtin:real-part z))

  (define (imag-part z)
    "Returns the imaginary part of complex number z."
    (builtin:imag-part z))

  (define (magnitude z)
    "Returns the magnitude of complex number z."
    (builtin:magnitude z))

  (define (angle z)
    "Returns the angle of complex number z."
    (builtin:angle z))

  ;; ============= Complex Number Predicates =============

  (define (complex? obj)
    "Returns #t if obj is a complex number."
    (builtin:complex? obj))

  (define (real? obj)
    "Returns #t if obj is a real number."
    (builtin:real? obj))

  (define (rational? obj)
    "Returns #t if obj is a rational number."
    (builtin:rational? obj))

  (define (integer? obj)
    "Returns #t if obj is an integer."
    (builtin:integer? obj))

  ;; ============= Arithmetic Operations =============

  (define (+ . numbers)
    "Addition of complex numbers."
    (if (null? numbers)
        0
        (builtin:complex-add-list numbers)))

  (define (- number . numbers)
    "Subtraction of complex numbers."
    (if (null? numbers)
        (builtin:complex-negate number)
        (builtin:complex-subtract number (apply + numbers))))

  (define (* . numbers)
    "Multiplication of complex numbers."
    (if (null? numbers)
        1
        (builtin:complex-multiply-list numbers)))

  (define (/ number . numbers)
    "Division of complex numbers."
    (if (null? numbers)
        (builtin:complex-reciprocal number)
        (builtin:complex-divide number (apply * numbers))))

  (define (abs z)
    "Absolute value (magnitude) of complex number."
    (magnitude z))

  ;; ============= Transcendental Functions =============

  (define (exp z)
    "Exponential function."
    (if (real? z)
        (builtin:real-exp z)
        (let ((r (real-part z))
              (i (imag-part z)))
          (make-polar (builtin:real-exp r) i))))

  (define (log z . base)
    "Natural logarithm or logarithm with specified base."
    (let ((ln-z (if (real? z)
                    (builtin:real-log z)
                    (make-rectangular (builtin:real-log (magnitude z))
                                     (angle z)))))
      (if (null? base)
          ln-z
          (/ ln-z (log (car base))))))

  (define (sin z)
    "Sine function."
    (if (real? z)
        (builtin:real-sin z)
        (let ((iz (* +i z)))
          (/ (- (exp iz) (exp (- iz))) (* 2 +i)))))

  (define (cos z)
    "Cosine function."
    (if (real? z)
        (builtin:real-cos z)
        (let ((iz (* +i z)))
          (/ (+ (exp iz) (exp (- iz))) 2))))

  (define (tan z)
    "Tangent function."
    (/ (sin z) (cos z)))

  (define (asin z)
    "Arcsine function."
    (if (real? z)
        (builtin:real-asin z)
        (* (- +i) (log (+ (* +i z) (sqrt (- 1 (* z z))))))))

  (define (acos z)
    "Arccosine function."
    (if (real? z)
        (builtin:real-acos z)
        (* (- +i) (log (+ z (* +i (sqrt (- 1 (* z z)))))))))

  (define (atan z . y)
    "Arctangent function."
    (if (null? y)
        (if (real? z)
            (builtin:real-atan z)
            (let ((iz (* +i z)))
              (/ (- (log (+ 1 iz)) (log (- 1 iz))) (* 2 +i))))
        (builtin:real-atan2 z (car y))))

  (define (sqrt z)
    "Square root function."
    (if (real? z)
        (if (negative? z)
            (make-rectangular 0 (builtin:real-sqrt (- z)))
            (builtin:real-sqrt z))
        (make-polar (builtin:real-sqrt (magnitude z))
                    (/ (angle z) 2))))

  (define (expt z1 z2)
    "Exponentiation function."
    (cond
      ((and (real? z1) (real? z2))
       (builtin:real-expt z1 z2))
      ((zero? z1)
       (if (zero? z2)
           1  ; 0^0 = 1 by convention
           0))
      (else
       (exp (* z2 (log z1))))))

  ;; ============= Additional Complex Operations =============

  (define (conjugate z)
    "Complex conjugate of z."
    (make-rectangular (real-part z) (- (imag-part z))))

  (define (phase z)
    "Phase (angle) of complex number z."
    (angle z))

  ;; ============= Conversion Functions =============

  (define (exact z)
    "Returns exact representation of z."
    (if (complex? z)
        (make-rectangular (exact (real-part z))
                         (exact (imag-part z)))
        (builtin:exact z)))

  (define (inexact z)
    "Returns inexact representation of z."
    (if (complex? z)
        (make-rectangular (inexact (real-part z))
                         (inexact (imag-part z)))
        (builtin:inexact z)))

  (define (exact->inexact z)
    "Converts exact number to inexact."
    (inexact z))

  (define (inexact->exact z)
    "Converts inexact number to exact."
    (exact z))

  ;; ============= Complex Constants =============

  (define +i (make-rectangular 0 1))
  (define -i (make-rectangular 0 -1))
  (define +1+i (make-rectangular 1 1))
  (define +1-i (make-rectangular 1 -1))
  (define -1+i (make-rectangular -1 1))
  (define -1-i (make-rectangular -1 -1))

  ;; ============= Utility Functions =============

  (define (zero? z)
    "Returns #t if z is zero."
    (and (zero? (real-part z))
         (zero? (imag-part z))))

  (define (positive? z)
    "Returns #t if z is positive real number."
    (and (real? z) (> z 0)))

  (define (negative? z)
    "Returns #t if z is negative real number."
    (and (real? z) (< z 0)))

  (define (finite? z)
    "Returns #t if z is finite."
    (and (finite? (real-part z))
         (finite? (imag-part z))))

  (define (infinite? z)
    "Returns #t if z is infinite."
    (or (infinite? (real-part z))
        (infinite? (imag-part z))))

  (define (nan? z)
    "Returns #t if z contains NaN."
    (or (nan? (real-part z))
        (nan? (imag-part z))))

  ;; ============= Complex Number Formatting =============

  (define (complex->string z . radix)
    "Converts complex number to string representation."
    (let ((r (if (null? radix) 10 (car radix))))
      (if (zero? (imag-part z))
          (number->string (real-part z) r)
          (let ((real-str (number->string (real-part z) r))
                (imag-val (imag-part z)))
            (cond
              ((zero? (real-part z))
               (if (= imag-val 1)
                   "+i"
                   (if (= imag-val -1)
                       "-i"
                       (string-append (number->string imag-val r) "i"))))
              ((positive? imag-val)
               (if (= imag-val 1)
                   (string-append real-str "+i")
                   (string-append real-str "+" (number->string imag-val r) "i")))
              (else
               (if (= imag-val -1)
                   (string-append real-str "-i")
                   (string-append real-str (number->string imag-val r) "i"))))))))

  ;; ============= Hyperbolic Functions =============

  (define (sinh z)
    "Hyperbolic sine function."
    (/ (- (exp z) (exp (- z))) 2))

  (define (cosh z)
    "Hyperbolic cosine function."
    (/ (+ (exp z) (exp (- z))) 2))

  (define (tanh z)
    "Hyperbolic tangent function."
    (/ (sinh z) (cosh z)))

  (define (asinh z)
    "Inverse hyperbolic sine function."
    (log (+ z (sqrt (+ (* z z) 1)))))

  (define (acosh z)
    "Inverse hyperbolic cosine function."
    (log (+ z (sqrt (- (* z z) 1)))))

  (define (atanh z)
    "Inverse hyperbolic tangent function."
    (/ (- (log (+ 1 z)) (log (- 1 z))) 2)))))