# scheme1

Minimal Scheme subset implemented by `scheme1/scheme1.P1pp`. A loose
subset of R7RS-small. The interpreter reads s-expressions from
`argv[1]`, evaluates them top-to-bottom in a single global env, and
exits.

`tests/boot-run-scheme1.sh` invokes `scheme1` with `prelude.scm`
catted in front of the user file. The prelude (`scheme1/prelude.scm`)
defines the R7RS surface that is expressible over the runtime
primitives — equivalence aliases, list/char/string helpers, and the
`shell.scm` process / file-I/O layer.

## Lexical syntax

- **Identifiers**: case-sensitive. Allowed bytes: ASCII letters,
  digits, and `! $ % & * + - . / : < = > ? @ ^ _ ~`. A token whose
  first byte is a digit (or sign-then-digit) is read as an integer; a
  bare `+` or `-` is a symbol. A lone `.` between list elements is the
  dotted-pair separator, not a symbol.
- **Booleans**: `#t`, `#f`.
- **Integers**: decimal (`42`, `-7`, `+3`) and hex (`#xff`, `#x-1a`).
  Word-size — 32-bit on 32-bit targets, 64-bit on 64-bit targets.
  No `#o`, no `#b`, no floats / rationals / bignums.
- **Strings**: `"…"`. Escapes: `\n \t \r \\ \"` and inline-hex `\xNN;`
  (1+ hex digits, value 0..255, terminated by `;`). A string is a
  bytevector; indexing is by u8.
- **Characters**: `#\a` through `#\~` for printable ASCII, plus
  `#\space` (32), `#\newline` (10), `#\tab` (9), `#\return` (13),
  `#\null` (0), and `#\xNN` for any byte. A character literal *is* a
  fixnum byte — there is no distinct character type. `(= #\a 97)` is
  `#t`.
- **Bytevector literal**: `#u8(b1 b2 ...)`. Each element must be a
  fixnum 0..255 (range unchecked).
- **Symbols**: bare. Globally interned — two symbols that print the
  same are `eq?`.
- **Pairs / lists**: `(a b c)`, `(a . b)`, `'()` for the empty list.
- **Quote sugar**: `'x` → `(quote x)`. **Comma sugar**: `,x` →
  `(unquote x)`. The unquote form has no effect outside `pmatch`
  patterns; evaluating it elsewhere fails as an unbound reference.
- **Comments**: `;` to end of line. No `#| … |#`, no `#;`.
- **No** vertical-bar identifiers, no quasiquote.

## Types

The runtime knows exactly:

| Type           | Notes                                                        |
|----------------|--------------------------------------------------------------|
| boolean        | `#t`, `#f`                                                   |
| integer        | word-size; 32- or 64-bit per target                          |
| symbol         | globally interned; `eq?`-comparable                          |
| string / bv    | same type (`HDR.BV`); contiguous u8 buffer                   |
| pair           | cons cell                                                    |
| empty list     | `'()`, disjoint from pair                                    |
| procedure      | closure or primitive                                         |
| record         | via `define-record-type`                                     |
| eof-object     | singleton; bound at top level as `eof`; also returned on EOF reads |
| unspecified    | singleton; result of `set!`, `define`, `(if #f x)`, etc.     |

Multiple-values packs flow through `values` / `call-with-values` / `let-values`
/ `let*-values`; they are not intended to be observed directly.

## Special forms

Top-level binding:

- `(define name expr)` and `(define (name arg ...) body ...)`,
  including variadic `.`-tails: `(define (f . rest) …)`,
  `(define (f a b . rest) …)`. **Top-level only** — internal
  `define` (inside `lambda` / `let` / `begin` body) is rejected at the
  start of the body interpreter. Use `let` / nested closures instead.

Procedures and binding:

- `(lambda (arg ...) body ...)`, with the same `.`-tail syntax.
  Captures its enclosing env by reference; `set!` on a captured
  variable is visible to all closures over it.
- `(let ((x v) ...) body ...)`, `(let name ((x v) ...) body ...)`
  (named let), `(let* …)`.
- `(let-values (((formals init) ...) body ...)`,
  `(let*-values …)`. `formals` is bound via the same matching as
  `lambda` parameters: list, dotted-tail, or bare symbol.
- `(set! name value)`. Walks the lexical env; on miss, rebinds the
  global slot.

Conditionals and sequencing:

- `(if test then else)` — `else` required for two-arm form;
  `(if test then)` returns the unspecified value when `test` is `#f`.
- `(cond (test body ...) ... (else body ...))`. A clause may also be
  `(test => proc-expr)` — when truthy, calls `proc-expr` on the test
  value.
