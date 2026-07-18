# libp1pp

## Scope

libp1pp is a small portable utility library for P1pp programs, written in
P1pp itself. It provides:

- M1PP control-flow macros that wrap P1 branches into structured forms
- Common byte and string primitives
- Integer parsing and formatting
- Character predicates
- Thin syscall wrappers and higher-level IO helpers
- A single-arena bump allocator
- Panic and assertion helpers

libp1pp is a single source file, `p1pp.P1pp`, composed via `catm` after the
target backend header and before user source:

    catm P1-aarch64.M1pp p1pp.P1pp usersrc.P1pp > program.M1

Because `p1pp.P1pp` passes through M1PP, it freely mixes P1 code and M1PP macro
definitions.

## Target and conventions

### Width

libp1pp targets **P1-64 only**. Word size is 8 bytes. Pointer values,
integer results, and syscall arguments are all one word.

### Syscall numbers

libp1pp does not hard-code syscall numbers. It relies on the backend header
to provide the `p1_sys_<name>` data-word macros already defined by existing
backends (e.g. `%p1_sys_read()`, `%p1_sys_write()`). User code should not
issue syscalls through raw numbers; it calls the libp1pp wrappers instead.

### Error style

IO functions follow kernel conventions: a negative return indicates an error
(typically `-errno`), a non-negative return is a success value.

Functions that return a pointer use `0` (`NULL`) to indicate failure.

Parsers return two words under the two-word direct-result convention:
`(value, consumed)`. `consumed == 0` means the input did not begin with a
syntactically valid token; the `value` word is then unspecified.

Functions whose return type is "nothing meaningful" return `0` in `a0`.

### String representation

libp1pp uses two string conventions, distinguished by the function name:

- **`(buf, len)` pair** — an explicit pointer/length pair. The default.
- **`cstr`** — a NUL-terminated byte pointer. Only functions whose name
  includes `cstr` expect or emit NUL termination.

libp1pp never mutates a caller-provided buffer it was not passed as an
output parameter.

### Allocation

libp1pp functions do not allocate. Anything that produces bytes writes into
a caller-provided buffer whose capacity is the caller's responsibility.
The single exception is the bump allocator, which allocates only from a
region the caller explicitly installed.

### Internal label namespace

libp1pp reserves the **global** label prefix `libp1pp__` for all internal
state and helper labels — bump allocator cursor/base/cap words, internal
scratch buffers used by `print_int` / `print_hex`, private helper routines.
User code must not define globals beginning with `libp1pp__`, and must not
reference them directly; everything libp1pp exposes is reachable through
its documented functions and macros.

Labels *inside* libp1pp functions are scope-local hex2++ dotted labels
(`:.loop`, `&.done`) and never appear in the global namespace, so they
cannot collide with user labels. The `libp1pp__` prefix matters only for
file-scope data and helper functions.

Public entry points (the functions and macros listed in this document,
such as `memcpy`, `bump_alloc`, `%if_eq`) are unprefixed. A user who
sees an undefined-label error for a `libp1pp__` name at link time has
almost certainly forgotten to `catm` `p1pp.P1pp` into the build.

### Initialization

libp1pp requires no global init step at program entry. Subsystems are
either self-initializing or require an explicit per-subsystem init
call, documented with that subsystem.

