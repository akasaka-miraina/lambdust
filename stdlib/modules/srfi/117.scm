;; SRFI-117: Queues
;; 
;; This library provides mutable queues (FIFOs) and list queues.
;; Queues are useful for implementing breadth-first algorithms,
;; producer-consumer patterns, and maintaining ordered processing.
;;
;; Reference: https://srfi.schemers.org/srfi-117/srfi-117.html

(define-library (srfi 117)
  (import (scheme base)
          (srfi 1))  ; For list operations
  
  (export
    ;; === Queue Type ===
    make-queue queue queue?
    
    ;; === List Queue Type ===
    make-list-queue list-queue list-queue?
    
    ;; === Common Queue Operations ===
    queue-empty? queue-front queue-back queue-length
    
    ;; === Mutating Operations ===
    queue-push-front! queue-push-back! queue-pop-front! queue-pop-back!
    
    ;; === List Queue Operations ===
    list-queue-list list-queue-set-list!
    
    ;; === Conversion Operations ===
    queue->list list->queue queue->list-queue list-queue->queue)

  (begin
    ;; ============= INTERNAL QUEUE REPRESENTATION =============
    
    ;; Queue representation: vector with front-list, back-list, and length
    ;; front-list: items to be removed from front
    ;; back-list: items added to back (in reverse order)
    ;; This representation provides amortized O(1) operations
    
    (define queue-type-tag 'queue)
    (define list-queue-type-tag 'list-queue)
    
    ;; Internal queue constructor
    ;; Format: #(queue-tag front-list back-list length)
    (define (%make-queue front back length)
      (vector queue-type-tag front back length))
    
    ;; Internal list queue constructor  
    ;; Format: #(list-queue-tag list)
    (define (%make-list-queue list)
      (vector list-queue-type-tag list))
    
    ;; ============= QUEUE CONSTRUCTORS =============
    
    ;; Create a new empty queue
    ;; Usage: (make-queue) -> queue
    (define (make-queue)
      "Create a new empty queue."
      (%make-queue '() '() 0))
    
    ;; Create a queue with initial elements
    ;; Usage: (queue element ...) -> queue
    (define (queue . elements)
      "Create a queue containing the given elements in order."
      (list->queue elements))
    
    ;; Create a new empty list queue
    ;; Usage: (make-list-queue list) -> list-queue
    (define (make-list-queue list)
      "Create a list queue containing the elements of list."
      (unless (list? list)
        (error "make-list-queue: argument must be a list" list))
      (%make-list-queue (list-copy list)))
    
    ;; Create a list queue with initial elements
    ;; Usage: (list-queue element ...) -> list-queue
    (define (list-queue . elements)
      "Create a list queue containing the given elements in order."
      (%make-list-queue elements))
    
    ;; ============= TYPE PREDICATES =============
    
    ;; Test if object is a queue
    ;; Usage: (queue? obj) -> boolean
    (define (queue? obj)
      "Test whether obj is a queue."
      (and (vector? obj)
           (= (vector-length obj) 4)
           (eq? (vector-ref obj 0) queue-type-tag)))
    
    ;; Test if object is a list queue
    ;; Usage: (list-queue? obj) -> boolean
    (define (list-queue? obj)
      "Test whether obj is a list queue."
      (and (vector? obj)
           (= (vector-length obj) 2)
           (eq? (vector-ref obj 0) list-queue-type-tag)))
    
    ;; ============= INTERNAL ACCESSORS =============
    
    (define (%queue-front q) (vector-ref q 1))
    (define (%queue-back q) (vector-ref q 2))
    (define (%queue-length q) (vector-ref q 3))
    (define (%queue-set-front! q front) (vector-set! q 1 front))
    (define (%queue-set-back! q back) (vector-set! q 2 back))
    (define (%queue-set-length! q length) (vector-set! q 3 length))
    
    (define (%list-queue-list q) (vector-ref q 1))
    (define (%list-queue-set-list! q list) (vector-set! q 1 list))
    
    ;; ============= QUEUE INSPECTION =============
    
    ;; Test if queue is empty
    ;; Usage: (queue-empty? queue) -> boolean
    (define (queue-empty? q)
      "Test whether queue is empty."
      (cond
        ((queue? q) (= (%queue-length q) 0))
        ((list-queue? q) (null? (%list-queue-list q)))
        (else (error "queue-empty?: not a queue" q))))
    
    ;; Get front element without removing it
    ;; Usage: (queue-front queue) -> any
    (define (queue-front q)
      "Return the front element of queue without removing it."
      (cond
        ((queue? q)
         (when (queue-empty? q)
           (error "queue-front: queue is empty" q))
         (%ensure-front-available! q)
         (car (%queue-front q)))
        ((list-queue? q)
         (let ((list (%list-queue-list q)))
           (when (null? list)
             (error "queue-front: queue is empty" q))
           (car list)))
        (else (error "queue-front: not a queue" q))))
    
    ;; Get back element without removing it
    ;; Usage: (queue-back queue) -> any
    (define (queue-back q)
      "Return the back element of queue without removing it."
      (cond
        ((queue? q)
         (when (queue-empty? q)
           (error "queue-back: queue is empty" q))
         (let ((back (%queue-back q))
               (front (%queue-front q)))
           (cond
             ((not (null? back)) (car back))
             ((not (null? front)) (last front))
             (else (error "queue-back: internal error" q)))))
        ((list-queue? q)
         (let ((list (%list-queue-list q)))
           (when (null? list)
             (error "queue-back: queue is empty" q))
           (last list)))
        (else (error "queue-back: not a queue" q))))
    
    ;; Get queue length
    ;; Usage: (queue-length queue) -> integer
    (define (queue-length q)
      "Return the number of elements in queue."
      (cond
        ((queue? q) (%queue-length q))
        ((list-queue? q) (length (%list-queue-list q)))
        (else (error "queue-length: not a queue" q))))
    
    ;; ============= INTERNAL QUEUE MANAGEMENT =============
    
    ;; Ensure front list has elements when queue is not empty
    (define (%ensure-front-available! q)
      (when (and (null? (%queue-front q))
                 (not (null? (%queue-back q))))
        (%queue-set-front! q (reverse (%queue-back q)))
        (%queue-set-back! q '())))
    
    ;; ============= MUTATING OPERATIONS =============
    
    ;; Add element to front of queue
    ;; Usage: (queue-push-front! queue element) -> unspecified
    (define (queue-push-front! q element)
      "Add element to the front of queue."
      (cond
        ((queue? q)
         (%queue-set-front! q (cons element (%queue-front q)))
         (%queue-set-length! q (+ (%queue-length q) 1)))
        ((list-queue? q)
         (%list-queue-set-list! q (cons element (%list-queue-list q))))
        (else (error "queue-push-front!: not a queue" q))))
    
    ;; Add element to back of queue  
    ;; Usage: (queue-push-back! queue element) -> unspecified
    (define (queue-push-back! q element)
      "Add element to the back of queue."
      (cond
        ((queue? q)
         (%queue-set-back! q (cons element (%queue-back q)))
         (%queue-set-length! q (+ (%queue-length q) 1)))
        ((list-queue? q)
         (%list-queue-set-list! q (append (%list-queue-list q) (list element))))
        (else (error "queue-push-back!: not a queue" q))))
    
    ;; Remove and return front element
    ;; Usage: (queue-pop-front! queue) -> any
    (define (queue-pop-front! q)
      "Remove and return the front element of queue."
      (cond
        ((queue? q)
         (when (queue-empty? q)
           (error "queue-pop-front!: queue is empty" q))
         (%ensure-front-available! q)
         (let ((element (car (%queue-front q))))
           (%queue-set-front! q (cdr (%queue-front q)))
           (%queue-set-length! q (- (%queue-length q) 1))
           element))
        ((list-queue? q)
         (let ((list (%list-queue-list q)))
           (when (null? list)
             (error "queue-pop-front!: queue is empty" q))
           (let ((element (car list)))
             (%list-queue-set-list! q (cdr list))
             element)))
        (else (error "queue-pop-front!: not a queue" q))))
    
    ;; Remove and return back element
    ;; Usage: (queue-pop-back! queue) -> any
    (define (queue-pop-back! q)
      "Remove and return the back element of queue."
      (cond
        ((queue? q)
         (when (queue-empty? q)
           (error "queue-pop-back!: queue is empty" q))
         (let ((back (%queue-back q)))
           (cond
             ((not (null? back))
              ;; Back list has elements, remove from there
              (let ((element (car back)))
                (%queue-set-back! q (cdr back))
                (%queue-set-length! q (- (%queue-length q) 1))
                element))
             (else
              ;; Back list empty, remove from end of front list
              (let* ((front (%queue-front q))
                     (new-front (reverse (cdr (reverse front))))
                     (element (last front)))
                (%queue-set-front! q new-front)
                (%queue-set-length! q (- (%queue-length q) 1))
                element)))))
        ((list-queue? q)
         (let ((list (%list-queue-list q)))
           (when (null? list)
             (error "queue-pop-back!: queue is empty" q))
           (let* ((rev-list (reverse list))
                  (element (car rev-list))
                  (new-list (reverse (cdr rev-list))))
             (%list-queue-set-list! q new-list)
             element)))
        (else (error "queue-pop-back!: not a queue" q))))
    
    ;; ============= LIST QUEUE SPECIFIC OPERATIONS =============
    
    ;; Get underlying list from list queue
    ;; Usage: (list-queue-list list-queue) -> list
    (define (list-queue-list q)
      "Return the list underlying the list queue."
      (if (list-queue? q)
          (list-copy (%list-queue-list q))
          (error "list-queue-list: not a list queue" q)))
    
    ;; Set underlying list of list queue
    ;; Usage: (list-queue-set-list! list-queue list) -> unspecified
    (define (list-queue-set-list! q list)
      "Set the list underlying the list queue."
      (cond
        ((not (list-queue? q))
         (error "list-queue-set-list!: not a list queue" q))
        ((not (list? list))
         (error "list-queue-set-list!: second argument must be a list" list))
        (else
         (%list-queue-set-list! q (list-copy list)))))
    
    ;; ============= CONVERSION OPERATIONS =============
    
    ;; Convert queue to list
    ;; Usage: (queue->list queue) -> list
    (define (queue->list q)
      "Convert queue to a list with same elements in same order."
      (cond
        ((queue? q)
         (append (%queue-front q) (reverse (%queue-back q))))
        ((list-queue? q)
         (list-copy (%list-queue-list q)))
        (else (error "queue->list: not a queue" q))))
    
    ;; Convert list to queue
    ;; Usage: (list->queue list) -> queue
    (define (list->queue list)
      "Convert list to a queue with same elements in same order."
      (unless (list? list)
        (error "list->queue: argument must be a list" list))
      (%make-queue list '() (length list)))
    
    ;; Convert queue to list queue
    ;; Usage: (queue->list-queue queue) -> list-queue
    (define (queue->list-queue q)
      "Convert queue to a list queue with same elements in same order."
      (if (queue? q)
          (%make-list-queue (queue->list q))
          (error "queue->list-queue: not a queue" q)))
    
    ;; Convert list queue to queue
    ;; Usage: (list-queue->queue list-queue) -> queue
    (define (list-queue->queue q)
      "Convert list queue to a queue with same elements in same order."
      (if (list-queue? q)
          (list->queue (%list-queue-list q))
          (error "list-queue->queue: not a list queue" q)))
    
    ;; ============= USAGE EXAMPLES =============
    
    ;; Example 1: Basic queue operations
    ;; (define q (make-queue))
    ;; (queue-push-back! q 'a)
    ;; (queue-push-back! q 'b) 
    ;; (queue-push-front! q 'x)
    ;; (queue->list q)  ; => (x a b)
    ;; (queue-pop-front! q)  ; => x
    ;; (queue-pop-back! q)   ; => b
    ;; (queue->list q)  ; => (a)
    
    ;; Example 2: List queue operations
    ;; (define lq (list-queue 1 2 3))
    ;; (queue-push-back! lq 4)
    ;; (list-queue-list lq)  ; => (1 2 3 4)
    ;; (queue-pop-front! lq)  ; => 1
    ;; (list-queue-list lq)  ; => (2 3 4)
    
    ;; Example 3: Queue conversions
    ;; (define q1 (queue 'a 'b 'c))
    ;; (define lq1 (queue->list-queue q1))
    ;; (define q2 (list-queue->queue lq1))
    ;; (equal? (queue->list q1) (queue->list q2))  ; => #t
    
    ;; Example 4: BFS traversal pattern
    ;; (define (bfs-traverse graph start)
    ;;   (let ((visited (make-hash-table))
    ;;         (queue (queue start)))
    ;;     (let loop ((result '()))
    ;;       (if (queue-empty? queue)
    ;;           (reverse result)
    ;;           (let ((node (queue-pop-front! queue)))
    ;;             (if (hash-table-contains? visited node)
    ;;                 (loop result)
    ;;                 (begin
    ;;                   (hash-table-set! visited node #t)
    ;;                   (for-each (lambda (neighbor)
    ;;                               (queue-push-back! queue neighbor))
    ;;                             (graph-neighbors graph node))
    ;;                   (loop (cons node result)))))))))
    
    ;; Example 5: Producer-consumer pattern
    ;; (define (make-buffer capacity)
    ;;   (let ((queue (make-queue))
    ;;         (max-size capacity))
    ;;     (lambda (op . args)
    ;;       (case op
    ;;         ((produce) 
    ;;          (when (>= (queue-length queue) max-size)
    ;;            (error "Buffer full"))
    ;;          (queue-push-back! queue (car args)))
    ;;         ((consume)
    ;;          (when (queue-empty? queue)
    ;;            (error "Buffer empty"))
    ;;          (queue-pop-front! queue))
    ;;         ((size) (queue-length queue))
    ;;         ((full?) (>= (queue-length queue) max-size))
    ;;         ((empty?) (queue-empty? queue))))))
    
    ;; ============= PERFORMANCE NOTES =============
    
    ;; Queue performance characteristics:
    ;; - queue-push-front!: O(1)
    ;; - queue-push-back!: O(1) 
    ;; - queue-pop-front!: Amortized O(1), worst case O(n)
    ;; - queue-pop-back!: O(1) for recent pushes, O(n) otherwise
    ;; - queue-length: O(1)
    ;; - queue-front/queue-back: O(1) amortized
    
    ;; List queue performance characteristics:
    ;; - queue-push-front!: O(1)
    ;; - queue-push-back!: O(n) - requires list traversal
    ;; - queue-pop-front!: O(1)
    ;; - queue-pop-back!: O(n) - requires list reversal
    ;; - queue-length: O(n) - requires list traversal
    
    ;; Choose queue for general use, list-queue when you need direct list access
    
    ;; ============= ALGORITHM NOTES =============
    
    ;; The queue implementation uses a two-list approach:
    ;; - Front list: elements ready for dequeue operations
    ;; - Back list: elements added via enqueue operations (stored in reverse)
    ;; 
    ;; When front list becomes empty, back list is reversed and becomes new front.
    ;; This provides amortized O(1) operations for typical FIFO usage patterns.
    
    ;; The list queue provides direct access to underlying list structure,
    ;; useful when queue needs to integrate with list-based algorithms.
    ;; However, it sacrifices performance for back operations.
    
    ;; ============= ERROR HANDLING =============
    
    ;; All operations include proper error checking:
    ;; - Type errors for non-queue arguments
    ;; - Empty queue errors for pop/front/back operations  
    ;; - List type errors for list-queue operations
    ;; - Informative error messages for debugging
    
    ;; ============= THREAD SAFETY NOTES =============
    
    ;; Current implementation is single-threaded
    ;; For multi-threaded use, external synchronization is required
    ;; Future versions may provide built-in thread safety options
    
    ))