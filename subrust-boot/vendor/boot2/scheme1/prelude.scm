; scheme1 prelude. catm'd in front of the user .scm before invoking the
; scheme1 binary (see tests/boot-run-scheme1.sh). Defines the R7RS
; surface that's expressible over scheme1's existing primitives --
; equivalence aliases, list helpers, characters as fixnum bytes,
; strings as NUL-terminated bytevectors -- plus the shell.scm process-
; management and file-I/O layer (port record + buffered reads/writes).
;
; Items that depend on primitives scheme1 doesn't yet have (the
; vector-* family) stay here as commented placeholders for re-enabling
; once those primitives land. See docs/SCHEME1-R7RS-TODO.md.

;; --- Arithmetic helpers (derivable from <, =, -) --------------------
(define (<= x y) (if (< y x) #f #t))
(define (>= x y) (if (< x y) #f #t))

(define (negative? x) (< x 0))
(define (positive? x) (> x 0))

;; scheme1 has only one numeric and one byte-string repr today, so these
;; predicates are exact aliases. They exist so callers can spell intent.
(define number?     integer?)
(define bytevector? string?)

(define (abs x) (if (< x 0) (- 0 x) x))

(define (min a b) (if (< a b) a b))
(define (max a b) (if (< a b) b a))

;; modulo has the sign of the divisor; remainder has the sign of the
;; dividend. They differ exactly when r is nonzero and r and b have
;; opposite signs -- in that case adjust by adding b.
(define (modulo a b)
  (let ((r (remainder a b)))
    (if (zero? r)
        0
        (if (eq? (negative? r) (negative? b))
            r
            (+ r b)))))

;; --- R7RS equivalence / equality predicates ------------------------
;; eqv? collapses to eq? for our value set: fixnums are immediate-
;; tagged, symbols are interned, and pairs/strings/closures use
;; pointer identity.
(define eqv? eq?)

(define (%all-eq? a xs)
  (if (null? xs) #t
      (if (eq? (car xs) a) (%all-eq? a (cdr xs)) #f)))

(define (boolean=? a b . rest) (and (eq? a b) (%all-eq? a rest)))
(define (symbol=?  a b . rest) (and (eq? a b) (%all-eq? a rest)))

;; --- c*r compositions ----------------------------------------------
(define (caar x)  (car (car x)))
(define (cadr x)  (car (cdr x)))
(define (cdar x)  (cdr (car x)))
(define (cddr x)  (cdr (cdr x)))

(define (caaar x) (car (caar x)))
(define (caadr x) (car (cadr x)))
(define (cadar x) (car (cdar x)))
(define (caddr x) (car (cddr x)))
(define (cdaar x) (cdr (caar x)))
(define (cdadr x) (cdr (cadr x)))
(define (cddar x) (cdr (cdar x)))
(define (cdddr x) (cdr (cddr x)))

(define (caaaar x) (car (caaar x)))
(define (caaadr x) (car (caadr x)))
(define (caadar x) (car (cadar x)))
(define (caaddr x) (car (caddr x)))
(define (cadaar x) (car (cdaar x)))
(define (cadadr x) (car (cdadr x)))
(define (caddar x) (car (cddar x)))
(define (cadddr x) (car (cdddr x)))
(define (cdaaar x) (cdr (caaar x)))
(define (cdaadr x) (cdr (caadr x)))
(define (cdadar x) (cdr (cadar x)))
(define (cdaddr x) (cdr (caddr x)))
(define (cddaar x) (cdr (cdaar x)))
(define (cddadr x) (cdr (cdadr x)))
(define (cdddar x) (cdr (cddar x)))
(define (cddddr x) (cdr (cdddr x)))

;; --- List helpers --------------------------------------------------
(define (list . xs) xs)

(define (list? x)
  (if (null? x)
      #t
      (if (pair? x) (list? (cdr x)) #f)))

(define (append-pair a b)
  (if (null? a) b (cons (car a) (append-pair (cdr a) b))))

(define (append . lists)
  (cond ((null? lists) (quote ()))
        ((null? (cdr lists)) (car lists))
        (else (append-pair (car lists) (apply append (cdr lists))))))

(define (make-list n . fill)
  (let ((v (if (null? fill) #f (car fill))))
    (let loop ((i 0) (acc (quote ())))
      (if (= i n) acc (loop (+ i 1) (cons v acc))))))

(define (list-tail xs k)
  (if (zero? k) xs (list-tail (cdr xs) (- k 1))))

(define (list-set! xs k v)
  (if (zero? k) (set-car! xs v) (list-set! (cdr xs) (- k 1) v)))

(define (list-copy xs)
  (if (pair? xs)
      (cons (car xs) (list-copy (cdr xs)))
      xs))

(define (memq x xs)
  (if (null? xs) #f
      (if (eq? (car xs) x) xs (memq x (cdr xs)))))
(define memv memq)
(define (member x xs)
  (if (null? xs) #f
      (if (equal? (car xs) x) xs (member x (cdr xs)))))

(define assv assq)

;; --- map / filter / fold / for-each --------------------------------
;; map and for-each accept N parallel lists per R7RS; iteration stops
;; at the shortest list. The %any-null?/%list-cars/%list-cdrs helpers
;; back the multi-list path.
(define (%any-null? xss)
  (if (null? xss) #f
      (if (null? (car xss)) #t (%any-null? (cdr xss)))))
(define (%list-cars xss)
  (if (null? xss) (quote ())
      (cons (car (car xss)) (%list-cars (cdr xss)))))
(define (%list-cdrs xss)
  (if (null? xss) (quote ())
      (cons (cdr (car xss)) (%list-cdrs (cdr xss)))))

(define (map f xs . rest)
  (if (null? rest)
      (let m ((xs xs))
        (if (null? xs) (quote ())
            (cons (f (car xs)) (m (cdr xs)))))
      (let m ((xss (cons xs rest)))
        (if (%any-null? xss) (quote ())
            (cons (apply f (%list-cars xss))
                  (m (%list-cdrs xss)))))))

(define (filter p xs)
  (if (null? xs)
      (quote ())
      (if (p (car xs))
          (cons (car xs) (filter p (cdr xs)))
          (filter p (cdr xs)))))

(define (fold f acc xs)
  (if (null? xs)
      acc
      (fold f (f acc (car xs)) (cdr xs))))

(define (for-each f xs . rest)
  (if (null? rest)
      (let m ((xs xs))
        (if (null? xs) (quote ())
            (begin (f (car xs)) (m (cdr xs)))))
      (let m ((xss (cons xs rest)))
        (if (%any-null? xss) (quote ())
            (begin (apply f (%list-cars xss))
                   (m (%list-cdrs xss)))))))

;; --- R7RS character procedures (ASCII over fixnum bytes) -----------
;; Chars are plain fixnums; char? is a 0..255 range check rather than
;; a disjoint type. char->integer / integer->char are the identity.
(define (char? x)
  (if (integer? x)
      (if (< x 0) #f (< x 256))
      #f))

(define (char->integer c) c)
(define (integer->char n) n)

(define (char-upper-case? c) (and (>= c 65) (<= c 90)))
(define (char-lower-case? c) (and (>= c 97) (<= c 122)))
(define (char-alphabetic? c) (or (char-upper-case? c) (char-lower-case? c)))
(define (char-numeric?    c) (and (>= c 48) (<= c 57)))
(define (char-whitespace? c)
  (or (= c 32) (= c 9) (= c 10) (= c 11) (= c 12) (= c 13)))

(define (digit-value c) (if (char-numeric? c) (- c 48) #f))

(define (char-upcase   c) (if (char-lower-case? c) (- c 32) c))
(define (char-downcase c) (if (char-upper-case? c) (+ c 32) c))
(define char-foldcase char-downcase)

(define (%chain-rel rel a b rest)
  (if (rel a b)
      (if (null? rest) #t (%chain-rel rel b (car rest) (cdr rest)))
      #f))

(define (char=?  a b . rest) (%chain-rel =  a b rest))
(define (char<?  a b . rest) (%chain-rel <  a b rest))
(define (char>?  a b . rest) (%chain-rel >  a b rest))
(define (char<=? a b . rest) (%chain-rel <= a b rest))
(define (char>=? a b . rest) (%chain-rel >= a b rest))

;; --- R7RS string procedures (over NUL-terminated bytevectors) ------
;; A scheme1 "string" is a bytevector whose first NUL byte marks the
;; logical end. Constructors allocate (n+1) bytes and store 0 at index
;; n. string-ref / string-set! are thin aliases over the bytevector
;; primitives; bounds against string-length aren't enforced (the user
;; can clobber the NUL terminator).
(define (make-string n . fill)
  (let ((c (if (null? fill) 32 (car fill))))
    (let ((bv (make-bytevector (+ n 1) c)))
      (bytevector-u8-set! bv n 0)
      bv)))

(define (string . cs)
  (let* ((n (length cs))
         (bv (make-bytevector (+ n 1) 0)))
    (let loop ((xs cs) (i 0))
      (if (null? xs) bv
          (begin
            (bytevector-u8-set! bv i (car xs))
            (loop (cdr xs) (+ i 1)))))))

(define string-ref  bytevector-u8-ref)
(define string-set! bytevector-u8-set!)

(define (substring s start end)
  (let* ((n (- end start))
         (out (make-bytevector (+ n 1) 0)))
    (bytevector-copy! out 0 s start end)
    out))

(define (string-append . ss)
  (let ((total (let sum ((xs ss) (n 0))
                 (if (null? xs) n
                     (sum (cdr xs) (+ n (string-length (car xs))))))))
    (let ((out (make-bytevector (+ total 1) 0)))
      (let loop ((xs ss) (off 0))
        (if (null? xs) out
            (let ((len (string-length (car xs))))
              (bytevector-copy! out off (car xs) 0 len)
              (loop (cdr xs) (+ off len))))))))

(define (string-copy s . args)
  (let* ((start (if (null? args) 0 (car args)))
         (rs    (if (null? args) (quote ()) (cdr args)))
         (end   (if (null? rs) (string-length s) (car rs))))
    (substring s start end)))

(define (string-copy! dst at src . args)
  (let* ((start (if (null? args) 0 (car args)))
         (rs    (if (null? args) (quote ()) (cdr args)))
         (end   (if (null? rs) (string-length src) (car rs))))
    (bytevector-copy! dst at src start end)))

(define (string-fill! s ch . args)
  (let* ((start (if (null? args) 0 (car args)))
         (rs    (if (null? args) (quote ()) (cdr args)))
         (end   (if (null? rs) (string-length s) (car rs))))
    (let loop ((i start))
      (if (>= i end) s
          (begin (bytevector-u8-set! s i ch) (loop (+ i 1)))))))

(define (string->list s . args)
  (let* ((start (if (null? args) 0 (car args)))
         (rs    (if (null? args) (quote ()) (cdr args)))
         (end   (if (null? rs) (string-length s) (car rs))))
    (let loop ((i (- end 1)) (acc (quote ())))
      (if (< i start) acc
          (loop (- i 1) (cons (bytevector-u8-ref s i) acc))))))

(define (list->string cs) (apply string cs))

(define (%string-cmp a b)
  (let ((alen (string-length a))
        (blen (string-length b)))
    (let loop ((i 0))
      (cond ((and (= i alen) (= i blen)) 0)
            ((= i alen) -1)
            ((= i blen) 1)
            (else
             (let ((d (- (bytevector-u8-ref a i) (bytevector-u8-ref b i))))
               (if (zero? d) (loop (+ i 1)) d)))))))

(define (%string-ci-cmp a b)
  (let ((alen (string-length a))
        (blen (string-length b)))
    (let loop ((i 0))
      (cond ((and (= i alen) (= i blen)) 0)
            ((= i alen) -1)
            ((= i blen) 1)
            (else
             (let ((d (- (char-foldcase (bytevector-u8-ref a i))
                         (char-foldcase (bytevector-u8-ref b i)))))
               (if (zero? d) (loop (+ i 1)) d)))))))

(define (%chain-cmp cmp rel a b rest)
  (if (rel (cmp a b) 0)
      (if (null? rest) #t (%chain-cmp cmp rel b (car rest) (cdr rest)))
      #f))

(define (string=?  a b . rest) (%chain-cmp %string-cmp =  a b rest))
(define (string<?  a b . rest) (%chain-cmp %string-cmp <  a b rest))
(define (string>?  a b . rest) (%chain-cmp %string-cmp >  a b rest))
(define (string<=? a b . rest) (%chain-cmp %string-cmp <= a b rest))
(define (string>=? a b . rest) (%chain-cmp %string-cmp >= a b rest))

(define (string-ci=?  a b . rest) (%chain-cmp %string-ci-cmp =  a b rest))
(define (string-ci<?  a b . rest) (%chain-cmp %string-ci-cmp <  a b rest))
(define (string-ci>?  a b . rest) (%chain-cmp %string-ci-cmp >  a b rest))
(define (string-ci<=? a b . rest) (%chain-cmp %string-ci-cmp <= a b rest))
(define (string-ci>=? a b . rest) (%chain-cmp %string-ci-cmp >= a b rest))

(define (string-upcase s)
  (let* ((n   (string-length s))
         (out (make-bytevector (+ n 1) 0)))
    (let loop ((i 0))
      (if (= i n) out
          (begin
            (bytevector-u8-set! out i (char-upcase (bytevector-u8-ref s i)))
            (loop (+ i 1)))))))

(define (string-downcase s)
  (let* ((n   (string-length s))
         (out (make-bytevector (+ n 1) 0)))
    (let loop ((i 0))
      (if (= i n) out
          (begin
            (bytevector-u8-set! out i (char-downcase (bytevector-u8-ref s i)))
            (loop (+ i 1)))))))

(define string-foldcase string-downcase)

(define (string-map f s)
  (let* ((n   (string-length s))
         (out (make-bytevector (+ n 1) 0)))
    (let loop ((i 0))
      (if (= i n) out
          (begin
            (bytevector-u8-set! out i (f (bytevector-u8-ref s i)))
            (loop (+ i 1)))))))

(define (string-for-each f s)
  (let ((n (string-length s)))
    (let loop ((i 0))
      (if (= i n) (quote ())
          (begin (f (bytevector-u8-ref s i)) (loop (+ i 1)))))))

;; --- R7RS bytevector constructor -----------------------------------
(define (bytevector . bytes)
  (let* ((n  (length bytes))
         (bv (make-bytevector n 0)))
    (let loop ((xs bytes) (i 0))
      (if (null? xs) bv
          (begin
            (bytevector-u8-set! bv i (car xs))
            (loop (cdr xs) (+ i 1)))))))

;; --- Generic deep-copy ---------------------------------------------
;; Structural clone of pair / bytevector / record graphs in the
;; currently-selected heap. Preserves eq? identity across shared
;; substructure and tolerates cycles via an eager stand-in registered
;; before recursion.
;;
;; The ctx is a one-cell box around an (orig . copy) alist; lookups
;; key off pointer identity (assq) so two structurally-equal but
;; physically-distinct objects are treated separately. Cells leak into
;; whichever heap is current when ctx is created — typically main
;; during cc.scm's parse-decl-or-fn promotion.
;;
;; Strict positive-list dispatch: pair / bytevector / record. Anything
;; else that masquerades as heap-allocated (closures, prims, MV-packs)
;; surfaces as an error rather than silently dangling.
(define (make-deep-copy-context) (cons '() #f))

(define (%dcc-lookup ctx obj)
  (let ((p (assq obj (car ctx))))
    (if p (cdr p) #f)))

(define (%dcc-register! ctx obj copy)
  (set-car! ctx (cons (cons obj copy) (car ctx)))
  copy)

(define (deep-copy ctx obj)
  (cond
    ((symbol? obj) obj)
    ((heap-in-current? obj) obj)
    ((pair? obj)
     (let ((c (%dcc-lookup ctx obj)))
       (cond
         (c c)
         (else
          (let ((p (cons #f #f)))
            (%dcc-register! ctx obj p)
            (set-car! p (deep-copy ctx (car obj)))
            (set-cdr! p (deep-copy ctx (cdr obj)))
            p)))))
    ((bytevector? obj)
     (let ((c (%dcc-lookup ctx obj)))
       (cond
         (c c)
         (else
          (%dcc-register! ctx obj
            (bytevector-copy obj 0 (bytevector-length obj)))))))
    ((record? obj)
     (let ((c (%dcc-lookup ctx obj)))
       (cond
         (c c)
         (else
          (let* ((td (record-td obj))
                 (n  (td-nfields td))
                 (s  (make-record/td td)))
            (%dcc-register! ctx obj s)
            (let fill ((i 0))
              (cond ((= i n) s)
                    (else
                     (record-set! s i (deep-copy ctx (record-ref obj i)))
                     (fill (+ i 1))))))))))
    ((procedure? obj)
     (error "deep-copy: cannot copy procedure" obj))
    (else obj)))

;; --- Heap arena wrappers -------------------------------------------
;; Two-pattern API on top of the raw heap-mark / heap-rewind! / scratch
;; primitives. Most callers should reach for these instead of driving
;; the primitives directly. See tests/scheme1/093-heap-mark-rewind.scm
;; and tests/scheme1/115-two-heap.scm for the underlying contract.

;; Pattern 1 — mark/rewind. Run thunk inside a heap-mark/rewind arena
;; on the current heap. All heap allocations performed by thunk are
;; reclaimed on return; thunk's return value MUST be either an immediate
;; (fixnum, boolean, symbol, '()) or a cell allocated by the caller
;; *before* call-with-heap-rewind ran. The classic A→B→C shape pre-
;; allocates an `out` cell, calls this with a thunk that mutates `out`,
;; and returns `out` to its own caller.
(define (call-with-heap-rewind thunk)
  (let ((mark (heap-mark)))
    (let ((r (thunk)))
      (heap-rewind! mark)
      r)))

;; Pattern 2a — scratch + deep-copy of a single root. Run thunk with
;; the scratch heap selected, switch back to main, deep-copy thunk's
;; result into main, reset scratch, return the main-heap copy. Use for
;; the common case of "build a graph in scratch, hand the caller a
;; main-heap clone, reclaim scratch".
(define (call-with-scratch-deep-copy thunk)
  (use-scratch-heap!)
  (let ((s (thunk)))
    (use-main-heap!)
    (let ((m (deep-copy (make-deep-copy-context) s)))
      (reset-scratch-heap!)
      m)))

;; Pattern 2b — scratch + multi-root promote. Lower-level cycle: select
;; scratch, run (in-scratch), select main, run (promote), reset scratch.
;; The (promote) thunk is responsible for deep-copying every survivor
;; root from scratch into main (typically across several caller-owned
;; slots, sharing a single deep-copy context). Returns unspec; survivors
;; must reach the caller via slots that promote rewrites in place.
(define (call-with-scratch-cycle in-scratch promote)
  (use-scratch-heap!)
  (in-scratch)
  (use-main-heap!)
  (promote)
  (reset-scratch-heap!))

;; --- Vector <-> list -- need make-vector / vector-ref / vector-set! /
;; vector-length, none of which are yet primitives. ------------------
; (define (vector->list-helper v i acc)
;   (if (< i 0)
;       acc
;       (vector->list-helper v (- i 1) (cons (vector-ref v i) acc))))
;
; (define (vector->list v)
;   (vector->list-helper v (- (vector-length v) 1) (quote ())))
;
; (define (list->vector-helper v xs i)
;   (if (null? xs)
;       v
;       (begin
;         (vector-set! v i (car xs))
;         (list->vector-helper v (cdr xs) (+ i 1)))))
;
; (define (list->vector xs)
;   (list->vector-helper (make-vector (length xs) 0) xs 0))

;; --- shell.scm port: process-management wrappers built on top of the
;; syscall primitives. sys-wait is a Scheme adapter over sys-waitid
;; that returns a wait4-style raw wstatus so decode-wait-status can
;; stay unchanged. --------------------------------------------------
(define (sys-wait pid)
  (let ((info (make-bytevector 128 0)))
    (let ((r (sys-waitid 1 pid info 4)))
      (if (car r)
          (let ((code (bytevector-u8-ref info 8))
                (status (bytevector-u8-ref info 24)))
            (if (= code 1)
                (cons #t (arithmetic-shift status 8))
                (cons #t (bit-and status #x7f))))
          r))))

(define (decode-wait-status s)
  (let ((termsig (bit-and s #x7f)))
    (if (zero? termsig)
        (bit-and (arithmetic-shift s -8) #xff)
        (+ 128 termsig))))

(define (wait pid)
  (let ((r (sys-wait pid)))
    (if (car r)
        (cons #t (decode-wait-status (cdr r)))
        r)))

(define (exit . rest)
  (sys-exit (if (null? rest) 0 (car rest))))

(define (argv) (sys-argv))
(define (command-line) (sys-argv))

;; scheme1 supports two process-creation paths:
;;   - sys-spawn: one atomic syscall (no userspace gap between fork and
;;     exec). Provided by the seed kernel; absent on Linux, where the
;;     syscall number returns -ENOSYS.
;;   - sys-clone + sys-execve: classic POSIX fork+exec. Provided by Linux;
;;     not implemented by the seed kernel.
;; Probe once at prelude-init time. The probe call uses an empty path,
;; so on the seed kernel it returns (#f . ENOENT) (the kernel finds the
;; argv/path checks before any side effect); on Linux it returns
;; (#f . ENOSYS). wrap_syscall_result already negates kernel -errno into
;; positive errno in cdr; treat anything other than ENOSYS as "available".
(define %has-sys-spawn?
  (let ((r (sys-spawn "" '())))
    (cond
      ((car r) #t)
      (else (not (= (cdr r) 38))))))

(define (spawn prog . args)
  (cond
    (%has-sys-spawn?
     (sys-spawn prog (cons prog args)))
    (else
     (let ((r (sys-clone)))
       (cond
         ((not (car r)) r)
         ((zero? (cdr r))
          (sys-execve prog (cons prog args))
          (sys-exit 127))
         (else r))))))

(define (run prog . args)
  (let ((r (apply spawn prog args)))
    (if (car r) (wait (cdr r)) r)))

;; --- shell.scm file-I/O constants ----------------------------------
(define BUFSIZE   4096)
(define AT_FDCWD  -100)
(define O_RDONLY  0)
(define O_WRONLY  1)
(define O_CREAT   #x40)     ; 0o100
(define O_TRUNC   #x200)    ; 0o1000
(define O_APPEND  #x400)    ; 0o2000
(define MODE_644  #x1a4)    ; 0o644
(define NL-BYTE   10)
(define NL-BV     (make-bytevector 1 10))

(define (file-exists? path)
  (let ((r (sys-openat AT_FDCWD path O_RDONLY 0)))
    (cond ((car r) (sys-close (cdr r)) #t)
          (else #f))))

;; --- shell.scm port record + handles -------------------------------
(define-record-type port
  (%port fd buf pos end)
  port?
  (fd  port-fd)
  (buf port-buf)
  (pos port-pos port-pos-set!)
  (end port-end port-end-set!))

(define stdin  (%port 0 (make-bytevector BUFSIZE) 0 0))
(define stdout (%port 1 #f 0 0))
(define stderr (%port 2 #f 0 0))

;; --- shell.scm port open/close -------------------------------------
(define (open-input path)
  (let ((r (sys-openat AT_FDCWD path O_RDONLY 0)))
    (if (car r)
        (cons #t (%port (cdr r) (make-bytevector BUFSIZE) 0 0))
        r)))

(define (open-output path)
  (let ((r (sys-openat AT_FDCWD path
                       (bit-or O_WRONLY O_CREAT O_TRUNC) MODE_644)))
    (if (car r) (cons #t (%port (cdr r) #f 0 0)) r)))

(define (open-append path)
  (let ((r (sys-openat AT_FDCWD path
                       (bit-or O_WRONLY O_CREAT O_APPEND) MODE_644)))
    (if (car r) (cons #t (%port (cdr r) #f 0 0)) r)))

(define (close p) (sys-close (port-fd p)))

;; --- shell.scm reads -----------------------------------------------
(define (refill! p)
  (let ((r (sys-read (port-fd p) (port-buf p) 0 BUFSIZE)))
    (cond
      ((not (car r)) r)
      (else (port-pos-set! p 0)
            (port-end-set! p (cdr r))
            r))))

(define (read-bytes p n)
  (let ((out (make-bytevector n)))
    (let loop ((i 0))
      (cond
        ((= i n) (cons #t out))
        ((< (port-pos p) (port-end p))
         (let* ((avail (- (port-end p) (port-pos p)))
                (take  (if (< avail (- n i)) avail (- n i))))
           (bytevector-copy! out i (port-buf p) (port-pos p) take)
           (port-pos-set! p (+ (port-pos p) take))
           (loop (+ i take))))
        (else
         (let ((r (refill! p)))
           (cond
             ((not (car r)) r)
             ((zero? (cdr r))
              (cons #t (if (zero? i) eof (bytevector-copy out 0 i))))
             (else (loop i)))))))))

(define (read-line p)
  (let loop ((acc (quote ())))
    (cond
      ((< (port-pos p) (port-end p))
       (let* ((buf   (port-buf p))
              (start (port-pos p))
              (end   (port-end p)))
         (let scan ((i start))
           (cond
             ((= i end)
              (port-pos-set! p i)
              (loop (cons (bytevector-copy buf start i) acc)))
             ((= (bytevector-u8-ref buf i) NL-BYTE)
              (port-pos-set! p (+ i 1))
              (cons #t (bv-concat-reverse
                        (cons (bytevector-copy buf start i) acc))))
             (else (scan (+ i 1)))))))
      (else
       (let ((r (refill! p)))
         (cond
           ((not (car r)) r)
           ((zero? (cdr r))
            (cons #t (if (null? acc) eof (bv-concat-reverse acc))))
           (else (loop acc))))))))

(define (read-all p)
  (let loop ((acc (quote ())))
    (cond
      ((< (port-pos p) (port-end p))
       (let ((chunk (bytevector-copy (port-buf p)
                                     (port-pos p) (port-end p))))
         (port-pos-set! p (port-end p))
         (loop (cons chunk acc))))
      (else
       (let ((r (refill! p)))
         (cond
           ((not (car r)) r)
           ((zero? (cdr r)) (cons #t (bv-concat-reverse acc)))
           (else (loop acc))))))))

(define (bv-concat-reverse chunks)
  (let* ((xs (reverse chunks))
         (total (let sum ((ys xs) (n 0))
                  (if (null? ys) n
                      (sum (cdr ys) (+ n (bytevector-length (car ys)))))))
         (out (make-bytevector total)))
    (let loop ((ys xs) (i 0))
      (if (null? ys)
          out
          (let ((len (bytevector-length (car ys))))
            (bytevector-copy! out i (car ys) 0 len)
            (loop (cdr ys) (+ i len)))))))

;; --- shell.scm writes (unbuffered; handle partial writes) ----------
;; sys-write takes an offset, so the partial-write fallback advances
;; the offset into the same bv instead of copying a tail.
(define (write-bytes p bv)
  (let ((len (bytevector-length bv)))
    (let loop ((off 0))
      (if (= off len)
          (cons #t len)
          (let ((r (sys-write (port-fd p) bv off (- len off))))
            (cond
              ((not (car r)) r)
              (else (loop (+ off (cdr r))))))))))

;; write-string assumes its input is a NUL-terminated bv (a "string")
;; and uses string-length, not bytevector-length, to bound the write.
(define (write-string p s)
  (let ((len (string-length s)))
    (let loop ((off 0))
      (if (= off len)
          (cons #t len)
          (let ((r (sys-write (port-fd p) s off (- len off))))
            (cond
              ((not (car r)) r)
              (else (loop (+ off (cdr r))))))))))

(define (write-line p s)
  (let ((r (write-string p s)))
    (if (car r) (write-bytes p NL-BV) r)))
