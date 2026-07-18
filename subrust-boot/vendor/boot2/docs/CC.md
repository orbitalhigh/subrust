# Minimal C subset (boot2)

Working doc. Baseline is C99; everything here is a delta against it. The
target is **just enough C** to compile

  `tcc-0.9.26-1147-gee75a10c/tcc.c`

with the same defines used at MesCC's `tcc-mes` stage in
[live-bootstrap](../../live-bootstrap/steps/tcc-0.9.26/pass1.kaem):

```
-D BOOTSTRAP=1
-D HAVE_LONG_LONG=1
-D ONE_SOURCE=1
-D TCC_TARGET_X86_64=1
-D inline=
-D CONFIG_TCCDIR="..."  ...etc
```

Notably **not** defined: `HAVE_FLOAT`, `HAVE_BITFIELD`, `HAVE_SETJMP`.
Those gate off entire code paths in tcc.c (floats, bitfield struct
support, setjmp-based error recovery), and we don't have to compile any
of it.

The accepted surface is shaped by two intersecting constraints:

1. **Lower bound** — what tcc.c (under those defines) actually uses.
2. **Upper bound** — what MesCC accepts, since MesCC already builds
   tcc-mes and we're its replacement. Anything MesCC strips silently
   (`const`, `inline`, `__attribute__`) we also strip silently.

Things outside both bounds are cut. Things admitted are load-bearing.

## Scope

