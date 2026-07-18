# M1PP

## Scope

M1PP is a tiny single-pass macro expander. Its output is consumed
directly by `hex2++` — there is no intermediate macro/hex stage. All
emission is in the byte/label/directive vocabulary `hex2++` accepts.

The implementation lives in `M1pp/M1pp.c`. It is one pass, allocation-free
(fixed static buffers), and stops at the first error.

## Features

- Function-like macros with parameters (`%macro` / `%endm`); zero-arg call
  as `%NAME` or `%NAME()`
- Brace-grouped arguments (`{ ... }`) to pass token spans containing commas
  or parens as a single argument
- Token paste with `##` after argument substitution
- Recursive expansion: expanded bodies are rescanned, so macros can call
  macros
- Local labels (`:@name` / `&@name`) rewritten per-expansion for hygienic
  intra-macro labels
- Struct and enum synthesis (`%struct`, `%enum`) generating per-field
  zero-arg macros plus `SIZE`/`COUNT`
- Named stack-frame access via `%frame` / `%endframe` + `%local(field)`,
  composing with `%struct`-generated `<frame>_FRAME.<field>` macros
- Compile-time integer expression language (Lisp S-expressions:
  arithmetic, bitwise, shift, comparison, `strlen`)
- Little-endian hex emission: `!` (1B), `@` (2B), `%` (4B), `$` (8B) —
  emits bare hex digits (e.g. `AABBCCDD`) consumable by `hex2++`
- Raw byte emission from string literals: a bare `"..."` token at
  statement position emits its decoded bytes
- Conditional token selection: `%select(cond, then, else)`
- Stringification: `%str(IDENT)` produces a `STRING` token holding the
  identifier text, which then emits as bytes
- Line comments (`#`, `;`); whitespace-insensitive output normalization
- Single-pass, allocation-free implementation with fixed static buffers;
  fail-fast on first error

Lexical scoping for control-flow labels is delegated to `hex2++`'s
nestable `.scope` / `.endscope`; M1PP itself only handles per-expansion
macro hygiene labels.

## Invocation

    m1pp input.M1 output.M1

Input is read whole into a fixed buffer (`MAX_INPUT` bytes); output
is written whole from another (`MAX_OUTPUT` bytes).

## Lexical structure

The lexer produces a flat token array. Token kinds:

- `WORD` — any run of non-special characters
- `STRING` — `"..."` or `'...'` (quotes included in the token text).
  Inside a string, a backslash plus the next character is consumed as
  one unit, so `\"` and `\\` do not end the literal. The escape's
  *meaning* is decoded at emission (see [String emission](#string-emission));
  the lexer only uses the backslash to find the right closing quote.
- `NEWLINE` — a single `\n`
- `LPAREN`, `RPAREN`, `COMMA`, `LBRACE`, `RBRACE`
- `PASTE` — the `##` marker

Whitespace other than newlines is discarded. Line comments start with `#` or
`;` and run to end-of-line. Output formatting is normalized to tokens
separated by spaces and newlines; original spacing is not preserved.

## Directives

Directives are recognized via `%X`.

### `%macro` / `%endm`

    %macro NAME(p1, p2, ...)
    ... body tokens ...
    %endm

Defines a function-like macro. Zero-parameter macros are written `%macro
NAME()`. Macros are define-before-use; there is no prescan. Recursive
macros are not detected and will loop until a buffer limit fires.

### `%struct`

    %struct NAME { f1 f2 f3 ... }

Synthesizes zero-parameter macros for fixed 8-byte-per-field layout:

- `%NAME.f1` → `0`
- `%NAME.f2` → `8`
- `%NAME.f3` → `16`
- `%NAME.SIZE` → `N * 8`

Fields are separated by whitespace, commas, or newlines.

### `%enum`

    %enum NAME { l1 l2 l3 ... }

Like `%struct` with stride 1 and a trailing `COUNT`:

- `%NAME.l1` → `0`, `%NAME.l2` → `1`, ...
- `%NAME.COUNT` → `N`

### `%frame` / `%endframe`

    %frame NAME
    ... body ...
    %endframe

Sets a single-slot "current frame" to `NAME`, consulted by `%local` to
look up named offsets in `<NAME>_FRAME.<field>` macros (typically
synthesized by `%struct`). Frames do not nest: a second `%frame` before
`%endframe` is an error. Every `%frame` must be closed before
end-of-input. `NAME` is a single `WORD` token and may come from
macro-argument substitution.

## Macro calls

    %NAME(arg, arg, ...)

Arguments are comma-separated token spans, with parentheses and braces
balanced inside an argument. A zero-parameter macro may be invoked either
as `%NAME()` or as a bare `%NAME`.

An argument wrapped in a single outer pair of `{ ... }` has the braces
stripped on substitution. This lets a comma-containing or paren-containing
token sequence be passed as a single argument: `%foo({a, b, c})` passes one
argument whose tokens are `a , b , c`.

Argument substitution happens inside the body. After substitution, `##`
token-paste is applied: `left ## right` becomes a single `WORD` token whose
text is the concatenation of the two operand tokens' text. Operands of
`##` must be exactly one non-braced token; newlines and other `##` tokens
are not valid neighbors.

The expanded body is then rescanned by pushing it onto the stream stack, so
macros can call other macros.

### Local labels

Inside a macro body, a token starting with `:@name` or `&@name` is a local
label definition or reference. On expansion, `@` is replaced by `__N` where
`N` is a monotonically increasing expansion id, so each call site gets a
fresh label namespace:

- `:@loop` → `:loop__7`
- `&@loop` → `&loop__7`

Each macro expansion gets a fresh `N`, so `:@loop` in two different call
sites (or two different macros) never collide. Argument-substituted tokens
keep their original text and are not rewritten, so a `:@name` literal
passed as a macro argument passes through verbatim.

These labels exist only to keep macro-internal symbols from colliding
with each other or with caller code. Lexical scoping for control-flow
labels (e.g. `loop`/`break` patterns where an inner macro must reference
a label defined by an outer macro) belongs to `hex2++`'s `.scope` —
emit `.scope` / `.endscope` and dotted local labels from your macro
bodies, and rely on `hex2++`'s innermost-out lookup to bind references.

## Built-in calls

These are recognized wherever a token matches, not only at line start.

### Integer emission: `!` `@` `%` `$`

    !(expr)    →  1-byte  little-endian hex, e.g. AB
    @(expr)    →  2-byte  little-endian hex
    %(expr)    →  4-byte  little-endian hex
    $(expr)    →  8-byte  little-endian hex

The expression is evaluated to a signed 64-bit integer and emitted as
bare hex digits (e.g. `AABBCCDD`). `hex2++` consumes whitespace-separated
hex bytes directly, so no quoting or wrapping is required.

### `%select(cond, then, else)`

Evaluates `cond` as an expression. If nonzero, the `then` argument's tokens
are pushed back for rescan; otherwise the `else` argument's tokens are. The
branches are raw token spans, not expressions.

### `%str(IDENT)`

Stringifies a single `WORD` token into a `STRING` token wrapping the
identifier text in double quotes. The argument must be exactly one word
token. The resulting `STRING` flows through emission like any bare
string literal: `%str(foo)` produces the same output bytes as `"foo"`
(`66 6F 6F`). Use it when the identifier is built up from macro
arguments or `##` paste and you want its text emitted as bytes.

### String emission

A `"..."` token reaching the output stream is decoded into raw bytes,
one two-hex-digit `WORD` token per byte. `hex2++` coalesces hex digits
across whitespace, so the result reassembles into a contiguous byte
sequence at link time. No NUL terminator is appended; write `00`
explicitly (or use `\0`) if you need one. Recognised escapes inside the
string are:

    \n  0x0A    \t  0x09    \r  0x0D    \0  0x00
    \\  0x5C    \"  0x22    \xNN  byte NN (two hex digits)

Any other backslash escape is an error. Example: `:msg "hi\n"` emits
`68 69 0A` immediately after defining `:msg`.

Strings inside expression arguments (e.g. `(strlen "literal")`) and
inside `%str(IDENT)` are not decoded — the string atom is read by the
expression evaluator instead.

### `%local(NAME)`

Looks up the zero-parameter macro `<frame>_FRAME.<NAME>`, where
`<frame>` is the currently active `%frame`, and emits its body. Errors
if no frame is active or the field is undefined. `NAME` must be exactly
one `WORD` token.

`%local` is also recognized as an expression atom, so it composes with
`%(...)` arithmetic: `%(+ %local(off) 4)` evaluates as expected.

The intended pattern combines `%struct`, `%frame`, and `%local` for
named stack-frame access:

    %struct foo_FRAME { saved_buf saved_len }
    :foo
    .scope
    %frame foo
    %enter(%foo_FRAME.SIZE)
    ;; %local(saved_buf) -> 0, %local(saved_len) -> 8
    %eret
    %endframe
    .endscope

## Expression language

Expressions are Lisp-shaped S-expressions. Atoms are integer literals
(decimal, or any base accepted by `strtoull`/`strtoll`, including `0x...`)
or zero-arg macro calls that evaluate to integer tokens.

Calls:

    (+ a b ...)   (- a b ...)   (* a b ...)   (~ a)
    (/ a b)       (% a b)       (<< a b)      (>> a b)
    (& a ...)     (| a ...)     (^ a ...)
    (= a b)       (!= a b)
    (< a b)       (<= a b)      (> a b)       (>= a b)
    (strlen "literal")

- `+ - * & | ^` are n-ary with at least one argument. Unary `-` negates.
- `/ % << >> = != < <= > >=` are strictly binary.
- `~` is unary.
- `strlen` takes one `STRING` token and returns the raw byte count of the
  contents between the quotes.

Inside an expression, a `%NAME` that names a zero-parameter (or invokable)
macro is expanded and its tokens are re-parsed as a sub-expression. This
is how `%struct` and `%enum`-generated names compose into arithmetic.

## Limits

Various limits are fixed at compile time. See the code for values.

| Resource              |
| --------------------- |
| input bytes           |
| output bytes          |
| total token text      |
| source tokens         |
| macro body tokens     |
| expansion pool tokens |
| macros                |
| parameters per macro  |
| stream stack depth    |
| expression frames     |

Exceeding any limit aborts with an error message on `stderr`.

## Errors

On failure, `m1pp` prints `m1macro: <reason>` to `stderr` and exits 1.
See code for reasons.