The only subsystem that requires explicit init is the bump allocator:
`bump_alloc` called before `bump_init` returns `0` (the "arena
exhausted" sentinel) because no arena is installed yet. Every other
libp1pp function is callable from the first instruction of `p1_main`.

`p1_main` itself inherits the portable entry contract from P1:
`a0` = `argc`, `a1` = `argv`. libp1pp does not wrap or interpose on
`p1_main`.

## Control-flow macros

All control-flow macros take braced blocks as arguments. The braces are
M1PP argument delimiters; they are stripped on substitution.

There are two flavors:

- **Unscoped** forms (`%if_<cc>`, `%while_<cc>`, `%for_lt`, `%loop`) use
  M1PP per-expansion `@`-mangled labels for their internal targets. They
  emit no hex2++ `.scope` and do not interact with `%break` /
  `%continue`. Use these when the body does not need mid-body exit.

- **Scoped** forms (`%loop_scoped`, `%while_scoped_<cc>`,
  `%for_lt_scoped`) open a nested hex2++ `.scope` and define dotted
  labels `.top` and `.end` inside it. The generic `%break` and
  `%continue` macros resolve through hex2++'s innermost-out scope
  lookup, so they always target the nearest enclosing scoped loop.

### Condition suffixes

Each conditional family is expanded once per condition. Suffixes:

- Two-operand: `eq`, `ne`, `lt`, `ltu`
- Zero-operand (implicit compare against zero): `eqz`, `nez`, `ltz`

`lt` and `ltz` are signed comparisons. `ltu` is unsigned. These mirror the
P1 branch opcodes `BEQ`, `BNE`, `BLT`, `BLTU`, `BEQZ`, `BNEZ`, `BLTZ`.

### `%if_<cc>` / `%ifelse_<cc>`

    %if_eq(ra, rb, { body })
    %if_ne(ra, rb, { body })
    %if_lt(ra, rb, { body })
    %if_ltu(ra, rb, { body })
    %if_eqz(ra, { body })
    %if_nez(ra, { body })
    %if_ltz(ra, { body })

    %ifelse_eq(ra, rb, { tblk }, { fblk })
    %ifelse_ne(ra, rb, { tblk }, { fblk })
    %ifelse_lt(ra, rb, { tblk }, { fblk })
    %ifelse_ltu(ra, rb, { tblk }, { fblk })
    %ifelse_eqz(ra, { tblk }, { fblk })
    %ifelse_nez(ra, { tblk }, { fblk })
    %ifelse_ltz(ra, { tblk }, { fblk })

`%if_<cc>` executes the block when the condition is true and falls through
otherwise. `%ifelse_<cc>` executes `tblk` on true and `fblk` on false, then
falls through to the code after the macro.

Neither form establishes a new frame or changes `sp`. A block that issues
a `CALL` must sit inside a function that has already established a frame
with `ENTER`. Neither form opens a `.scope`, so `%break` / `%continue`
inside the body resolve through to the enclosing scoped loop (if any).

### `%while_<cc>` / `%do_while_<cc>`

    %while_eq(ra, rb, { body })
    %while_ne(ra, rb, { body })
    %while_lt(ra, rb, { body })
    %while_ltu(ra, rb, { body })
    %while_eqz(ra, { body })
    %while_nez(ra, { body })
    %while_ltz(ra, { body })

    %do_while_eq(ra, rb, { body })
    %do_while_ne(ra, rb, { body })
    %do_while_lt(ra, rb, { body })
    %do_while_ltu(ra, rb, { body })
    %do_while_eqz(ra, { body })
    %do_while_nez(ra, { body })
    %do_while_ltz(ra, { body })

`%while_<cc>` tests the condition before the body; `%do_while_<cc>` after.
In both, the condition is a positive sense ("continue while `ra == rb`").
The operand registers are re-read on every iteration, so body may update
them.

All `%while_<cc>` macros share a single lowering pattern so they work
uniformly across conditions, including `lt`, `ltu`, and `ltz` which have no
inverted P1 branches.

These are unscoped: they do not support `%break` / `%continue`. Use the
`%while_scoped_<cc>` family below if the body needs mid-body exit.

### `%for_lt`

    %for_lt(i_reg, n_reg, { body })

Counts `i_reg` from `0` up to but not including `n_reg`, with step `+1`,
under signed comparison. On entry, `i_reg` is set to `0`; after each body
iteration, `i_reg` is incremented by `1`; the loop exits once
`i_reg < n_reg` is false.

`n_reg` is re-read each iteration, so body may update the bound. Body may
read `i_reg` but must not otherwise modify it. If body issues a `CALL`,
the caller is responsible for keeping `i_reg` live across the call — in
practice, this means `i_reg` should be a callee-saved register (`s0`–`s3`)
or explicitly spilled.

libp1pp does not provide an unsigned variant, an immediate-bound variant, a
step-by-`k` variant, or a count-down variant. Pointer iteration and
other shapes are better expressed as `%while_<cc>` plus explicit
increments.

### `%loop`

    %loop({ body })

An unconditional unscoped loop with no built-in exit. The body runs
forever unless it transfers control out by another mechanism. Use
`%loop_scoped` if the body needs `%break`.

### Scoped loops: `%loop_scoped`, `%while_scoped_<cc>`, `%for_lt_scoped`

    %loop_scoped({ body })

    %while_scoped_eq(ra, rb, { body })
    %while_scoped_ne(ra, rb, { body })
    %while_scoped_lt(ra, rb, { body })
    %while_scoped_ltu(ra, rb, { body })
    %while_scoped_eqz(ra, { body })
    %while_scoped_nez(ra, { body })
    %while_scoped_ltz(ra, { body })

    %for_lt_scoped(i_reg, n_reg, { body })

Each scoped loop opens a hex2++ `.scope` around its expansion and defines
two dotted labels inside it: `:.top` at the point where `%continue` should
land, and `:.end` immediately after the loop. For top-tested
`%while_scoped_<cc>`, `.top` names the condition test; for
`%for_lt_scoped`, `.top` names the increment-and-test block; for
`%loop_scoped`, `.top` names the head of the body.

    %break

Emits `B &.end`. Transfers control to the `.end` of the innermost
enclosing scoped loop, resolved by hex2++'s scope-walk.

    %continue

Emits `B &.top`. Transfers control to the innermost enclosing scoped
loop's re-test / increment point.

`%break` and `%continue` work from arbitrary depth inside a scoped loop,
including inside `%if_<cc>`, `%ifelse_<cc>`, or another nested loop's
body — because none of those forms open their own `.scope`, the lookup
walks past them to the enclosing scoped loop.

A nested scoped loop *does* open its own `.scope` and shadows the outer
`.top` / `.end`. Inside the inner loop, `%break` / `%continue` target the
inner loop. To break out of an outer loop from inside an inner one, fall
through with a manual branch or a status flag — libp1pp does not provide
named-label break.

Unscoped forms (`%while_<cc>`, `%for_lt`, `%loop`) are preferred when the
body does not need `%break` or `%continue`. They emit no `.scope` and use
per-expansion local labels that cannot collide.

## Frame locals

libp1pp does not introduce a new local-variable macro. Use M1PP's `%struct`
directly: its 8-byte stride matches `WORD` on P1-64, and it already
synthesizes `%name.SIZE` for `ENTER`.

    %struct parse_f { state cursor endp tmp }

    :parse_one
    ENTER %parse_f.SIZE
      ST   a0, [sp + %parse_f.state]
      ST   a1, [sp + %parse_f.cursor]
      ...
    ERET

If the function stages stack-passed outgoing arguments for calls with more
than four word arguments, reserve the low-addressed fields for that
staging:

    %struct parse_f { _o0 _o1 state cursor endp tmp }

The caller places outgoing argument word `k` at `[sp + k * 8]` immediately
before the `CALL`, then reads locals from higher offsets. libp1pp does not
otherwise enforce this convention.

## Function definition

    %fn(name, size, { body })

Defines a non-leaf function named `name` with `size` bytes of
frame-local storage. Expands to:

- a global label `:name` at the function entry,
- a `.scope` push, so dotted labels inside `body` (`:.start`, `:.done`)
  are local to the function and never collide with sibling functions,
- an `%enter(size)` prologue,
- the body,
- an `%eret()` epilogue,
- a matching `.endscope`.

`%fn` does not itself define `.top` or `.end`, so a bare `%break` /
`%continue` directly in `body` would resolve outside the function (or
fail to resolve) — they should appear only inside a nested scoped loop.

Example:

    %struct parse_f { state cursor }

    %fn(parse_number, %parse_f.SIZE, {
      ST a0, [sp + %parse_f.state]
      ST a1, [sp + %parse_f.cursor]
      ...
      BEQZ t0, &.done
      ...
      :.done
      LD a0, [sp + %parse_f.state]
    })

`size` may be a literal byte count, a `%struct` `SIZE` reference, or any
M1PP-time integer expression that the backend `%enter` macro accepts.

    %fn2(name, { local1 local2 ... }, { body })

Like `%fn`, but the second argument is a braced list of local names
instead of a byte frame size. Synthesizes a `name_FRAME` `%struct` (one
8-byte slot per local), opens both a hex2++ `.scope` and an M1PP
`%frame` named after the function, and sizes the stack frame from
`%name_FRAME.SIZE`.

Inside the body these helpers resolve against the enclosing `%frame`:

    %local(slot)     byte offset of local `slot`
    %stl(reg, slot)  store reg into local `slot`
    %ldl(reg, slot)  load local `slot` into reg

A zero-local function uses `{}` for the locals list.

Leaf functions that need no frame do not use `%fn`: they write the
entry label, body, and `%ret()` directly, and may optionally wrap the
body in `.scope` / `.endscope` if they want scope-local dotted labels.

## Memory and strings

### Byte-buffer primitives

    memcpy(dst, src, n)         -> dst
    memmove(dst, src, n)        -> dst
    memset(dst, byte, n)        -> dst
    memcmp(a, b, n)             -> sign        # -1 / 0 / 1

These four entries are the **canonical compiler-builtin mem* runtime**
for every build chain in this tree. cc.scm + libp1pp, cc-libc (libp1pp
+ libc), tcc-cc, and tcc-gcc all resolve bare `extern memcpy` against
libp1pp here; the vendored mes-libc is flattened with its own copies
omitted so the symbols are not duplicated at hex2++ time, and the
gcc-built tcc-gcc binary links `tcc/cc/mem.c` for the same reason.

`memcpy` does not support overlapping ranges where `dst > src && dst < src + n`;
use `memmove` for overlap.

`memmove` picks the safe direction based on `dst` vs `src`.

`memset` stores only the low 8 bits of `byte`.

`memcmp` performs an unsigned byte-wise three-way compare and returns
`-1`, `0`, or `1`. It stops at the first differing byte.

### NUL-terminated strings

    strlen(cstr)                -> n
    streq(a_cstr, b_cstr)       -> 0 or 1
    strcmp(a_cstr, b_cstr)      -> sign        # -1 / 0 / 1

`strlen` returns the byte count up to but not including the terminating NUL.

`streq` returns `1` iff the two strings are byte-equal including length.

`strcmp` compares byte-wise until either a differing byte is found or one
side's NUL is reached, and returns the sign of the first difference (the
shorter string compares less when it is a prefix of the other).