- **Single translation unit.** Input is one bytestream. The
  preprocessor does no file I/O — `#include` is an external
  pre-flattening pass (system headers + tcc.c's `#include "libtcc.c"`
  / `"tcctools.c"` are spliced upstream of our compiler). See
  [§Toolchain envelope](#toolchain-envelope).
- **P1-64 only.** Sizes assume LP64. Porting to P1-32 is out of scope.
- **No optimization.** Output P1pp is a stack-machine lowering with
  every operand spilled to a frame slot. Codegen quality is a v2
  problem.

## Toolchain envelope

```
tcc.c + system headers
       │
       │ pre-flatten: resolve #include recursively, splice into one file
       │ (separate tool: scheme1 or shell; not part of cc.scm)
       ▼
tcc.flat.c                          single bytestream, no #include
       │
       │ scheme1 cc.scm
       ▼
tcc.P1pp                            our compiler's output
       │
       │ catm with arch backend + libp1pp.P1pp
       │ m1pp
       ▼
tcc.M1
       │ M0
       ▼
tcc.hex2
       │ hex2
       ▼
tcc-mes                             native ELF, replaces MesCC's tcc-mes
```

The pre-flatten pass is *not* a C preprocessor — it only resolves
`#include`. All other directives (`#define`, `#if`, …) are handled by
the in-Scheme preprocessor in pass 2.

## Translation phases

The C standard names eight phases. We collapse them to three:

1. **Lex** — bytestream → token list. Trigraphs and line-splicing
   (backslash-newline) are handled here, alongside numbers / strings /
   identifiers / punctuators. Comments removed. Newlines preserved as
   `NL` tokens (the preprocessor needs them to delimit directives).
2. **Preprocess** — token list → expanded token list. Directives
   consumed, macros expanded, `NL` tokens stripped on exit.
3. **Parse + emit** — token list → P1pp text. xcc-style direct emit;
   no AST.

## Lexical syntax

Subset of C99 lexical grammar.

- **Identifiers**: `[a-zA-Z_][a-zA-Z_0-9]*`. Universal character names
  (`\uXXXX`) **not** supported.
- **Integers**: decimal, octal (`0…`), hex (`0x…`); suffixes
  `u`, `U`, `l`, `L`, `ll`, `LL`, `ul`, `ull`, etc. (case-insensitive).
  All values fit in `unsigned long long` (64 bits).
- **Floats**: **not** present. The lexer rejects floating-point
  literals. (HAVE_FLOAT is off.)
- **Characters**: `'c'` and standard escapes `\n \t \r \\ \' \" \0
  \xNN \NNN`. `'\xNN'` is a `char`-typed value, not multi-character.
  Multi-character constants (`'AB'`) are **not** supported.
- **Strings**: `"…"` with same escapes. Adjacent string literals
  concatenate (`"a" "b"` ≡ `"ab"`). Wide strings (`L"…"`), UTF-8
  strings (`u8"…"`), UTF-16/32 (`u"…"`, `U"…"`) **not** supported.
- **Punctuators**: full C99 set, including digraphs `<: :> <% %> %:`.
  Trigraphs are handled in lex. `##` and `#` are preprocessor-only.
- **Comments**: `// …` to end of line; `/* … */` block (no nesting).
- **Line splicing**: `\` immediately before newline removes both,
  per the standard.
- **Whitespace**: space, tab, vertical tab, form feed, newline.

## Preprocessor

Directive set:

- `#define NAME …` — object-like
- `#define NAME(p1, p2, …) …` — function-like
- `#define NAME(p1, …, …) …` — variadic, with `__VA_ARGS__` in body
- `#undef NAME`
- `#if expr`, `#ifdef NAME`, `#ifndef NAME`
- `#elif expr`, `#else`, `#endif`
- `#error msg…` — flush and exit nonzero
- `#line NN ["file"]` — accepted; only `__LINE__` / `__FILE__` honor it
- `#pragma …` — accepted and ignored (whole line consumed)
- `#include …` — **rejected**. Pre-flattening handles this upstream.
  We refuse rather than silently ignore so an unflattened input fails
  loudly.

Operators inside the body of a function-like macro:

- `#param` — stringize. Result is a string literal of `param`'s
  pre-expansion tokens.
- `a##b` — token paste. Performed before rescanning for further
  expansion.

Built-in macros:

- `__FILE__` — current source file (a string literal)
- `__LINE__` — current line number (a decimal integer)
- `__DATE__` — `"Jan  1 1970"` (fixed; we don't read the wall clock)
- `__TIME__` — `"00:00:00"` (fixed)
- `__STDC__` — `1`
- `__STDC_VERSION__` — `199901`
- `__STDC_HOSTED__` — `1`
- `__LISPCC__` — `1` (our analogue of MesCC's `__MESC__`)

Adjacent string-literal tokens in the post-expansion stream are
concatenated (translation phase 6).

Expression evaluator (used by `#if`/`#elif`):

- All integer operators including `defined NAME` / `defined(NAME)`.
- Identifiers that aren't macros evaluate to `0`. (Standard.)
- Result is a 64-bit signed integer.

Macro expansion uses C11 6.10.3.4 hide-set discipline. Each token
carries the set of macro names already expanded into it; an identifier
inside its own hide-set is not re-expanded. This is the standard
defense against `#define A B\n#define B A`.

## Types

### Primitives (P1-64)

| Type                  | Size (bytes) | Align | Notes                        |
|-----------------------|--------------|-------|------------------------------|
| `void`                | —            | —     | only as ptr-target / fn-ret  |
| `char`                | 1            | 1     | signed by default            |
| `signed char`         | 1            | 1     |                              |
| `unsigned char`       | 1            | 1     |                              |
| `short`               | 2            | 2     |                              |
| `unsigned short`      | 2            | 2     |                              |
| `int`                 | 4            | 4     |                              |
| `unsigned int`        | 4            | 4     |                              |
| `long`                | 8            | 8     | LP64                         |
| `unsigned long`       | 8            | 8     |                              |
| `long long`           | 8            | 8     | same as `long` in LP64       |
| `unsigned long long`  | 8            | 8     |                              |
| pointer               | 8            | 8     | tag-free; raw native address |
| `_Bool`               | 1            | 1     | values: `0`, `1`             |

`size_t` is `unsigned long`; `ptrdiff_t` is `long`; `intptr_t` /
`uintptr_t` are `long` / `unsigned long`. These typedefs come from the
flattened headers; the language doesn't bake them in.

**Floating-point types** (`float`, `double`, `long double`,
`_Complex`, `_Imaginary`) are **parsed but never codegen'd**: prototypes
and struct fields involving them are accepted (so the flattened tcc.c
TU can be ingested), and `sizeof` reports the standard SysV widths
(4/8/8). Any attempt to materialize an fp value — load, store, cast,
arithmetic, call/return — dies with `fp not codegen'd`. tcc.c only uses
fp under `HAVE_FLOAT`, which is off, so live code never trips the cg
guard. **Not present**: `__int128`. `float.h` macros and `<math.h>` are
unavailable to the input.

### Derived types

- **Pointer**: `T *`, multi-level. `void *` is a generic pointer that
  freely converts to and from any other object pointer.
- **Array**: `T[N]` with `N` a constant expression evaluating to a
  positive integer. `T[]` is allowed in function parameter position
  (decays to `T*`) and as a flexible-array tail field. **VLAs**
  (`T[expr]` with non-constant `expr`) are **not** supported.
- **Function**: `T(P1, P2, ..., Pn)` and `T(P1, ..., ...)` (variadic).
  Pointers to functions, arrays of pointers to functions, and
  functions returning pointers to functions all parse via the
  spiral-declarator grammar. Old-style (K&R) function definitions are
  **not** supported.
- **Struct / union**: declared with `struct tag { ... }` or
  `union tag { ... }`. Tag and member namespaces are separate from
  identifiers. Forward declarations (`struct tag;`) supported.
  Anonymous structs/unions inside other structs are **not** supported.
  **Bitfields** (`int x : 3`) are **not** supported (HAVE_BITFIELD off
  in our target). Flexible array member as last field allowed:
  `struct s { int n; T data[]; }`.
- **Enum**: `enum tag { A, B = 7, C }`. Underlying type is `int`.
  Constants are usable in constant expressions.
- **Typedef**: `typedef T name;` — name becomes a type-name token in
  later declarations. Must be visible at parse time of any use
  (lexer/parser cooperation: typedef names are tracked in the
  current scope).

### Qualifiers

- `const`, `volatile`, `restrict` — **parsed and discarded**.
  We don't enforce const-correctness, don't suppress optimization
  on volatile (no optimizer to suppress), and don't honor restrict.
  Same as MesCC.
- `_Atomic`, `_Thread_local` — **rejected** (lex error if they appear;
  tcc.c doesn't use them, so this won't fire).

## Declarations and storage

### Declarators

Full C99 spiral-declarator grammar:

```
int  *p             // pointer to int
int  *p[10]         // array of 10 pointers to int
int (*p)[10]        // pointer to array of 10 ints
int (*f)(int, int)  // pointer to function (int,int) returning int
int  *f(int)        // function (int) returning pointer to int
char *(*tab[5])(int) // array of 5 pointers to function (int) returning char*
```

### Storage classes

- `extern` — declares without defining. References resolve at link
  time. Honored.
- `static` at file scope — gives internal linkage; prevents the symbol
  from being emitted as a P1pp `:public_label`. Honored.
- `static` at block scope — single shared instance, zero-initialized
  by default. Honored.
- `auto` — accepted, no effect (the default for block scope).
- `register` — accepted, no effect.
- `typedef` — handled specially (see Types).

### Function definitions

```
[storage] [type-quals] return-type name(params) { body }
```

Parameter list forms:

- `void` (zero parameters)
- `T1 p1, T2 p2, ...`
- `T1 p1, T2 p2, ..., ...` (variadic, `va_list` discipline below)

K&R-style (`int f(a, b) int a, b; { … }`) is **not** supported.

### Variable initializers

- Scalars: `T x = expr;` — `expr` must be a constant for static-storage
  variables; arbitrary for auto-storage.
- Arrays: `T a[N] = { e0, e1, ... };` and `T a[] = { ... };` (size
  inferred). String-literal initializer for `char[]` allowed.
- Structs: `S s = { e0, e1, ... };` (positional). Designated
  initializers (`{ .field = ... }`) **supported** at struct top level
  only — required by tcc.c.
- Nested initializers brace-flatten the obvious way.

### Inline / attributes

- `inline` — already removed by `-D inline=` in the bootstrap. Our
  preprocessor would also strip the keyword if it appeared. No
  effect on codegen either way.
- `__attribute__((...))` — parsed and discarded everywhere it
  appears in declarations.

## Statements

All standard C statements:

- expression statement, including the empty `;`
- compound statement `{ ... }`, with declarations interleaved with
  statements (C99-style, not K&R block prologue)
- `if (e) S` / `if (e) S else S`
- `while (e) S`, `do S while (e);`
- `for (init; cond; step) S` — `init` may be a declaration (C99)
- `switch (e) { case K: ... default: ... }` — `case K` requires `K`
  constant-integer; fall-through is the default; no implicit break
- `break;`, `continue;`
- `goto label;`, `label:` — function-scope labels
- `return;`, `return e;`
- declaration as statement (C99)

Cut:

- statement expressions `({ ... })` (GCC ext) — tcc.c doesn't use them
- `__label__` (GCC) — N/A
- compound literals `(T){ ... }` — tcc.c doesn't use them
- `_Generic` selection — tcc.c doesn't use it
- inline asm `__asm__(...)` — N/A; tcc.c gates this on conditions
  that aren't active at the tcc-mes stage

## Expressions

All standard C operators with standard precedence and associativity:

| Tier (high → low) | Operators |
|-------------------|-----------|
| postfix           | `a[i]`, `f(a,...)`, `s.m`, `p->m`, `e++`, `e--` |
| unary             | `++e`, `--e`, `&e`, `*e`, `+e`, `-e`, `~e`, `!e`, `sizeof`, `(T)e` |
| multiplicative    | `*`, `/`, `%` |
| additive          | `+`, `-` |
| shift             | `<<`, `>>` |
| relational        | `<`, `<=`, `>`, `>=` |
| equality          | `==`, `!=` |
| bitwise           | `&`, `^`, `|` (in that order) |
| logical           | `&&`, `||` |
| conditional       | `?:` |
| assignment        | `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `<<=`, `>>=`, `&=`, `^=`, `|=` |
| comma             | `,` |

Notes:

- `sizeof T` and `sizeof e` both supported. `sizeof e` does **not**
  evaluate `e` (standard).
- Integer promotion (rank ≤ `int` → `int`) and usual arithmetic
  conversions performed automatically. Pointer arithmetic scales by
  pointee size.
- Implicit conversions for assignment, return, and function arguments
  (incl. promotion of variadic args to `int` / `unsigned int` /
  pointer / `long` / `unsigned long`).
- String literals have type `char *` (not `const char[N]`) for our
  purposes — we strip const, and tcc.c writes through string literals
  in a few places.
- `_Alignof` — **not** supported. tcc.c uses no alignment intrinsics.

### Variadic argument access

```
#include <stdarg.h>     // pre-flattened in
void f(int n, ...) {
    va_list ap; va_start(ap, n);
    int x = va_arg(ap, int);
    va_end(ap);
}
```

`va_list`, `va_start`, `va_arg`, `va_end` are macros from the
flattened header. They expand to direct frame-slot reads keyed off the
`...` slot offset our codegen exposes. Implementation detail: our
`stdarg.h` substitute is one of the headers shipped with the
compiler.

## Standard library expectations

Our compiler doesn't bundle libc. The bootstrap script links the
output against the same `libc+tcc` archive MesCC uses, which provides:

- `<stdio.h>`: `FILE`, `fopen`, `fclose`, `fread`, `fwrite`, `fprintf`,
  `fputs`, `fgetc`, `getc`, `printf`, `sprintf`, `vsnprintf`, …
- `<stdlib.h>`: `malloc`, `free`, `realloc`, `exit`, `atoi`, `strtol`,
  `qsort`, …
- `<string.h>`: `strlen`, `strcpy`, `strncpy`, `strcmp`, `strncmp`,
  `strcat`, `strchr`, `strrchr`, `strstr`, `memset`, `memcpy`,
  `memmove`, `memcmp`, …
- `<ctype.h>`, `<errno.h>`, `<unistd.h>`, `<fcntl.h>`, `<sys/stat.h>`,
  …
- `<stdarg.h>`, `<stddef.h>`, `<limits.h>` — supplied by us.

Anything `<setjmp.h>` is **not** required at the tcc-mes stage
(`HAVE_SETJMP` off). `<math.h>` is not required (`HAVE_FLOAT` off).

Built-in functions our compiler *recognizes* (vs. linking against):

- `__builtin_va_start`, `__builtin_va_arg`, `__builtin_va_end` —
  expanded inline by the codegen. The `<stdarg.h>` we ship aliases
  the standard names to these.
- `alloca` — left as a library call. tcc.c only references it via
  `__builtin_alloca` definition for compiled programs, not for itself.

## Cut from C99 / C11

Kept explicit so additions are deliberate.

| Feature                                       | Status   | Rationale                                      |
|-----------------------------------------------|----------|------------------------------------------------|
| Floats / doubles / `_Complex`                 | parse-only | parsed as types; cg rejects fp ops (HAVE_FLOAT off) |
| `long double`                                 | parse-only | same softening; sized as 8 bytes              |
| Bitfields                                     | rejected | HAVE_BITFIELD off                              |
| `setjmp` / `longjmp`                          | not lib  | HAVE_SETJMP off                                |
| VLAs                                          | rejected | tcc.c doesn't use; complicates frame layout    |
| Compound literals `(T){...}`                  | rejected | tcc.c doesn't use                              |
| Statement expressions `({...})` (GCC)         | rejected | tcc.c doesn't use                              |
| `_Generic`                                    | rejected | not used                                       |
| `_Atomic`, `_Thread_local`                    | rejected | not used                                       |
| `_Alignof`, `_Alignas`                        | rejected | not used                                       |
| `_Static_assert`                              | rejected | not used                                       |
| Wide / UTF strings (`L"…"`, `u8"…"`)          | rejected | not used                                       |
| Anonymous struct/union members                | rejected | not used                                       |
| Multi-character constants (`'AB'`)            | rejected | not used                                       |
| Universal character names (`\uXXXX`)          | rejected | identifier set is ASCII only                   |
| K&R-style function definitions                | rejected | tcc.c uses ANSI                                |
| Nested function definitions (GCC)             | rejected | not used                                       |
| Inline assembly (`__asm__`)                   | rejected | not used at this stage                         |
| `__label__` (GCC)                             | rejected | not used                                       |
| `#include`                                    | rejected | external pre-flatten step                      |
| `const`, `volatile`, `restrict`               | parsed, discarded | match MesCC                          |
| `inline`                                      | parsed, discarded | -D inline= in bootstrap              |
| `__attribute__((...))`                        | parsed, discarded | match MesCC                          |
| `register`, `auto` storage classes            | parsed, no effect |                                       |

## Undefined behavior policy

Following [LISP.md](LISP.md)'s "Primitive failure" stance: out-of-bounds
array access, signed integer overflow, dereferencing a null or
uninitialized pointer, integer division by zero, and modifying a string
literal are **undefined**. The compiler emits no runtime checks; the
generated P1pp will crash, loop, or produce nonsense, and that's
acceptable.

The compiler itself aims to be **deterministic**: the same input bytes
produce identical output bytes. Errors detected at compile time
(syntax errors, type errors, unresolved identifiers) abort with a
diagnostic on stderr and a nonzero exit code. No partial output is
written.

## Validation milestones

Status legend: `[x]` done · `[~]` in progress · `[ ]` not started.

1. [ ] Self-tests: a tests/cc/ tree mirroring tests/scheme1/ — one
   tiny `.c` file per language feature, exit-status-driven.
2. [ ] Compile a hand-written single-file C "hello world" through to
   ELF.
3. [ ] Compile the mes libc unified-libc.c (the same file MesCC builds
   into libc.a).
4. [ ] Compile tcc.c (under the tcc-mes defines) → tcc-boot2; verify
   `tcc-boot2 -version` runs.
5. [ ] Use tcc-boot2 to build tcc-boot0; verify checksum matches the
   live-bootstrap reference.

Hitting (5) is the bootstrap milestone — at that point boot2 has
fully replaced MesCC in the chain.