- `(when test body ...)` — body runs when test is truthy, else
  unspecified.
- `(case key (datum-list body ...) ... (else body ...))` — datums
  matched by `eq?`.
- `(and e ...)` / `(or e ...)` — short-circuiting; return the
  deciding value, not a coerced boolean.
- `(begin e ...)`.
- `(do ((var init step?) ...) (test result ...) body ...)` —
  R7RS iteration construct; steps update in parallel.

Quote, records, matching:

- `(quote datum)` / `'datum`.
- `(define-record-type name (ctor f1 ...) pred (field accessor [mutator]) ...)`
  Creates a disjoint type. Binds `ctor`, `pred`, and each `accessor` /
  optional `mutator` at top level. Records are `equal?` iff TDs are
  `eq?` and all fields are `equal?`.
- `(pmatch expr clause ...)` — pattern matcher. Clause forms:

  ```
  (pattern body ...)
  (pattern (guard g-expr ...) body ...)
  (else body ...)
  ```

  Patterns:

  | Pattern              | Matches                                          |
  |----------------------|--------------------------------------------------|
  | `()`                 | the empty list                                   |
  | `<literal>`          | fixnum / bv / immediate by `equal?`              |
  | `<symbol>`           | that exact symbol (*not* a binder)               |
  | `,<ident>`           | anything; binds to `<ident>`                     |
  | `,_`                 | anything; no binding (wildcard)                  |
  | `(p1 p2 ...)`        | proper list of exactly that length               |
  | `(p1 ... . ptail)`   | improper list; `ptail` binds the rest            |
  | `($ pred (f p) ...)` | record whose predicate is `pred`; field `f` matches `p` |

  Clauses are tried top-to-bottom. A guard that evaluates to `#f`
  falls through. No-match with no `else` aborts.

## Primitives

The runtime built-ins — registered at startup from `prim_table` in
`scheme1.P1pp`. The prelude builds the wider R7RS surface on top of
these.

**Equality / predicates**
`eq?`, `equal?`, `not`, `null?`, `pair?`, `boolean?`, `integer?`,
`symbol?`, `string?` (≡ `bytevector?`), `procedure?`, `zero?`, `eof?`.

**Pairs**
`cons`, `car`, `cdr`, `set-car!`, `set-cdr!`, `length`, `list-ref`,
`assq`, `assoc`, `reverse`. `assq` compares alist keys by `eq?`;
`assoc` compares keys by `equal?`; both return the matching alist pair
or `#f`. `reverse` returns a fresh reversed list.

**Integers** (word-size; overflow / divide-by-zero are UB)
`+ - *`, `quotient`, `remainder`, `=`, `<`, `>`, `bit-and`, `bit-or`,
`bit-xor`, `bit-not`, `arithmetic-shift`. Arities: `+ * bit-and
bit-or bit-xor` accept 0+ args (identities `0 1 -1 0 0`); `-` accepts
1+ (`(- x)` is unary negate); `= < >` accept 2+ and chain pairwise.
`quotient` / `remainder` / `arithmetic-shift` are binary; `bit-not`
is unary. `quotient` truncates toward zero; `remainder` has the sign
of the dividend.

**Bytevectors / strings**
`make-bytevector`, `bytevector-length`, `bytevector-u8-ref`,
`bytevector-u8-set!`, `bytevector-copy` (3-arg `src start end` →
fresh bv), `bytevector-copy!` (`dst dst-start src src-start
src-end`), `bytevector-append` (variadic), `bytevector=?`,
`string-length` (strlen of the data buffer up to the first NUL).

**Symbols / numbers as text**
`string->symbol`, `symbol->string`, `number->string` (decimal by
default; lowercase hex when the optional radix arg is `16`, with a
leading `-` for negatives; any other radix value falls back to
decimal), `string->number` (decimal by default; hex when radix is
`16`, accepting upper- or lowercase digits and an optional leading
`+`/`-`; returns `#f` on parse failure).

**I/O and error**
`display`, `write`, `format`, `error`. `format` understands `~a`
(display), `~s` (write), `~d` (decimal fixnum), `~x` (lowercase hex
fixnum, signed: leading `-` for negatives), `~%` (newline), `~~`
(literal tilde); unknown directives pass through verbatim. `error`
writes `scheme1: error: <msg> <irritants…>` to stderr and exits with
status 1.

**EOF**
`eof` (the singleton, bound at startup), `eof?`.

**Multiple values**
`values`, `call-with-values`. `(values x)` is identical to `x` in
single-value context; 0 or 2+ args produce an MV-pack consumable by
`call-with-values` / `let-values` / `let*-values`.