## Integer parsing and formatting

### Parsers

    parse_dec(buf, len)         -> (value, consumed)
    parse_hex(buf, len)         -> (value, consumed)

Both use the two-word direct-result convention: `a0` holds the parsed
integer value and `a1` holds the number of bytes consumed. `consumed == 0`
means the input did not start with a valid literal.

`parse_dec` accepts an optional leading `-` followed by one or more decimal
digits. On overflow, the result is truncated to 64 bits modulo 2^64;
detection of overflow is not part of the portable contract.

`parse_hex` accepts one or more hex digits (`0-9`, `a-f`, `A-F`). It does
not consume a `0x` prefix; callers handle any prefix themselves. The
result is the unsigned value of the parsed digits, truncated to 64 bits.

Parsers do not skip leading whitespace.

### Formatters

    fmt_dec(buf, value)         -> n_bytes
    fmt_hex(buf, value)         -> n_bytes

Both write a human-readable representation into `buf`, starting at offset
`0`, and return the number of bytes written. Neither writes a terminating
NUL.

`fmt_dec` emits a signed decimal representation: a leading `-` for
negative values, then one or more decimal digits. At most 20 bytes are
written.

`fmt_hex` emits an unsigned lowercase hex representation with no prefix
and no leading zeros (except that `0` is rendered as `0`). At most 16
bytes are written.

