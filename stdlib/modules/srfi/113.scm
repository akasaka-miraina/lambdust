;; SRFI-113: Sets and Bags
;;
;; This library provides immutable and mutable data structures called sets
;; and bags. A set contains any number of distinct elements; a bag is like
;; a set, but can contain duplicate elements.
;;
;; Reference: https://srfi.schemers.org/srfi-113/srfi-113.html

(define-library (srfi 113)
  (import (scheme base)
          (scheme case-lambda)
          (srfi 128))  ; Comparators
  
  (export
    ;; ============= SET PROCEDURES =============
    
    ;; Constructors
    set list->set
    
    ;; Predicates
    set? set-contains? set-empty? set-disjoint?
    
    ;; Accessors
    set-member set-element-at
    
    ;; Updaters
    set-adjoin set-adjoin! set-replace set-replace!
    set-delete set-delete! set-delete-all set-delete-all!
    set-search!
    
    ;; Whole set procedures
    set-size set-find set-count set-any? set-every?
    
    ;; Mapping and folding
    set-map set-for-each set-fold
    set-filter set-filter! set-remove set-remove!
    set-partition set-partition!
    
    ;; Copying and conversion
    set-copy set->list list->set set->bag
    
    ;; Subsets
    set=? set<? set>? set<=? set>=?
    
    ;; Set theory operations
    set-union set-intersection set-difference set-xor
    set-union! set-intersection! set-difference! set-xor!
    
    ;; ============= BAG PROCEDURES =============
    
    ;; Constructors  
    bag list->bag
    
    ;; Predicates
    bag? bag-contains? bag-empty? bag-disjoint?
    
    ;; Accessors
    bag-member bag-element-at bag-element-count
    
    ;; Updaters
    bag-adjoin bag-adjoin! bag-replace bag-replace!
    bag-delete bag-delete! bag-delete-all bag-delete-all!
    bag-increment! bag-decrement! bag-search!
    
    ;; Whole bag procedures
    bag-size bag-unique-size bag-find bag-count bag-any? bag-every?
    
    ;; Mapping and folding
    bag-map bag-for-each bag-fold
    bag-filter bag-filter! bag-remove bag-remove!
    bag-partition bag-partition!
    
    ;; Copying and conversion
    bag-copy bag->list list->bag bag->set
    
    ;; Bag theory operations
    bag-sum bag-sum! bag-product bag-product!
    bag-unique-size bag-element-count
    bag-union bag-intersection bag-difference
    bag-union! bag-intersection! bag-difference!
    
    ;; Comparison
    bag=? bag<? bag>? bag<=? bag>=?
    
    ;; ============= UNFOLD CONSTRUCTORS =============
    set-unfold bag-unfold
    )
  
  (begin
    
    ;; ============= INTERNAL UTILITIES =============
    
    ;; Default comparator for sets and bags
    (define *default-comparator* default-comparator)
    
    ;; Helper to get comparator from optional arguments
    (define (get-comparator args default-comp)
      (if (null? args)
          default-comp
          (car args)))
    
    ;; Helper for validation
    (define (validate-set obj proc-name)
      (unless (set? obj)
        (error proc-name "expected a set" obj)))
    
    (define (validate-bag obj proc-name)
      (unless (bag? obj)
        (error proc-name "expected a bag" obj)))
    
    ;; ============= SET PROCEDURES =============
    
    ;; Set constructors
    (define set 
      (case-lambda
        (() (primitive-set))
        (args (primitive-set args))))
    
    (define list->set
      (case-lambda
        ((lst) (primitive-list->set lst))
        ((lst comparator) (primitive-list->set lst comparator))))
    
    ;; Set predicates  
    (define (set? obj)
      (primitive-set? obj))
    
    (define (set-contains? set element)
      (validate-set set 'set-contains?)
      (primitive-set-contains? set element))
    
    (define (set-empty? set)
      (validate-set set 'set-empty?)
      (primitive-set-empty? set))
    
    (define (set-disjoint? set1 set2)
      (validate-set set1 'set-disjoint?)
      (validate-set set2 'set-disjoint?)
      (primitive-set-disjoint? set1 set2))
    
    ;; Set accessors
    (define (set-member set element default)
      (validate-set set 'set-member)
      (if (set-contains? set element)
          element
          default))
    
    (define (set-element-at set element)
      (validate-set set 'set-element-at)
      (set-member set element #f))
    
    ;; Set updaters
    (define set-adjoin
      (case-lambda
        ((set) set)
        ((set . elements)
         (validate-set set 'set-adjoin)
         (apply primitive-set-adjoin set elements))))
    
    (define set-adjoin!
      (case-lambda
        ((set) set)
        ((set . elements)
         (validate-set set 'set-adjoin!)
         (apply primitive-set-adjoin! set elements)
         set)))
    
    (define (set-replace set element)
      (validate-set set 'set-replace)
      (set-adjoin set element))
    
    (define (set-replace! set element)
      (validate-set set 'set-replace!)
      (set-adjoin! set element))
    
    (define set-delete
      (case-lambda
        ((set) set)
        ((set . elements)
         (validate-set set 'set-delete)
         (apply primitive-set-delete set elements))))
    
    (define set-delete!
      (case-lambda
        ((set) set)
        ((set . elements)
         (validate-set set 'set-delete!)
         (apply primitive-set-delete! set elements)
         set)))
    
    (define (set-delete-all set element-list)
      (validate-set set 'set-delete-all)
      (primitive-set-delete-all set element-list))
    
    (define (set-delete-all! set element-list)
      (validate-set set 'set-delete-all!)
      (primitive-set-delete-all! set element-list)
      set)
    
    ;; Set whole procedures
    (define (set-size set)
      (validate-set set 'set-size)
      (primitive-set-size set))
    
    (define (set-find predicate set failure)
      (validate-set set 'set-find)
      (call-with-current-continuation
        (lambda (return)
          (set-for-each 
            (lambda (element)
              (when (predicate element)
                (return element)))
            set)
          (failure))))
    
    (define (set-count predicate set)
      (validate-set set 'set-count)
      (set-fold
        (lambda (element count)
          (if (predicate element)
              (+ count 1)
              count))
        0
        set))
    
    (define (set-any? predicate set)
      (validate-set set 'set-any?)
      (call-with-current-continuation
        (lambda (return)
          (set-for-each
            (lambda (element)
              (when (predicate element)
                (return #t)))
            set)
          #f)))
    
    (define (set-every? predicate set)
      (validate-set set 'set-every?)
      (call-with-current-continuation
        (lambda (return)
          (set-for-each
            (lambda (element)
              (unless (predicate element)
                (return #f)))
            set)
          #t)))
    
    ;; Set mapping and folding (to be implemented with higher-order functions)
    (define set-map
      (case-lambda
        ((proc set)
         (validate-set set 'set-map)
         (primitive-set-map proc set))
        ((proc set comparator)
         (validate-set set 'set-map)
         (primitive-set-map proc set comparator))))
    
    (define (set-for-each proc set)
      (validate-set set 'set-for-each)
      (primitive-set-for-each proc set))
    
    (define (set-fold proc nil set)
      (validate-set set 'set-fold)
      (primitive-set-fold proc nil set))
    
    (define (set-filter predicate set)
      (validate-set set 'set-filter)
      (primitive-set-filter predicate set))
    
    (define (set-filter! predicate set)
      (validate-set set 'set-filter!)
      (primitive-set-filter! predicate set)
      set)
    
    (define (set-remove predicate set)
      (validate-set set 'set-remove)
      (primitive-set-remove predicate set))
    
    (define (set-remove! predicate set)
      (validate-set set 'set-remove!)
      (primitive-set-remove! predicate set)
      set)
    
    (define (set-partition predicate set)
      (validate-set set 'set-partition)
      (primitive-set-partition predicate set))
    
    (define (set-partition! predicate set)
      (validate-set set 'set-partition!)
      (primitive-set-partition! predicate set))
    
    ;; Set copying and conversion
    (define (set-copy set)
      (validate-set set 'set-copy)
      (primitive-set-copy set))
    
    (define (set->list set)
      (validate-set set 'set->list)
      (primitive-set->list set))
    
    (define (set->bag set)
      (validate-set set 'set->bag)
      (primitive-set->bag set))
    
    ;; Set comparison
    (define (set=? . sets)
      (apply primitive-set=? sets))
    
    (define (set<? . sets)
      (apply primitive-set<? sets))
    
    (define (set>? . sets)
      (apply primitive-set>? sets))
    
    (define (set<=? . sets)
      (apply primitive-set<=? sets))
    
    (define (set>=? . sets)
      (apply primitive-set>=? sets))
    
    ;; Set theory operations
    (define (set-union . sets)
      (apply primitive-set-union sets))
    
    (define (set-intersection . sets)
      (apply primitive-set-intersection sets))
    
    (define (set-difference . sets)
      (apply primitive-set-difference sets))
    
    (define (set-xor set1 set2)
      (primitive-set-xor set1 set2))
    
    (define (set-union! . sets)
      (apply primitive-set-union! sets)
      (if (null? sets) #f (car sets)))
    
    (define (set-intersection! . sets)
      (apply primitive-set-intersection! sets)
      (if (null? sets) #f (car sets)))
    
    (define (set-difference! . sets)
      (apply primitive-set-difference! sets)
      (if (null? sets) #f (car sets)))
    
    (define (set-xor! set1 set2)
      (primitive-set-xor! set1 set2)
      set1)
    
    ;; Set unfold constructor
    (define set-unfold
      (case-lambda
        ((stop? mapper successor seed)
         (primitive-set-unfold stop? mapper successor seed))
        ((stop? mapper successor seed comparator)
         (primitive-set-unfold stop? mapper successor seed comparator))))
    
    ;; Set search (to be implemented)
    (define (set-search! set element failure success)
      (validate-set set 'set-search!)
      (if (set-contains? set element)
          (success element 
                   (lambda (new-element) (set-adjoin! set new-element))
                   (lambda () (set-delete! set element)))
          (failure (lambda (new-element) (set-adjoin! set new-element)))))
    
    ;; Set copying and conversion
    (define (set-copy set)
      (validate-set set 'set-copy)
      (primitive-set-copy set))
    
    (define (set->list set)
      (validate-set set 'set->list)
      (primitive-set->list set))
    
    (define list->set
      (case-lambda
        ((list)
         (primitive-list->set list))
        ((list comparator)
         (primitive-list->set list comparator))))
    
    ;; ============= BAG PROCEDURES =============
    
    ;; Bag constructors
    (define bag
      (case-lambda
        (() (primitive-bag))
        (args (apply primitive-bag args))))
    
    (define list->bag
      (case-lambda
        ((lst) (primitive-list->bag lst))
        ((lst comparator) (primitive-list->bag lst comparator))))
    
    ;; Bag predicates
    (define (bag? obj)
      (primitive-bag? obj))
    
    (define (bag-contains? bag element)
      (validate-bag bag 'bag-contains?)
      (primitive-bag-contains? bag element))
    
    (define (bag-empty? bag)
      (validate-bag bag 'bag-empty?)
      (primitive-bag-empty? bag))
    
    (define (bag-disjoint? bag1 bag2)
      (validate-bag bag1 'bag-disjoint?)
      (validate-bag bag2 'bag-disjoint?)
      (primitive-bag-disjoint? bag1 bag2))
    
    ;; Bag accessors
    (define (bag-member bag element default)
      (validate-bag bag 'bag-member)
      (if (bag-contains? bag element)
          element
          default))
    
    (define (bag-element-at bag element)
      (validate-bag bag 'bag-element-at)
      (bag-member bag element #f))
    
    (define (bag-element-count bag element)
      (validate-bag bag 'bag-element-count)
      (primitive-bag-element-count bag element))
    
    ;; Bag updaters
    (define bag-adjoin
      (case-lambda
        ((bag) bag)
        ((bag . elements)
         (validate-bag bag 'bag-adjoin)
         (apply primitive-bag-adjoin bag elements))))
    
    (define bag-adjoin!
      (case-lambda
        ((bag) bag)
        ((bag . elements)
         (validate-bag bag 'bag-adjoin!)
         (apply primitive-bag-adjoin! bag elements)
         bag)))
    
    (define (bag-replace bag element)
      (validate-bag bag 'bag-replace)
      (bag-adjoin bag element))
    
    (define (bag-replace! bag element)
      (validate-bag bag 'bag-replace!)
      (bag-adjoin! bag element))
    
    (define bag-delete
      (case-lambda
        ((bag) bag)
        ((bag . elements)
         (validate-bag bag 'bag-delete)
         (apply primitive-bag-delete bag elements))))
    
    (define bag-delete!
      (case-lambda
        ((bag) bag)
        ((bag . elements)
         (validate-bag bag 'bag-delete!)
         (apply primitive-bag-delete! bag elements)
         bag)))
    
    (define (bag-delete-all bag element)
      (validate-bag bag 'bag-delete-all)
      (primitive-bag-delete-all bag element))
    
    (define (bag-delete-all! bag element)
      (validate-bag bag 'bag-delete-all!)
      (primitive-bag-delete-all! bag element)
      bag)
    
    (define (bag-increment! bag element count)
      (validate-bag bag 'bag-increment!)
      (primitive-bag-increment! bag element count)
      bag)
    
    (define (bag-decrement! bag element count)
      (validate-bag bag 'bag-decrement!)
      (primitive-bag-decrement! bag element count)
      bag)
    
    ;; Bag whole procedures
    (define (bag-size bag)
      (validate-bag bag 'bag-size)
      (primitive-bag-size bag))
    
    (define (bag-unique-size bag)
      (validate-bag bag 'bag-unique-size)
      (primitive-bag-unique-size bag))
    
    (define (bag-find predicate bag failure)
      (validate-bag bag 'bag-find)
      (call-with-current-continuation
        (lambda (return)
          (bag-for-each 
            (lambda (element)
              (when (predicate element)
                (return element)))
            bag)
          (failure))))
    
    (define (bag-count predicate bag)
      (validate-bag bag 'bag-count)
      (bag-fold
        (lambda (element count)
          (if (predicate element)
              (+ count 1)
              count))
        0
        bag))
    
    (define (bag-any? predicate bag)
      (validate-bag bag 'bag-any?)
      (call-with-current-continuation
        (lambda (return)
          (bag-for-each
            (lambda (element)
              (when (predicate element)
                (return #t)))
            bag)
          #f)))
    
    (define (bag-every? predicate bag)
      (validate-bag bag 'bag-every?)
      (call-with-current-continuation
        (lambda (return)
          (bag-for-each
            (lambda (element)
              (unless (predicate element)
                (return #f)))
            bag)
          #t)))
    
    ;; Bag mapping and folding (to be implemented with higher-order functions)
    (define bag-map
      (case-lambda
        ((proc bag)
         (validate-bag bag 'bag-map)
         (primitive-bag-map proc bag))
        ((proc bag comparator)
         (validate-bag bag 'bag-map)
         (primitive-bag-map proc bag comparator))))
    
    (define (bag-for-each proc bag)
      (validate-bag bag 'bag-for-each)
      (primitive-bag-for-each proc bag))
    
    (define (bag-fold proc nil bag)
      (validate-bag bag 'bag-fold)
      (primitive-bag-fold proc nil bag))
    
    (define (bag-filter predicate bag)
      (validate-bag bag 'bag-filter)
      (primitive-bag-filter predicate bag))
    
    (define (bag-filter! predicate bag)
      (validate-bag bag 'bag-filter!)
      (primitive-bag-filter! predicate bag)
      bag)
    
    (define (bag-remove predicate bag)
      (validate-bag bag 'bag-remove)
      (primitive-bag-remove predicate bag))
    
    (define (bag-remove! predicate bag)
      (validate-bag bag 'bag-remove!)
      (primitive-bag-remove! predicate bag)
      bag)
    
    (define (bag-partition predicate bag)
      (validate-bag bag 'bag-partition)
      (primitive-bag-partition predicate bag))
    
    (define (bag-partition! predicate bag)
      (validate-bag bag 'bag-partition!)
      (primitive-bag-partition! predicate bag))
    
    ;; Bag copying and conversion
    (define (bag-copy bag)
      (validate-bag bag 'bag-copy)
      (primitive-bag-copy bag))
    
    (define (bag->list bag)
      (validate-bag bag 'bag->list)
      (primitive-bag->list bag))
    
    (define (bag->set bag)
      (validate-bag bag 'bag->set)
      (primitive-bag->set bag))
    
    (define list->bag
      (case-lambda
        ((list)
         (primitive-list->bag list))
        ((list comparator)
         (primitive-list->bag list comparator))))
    
    (define (set->bag set)
      (validate-set set 'set->bag)
      (primitive-set->bag set))
    
    ;; Bag theory operations
    (define (bag-sum . bags)
      (apply primitive-bag-sum bags))
    
    (define (bag-sum! . bags)
      (apply primitive-bag-sum! bags)
      (if (null? bags) #f (car bags)))
    
    (define (bag-product . bags)
      (apply primitive-bag-product bags))
    
    (define (bag-product! . bags)
      (apply primitive-bag-product! bags)
      (if (null? bags) #f (car bags)))
    
    (define (bag-union . bags)
      (apply primitive-bag-union bags))
    
    (define (bag-intersection . bags)
      (apply primitive-bag-intersection bags))
    
    (define (bag-difference . bags)
      (apply primitive-bag-difference bags))
    
    (define (bag-union! . bags)
      (apply primitive-bag-union! bags)
      (if (null? bags) #f (car bags)))
    
    (define (bag-intersection! . bags)
      (apply primitive-bag-intersection! bags)
      (if (null? bags) #f (car bags)))
    
    (define (bag-difference! . bags)
      (apply primitive-bag-difference! bags)
      (if (null? bags) #f (car bags)))
    
    ;; Bag comparison
    (define (bag=? . bags)
      (apply primitive-bag=? bags))
    
    (define (bag<? . bags)
      (apply primitive-bag<? bags))
    
    (define (bag>? . bags)
      (apply primitive-bag>? bags))
    
    (define (bag<=? . bags)
      (apply primitive-bag<=? bags))
    
    (define (bag>=? . bags)
      (apply primitive-bag>=? bags))
    
    ;; Bag unfold constructor
    (define bag-unfold
      (case-lambda
        ((stop? mapper successor seed)
         (primitive-bag-unfold stop? mapper successor seed))
        ((stop? mapper successor seed comparator)
         (primitive-bag-unfold stop? mapper successor seed comparator))))
    
    ;; Bag search (to be implemented)
    (define (bag-search! bag element failure success)
      (validate-bag bag 'bag-search!)
      (if (bag-contains? bag element)
          (success element 
                   (lambda (new-element) (bag-adjoin! bag new-element))
                   (lambda () (bag-delete! bag element)))
          (failure (lambda (new-element) (bag-adjoin! bag new-element)))))
    
    )) ; end define-library