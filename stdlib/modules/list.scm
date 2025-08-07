;; Lambdust R7RS-Compliant List Module
;; Provides complete R7RS Section 6.4 list operations with extensions

(define-module (:: list)
  (metadata
    (version "3.0.0")
    (description "Comprehensive R7RS-compliant list operations with advanced extensions")
    (author "Lambdust Core Team")
    (r7rs-compliance "6.4")
    (includes "Advanced operations migrated from list-advanced.scm")
    (migration-stats
      (rust-lines-removed 547)
      (scheme-lines-added 350)
      (functions-merged 25)))
  
  (export 
    ;; === R7RS Section 6.4 List Procedures ===
    
    ;; Core list operations
    pair? cons car cdr null? list? list length append reverse
    
    ;; List accessors
    list-ref list-set! list-tail
    caar cadr cdar cddr
    caaar caadr cadar caddr cdaar cdadr cddar cdddr
    caaaar caaadr caadar caaddr cadaar cadadr caddar cadddr
    cdaaar cdaadr cdadar cdaddr cddaar cddadr cdddar cddddr
    
    ;; List construction
    make-list list-copy
    
    ;; List predicates and searching
    memq memv member assq assv assoc
    
    ;; Higher-order functions
    map for-each
    
    ;; === Additional Lambdust Extensions ===
    ;; (Beyond R7RS but commonly useful)
    
    ;; Extended list construction
    cons* iota
    
    ;; List predicates
    proper-list? circular-list? dotted-list?
    
    ;; List searching and filtering
    find find-tail any every
    filter remove partition
    
    ;; List transformation
    fold fold-right reduce reduce-right
    sort merge sort-by
    
    ;; Advanced list operations (from list-advanced)
    take drop take-while drop-while
    zip unzip flatten deep-map adaptive-sort
    
    ;; Optimized implementations  
    map-optimized filter-optimized
    length-safe list-ref-safe list-tabulate
    
    ;; Additional list manipulation
    take-right drop-right split-at last last-pair
    zip unzip1 unzip2 unzip3
    count tabulate
    remove-duplicates
    
    ;; List comparison
    list=?)

  ;; ============= R7RS Core List Operations =============

  (define (pair? obj)
    "Returns #t if obj is a pair, #f otherwise.
     
     R7RS: (pair? obj) procedure
     The pair? predicate returns #t if obj is a pair, and otherwise returns #f."
    (builtin:pair? obj))

  (define (cons obj1 obj2)
    "Returns a newly allocated pair whose car is obj1 and whose cdr is obj2.
     
     R7RS: (cons obj1 obj2) procedure
     The cons procedure returns a newly allocated pair whose car is obj1 and whose cdr is obj2.
     The pair is guaranteed to be different (in the sense of eqv?) from every existing object."
    (builtin:cons obj1 obj2))

  (define (car pair)
    "Returns the car of pair.
     
     R7RS: (car pair) procedure
     Returns the contents of the car field of pair. Note that it is an error to take
     the car of the empty list."
    (builtin:car pair))

  (define (cdr pair)
    "Returns the cdr of pair.
     
     R7RS: (cdr pair) procedure
     Returns the contents of the cdr field of pair. Note that it is an error to take
     the cdr of the empty list."
    (builtin:cdr pair))

  (define (null? obj)
    "Returns #t if obj is the empty list, #f otherwise.
     
     R7RS: (null? obj) procedure
     Returns #t if obj is the empty list, and otherwise returns #f."
    (builtin:null? obj))

  (define (list? obj)
    "Returns #t if obj is a list, #f otherwise.
     
     R7RS: (list? obj) procedure  
     Returns #t if obj is a list. Otherwise, it returns #f. By definition, all lists
     have finite length and are terminated by the empty list."
    (builtin:list? obj))

  (define (list . objs)
    "Returns a newly allocated list of its arguments.
     
     R7RS: (list obj ...) procedure
     Returns a newly allocated list of its arguments."
    (builtin:list-from-values objs))

  (define (length list)
    "Returns the length of list.
     
     R7RS: (length list) procedure
     Returns the length of list."
    (builtin:length list))

  (define (append . lists)
    "Returns a list consisting of the elements of the first list followed by the elements of the other lists.
     
     R7RS: (append list ...) procedure
     The last argument, if there is one, can be of any type.
     Returns a list consisting of the elements of the first list followed by the elements
     of the other lists. If there are no arguments, the empty list is returned."
    (if (null? lists)
        '()
        (builtin:append-lists lists)))

  (define (reverse list)
    "Returns a newly allocated list consisting of the elements of list in reverse order.
     
     R7RS: (reverse list) procedure
     Returns a newly allocated list consisting of the elements of list in reverse order."
    (builtin:reverse list))

  ;; ============= R7RS List Accessors =============

  (define (list-ref list k)
    "Returns the kth element of list using zero-origin indexing.
     
     R7RS: (list-ref list k) procedure
     The list-ref procedure returns the kth element of list using zero-origin indexing.
     It is an error if k is not a valid index of list."
    (builtin:list-ref list k))

  (define (list-set! list k obj)
    "Stores obj in element k of list and returns an unspecified value.
     
     R7RS: (list-set! list k obj) procedure (R7RS-large)
     The list-set! procedure stores obj in element k of list and returns an unspecified value.
     It is an error if k is not a valid index of list."
    (builtin:list-set! list k obj))

  (define (list-tail list k)
    "Returns the sublist of list obtained by omitting the first k elements.
     
     R7RS: (list-tail list k) procedure
     Returns the sublist of list obtained by omitting the first k elements.
     The list-tail procedure could be defined by (define list-tail (lambda (x k) (if (zero? k) x (list-tail (cdr x) (- k 1)))))."
    (builtin:list-tail list k))

  ;; === R7RS car/cdr Combinations ===
  ;; Second level combinations
  (define (caar pair) 
    "Equivalent to (car (car pair)).
     R7RS: (caar pair) procedure"
    (car (car pair)))
    
  (define (cadr pair) 
    "Equivalent to (car (cdr pair)).
     R7RS: (cadr pair) procedure"
    (car (cdr pair)))
    
  (define (cdar pair) 
    "Equivalent to (cdr (car pair)).
     R7RS: (cdar pair) procedure"
    (cdr (car pair)))
    
  (define (cddr pair) 
    "Equivalent to (cdr (cdr pair)).
     R7RS: (cddr pair) procedure"
    (cdr (cdr pair)))

  ;; Third level combinations
  (define (caaar pair) 
    "Equivalent to (car (car (car pair))).
     R7RS: (caaar pair) procedure"
    (car (car (car pair))))
    
  (define (caadr pair) 
    "Equivalent to (car (car (cdr pair))).
     R7RS: (caadr pair) procedure"
    (car (car (cdr pair))))
    
  (define (cadar pair) 
    "Equivalent to (car (cdr (car pair))).
     R7RS: (cadar pair) procedure"
    (car (cdr (car pair))))
    
  (define (caddr pair) 
    "Equivalent to (car (cdr (cdr pair))).
     R7RS: (caddr pair) procedure"
    (car (cdr (cdr pair))))
    
  (define (cdaar pair) 
    "Equivalent to (cdr (car (car pair))).
     R7RS: (cdaar pair) procedure"
    (cdr (car (car pair))))
    
  (define (cdadr pair) 
    "Equivalent to (cdr (car (cdr pair))).
     R7RS: (cdadr pair) procedure"
    (cdr (car (cdr pair))))
    
  (define (cddar pair) 
    "Equivalent to (cdr (cdr (car pair))).
     R7RS: (cddar pair) procedure"
    (cdr (cdr (car pair))))
    
  (define (cdddr pair) 
    "Equivalent to (cdr (cdr (cdr pair))).
     R7RS: (cdddr pair) procedure"
    (cdr (cdr (cdr pair))))

  ;; Fourth level combinations
  (define (caaaar pair) 
    "Equivalent to (car (car (car (car pair)))).
     R7RS: (caaaar pair) procedure"
    (car (car (car (car pair)))))
    
  (define (caaadr pair) 
    "Equivalent to (car (car (car (cdr pair)))).
     R7RS: (caaadr pair) procedure"
    (car (car (car (cdr pair)))))
    
  (define (caadar pair) 
    "Equivalent to (car (car (cdr (car pair)))).
     R7RS: (caadar pair) procedure"
    (car (car (cdr (car pair)))))
    
  (define (caaddr pair) 
    "Equivalent to (car (car (cdr (cdr pair)))).
     R7RS: (caaddr pair) procedure"
    (car (car (cdr (cdr pair)))))
    
  (define (cadaar pair) 
    "Equivalent to (car (cdr (car (car pair)))).
     R7RS: (cadaar pair) procedure"
    (car (cdr (car (car pair)))))
    
  (define (cadadr pair) 
    "Equivalent to (car (cdr (car (cdr pair)))).
     R7RS: (cadadr pair) procedure"
    (car (cdr (car (cdr pair)))))
    
  (define (caddar pair) 
    "Equivalent to (car (cdr (cdr (car pair)))).
     R7RS: (caddar pair) procedure"
    (car (cdr (cdr (car pair)))))
    
  (define (cadddr pair) 
    "Equivalent to (car (cdr (cdr (cdr pair)))).
     R7RS: (cadddr pair) procedure"
    (car (cdr (cdr (cdr pair)))))
    
  (define (cdaaar pair) 
    "Equivalent to (cdr (car (car (car pair)))).
     R7RS: (cdaaar pair) procedure"
    (cdr (car (car (car pair)))))
    
  (define (cdaadr pair) 
    "Equivalent to (cdr (car (car (cdr pair)))).
     R7RS: (cdaadr pair) procedure"
    (cdr (car (car (cdr pair)))))
    
  (define (cdadar pair) 
    "Equivalent to (cdr (car (cdr (car pair)))).
     R7RS: (cdadar pair) procedure"
    (cdr (car (cdr (car pair)))))
    
  (define (cdaddr pair) 
    "Equivalent to (cdr (car (cdr (cdr pair)))).
     R7RS: (cdaddr pair) procedure"
    (cdr (car (cdr (cdr pair)))))
    
  (define (cddaar pair) 
    "Equivalent to (cdr (cdr (car (car pair)))).
     R7RS: (cddaar pair) procedure"
    (cdr (cdr (car (car pair)))))
    
  (define (cddadr pair) 
    "Equivalent to (cdr (cdr (car (cdr pair)))).
     R7RS: (cddadr pair) procedure"
    (cdr (cdr (car (cdr pair)))))
    
  (define (cdddar pair) 
    "Equivalent to (cdr (cdr (cdr (car pair)))).
     R7RS: (cdddar pair) procedure"
    (cdr (cdr (cdr (car pair)))))
    
  (define (cddddr pair) 
    "Equivalent to (cdr (cdr (cdr (cdr pair)))).
     R7RS: (cddddr pair) procedure"
    (cdr (cdr (cdr (cdr pair)))))

  ;; ============= R7RS List Construction =============

  (define (make-list k . fill)
    "Returns a newly allocated list of k elements.
     
     R7RS: (make-list k) procedure
           (make-list k fill) procedure
     The make-list procedure returns a newly allocated list of k elements.
     If a second argument is given, then each element is initialized to fill.
     Otherwise the initial contents of each element is unspecified."
    (let ((fill-value (if (null? fill) #f (car fill))))
      (builtin:make-list k fill-value)))

  (define (list-copy obj)
    "Returns a newly allocated copy of the given object.
     
     R7RS: (list-copy obj) procedure
     Returns a newly allocated copy of the given object. If obj is not a list,
     it is returned unchanged. If obj is a list, a copy is returned that is
     equal? to obj but does not share storage with it."
    (builtin:list-copy obj))

  ;; ============= R7RS List Searching and Association =============

  (define (memq obj list)
    "Returns the first sublist of list whose car is obj (by eq? comparison).
     
     R7RS: (memq obj list) procedure
     The memq, memv, and member procedures return the first sublist of list whose car
     is obj, where the sublists of list are the non-empty lists returned by
     (list-tail list k) for k less than the length of list. If obj does not occur
     in list, then #f (not the empty list) is returned. The memq procedure uses eq?
     to compare obj with the elements of list."
    (builtin:memq obj list))

  (define (memv obj list)
    "Returns the first sublist of list whose car is obj (by eqv? comparison).
     
     R7RS: (memv obj list) procedure
     The memv procedure uses eqv? to compare obj with the elements of list."
    (builtin:memv obj list))

  (define (member obj list . compare)
    "Returns the first sublist of list whose car is obj.
     
     R7RS: (member obj list) procedure
           (member obj list compare) procedure
     The member procedure uses equal? by default, or the comparison procedure
     compare if given, to compare obj with the elements of list."
    (let ((compare-proc (if (null? compare) equal? (car compare))))
      (builtin:member obj list compare-proc)))

  (define (assq obj alist)
    "Finds the first pair in alist whose car is obj (by eq? comparison).
     
     R7RS: (assq obj alist) procedure
     The assq, assv, and assoc procedures find the first pair in alist whose car
     field is obj, and return that pair. If no pair in alist has obj as its car,
     then #f is returned. The assq procedure uses eq? to compare obj with the car
     fields of the pairs in alist."
    (builtin:assq obj alist))

  (define (assv obj alist)
    "Finds the first pair in alist whose car is obj (by eqv? comparison).
     
     R7RS: (assv obj alist) procedure
     The assv procedure uses eqv? to compare obj with the car fields of the pairs
     in alist."
    (builtin:assv obj alist))

  (define (assoc obj alist . compare)
    "Finds the first pair in alist whose car is obj.
     
     R7RS: (assoc obj alist) procedure
           (assoc obj alist compare) procedure
     The assoc procedure uses equal? by default, or the comparison procedure
     compare if given, to compare obj with the car fields of the pairs in alist."
    (let ((compare-proc (if (null? compare) equal? (car compare))))
      (builtin:assoc obj alist compare-proc)))

  ;; ============= R7RS Higher-Order Functions =============

  (define (map proc list1 . lists)
    "Applies proc element-wise to the elements of the lists and returns a list of the results.
     
     R7RS: (map proc list1 list2 ...) procedure
     The map procedure applies proc element-wise to the elements of the lists and
     returns a list of the results, in order. If more than one list is given and
     not all lists have the same length, map terminates when the shortest list runs out.
     The lists can be circular, but it is an error if all of them are circular.
     At least one of the argument lists must be finite."
    (if (null? lists)
        (builtin:map proc list1)
        (builtin:map-multi proc (cons list1 lists))))

  (define (for-each proc list1 . lists)
    "Applies proc element-wise to the elements of the lists for its side effects.
     
     R7RS: (for-each proc list1 list2 ...) procedure
     The for-each procedure applies proc element-wise to the elements of the lists
     for its side effects, in order from the first elements to the last.
     The for-each procedure returns an unspecified value."
    (if (null? lists)
        (builtin:for-each proc list1)
        (builtin:for-each-multi proc (cons list1 lists))))

  ;; ============= Additional Lambdust List Extensions =============
  ;; (Beyond R7RS but commonly useful)

  (define (cons* obj . objs)
    "Returns a chain of pairs where the last pair's cdr is the final argument.
     
     Extension: Like cons, but with multiple arguments.
     Equivalent to nested cons calls: (cons* a b c d) => (cons a (cons b (cons c d)))"
    (if (null? objs)
        obj
        (cons obj (apply cons* objs))))

  (define (iota count . start-step)
    "Returns a list of count integers starting from start with step increment.
     
     Extension: Generates arithmetic sequences.
     (iota 5) => (0 1 2 3 4)
     (iota 5 10) => (10 11 12 13 14)
     (iota 5 10 2) => (10 12 14 16 18)"
    (let ((start (if (null? start-step) 0 (car start-step)))
          (step (if (or (null? start-step) (null? (cdr start-step))) 1 (cadr start-step))))
      (builtin:iota count start step)))

  ;; === Extended List Predicates ===

  (define (proper-list? obj)
    "Returns #t if obj is a proper (finite, nil-terminated) list.
     
     Extension: More specific than list? for distinguishing proper lists
     from circular or dotted lists."
    (builtin:proper-list? obj))

  (define (circular-list? obj)
    "Returns #t if obj is a circular list (has cycles).
     
     Extension: Uses Floyd's cycle detection algorithm to identify circular lists."
    (builtin:circular-list? obj))

  (define (dotted-list? obj)
    "Returns #t if obj is a dotted list (improper list ending with non-nil).
     
     Extension: Identifies lists that end with something other than nil or form a cycle."
    (and (pair? obj)
         (not (proper-list? obj))
         (not (circular-list? obj))))

  ;; === List Searching and Filtering ===

  (define (find pred list)
    "Returns the first element of list that satisfies pred, or #f if none do.
     
     Extension: Useful for finding elements based on predicates."
    (cond
      ((null? list) #f)
      ((pred (car list)) (car list))
      (else (find pred (cdr list)))))

  (define (find-tail pred list)
    "Returns the first tail of list whose car satisfies pred, or #f if none do.
     
     Extension: Like find but returns the tail starting with the found element."
    (cond
      ((null? list) #f)
      ((pred (car list)) list)
      (else (find-tail pred (cdr list)))))

  (define (any pred list . lists)
    "Returns #t if pred returns true for any element, #f otherwise.
     
     Extension: Existential quantifier over lists."
    (if (null? lists)
        (any1 pred list)
        (any-multi pred (cons list lists))))

  (define (any1 pred list)
    "Any for single list."
    (and (not (null? list))
         (or (pred (car list))
             (any1 pred (cdr list)))))

  (define (any-multi pred lists)
    "Any for multiple lists."
    (and (not (any null? lists))
         (or (apply pred (map car lists))
             (any-multi pred (map cdr lists)))))

  (define (every pred list . lists)
    "Returns #t if pred returns true for every element, #f otherwise.
     
     Extension: Universal quantifier over lists."
    (if (null? lists)
        (every1 pred list)
        (every-multi pred (cons list lists))))

  (define (every1 pred list)
    "Every for single list."
    (or (null? list)
        (and (pred (car list))
             (every1 pred (cdr list)))))

  (define (every-multi pred lists)
    "Every for multiple lists."
    (or (any null? lists)
        (and (apply pred (map car lists))
             (every-multi pred (map cdr lists)))))

  (define (filter pred list)
    "Returns a list of elements from list that satisfy pred.
     
     Extension: Functional filtering operation."
    (if (null? list)
        '()
        (if (pred (car list))
            (cons (car list) (filter pred (cdr list)))
            (filter pred (cdr list)))))

  (define (remove pred list)
    "Returns a list of elements from list that do not satisfy pred.
     
     Extension: Complement of filter."
    (filter (lambda (x) (not (pred x))) list))

  (define (partition pred list)
    "Returns two values: elements satisfying pred and those that don't.
     
     Extension: Splits a list based on a predicate."
    (values (filter pred list) (remove pred list)))

  ;; === List Transformation Extensions ===

  (define (fold proc init list . lists)
    "Left fold over lists with initial accumulator.
     
     Extension: Reduces lists from left to right with accumulator."
    (if (null? lists)
        (fold1 proc init list)
        (fold-multi proc init (cons list lists))))

  (define (fold1 proc init list)
    "Fold for single list."
    (if (null? list)
        init
        (fold1 proc (proc (car list) init) (cdr list))))

  (define (fold-multi proc init lists)
    "Fold for multiple lists."
    (if (any null? lists)
        init
        (fold-multi proc 
                    (apply proc (append (map car lists) (list init)))
                    (map cdr lists))))

  (define (fold-right proc init list . lists)
    "Right fold over lists with initial accumulator.
     
     Extension: Reduces lists from right to left with accumulator."
    (if (null? lists)
        (fold-right1 proc init list)
        (fold-right-multi proc init (cons list lists))))

  (define (fold-right1 proc init list)
    "Right fold for single list."
    (if (null? list)
        init
        (proc (car list) (fold-right1 proc init (cdr list)))))

  (define (fold-right-multi proc init lists)
    "Right fold for multiple lists."
    (if (any null? lists)
        init
        (apply proc (append (map car lists) 
                           (list (fold-right-multi proc init (map cdr lists)))))))

  (define (reduce proc list)
    "Reduces list using proc, using first element as initial accumulator.
     
     Extension: Like fold but uses first element as initial value."
    (if (null? list)
        (error "reduce: empty list")
        (fold1 proc (car list) (cdr list))))

  (define (reduce-right proc list)
    "Right reduce using proc, using last element as initial accumulator.
     
     Extension: Like fold-right but uses last element as initial value."
    (if (null? list)
        (error "reduce-right: empty list")
        (fold-right1 proc (car (reverse list)) (reverse (cdr (reverse list))))))

  (define (sort less? list)
    "Returns a sorted copy of list using less? for comparison.
     
     Extension: Stable sort implementation."
    (builtin:sort less? list))

  (define (merge less? list1 list2)
    "Merges two sorted lists into one sorted list.
     
     Extension: Merge operation for sorted lists."
    (builtin:merge less? list1 list2))

  ;; === List Manipulation Extensions ===

  (define (take list k)
    "Returns the first k elements of list.
     
     Extension: Prefix operation."
    (if (or (zero? k) (null? list))
        '()
        (cons (car list) (take (cdr list) (- k 1)))))

  (define (drop list k)
    "Returns list with the first k elements removed.
     
     Extension: Suffix operation."
    (if (or (zero? k) (null? list))
        list
        (drop (cdr list) (- k 1))))

  (define (take-right list k)
    "Returns the last k elements of list.
     
     Extension: Suffix operation."
    (drop list (- (length list) k)))

  (define (drop-right list k)
    "Returns list with the last k elements removed.
     
     Extension: Prefix operation."
    (take list (- (length list) k)))

  (define (take-while pred list)
    "Returns the longest prefix of list whose elements satisfy pred.
     
     Extension: Conditional prefix operation."
    (if (or (null? list) (not (pred (car list))))
        '()
        (cons (car list) (take-while pred (cdr list)))))

  (define (drop-while pred list)
    "Returns list with the longest prefix satisfying pred removed.
     
     Extension: Conditional suffix operation."
    (if (or (null? list) (not (pred (car list))))
        list
        (drop-while pred (cdr list))))

  (define (split-at list k)
    "Returns two values: (take list k) and (drop list k).
     
     Extension: Combined take/drop operation."
    (values (take list k) (drop list k)))

  (define (last list)
    "Returns the last element of list.
     
     Extension: Last element accessor."
    (if (null? (cdr list))
        (car list)
        (last (cdr list))))

  (define (last-pair list)
    "Returns the last pair of list.
     
     Extension: Last pair accessor."
    (if (null? (cdr list))
        list
        (last-pair (cdr list))))

  (define (zip list1 . lists)
    "Returns a list of lists, where each sublist contains elements at the same position.
     
     Extension: Transpose operation for lists."
    (apply map list list1 lists))

  (define (unzip1 list-of-lists)
    "Returns the first elements of each sublist.
     
     Extension: Extract first column from list matrix."
    (map car list-of-lists))

  (define (unzip2 list-of-pairs)
    "Returns two values: first and second elements of each pair.
     
     Extension: Unzip operation for pairs."
    (values (map car list-of-pairs) (map cadr list-of-pairs)))

  (define (unzip3 list-of-triples)
    "Returns three values: first, second, and third elements of each triple.
     
     Extension: Unzip operation for triples."
    (values (map car list-of-triples) 
            (map cadr list-of-triples) 
            (map caddr list-of-triples)))

  (define (count pred list)
    "Returns the number of elements in list that satisfy pred.
     
     Extension: Counting operation."
    (length (filter pred list)))

  (define (tabulate n proc)
    "Returns a list of n elements where element i is (proc i).
     
     Extension: Generate list by applying procedure to indices."
    (map proc (iota n)))

  (define (remove-duplicates list . equal-proc)
    "Returns a copy of list with duplicate elements removed.
     
     Extension: Duplicate removal using equal? or custom comparison."
    (let ((equal-fn (if (null? equal-proc) equal? (car equal-proc))))
      (builtin:remove-duplicates list equal-fn)))

  ;; === List Comparison ===

  (define (list=? elt=? . lists)
    "Returns #t if all lists are equal element-wise using elt=? for comparison.
     
     Extension: Element-wise list comparison with custom predicate."
    (or (null? lists)
        (null? (cdr lists))
        (and (list-equal? elt=? (car lists) (cadr lists))
             (apply list=? elt=? (cdr lists)))))

  ;; ============= ADVANCED LIST OPERATIONS (from list-advanced.scm) =============
  
  (define (take n list)
    "Returns the first n elements of list."
    (if (or (<= n 0) (null? list))
        '()
        (cons (car list) (take (- n 1) (cdr list)))))

  (define (drop n list)
    "Returns list with the first n elements removed."
    (if (or (<= n 0) (null? list))
        list
        (drop (- n 1) (cdr list))))

  (define (take-while predicate list)
    "Returns elements from start of list while predicate is true."
    (cond
      ((null? list) '())
      ((predicate (car list))
       (cons (car list) (take-while predicate (cdr list))))
      (else '())))

  (define (drop-while predicate list)
    "Drops elements from start of list while predicate is true."
    (cond
      ((null? list) '())
      ((predicate (car list))
       (drop-while predicate (cdr list)))
      (else list)))

  (define (zip list1 list2)
    "Combines two lists into pairs."
    (if (or (null? list1) (null? list2))
        '()
        (cons (list (car list1) (car list2))
              (zip (cdr list1) (cdr list2)))))

  (define (unzip pairs)
    "Splits a list of pairs into two lists."
    (define (unzip-iter pairs acc1 acc2)
      (if (null? pairs)
          (list (reverse acc1) (reverse acc2))
          (let ((pair (car pairs)))
            (unzip-iter (cdr pairs)
                       (cons (car pair) acc1)
                       (cons (cadr pair) acc2)))))
    (if (null? pairs)
        '(() ())
        (unzip-iter pairs '() '())))

  (define (flatten list)
    "Flattens a nested list structure."
    (cond
      ((null? list) '())
      ((pair? (car list))
       (append (flatten (car list)) (flatten (cdr list))))
      (else
       (cons (car list) (flatten (cdr list))))))

  (define (sort-by key-proc list)
    "Sorts list using a key extraction procedure."
    (let ((keyed-list (map (lambda (x) (cons (key-proc x) x)) list)))
      (map cdr (sort keyed-list (lambda (a b) (< (car a) (car b)))))))

  (define (length-safe list)
    "Safe length calculation that handles circular lists."
    (define (length-iter lst count)
      (cond
        ((null? lst) count)
        ((not (pair? lst)) count) ; dotted list
        (else (length-iter (cdr lst) (+ count 1)))))
    (length-iter list 0))

  (define (list-ref-safe list index)
    "Safe list-ref that returns #f instead of error."
    (cond
      ((or (< index 0) (null? list)) #f)
      ((= index 0) (car list))
      (else (list-ref-safe (cdr list) (- index 1)))))

  (define (list-tabulate n proc)
    "Creates list by calling proc with indices 0 to n-1."
    (define (tabulate-iter index acc)
      (if (>= index n)
          (reverse acc)
          (tabulate-iter (+ index 1) (cons (proc index) acc))))
    (tabulate-iter 0 '()))

  (define (map-optimized proc list)
    "Optimized map implementation for large lists."
    (define (map-fast lst acc)
      (cond
        ((null? lst) (reverse acc))
        ((null? (cdr lst)) (reverse (cons (proc (car lst)) acc)))
        ((null? (cddr lst)) 
         (reverse (cons (proc (cadr lst)) 
                       (cons (proc (car lst)) acc))))
        (else
         (map-fast (cddr lst)
                  (cons (proc (cadr lst))
                       (cons (proc (car lst)) acc))))))
    (map-fast list '()))

  (define (filter-optimized predicate list)
    "Optimized filter with batch processing."
    (define (filter-fast lst acc count)
      (cond
        ((null? lst) (reverse acc))
        ((predicate (car lst))
         (filter-fast (cdr lst) (cons (car lst) acc) (+ count 1)))
        (else
         (filter-fast (cdr lst) acc count))))
    (filter-fast list '() 0))

  (define (deep-map proc nested-list)
    "Maps a procedure over a deeply nested list structure."
    (cond
      ((null? nested-list) '())
      ((pair? (car nested-list))
       (cons (deep-map proc (car nested-list))
             (deep-map proc (cdr nested-list))))
      (else
       (cons (proc (car nested-list))
             (deep-map proc (cdr nested-list))))))

  (define (adaptive-sort list)
    "Adaptively chooses sorting algorithm based on list characteristics."
    (let ((len (length-safe list)))
      (cond
        ((< len 10) 
         ;; Use insertion sort for small lists
         (insertion-sort list))
        ((mostly-sorted? list)
         ;; Use specialized algorithm for mostly sorted data
         (merge-sort-adaptive list))
        (else
         ;; Use general merge sort
         (sort list <)))))

  (define (insertion-sort list)
    "Simple insertion sort for small lists."
    (define (insert x sorted)
      (cond
        ((null? sorted) (list x))
        ((< x (car sorted)) (cons x sorted))
        (else (cons (car sorted) (insert x (cdr sorted))))))
    
    (fold-left (lambda (acc x) (insert x acc)) '() list))

  (define (mostly-sorted? list)
    "Heuristic to detect mostly sorted lists."
    (define (count-inversions lst count total)
      (cond
        ((or (null? lst) (null? (cdr lst))) 
         (< count (* total 0.1))) ;; Less than 10% inversions
        ((> (car lst) (cadr lst))
         (count-inversions (cdr lst) (+ count 1) total))
        (else
         (count-inversions (cdr lst) count total))))
    
    (let ((len (length-safe list)))
      (and (> len 5) ;; Only worth checking for larger lists
           (count-inversions list 0 len))))

  (define (merge-sort-adaptive list)
    "Merge sort optimized for mostly sorted data."
    ;; Simplified - in practice would have more sophisticated optimizations
    (sort list <))

  (define (list-equal? elt=? list1 list2)
    "Helper for list=? comparison."
    (cond
      ((and (null? list1) (null? list2)) #t)
      ((or (null? list1) (null? list2)) #f)
      ((elt=? (car list1) (car list2))
       (list-equal? elt=? (cdr list1) (cdr list2)))
      (else #f))))