Callers provide a buffer at least as large as the documented maximum.

## Character predicates

All predicates take a single one-byte value (passed as a word; the high
bits are ignored) and return `1` or `0`.

    is_digit(c)                 -> 0 or 1     # '0'..'9'
    is_hex_digit(c)             -> 0 or 1     # 0-9, a-f, A-F
    is_space(c)                 -> 0 or 1     # ' ', '\t', '\n', '\r', '\v', '\f'
    is_alpha(c)                 -> 0 or 1     # a-z, A-Z
    is_alnum(c)                 -> 0 or 1     # is_alpha OR is_digit

Predicates are functions.

## IO

### Raw syscall wrappers

    sys_read(fd, buf, len)      -> n          # bytes read; 0 at EOF; <0 error
    sys_write(fd, buf, len)     -> n          # bytes written; <0 error
    sys_open(path_cstr, flags, mode)
                                -> fd         # fd >= 0 on success; <0 error
    sys_close(fd)               -> r          # 0 on success; <0 error
    sys_exit(code)              -> !          # does not return

These are thin wrappers over the P1 `SYSCALL` op. They set the syscall
number themselves using the backend's `%p1_sys_<name>` data-word macros,
marshal arguments into the syscall-argument registers, and return the raw
kernel return value unchanged.

`sys_open` is a logical open: the backend may implement it via `open` or
`openat(AT_FDCWD, ...)` as appropriate for the target.

