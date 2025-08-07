;; SRFI-1: List Library
;; 
;; This library provides comprehensive list processing procedures that extend
;; the basic list operations of R7RS. SRFI-1 is one of the most widely used
;; and fundamental SRFIs, providing essential functional programming tools.
;;
;; Reference: https://srfi.schemers.org/srfi-1/srfi-1.html

(define-library (srfi 1)
  (import (scheme base)
          (scheme case-lambda))
  
  (export
    ;; === Constructors ===
    cons list xcons cons* make-list list-tabulate list-copy circular-list iota
    
    ;; === Predicates ===
    proper-list? circular-list? dotted-list? not-pair? null-list? list=
    
    ;; === Selectors ===
    first second third fourth fifth sixth seventh eighth ninth tenth
    car+cdr take drop take-right drop-right take! drop-right!
    split-at split-at! last last-pair
    
    ;; === Miscellaneous ===
    length+
    
    ;; === Fold, unfold & map ===
    fold fold-right pair-fold pair-fold-right reduce reduce-right
    unfold unfold-right map for-each append-map append-map! 
    map! pair-for-each filter-map map-in-order
    
    ;; === Filtering & partitioning ===
    filter partition remove filter! partition! remove!
    
    ;; === Searching ===
    find find-tail any every list-index take-while drop-while
    take-while! span break span! break!
    
    ;; === Deleting ===
    delete delete-duplicates delete! delete-duplicates!
    
    ;; === Association lists ===
    assoc alist-cons alist-copy alist-delete alist-delete!
    
    ;; === Set operations ===
    lset<= lset= lset-adjoin lset-union lset-intersection 
    lset-difference lset-xor lset-diff+intersection
    lset-union! lset-intersection! lset-difference! lset-xor!
    lset-diff+intersection!
    
    ;; === Primitive side effects ===
    set-car! set-cdr!)

  (begin
    ;; ============= CONSTRUCTORS =============
    
    (define (xcons d a) (cons a d))
    
    (define cons*
      (case-lambda
        ((x) x)
        ((x y) (cons x y))
        ((x y z . rest)
         (cons x (apply cons* y z rest)))))
    
    (define (make-list n . maybe-fill)
      (let ((fill (if (null? maybe-fill) #f (car maybe-fill))))
        (do ((i n (- i 1))
             (result '() (cons fill result)))
            ((= i 0) result))))
    
    (define (list-tabulate n init-proc)
      (do ((i (- n 1) (- i 1))
           (result '() (cons (init-proc i) result)))
          ((< i 0) result)))
    
    (define (list-copy lst)
      (if (null? lst)
          '()
          (cons (car lst) (list-copy (cdr lst)))))
    
    (define (circular-list val1 . vals)
      (let ((result (cons val1 vals)))
        (let loop ((lst result))
          (if (null? (cdr lst))
              (set-cdr! lst result)
              (loop (cdr lst))))
        result))
    
    (define iota
      (case-lambda
        ((count) (iota count 0 1))
        ((count start) (iota count start 1))
        ((count start step)
         (do ((i (- count 1) (- i 1))
              (result '() (cons (+ start (* i step)) result)))
             ((< i 0) result)))))
    
    ;; ============= PREDICATES =============
    
    (define (proper-list? x)
      (let loop ((x x) (lag x))
        (cond
          ((null? x) #t)
          ((not (pair? x)) #f)
          ((null? (cdr x)) #t)
          ((not (pair? (cdr x))) #f)
          ((eq? x lag) #f)  ; circular
          (else (loop (cddr x) (cdr lag))))))
    
    (define (circular-list? x)
      (and (pair? x)
           (let loop ((fast x) (slow x))
             (cond
               ((null? fast) #f)
               ((not (pair? fast)) #f)
               ((null? (cdr fast)) #f)
               ((not (pair? (cdr fast))) #f)
               ((eq? fast slow) #t)
               (else (loop (cddr fast) (cdr slow)))))))
    
    (define (dotted-list? x)
      (and (pair? x)
           (not (proper-list? x))
           (not (circular-list? x))))
    
    (define (not-pair? x) (not (pair? x)))
    
    (define (null-list? x)
      (cond
        ((null? x) #t)
        ((pair? x) #f)
        (else (error "null-list?: argument must be a list" x))))
    
    (define list=
      (case-lambda
        ((elt=) #t)
        ((elt= list1) #t)
        ((elt= list1 list2)
         (let loop ((l1 list1) (l2 list2))
           (cond
             ((and (null? l1) (null? l2)) #t)
             ((or (null? l1) (null? l2)) #f)
             ((elt= (car l1) (car l2)) (loop (cdr l1) (cdr l2)))
             (else #f))))
        ((elt= list1 list2 . lists)
         (let loop ((lists (cons list1 (cons list2 lists))))
           (or (null? (cdr lists))
               (and (list= elt= (car lists) (cadr lists))
                    (loop (cdr lists))))))))
    
    ;; ============= SELECTORS =============
    
    (define first car)
    (define second cadr)
    (define third caddr)
    (define fourth cadddr)
    (define (fifth x) (car (cddddr x)))
    (define (sixth x) (cadr (cddddr x)))
    (define (seventh x) (caddr (cddddr x)))
    (define (eighth x) (cadddr (cddddr x)))
    (define (ninth x) (car (cddddr (cddddr x))))
    (define (tenth x) (cadr (cddddr (cddddr x))))
    
    (define (car+cdr pair)
      (values (car pair) (cdr pair)))
    
    (define (take lst k)
      (if (= k 0)
          '()
          (cons (car lst) (take (cdr lst) (- k 1)))))
    
    (define (drop lst k)
      (if (= k 0)
          lst
          (drop (cdr lst) (- k 1))))
    
    (define (take-right lst k)
      (let ((len (length lst)))
        (drop lst (- len k))))
    
    (define (drop-right lst k)
      (let ((len (length lst)))
        (take lst (- len k))))
    
    (define (take! lst k)
      (if (= k 0)
          (begin (set-cdr! lst '()) lst)
          (begin (take! (cdr lst) (- k 1)) lst)))
    
    (define (drop-right! lst k)
      (let ((len (length lst)))
        (if (<= len k)
            '()
            (begin
              (let loop ((lst lst) (n (- len k 1)))
                (if (= n 0)
                    (set-cdr! lst '())
                    (loop (cdr lst) (- n 1))))
              lst))))
    
    (define (split-at lst k)
      (values (take lst k) (drop lst k)))
    
    (define (split-at! lst k)
      (if (= k 0)
          (values '() lst)
          (let ((prefix lst))
            (let loop ((lst lst) (k (- k 1)))
              (if (= k 0)
                  (let ((suffix (cdr lst)))
                    (set-cdr! lst '())
                    (values prefix suffix))
                  (loop (cdr lst) (- k 1)))))))
    
    (define (last lst)
      (if (null? (cdr lst))
          (car lst)
          (last (cdr lst))))
    
    (define (last-pair lst)
      (if (null? (cdr lst))
          lst
          (last-pair (cdr lst))))
    
    ;; ============= MISCELLANEOUS =============
    
    (define (length+ lst)
      (if (proper-list? lst)
          (length lst)
          #f))
    
    ;; ============= FOLD, UNFOLD & MAP =============
    
    (define fold
      (case-lambda
        ((kons knil lst)
         (let loop ((lst lst) (acc knil))
           (if (null? lst)
               acc
               (loop (cdr lst) (kons (car lst) acc)))))
        ((kons knil lst1 lst2 . lists)
         (let loop ((lists (cons lst1 (cons lst2 lists))) (acc knil))
           (if (any null? lists)
               acc
               (loop (map cdr lists)
                     (apply kons (append (map car lists) (list acc)))))))))
    
    (define fold-right
      (case-lambda
        ((kons knil lst)
         (if (null? lst)
             knil
             (kons (car lst) (fold-right kons knil (cdr lst)))))
        ((kons knil lst1 lst2 . lists)
         (let ((all-lists (cons lst1 (cons lst2 lists))))
           (if (any null? all-lists)
               knil
               (apply kons (append (map car all-lists)
                                   (list (apply fold-right kons knil (map cdr all-lists))))))))))
    
    (define (pair-fold kons knil lst)
      (let loop ((lst lst) (acc knil))
        (if (null? lst)
            acc
            (loop (cdr lst) (kons lst acc)))))
    
    (define (pair-fold-right kons knil lst)
      (if (null? lst)
          knil
          (kons lst (pair-fold-right kons knil (cdr lst)))))
    
    (define (reduce f ridentity lst)
      (if (null? lst)
          ridentity
          (fold f (car lst) (cdr lst))))
    
    (define (reduce-right f ridentity lst)
      (if (null? lst)
          ridentity
          (fold-right f (last lst) (drop-right lst 1))))
    
    (define unfold
      (case-lambda
        ((p f g seed)
         (unfold p f g seed '()))
        ((p f g seed tail-gen)
         (let loop ((seed seed) (result '()))
           (if (p seed)
               (if (procedure? tail-gen)
                   (append (reverse result) (tail-gen seed))
                   (append (reverse result) tail-gen))
               (loop (g seed) (cons (f seed) result)))))))
    
    (define (unfold-right p f g seed . maybe-tail)
      (let ((tail (if (null? maybe-tail) '() (car maybe-tail))))
        (let loop ((seed seed) (result tail))
          (if (p seed)
              result
              (loop (g seed) (cons (f seed) result))))))
    
    (define (append-map f lst . lists)
      (apply append (apply map f lst lists)))
    
    (define (append-map! f lst . lists)
      (apply append! (apply map f lst lists)))
    
    (define (map! f lst . lists)
      (if (null? lists)
          (let loop ((lst lst))
            (if (not (null? lst))
                (begin
                  (set-car! lst (f (car lst)))
                  (loop (cdr lst)))))
          (let loop ((lists (cons lst lists)))
            (if (not (any null? lists))
                (begin
                  (set-car! (car lists) (apply f (map car lists)))
                  (loop (map cdr lists))))))
      lst)
    
    (define (pair-for-each f lst . lists)
      (if (null? lists)
          (let loop ((lst lst))
            (if (not (null? lst))
                (begin
                  (f lst)
                  (loop (cdr lst)))))
          (let loop ((lists (cons lst lists)))
            (if (not (any null? lists))
                (begin
                  (apply f lists)
                  (loop (map cdr lists)))))))
    
    (define (filter-map f lst . lists)
      (if (null? lists)
          (let loop ((lst lst) (result '()))
            (if (null? lst)
                (reverse result)
                (let ((val (f (car lst))))
                  (if val
                      (loop (cdr lst) (cons val result))
                      (loop (cdr lst) result)))))
          (let loop ((lists (cons lst lists)) (result '()))
            (if (any null? lists)
                (reverse result)
                (let ((val (apply f (map car lists))))
                  (if val
                      (loop (map cdr lists) (cons val result))
                      (loop (map cdr lists) result)))))))
    
    (define (map-in-order f lst . lists)
      (apply map f lst lists))  ; In single-threaded context, same as map
    
    ;; ============= FILTERING & PARTITIONING =============
    
    (define (filter pred lst)
      (let loop ((lst lst) (result '()))
        (cond
          ((null? lst) (reverse result))
          ((pred (car lst)) (loop (cdr lst) (cons (car lst) result)))
          (else (loop (cdr lst) result)))))
    
    (define (partition pred lst)
      (let loop ((lst lst) (in '()) (out '()))
        (cond
          ((null? lst) (values (reverse in) (reverse out)))
          ((pred (car lst)) (loop (cdr lst) (cons (car lst) in) out))
          (else (loop (cdr lst) in (cons (car lst) out))))))
    
    (define (remove pred lst)
      (filter (lambda (x) (not (pred x))) lst))
    
    (define (filter! pred lst)
      (let loop ((lst lst) (prev #f))
        (cond
          ((null? lst) lst)
          ((pred (car lst))
           (loop (cdr lst) lst))
          (else
           (if prev
               (set-cdr! prev (cdr lst))
               (set! lst (cdr lst)))
           (loop (cdr lst) prev))))
      lst)
    
    (define (partition! pred lst)
      (let ((in-head (cons #f '()))
            (out-head (cons #f '())))
        (let loop ((lst lst) (in-tail in-head) (out-tail out-head))
          (cond
            ((null? lst)
             (values (cdr in-head) (cdr out-head)))
            ((pred (car lst))
             (set-cdr! in-tail lst)
             (loop (cdr lst) lst out-tail))
            (else
             (set-cdr! out-tail lst)
             (loop (cdr lst) in-tail lst))))))
    
    (define (remove! pred lst)
      (filter! (lambda (x) (not (pred x))) lst))
    
    ;; ============= SEARCHING =============
    
    (define (find pred lst)
      (let loop ((lst lst))
        (cond
          ((null? lst) #f)
          ((pred (car lst)) (car lst))
          (else (loop (cdr lst))))))
    
    (define (find-tail pred lst)
      (let loop ((lst lst))
        (cond
          ((null? lst) #f)
          ((pred (car lst)) lst)
          (else (loop (cdr lst))))))
    
    (define (any pred lst . lists)
      (if (null? lists)
          (let loop ((lst lst))
            (cond
              ((null? lst) #f)
              ((pred (car lst)) #t)
              (else (loop (cdr lst)))))
          (let loop ((lists (cons lst lists)))
            (cond
              ((any null? lists) #f)
              ((apply pred (map car lists)) #t)
              (else (loop (map cdr lists)))))))
    
    (define (every pred lst . lists)
      (if (null? lists)
          (let loop ((lst lst))
            (cond
              ((null? lst) #t)
              ((pred (car lst)) (loop (cdr lst)))
              (else #f)))
          (let loop ((lists (cons lst lists)))
            (cond
              ((any null? lists) #t)
              ((apply pred (map car lists)) (loop (map cdr lists)))
              (else #f)))))
    
    (define (list-index pred lst . lists)
      (if (null? lists)
          (let loop ((lst lst) (i 0))
            (cond
              ((null? lst) #f)
              ((pred (car lst)) i)
              (else (loop (cdr lst) (+ i 1)))))
          (let loop ((lists (cons lst lists)) (i 0))
            (cond
              ((any null? lists) #f)
              ((apply pred (map car lists)) i)
              (else (loop (map cdr lists) (+ i 1)))))))
    
    (define (take-while pred lst)
      (let loop ((lst lst) (result '()))
        (cond
          ((null? lst) (reverse result))
          ((pred (car lst)) (loop (cdr lst) (cons (car lst) result)))
          (else (reverse result)))))
    
    (define (drop-while pred lst)
      (let loop ((lst lst))
        (cond
          ((null? lst) '())
          ((pred (car lst)) (loop (cdr lst)))
          (else lst))))
    
    (define (take-while! pred lst)
      (let loop ((lst lst) (prev #f))
        (cond
          ((null? lst) lst)
          ((pred (car lst)) (loop (cdr lst) lst))
          (else
           (if prev
               (set-cdr! prev '())
               (set! lst '()))
           lst))))
    
    (define (span pred lst)
      (let loop ((lst lst) (prefix '()))
        (cond
          ((null? lst) (values (reverse prefix) '()))
          ((pred (car lst)) (loop (cdr lst) (cons (car lst) prefix)))
          (else (values (reverse prefix) lst)))))
    
    (define (break pred lst)
      (span (lambda (x) (not (pred x))) lst))
    
    (define (span! pred lst)
      (let loop ((lst lst) (prev #f))
        (cond
          ((null? lst) (values lst '()))
          ((pred (car lst)) (loop (cdr lst) lst))
          (else
           (if prev
               (let ((suffix lst))
                 (set-cdr! prev '())
                 (values lst suffix))
               (values '() lst))))))
    
    (define (break! pred lst)
      (span! (lambda (x) (not (pred x))) lst))
    
    ;; ============= DELETING =============
    
    (define delete
      (case-lambda
        ((x lst) (delete x lst equal?))
        ((x lst =)
         (filter (lambda (y) (not (= x y))) lst))))
    
    (define (delete-duplicates lst . maybe-=)
      (let ((= (if (null? maybe-=) equal? (car maybe-=))))
        (let loop ((lst lst) (result '()))
          (cond
            ((null? lst) (reverse result))
            ((find (lambda (y) (= (car lst) y)) result)
             (loop (cdr lst) result))
            (else (loop (cdr lst) (cons (car lst) result)))))))
    
    (define delete!
      (case-lambda
        ((x lst) (delete! x lst equal?))
        ((x lst =)
         (filter! (lambda (y) (not (= x y))) lst))))
    
    (define (delete-duplicates! lst . maybe-=)
      (let ((= (if (null? maybe-=) equal? (car maybe-=))))
        (let loop ((lst lst))
          (if (not (null? lst))
              (begin
                (set-cdr! lst (delete! (car lst) (cdr lst) =))
                (loop (cdr lst)))))
        lst))
    
    ;; ============= ASSOCIATION LISTS =============
    
    (define (alist-cons key datum alist)
      (cons (cons key datum) alist))
    
    (define (alist-copy alist)
      (map (lambda (pair) (cons (car pair) (cdr pair))) alist))
    
    (define alist-delete
      (case-lambda
        ((key alist) (alist-delete key alist equal?))
        ((key alist =)
         (filter (lambda (pair) (not (= key (car pair)))) alist))))
    
    (define alist-delete!
      (case-lambda
        ((key alist) (alist-delete! key alist equal?))
        ((key alist =)
         (filter! (lambda (pair) (not (= key (car pair)))) alist))))
    
    ;; ============= SET OPERATIONS =============
    
    (define lset<=
      (case-lambda
        ((=) #t)
        ((= set1) #t)
        ((= set1 set2)
         (every (lambda (x) (find (lambda (y) (= x y)) set2)) set1))
        ((= set1 set2 . sets)
         (let loop ((sets (cons set1 (cons set2 sets))))
           (or (null? (cdr sets))
               (and (lset<= = (car sets) (cadr sets))
                    (loop (cdr sets))))))))
    
    (define lset=
      (case-lambda
        ((=) #t)
        ((= set1) #t)
        ((= set1 set2)
         (and (lset<= = set1 set2) (lset<= = set2 set1)))
        ((= set1 set2 . sets)
         (let loop ((sets (cons set1 (cons set2 sets))))
           (or (null? (cdr sets))
               (and (lset= = (car sets) (cadr sets))
                    (loop (cdr sets))))))))
    
    (define (lset-adjoin = set . elts)
      (fold (lambda (elt set)
              (if (find (lambda (x) (= elt x)) set)
                  set
                  (cons elt set)))
            set elts))
    
    (define lset-union
      (case-lambda
        ((=) '())
        ((= set1) set1)
        ((= set1 set2 . sets)
         (let loop ((sets (cons set1 (cons set2 sets))) (result '()))
           (if (null? sets)
               result
               (loop (cdr sets)
                     (fold (lambda (x acc)
                             (if (find (lambda (y) (= x y)) acc)
                                 acc
                                 (cons x acc)))
                           result (car sets))))))))
    
    (define lset-intersection
      (case-lambda
        ((=) '())
        ((= set1) set1)
        ((= set1 set2 . sets)
         (let ((all-sets (cons set1 (cons set2 sets))))
           (filter (lambda (x)
                     (every (lambda (set)
                              (find (lambda (y) (= x y)) set))
                            (cdr all-sets)))
                   (car all-sets))))))
    
    (define lset-difference
      (case-lambda
        ((= set1) set1)
        ((= set1 set2 . sets)
         (let ((other-sets (cons set2 sets)))
           (filter (lambda (x)
                     (not (any (lambda (set)
                                 (find (lambda (y) (= x y)) set))
                               other-sets)))
                   set1)))))
    
    (define (lset-xor = . sets)
      (fold (lambda (set1 set2)
              (append (lset-difference = set1 set2)
                      (lset-difference = set2 set1)))
            '() sets))
    
    (define (lset-diff+intersection = set1 . sets)
      (values (apply lset-difference = set1 sets)
              (apply lset-intersection = set1 sets)))
    
    ;; Destructive versions - these modify the original lists
    (define (lset-union! = . sets)
      (apply lset-union = sets))  ; Simplified implementation
    
    (define (lset-intersection! = . sets)
      (apply lset-intersection = sets))  ; Simplified implementation
    
    (define (lset-difference! = set1 . sets)
      (apply lset-difference = set1 sets))  ; Simplified implementation
    
    (define (lset-xor! = . sets)
      (apply lset-xor = sets))  ; Simplified implementation
    
    (define (lset-diff+intersection! = set1 . sets)
      (apply lset-diff+intersection = set1 sets))  ; Simplified implementation
    
    ;; Helper function for append! - mutating append
    (define (append! . lists)
      (if (null? lists)
          '()
          (let ((first (car lists)))
            (if (null? (cdr lists))
                first
                (begin
                  (set-cdr! (last-pair first) (apply append! (cdr lists)))
                  first)))))))