**Apply**
`apply`. Tail calls are guaranteed proper.

**Syscalls** (Linux). Each returns `(#t . val)` on success or
`(#f . errno)` on failure.
`sys-read fd buf offset count`,
`sys-write fd buf offset count`,
`sys-close fd`,
`sys-openat dirfd path-bv flags mode`,
`sys-clone` (fork-style, no args),
`sys-execve path-bv argv-list`,
`sys-waitid idtype id infop options`,
`sys-argv` (no args; returns the process's argv as a list of bvs),
`sys-exit code` (does not return).

**Heap control** (used by the cc compiler for arena-style allocation)
`heap-usage`, `heap-mark`, `heap-rewind!`, `use-scratch-heap!`,
`use-main-heap!`, `reset-scratch-heap!`, `heap-in-main?`.
`heap-mark` / `heap-rewind!` discard everything allocated after the
mark on whichever heap is current; the scratch heap can be reset
wholesale. UNSAFE: the runtime does not track liveness, so any
surviving reference into a freed region becomes dangling. Most callers
should reach for the prelude wrappers `call-with-heap-rewind`,
`call-with-scratch-deep-copy`, and `call-with-scratch-cycle` rather
than driving these primitives directly.

## Error semantics

`error` is the only structured error path. Everything else —
`(car '())`, out-of-range `bytevector-u8-ref`, `(quotient 1 0)`,
mutating immutable state, integer overflow, unknown-form `pmatch`
fallthrough — is **primitive failure**: the runtime aborts with a
short message on stderr. Callers should not rely on any particular
outcome.

There is no `raise` / `guard` / handlers, no `call/cc`, no
exceptions. Wrap-and-return through `(ok . val)` pairs (the syscall
convention) when failure needs to be observable.

## Prelude surface

`scheme1/prelude.scm` is bundled in front of every user program by
`tests/boot-run-scheme1.sh`. It adds:

- **R7RS aliases**: `eqv?` ≡ `eq?`, `number?` ≡ `integer?`,
  `bytevector?` ≡ `string?`.
- **Arithmetic**: `<=`, `>=`, `negative?`, `positive?`, `abs`, `min`,
  `max`, `modulo`.
- **Equivalence chains**: `boolean=?`, `symbol=?`.
- **List helpers**: `list`, `list?`, `append`, `make-list`, `list-tail`,
  `list-set!`, `list-copy`, `memq` / `memv` / `member`, `assv` (alias
  of primitive `assq`), `map`, `for-each`, `filter`, `fold`, plus the
  full `c[ad]+r` family up to four levels. Primitive list helpers
  available before the prelude are listed above.
- **Characters as fixnums**: `char?`, `char->integer`,
  `integer->char` (identity), `char-upper-case?`, `char-lower-case?`,
  `char-alphabetic?`, `char-numeric?`, `char-whitespace?`,
  `digit-value`, `char-upcase`, `char-downcase`, `char-foldcase`,
  `char=?`, `char<?`, `char>?`, `char<=?`, `char>=?`.
- **Strings as NUL-terminated bytevectors**: `make-string`, `string`,
  `string-ref`, `string-set!`, `substring`, `string-append`,
  `string-copy`, `string-copy!`, `string-fill!`, `string->list`,
  `list->string`, `string-upcase`, `string-downcase`,
  `string-foldcase`, `string-map`, `string-for-each`,
  `string=?` / `string<?` / `string>?` / `string<=?` / `string>=?`
  (and `-ci` variants).
- **Bytevectors**: `bytevector` constructor.
- **`shell.scm`** — process and file I/O layer:
  - `argv`, `command-line`, `exit`, `spawn`, `run`, `wait`,
    `decode-wait-status`.
  - `port` record (via `define-record-type`) with a 4 KiB read buffer.
    `stdin` / `stdout` / `stderr` are pre-built ports on fds 0 / 1 / 2.
  - `open-input`, `open-output`, `open-append`, `close`,
    `file-exists?`.
  - Buffered reads: `read-bytes`, `read-line`, `read-all`. Each
    returns either `(#t . value)` (where `value` may be `eof`) or
    `(#f . errno)` from the underlying syscall.
  - Unbuffered writes: `write-bytes`, `write-string`, `write-line`.
    Writes loop until the requested length is delivered or a syscall
    error surfaces.
- **Constants**: `BUFSIZE`, `AT_FDCWD`, `O_RDONLY`, `O_WRONLY`,
  `O_CREAT`, `O_TRUNC`, `O_APPEND`, `MODE_644`, `NL-BYTE`, `NL-BV`.