`sys_exit` terminates the process with the low 8 bits of `code` as the
exit status. It never returns.

No wrapper interprets the negative return as a specific errno. Callers
that need such detail inspect `a0` directly.

### Print helpers

    print(buf, len)             -> r          # 0 on success; <0 error
    println(buf, len)           -> r          # writes buf then "\n"
    print_cstr(cstr)            -> r          # writes strlen(cstr) bytes
    print_int(value)            -> r          # decimal
    print_hex(value)            -> r          # hex, no prefix
    eprint(buf, len)            -> r
    eprintln(buf, len)          -> r
    eprint_cstr(cstr)           -> r

`print*` helpers write to fd `1`; `eprint*` to fd `2`. All return `0` on a
successful write of all bytes, or a negative value if the underlying
`sys_write` reported an error. A partial write is retried until complete
or the kernel returns an error.

`print_int` and `print_hex` render into a small internal stack buffer,
then write. They allocate no heap memory.

### File helpers

    read_file(path_cstr, buf, cap)
                                -> n          # bytes read, or -1

Opens `path_cstr` read-only, reads up to `cap` bytes into `buf`, and
closes the fd. Returns the number of bytes read on success, or `-1` if
the file could not be opened, a read failed, or the file exceeds `cap`
(in which case `buf` may have been partially written).

    write_file(path_cstr, buf, len)
                                -> r          # 0 on success; -1 on error

Creates or truncates `path_cstr`, writes `len` bytes from `buf`, and
closes the fd. Returns `0` on success or `-1` if any step failed. The
created file's mode is implementation-defined but intended to be a
reasonable default (typically `0644`).

## Bump allocator

libp1pp provides a single global bump allocator. Memory is carved from a
caller-supplied region; libp1pp does not own or reserve storage itself.

    bump_init(base, cap)        -> 0

Installs `[base, base + cap)` as the live arena and sets the cursor to
`base`. Discards any prior state. `base` should be word-aligned; `cap`
should be a multiple of 8. libp1pp does not validate these.

    bump_alloc(n)               -> ptr        # 0 on exhaustion

Advances the cursor by `n` bytes rounded up to the next multiple of 8 and
returns the pre-advance cursor. Returns `0` and leaves the cursor
unchanged if the rounded-up request would exceed the arena.

Returned memory is not zeroed. Callers that need zero-init memset
themselves.

    bump_mark()                 -> saved

Returns the current cursor value as an opaque word.

    bump_release(saved)         -> 0

Rewinds the cursor to a value previously returned by `bump_mark`. Any
pointers handed out since that mark become invalid. Passing a value that
was not produced by `bump_mark` against the currently installed arena is
undefined behavior.

    bump_reset()                -> 0

Rewinds the cursor to the arena's `base`.

libp1pp provides exactly one arena.

## Panic and assertions

### `panic`

    panic(msg_cstr)             -> !

Writes `msg_cstr` followed by `"\n"` to fd `2`, then calls `sys_exit(1)`.
Does not return.

User code is encouraged to use `panic` for its own invariant violations.

### `%assert_<cc>` macros

    %assert_eq(ra, rb, msg_label)
    %assert_ne(ra, rb, msg_label)
    %assert_lt(ra, rb, msg_label)
    %assert_ltu(ra, rb, msg_label)
    %assert_eqz(ra, msg_label)
    %assert_nez(ra, msg_label)
    %assert_ltz(ra, msg_label)

Each macro asserts that the named condition holds and calls `panic` with
`msg_label` (a NUL-terminated string label in the program image) if it
does not. They lower to a `B<cc>` past an `LA a0, msg_label` /
`CALL &panic` sequence, so the non-failure path adds no runtime cost
beyond the original branch.

Because the failure path issues a `CALL`, `%assert_*` may be used only in
functions that have established a frame with `ENTER`.

