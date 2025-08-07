;; Lambdust R7RS-Compliant I/O Module
;; Provides complete R7RS Section 6.13 input/output operations
;; Implements textual and binary I/O, port operations, and file system utilities

(define-module (:: io)
  (metadata
    (version "2.0.0")
    (description "R7RS-compliant input/output operations with port-based I/O")
    (author "Lambdust Core Team")
    (r7rs-compliance "6.13"))
  
  (export 
    ;; === R7RS Section 6.13.1 Port Predicates ===
    input-port? output-port? textual-port? binary-port?
    port? port-open?
    
    ;; === R7RS Section 6.13.2 Current Ports ===
    current-input-port current-output-port current-error-port
    
    ;; === R7RS Section 6.13.3 File I/O ===
    open-input-file open-output-file open-binary-input-file open-binary-output-file
    close-port close-input-port close-output-port
    with-input-from-file with-output-to-file
    call-with-input-file call-with-output-file
    call-with-port
    
    ;; === R7RS Section 6.13.4 String I/O ===
    open-input-string open-output-string get-output-string
    open-input-bytevector open-output-bytevector get-output-bytevector
    
    ;; === R7RS Section 6.13.5 Input Operations ===
    read read-char peek-char read-line read-string read-u8 peek-u8
    read-bytevector read-bytevector!
    char-ready? u8-ready? eof-object eof-object?
    
    ;; === R7RS Section 6.13.6 Output Operations ===
    write write-char write-string write-u8 newline display
    write-bytevector flush-output-port
    
    ;; === Additional Lambdust Extensions ===
    ;; (Beyond R7RS but commonly useful)
    
    ;; Format operations
    format fprintf sprintf
    
    ;; File system operations
    file-exists? delete-file rename-file
    
    ;; I/O utilities
    copy-port port-position set-port-position!
    port-has-port-position? port-has-set-port-position!?
    
    ;; Reading utilities
    read-all read-file write-file read-string-all)

  ;; ============= R7RS Port Operations =============

  (define (input-port? obj)
    "Returns #t if obj is an input port.
     
     R7RS: (input-port? obj) procedure
     Returns #t if obj is an input port, otherwise returns #f."
    (builtin:input-port? obj))

  (define (output-port? obj)
    "Returns #t if obj is an output port.
     
     R7RS: (output-port? obj) procedure
     Returns #t if obj is an output port, otherwise returns #f."
    (builtin:output-port? obj))

  (define (textual-port? obj)
    "Returns #t if obj is a textual port.
     
     R7RS: (textual-port? obj) procedure
     Returns #t if obj is a textual port, otherwise returns #f.
     A textual port supports textual input and output operations."
    (builtin:textual-port? obj))

  (define (binary-port? obj)
    "Returns #t if obj is a binary port.
     
     R7RS: (binary-port? obj) procedure
     Returns #t if obj is a binary port, otherwise returns #f.
     A binary port supports binary input and output operations."
    (builtin:binary-port? obj))

  (define (port? obj)
    "Returns #t if obj is a port.
     
     R7RS: (port? obj) procedure
     Returns #t if obj is either an input port or an output port,
     otherwise returns #f."
    (or (input-port? obj) (output-port? obj)))

  (define (port-open? port)
    "Returns #t if port is still open and capable of I/O operations.
     
     R7RS: (port-open? port) procedure
     Returns #t if port is still open and capable of performing I/O,
     and #f otherwise."
    (builtin:port-open? port))

  (define (close-port port)
    "Closes port, rendering it incapable of I/O operations.
     
     R7RS: (close-port port) procedure
     Closes the resource associated with port, rendering the port
     incapable of delivering or accepting data. It is an error to
     apply close-port to a port that is not open."
    (builtin:close-port port))

  (define (close-input-port port)
    "Closes input port.
     
     R7RS: (close-input-port port) procedure
     Closes the resource associated with port, rendering the port
     incapable of delivering data. It is an error to apply
     close-input-port to a port that is not an open input port."
    (builtin:close-input-port port))

  (define (close-output-port port)
    "Closes output port.
     
     R7RS: (close-output-port port) procedure
     Closes the resource associated with port, rendering the port
     incapable of accepting data. It is an error to apply
     close-output-port to a port that is not an open output port."
    (builtin:close-output-port port))

  ;; ============= R7RS Current Ports =============

  (define (current-input-port)
    "Returns the current default input port.
     
     R7RS: (current-input-port) procedure
     Returns the current default input port. The current default
     input port is the input port used by many input procedures
     when no port argument is supplied."
    (builtin:current-input-port))

  (define (current-output-port)
    "Returns the current default output port.
     
     R7RS: (current-output-port) procedure
     Returns the current default output port. The current default
     output port is the output port used by many output procedures
     when no port argument is supplied."
    (builtin:current-output-port))

  (define (current-error-port)
    "Returns the current default error port.
     
     R7RS: (current-error-port) procedure
     Returns the current default error port. The current default
     error port is the output port used by error reporting
     procedures when no port argument is supplied."
    (builtin:current-error-port))

  ;; ============= R7RS File I/O Operations =============

  (define (open-input-file filename)
    "Opens a textual input port for the named file.
     
     R7RS: (open-input-file filename) file library procedure
     Takes a string naming an existing file and returns a textual
     input port that is capable of delivering data from the file.
     If the file does not exist or cannot be opened, an error that
     satisfies file-error? is signaled."
    (builtin:open-input-file filename))

  (define (open-output-file filename)
    "Opens a textual output port for the named file.
     
     R7RS: (open-output-file filename) file library procedure
     Takes a string naming an output file to be created and returns a
     textual output port that is capable of writing data to a new file
     by that name. If a file with the given name already exists, the
     effect is unspecified. If the file cannot be opened, an error that
     satisfies file-error? is signaled."
    (builtin:open-output-file filename))

  (define (open-binary-input-file filename)
    "Opens a binary input port for the named file.
     
     R7RS: (open-binary-input-file filename) file library procedure
     Takes a string naming an existing file and returns a binary input
     port that is capable of delivering data from the file. If the file
     does not exist or cannot be opened, an error that satisfies
     file-error? is signaled."
    (builtin:open-binary-input-file filename))

  (define (open-binary-output-file filename)
    "Opens a binary output port for the named file.
     
     R7RS: (open-binary-output-file filename) file library procedure
     Takes a string naming an output file to be created and returns a
     binary output port that is capable of writing data to a new file
     by that name. If a file with the given name already exists, the
     effect is unspecified. If the file cannot be opened, an error that
     satisfies file-error? is signaled."
    (builtin:open-binary-output-file filename))

  (define (with-input-from-file filename thunk)
    "Temporarily redirects current input to read from filename.
     
     R7RS: (with-input-from-file filename thunk) file library procedure
     Filename should be a string naming a file, and proc should be a
     procedure that accepts no arguments. The file is opened for input,
     an input port connected to it is made the default value returned
     by current-input-port, and thunk is called with no arguments.
     When thunk returns, the port is closed and the previous default
     is restored."
    (let ((old-port (current-input-port))
          (port (open-input-file filename)))
      (dynamic-wind
        (lambda () (builtin:set-current-input-port! port))
        thunk
        (lambda () 
          (close-input-port port)
          (builtin:set-current-input-port! old-port)))))

  (define (with-output-to-file filename thunk)
    "Temporarily redirects current output to write to filename.
     
     R7RS: (with-output-to-file filename thunk) file library procedure
     Filename should be a string naming a file, and proc should be a
     procedure that accepts no arguments. The file is opened for output,
     an output port connected to it is made the default value returned
     by current-output-port, and thunk is called with no arguments.
     When thunk returns, the port is closed and the previous default
     is restored."
    (let ((old-port (current-output-port))
          (port (open-output-file filename)))
      (dynamic-wind
        (lambda () (builtin:set-current-output-port! port))
        thunk
        (lambda () 
          (close-output-port port)
          (builtin:set-current-output-port! old-port)))))

  (define (call-with-input-file filename proc)
    "Calls proc with an input port for the named file.
     
     R7RS: (call-with-input-file filename proc) file library procedure
     Filename should be a string naming a file, and proc should be a
     procedure that accepts one argument. The file is opened for input,
     an input port connected to it is passed to proc. When proc returns,
     the port is closed and the value(s) yielded by proc are returned."
    (let ((port (open-input-file filename)))
      (call-with-port port proc)))

  (define (call-with-output-file filename proc)
    "Calls proc with an output port for the named file.
     
     R7RS: (call-with-output-file filename proc) file library procedure
     Filename should be a string naming a file, and proc should be a
     procedure that accepts one argument. The file is opened for output,
     an output port connected to it is passed to proc. When proc returns,
     the port is closed and the value(s) yielded by proc are returned."
    (let ((port (open-output-file filename)))
      (call-with-port port proc)))

  (define (call-with-port port proc)
    "Calls proc with port, ensuring the port is closed afterwards.
     
     R7RS: (call-with-port port proc) procedure
     Calls proc with port as an argument. If proc returns, then the
     port is closed automatically and the value(s) yielded by proc
     are returned. If proc does not return, then the port will not
     be closed automatically, unless it is possible to prove that
     the port will never again be used for a read or write operation."
    (dynamic-wind
      (lambda () #f)
      (lambda () (proc port))
      (lambda () (close-port port))))

  ;; ============= R7RS String Ports =============

  (define (open-input-string string)
    "Opens a textual input port that reads from the given string.
     
     R7RS: (open-input-string string) procedure
     Takes a string and returns a textual input port that delivers
     characters from the string. If the string is modified after
     the port is created, the effect is unspecified."
    (builtin:open-input-string string))

  (define (open-output-string)
    "Opens a textual output port that accumulates characters into a string.
     
     R7RS: (open-output-string) procedure
     Returns a textual output port that will accumulate characters for
     retrieval by get-output-string. The port should be closed
     after use, although it is safe to continue using it after
     calling get-output-string."
    (builtin:open-output-string))

  (define (get-output-string port)
    "Retrieves the characters accumulated in the given output string port.
     
     R7RS: (get-output-string port) procedure
     Given an output port created by open-output-string, returns a
     string consisting of the characters that have been output to
     the port so far. After calling get-output-string, the port
     will still accept new output but will begin accumulating again
     from an empty string."
    (builtin:get-output-string port))

  ;; ============= R7RS Binary String Ports =============

  (define (open-input-bytevector bytevector)
    "Opens a binary input port that reads from the given bytevector.
     
     R7RS: (open-input-bytevector bytevector) procedure
     Takes a bytevector and returns a binary input port that delivers
     bytes from the bytevector. If the bytevector is modified after
     the port is created, the effect is unspecified."
    (builtin:open-input-bytevector bytevector))

  (define (open-output-bytevector)
    "Opens a binary output port that accumulates bytes into a bytevector.
     
     R7RS: (open-output-bytevector) procedure
     Returns a binary output port that will accumulate bytes for
     retrieval by get-output-bytevector. The port should be closed
     after use, although it is safe to continue using it after
     calling get-output-bytevector."
    (builtin:open-output-bytevector))

  (define (get-output-bytevector port)
    "Retrieves the bytes accumulated in the given output bytevector port.
     
     R7RS: (get-output-bytevector port) procedure
     Given an output port created by open-output-bytevector, returns a
     bytevector consisting of the bytes that have been output to the
     port so far. After calling get-output-bytevector, the port will
     still accept new output but will begin accumulating again from
     an empty bytevector."
    (builtin:get-output-bytevector port))

  ;; ============= R7RS Input Operations =============

  (define (read . port)
    "Reads and returns an object from the textual input port.
     
     R7RS: (read) procedure
           (read port) procedure
     The read procedure converts external representations of Scheme
     objects into the objects themselves. It reads an external
     representation from the given textual input port and returns the
     datum that it represents. If port is omitted, it defaults to the
     value returned by current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:read p)))

  (define (read-char . port)
    "Reads and returns a character from the textual input port.
     
     R7RS: (read-char) procedure
           (read-char port) procedure
     Returns the next character available from the textual input port,
     updating the port to point to the following character. If no more
     characters are available, an end-of-file object is returned.
     If port is omitted, it defaults to the value returned by
     current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:read-char p)))

  (define (peek-char . port)
    "Returns the next character from the textual input port without updating the port.
     
     R7RS: (peek-char) procedure
           (peek-char port) procedure
     Returns the next character available from the textual input port,
     but does not update the port to point to the following character.
     If no more characters are available, an end-of-file object is returned.
     If port is omitted, it defaults to the value returned by
     current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:peek-char p)))

  (define (read-line . port)
    "Reads a line of text from the textual input port.
     
     R7RS: (read-line) procedure
           (read-line port) procedure
     Returns the next line of text available from the textual input port,
     updating the port to point to the following character. If end of line
     is read, a string containing all of the text up to (but not including)
     the end of line is returned, and the port is updated to point just past
     the end of line. If end of file is encountered before any characters
     are read, an end-of-file object is returned.
     If port is omitted, it defaults to the value returned by
     current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:read-line p)))

  (define (read-string k . port)
    "Reads at most k characters from the textual input port.
     
     R7RS: (read-string k) procedure
           (read-string k port) procedure
     Reads the next k characters, or as many as are available before the
     end of file, from the textual input port into a newly allocated string
     in left-to-right order and returns the string. If no characters are
     available before the end of file, an end-of-file object is returned.
     If port is omitted, it defaults to the value returned by
     current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:read-string k p)))

  (define (read-u8 . port)
    "Reads and returns a byte from the binary input port.
     
     R7RS: (read-u8) procedure
           (read-u8 port) procedure
     Returns the next byte available from the binary input port, updating
     the port to point to the following byte. If no more bytes are available,
     an end-of-file object is returned. If port is omitted, it defaults to
     the value returned by current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:read-u8 p)))

  (define (peek-u8 . port)
    "Returns the next byte from the binary input port without updating the port.
     
     R7RS: (peek-u8) procedure
           (peek-u8 port) procedure
     Returns the next byte available from the binary input port, but does not
     update the port to point to the following byte. If no more bytes are
     available, an end-of-file object is returned. If port is omitted, it
     defaults to the value returned by current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:peek-u8 p)))

  (define (char-ready? . port)
    "Returns #t if a character is ready from the textual input port.
     
     R7RS: (char-ready?) procedure
           (char-ready? port) procedure
     Returns #t if a character is ready on the textual input port and returns
     #f otherwise. If char-ready returns #t then the next read-char operation
     on the given port is guaranteed not to hang. If the port is at end of file
     then char-ready? returns #t. If port is omitted, it defaults to the value
     returned by current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:char-ready? p)))

  (define (u8-ready? . port)
    "Returns #t if a byte is ready from the binary input port.
     
     R7RS: (u8-ready?) procedure
           (u8-ready? port) procedure
     Returns #t if a byte is ready on the binary input port and returns #f
     otherwise. If u8-ready? returns #t then the next read-u8 operation on
     the given port is guaranteed not to hang. If the port is at end of file
     then u8-ready? returns #t. If port is omitted, it defaults to the value
     returned by current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:u8-ready? p)))

  (define (eof-object)
    "Returns the end-of-file object.
     
     R7RS: (eof-object) procedure
     Returns an end-of-file object, not necessarily unique."
    (builtin:eof-object))

  (define (eof-object? obj)
    "Returns #t if obj is an end-of-file object.
     
     R7RS: (eof-object? obj) procedure
     Returns #t if obj is an end-of-file object, otherwise returns #f.
     The end-of-file object returned by read, read-char, read-line, and
     read-u8 (and others) is guaranteed to satisfy this predicate, but no other
     objects do (and not all Scheme implementations distinguish the end-of-file
     object from other objects)."
    (builtin:eof-object? obj))

  ;; ============= R7RS Output Operations =============

  (define (write obj . port)
    "Writes obj to the textual output port using the external representation.
     
     R7RS: (write obj) procedure
           (write obj port) procedure
     Writes a representation of obj to the given textual output port. Strings
     that appear in the written representation are enclosed in quotation marks,
     and within those strings backslash and quotation mark characters are
     escaped by backslashes. Character objects are written using the #\\
     notation. If port is omitted, it defaults to the value returned by
     current-output-port."
    (let ((p (if (null? port) (current-output-port) (car port))))
      (builtin:write obj p)))

  (define (write-char char . port)
    "Writes the character to the textual output port.
     
     R7RS: (write-char char) procedure
           (write-char char port) procedure
     Writes the character char (not an external representation of the character)
     to the given textual output port and returns an unspecified value.
     If port is omitted, it defaults to the value returned by
     current-output-port."
    (let ((p (if (null? port) (current-output-port) (car port))))
      (builtin:write-char char p)))

  (define (write-string string . port-start-end)
    "Writes the characters of string to the textual output port.
     
     R7RS: (write-string string) procedure
           (write-string string port) procedure
           (write-string string port start) procedure
           (write-string string port start end) procedure
     Writes the characters of string from start to end in left-to-right order
     to the textual output port. If port is omitted, it defaults to the value
     returned by current-output-port. If start is omitted, it defaults to 0.
     If end is omitted, it defaults to the length of string."
    (let ((p (if (null? port-start-end) (current-output-port) (car port-start-end)))
          (start (if (< (length port-start-end) 2) 0 (cadr port-start-end)))
          (end (if (< (length port-start-end) 3) 
                   (string-length string) 
                   (caddr port-start-end))))
      (builtin:write-string string p start end)))

  (define (write-u8 byte . port)
    "Writes the byte to the binary output port.
     
     R7RS: (write-u8 byte) procedure
           (write-u8 byte port) procedure
     Writes the byte given by byte to the given binary output port and returns
     an unspecified value. If port is omitted, it defaults to the value
     returned by current-output-port."
    (let ((p (if (null? port) (current-output-port) (car port))))
      (builtin:write-u8 byte p)))

  (define (newline . port)
    "Writes an end of line to the textual output port.
     
     R7RS: (newline) procedure
           (newline port) procedure
     Writes an end of line to textual output port. Exactly how this is done
     differs from one operating system to another. Returns an unspecified value.
     If port is omitted, it defaults to the value returned by
     current-output-port."
    (let ((p (if (null? port) (current-output-port) (car port))))
      (builtin:newline p)))

  (define (display obj . port)
    "Writes obj to the textual output port using a human-readable representation.
     
     R7RS: (display obj) procedure
           (display obj port) procedure
     Writes a representation of obj to the given textual output port. Strings
     that appear in the written representation are output as if by write-string
     instead of by write. Character objects appear in the representation as if
     written by write-char instead of by write. If port is omitted, it defaults
     to the value returned by current-output-port."
    (let ((p (if (null? port) (current-output-port) (car port))))
      (builtin:display obj p)))

  (define (flush-output-port . port)
    "Forces any pending output on the textual output port to be delivered.
     
     R7RS: (flush-output-port) procedure
           (flush-output-port port) procedure
     Forces any pending output on port to be delivered to the underlying
     operating system and returns an unspecified value. If port is omitted,
     it defaults to the value returned by current-output-port."
    (let ((p (if (null? port) (current-output-port) (car port))))
      (builtin:flush-output-port p)))

  ;; ============= R7RS Binary I/O Operations =============

  (define (read-bytevector k . port)
    "Reads at most k bytes from the binary input port into a fresh bytevector.
     
     R7RS: (read-bytevector k) procedure
           (read-bytevector k port) procedure
     Reads the next k bytes, or as many as are available before the end of file,
     from the binary input port into a newly allocated bytevector in left-to-right
     order and returns the bytevector. If no bytes are available before the end
     of file, an end-of-file object is returned. If port is omitted, it defaults
     to the value returned by current-input-port."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (builtin:read-bytevector k p)))

  (define (read-bytevector! bytevector . port-start-end)
    "Reads bytes from the binary input port into the given bytevector.
     
     R7RS: (read-bytevector! bytevector) procedure
           (read-bytevector! bytevector port) procedure
           (read-bytevector! bytevector port start) procedure
           (read-bytevector! bytevector port start end) procedure
     Reads the next end − start bytes, or as many as are available before the
     end of file, from the binary input port into bytevector in left-to-right
     order beginning at the start position. If end is not supplied, reads until
     the end of bytevector has been reached. If start is not supplied, reads
     beginning at position 0. Returns the number of bytes read. If no bytes are
     available, an end-of-file object is returned. If port is omitted, it defaults
     to the value returned by current-input-port."
    (let ((p (if (null? port-start-end) (current-input-port) (car port-start-end)))
          (start (if (< (length port-start-end) 2) 0 (cadr port-start-end)))
          (end (if (< (length port-start-end) 3) 
                   (bytevector-length bytevector) 
                   (caddr port-start-end))))
      (builtin:read-bytevector! bytevector p start end)))

  (define (write-bytevector bytevector . port-start-end)
    "Writes the bytes of bytevector to the binary output port.
     
     R7RS: (write-bytevector bytevector) procedure
           (write-bytevector bytevector port) procedure
           (write-bytevector bytevector port start) procedure
           (write-bytevector bytevector port start end) procedure
     Writes the bytes of bytevector from start to end in left-to-right order
     to the binary output port. If port is omitted, it defaults to the value
     returned by current-output-port. If start is omitted, it defaults to 0.
     If end is omitted, it defaults to the length of bytevector."
    (let ((p (if (null? port-start-end) (current-output-port) (car port-start-end)))
          (start (if (< (length port-start-end) 2) 0 (cadr port-start-end)))
          (end (if (< (length port-start-end) 3) 
                   (bytevector-length bytevector) 
                   (caddr port-start-end))))
      (builtin:write-bytevector bytevector p start end)))

  ;; ============= Additional Lambdust Format Operations =============
  ;; (Extensions beyond R7RS for practical formatting)

  (define (format format-string . args)
    "Returns a formatted string using printf-style format specifiers.
     
     Extension: Provides C-style printf formatting for convenience.
     Format specifiers: ~a (any), ~s (S-expr), ~d (decimal), ~x (hex), ~o (octal), ~b (binary), ~f (float), ~% (newline)"
    (apply sprintf format-string args))

  (define (fprintf port format-string . args)
    "Writes formatted output directly to the specified port.
     
     Extension: Combines formatting with direct port output for efficiency."
    (write-string (apply sprintf format-string args) port))

  (define (sprintf format-string . args)
    "Returns a formatted string using printf-style format specifiers.
     
     Extension: Core string formatting function with format specifier support."
    (builtin:sprintf format-string args))

  ;; ============= Additional Lambdust File System Operations =============
  ;; (Extensions beyond R7RS for file system interaction)

  (define (file-exists? filename)
    "Returns #t if the named file exists in the file system.
     
     Extension: File system query operation for checking file existence.
     Useful for conditional file operations and error prevention."
    (builtin:file-exists? filename))

  (define (delete-file filename)
    "Deletes the named file from the file system.
     
     Extension: File system modification operation for file removal.
     Raises an error if the file does not exist or cannot be deleted."
    (builtin:delete-file filename))

  (define (rename-file old-filename new-filename)
    "Renames or moves a file from old-filename to new-filename.
     
     Extension: File system operation for renaming/moving files.
     Raises an error if the source file does not exist or the operation fails."
    (builtin:rename-file old-filename new-filename))

  ;; ============= Additional Lambdust I/O Utilities =============
  ;; (Extensions beyond R7RS for practical I/O operations)

  (define (copy-port input-port output-port . buffer-size)
    "Copies all data from input port to output port efficiently.
     
     Extension: High-level utility for transferring data between ports.
     Uses buffered I/O for performance with configurable buffer size."
    (let ((buf-size (if (null? buffer-size) 4096 (car buffer-size))))
      (copy-port-helper input-port output-port buf-size)))

  (define (copy-port-helper input-port output-port buffer-size)
    "Internal helper function for copy-port implementation."
    (let ((buffer (read-string buffer-size input-port)))
      (unless (eof-object? buffer)
        (write-string buffer output-port)
        (copy-port-helper input-port output-port buffer-size))))

  (define (port-position port)
    "Returns the current position in the port as a non-negative integer.
     
     Extension: Port positioning support for seekable ports.
     Useful for random access I/O operations."
    (builtin:port-position port))

  (define (set-port-position! port position)
    "Sets the position in the port to the specified non-negative integer.
     
     Extension: Port seeking operation for random access.
     Only works on ports that support positioning."
    (builtin:set-port-position! port position))

  (define (port-has-port-position? port)
    "Returns #t if port supports the port-position operation.
     
     Extension: Capability check for port positioning support.
     Not all ports support random access positioning."
    (builtin:port-has-port-position? port))

  (define (port-has-set-port-position!? port)
    "Returns #t if port supports the set-port-position! operation.
     
     Extension: Capability check for port seeking support.
     Some ports may support reading position but not setting it."
    (builtin:port-has-set-port-position!? port))

  ;; ============= Additional Lambdust Reading Utilities =============
  ;; (Extensions beyond R7RS for convenient file and data operations)

  (define (read-all . port)
    "Reads all S-expressions from port and returns them as a list.
     
     Extension: Convenient utility for reading multiple objects.
     Continues reading until EOF is encountered."
    (let ((p (if (null? port) (current-input-port) (car port))))
      (read-all-helper p '())))

  (define (read-all-helper port acc)
    "Internal helper function for read-all implementation."
    (let ((obj (read port)))
      (if (eof-object? obj)
          (reverse acc)
          (read-all-helper port (cons obj acc)))))

  (define (read-file filename)
    "Reads the entire contents of the named file as a string.
     
     Extension: High-level file reading utility for text files.
     Opens the file, reads all content, and closes the file automatically."
    (call-with-input-file filename
      (lambda (port)
        (read-string-all port))))

  (define (read-string-all port)
    "Reads all remaining characters from port as a single string.
     
     Extension: Utility for reading entire port contents efficiently.
     Uses string port accumulation for optimal performance."
    (let ((result (open-output-string)))
      (copy-port port result)
      (get-output-string result)))

  (define (write-file filename content)
    "Writes the string content to the named file.
     
     Extension: High-level file writing utility for text files.
     Creates/overwrites the file, writes the content, and closes automatically."
    (call-with-output-file filename
      (lambda (port)
        (write-string content port)))))