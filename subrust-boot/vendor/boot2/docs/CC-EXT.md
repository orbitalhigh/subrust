# cc-ext — c-testsuite coverage for cc.scm

External breadth-coverage for `cc.scm`, on top of the hand-written
`tests/cc/` and `tests/cc-libc/` fixtures. Each fixture is a small,
self-contained C program from
[c-testsuite](https://github.com/c-testsuite/c-testsuite) (vendored
under `vendor/c-testsuite/single-exec/`); per the suite's spec the
program returns 0 on success and the `.expected` file pins
stdout+stderr.

The point of having someone else's tests in the tree is that they
exercise corners of C we would not naturally write tests for — they
surface bugs we don't know to look for.

## Running

```
make test SUITE=cc-ext                              # full suite, every arch
make test SUITE=cc-ext ARCH=aarch64                 # one arch
sh scripts/run-tests.sh --suite=cc-ext \
    --arch=aarch64 00050 00089                      # specific fixtures
```

`make test SUITE=cc-ext` is opt-in — it is not part of the default
`make test` loop, since most fixtures still fail and we don't want
that drowning out signal from the curated suites.

Each fixture's `.tags` file lists feature requirements (`portable`,
`c89`, `c99`, `c11`, `needs-libc`, `needs-cpp`). The runner switches
pipelines on the tags:

- `needs-libc` → compiled with `--lib=app__` and linked through the
  same chain as the `cc-libc` suite (entry-libc + mes-libc.P1pp +
  client + elf-end).
- otherwise → bare `cc.scm` -> P1pp -> ELF.

Compile, assemble, and runtime errors all count as FAIL. There are no
auto-skips: the goal is an honest pass/fail count that drops as
`cc.scm` grows.

## Current status (aarch64, snapshot 2026-05-02)

- **143 PASS / 77 FAIL** out of 220 fixtures.

Failure groups, largest first:

| count | error | likely root cause |
|------:|-------|-------------------|
|    62 | `#include: file inclusion is handled upstream by pre-flatten` | preprocessor doesn't resolve system headers; needs-libc tests use `#include <stdio.h>`. The `cc-libc` suite sidesteps this with explicit `extern int printf(...)` declarations. Either (a) add an include search path + minimal `stdio.h`/`stdlib.h`/`string.h` shims that emit the same `extern` declarations, or (b) pre-flatten these fixtures before handing them to `cc.scm`. |
|     5 | `const-expr: bad operand: lbrack` | designated array initializer `[N] = ...`. Out of scope — cc.scm intentionally doesn't support designated array init. |
|     4 | P1pp assemble (hex2 link) failed — fixtures 00210/00211/00215/00217 | undefined symbols against libc. 00211/00215/00217 are tagged `needs-libc` but call routines we haven't wired up; 00210 uses `printf` without the `needs-libc` tag, so the runner pipelines it bare. |
|     2 | `unexp: lbrace` — fixtures 00213, 00214 | GCC statement expression `({ ... })`. Substantial feature; not on the path for tcc.c bootstrap. |
|     1 | `init: too many fields` — fixture 00216 | extensive use of designated/range/flex-array initializer features (`[1 ... 5] = 9`, etc.). |
|     1 | `field` — fixture 00218 | bit-field declaration (`enum tree_code code : 8`). Bit-fields not implemented. |
|     1 | `floating-point literal not supported` — fixture 00123 | no float support in cc.scm. Out of scope. |
|     1 | `undecl: L` — fixture 00098 | wide-char literal `L"..."` / `L'x'`. Out of scope. |

## Recently fixed

The 2026-05-02 sweep flipped 5 singletons green via small targeted
changes:

- 00050 — anon-union inside struct init: `%parse-init-struct-list/mode`
  in brace-elision mode now terminates after one element when the
  aggregate is a union, leaving the next sibling for the parent.
- 00089, 00095 — `&function`: `cg-take-addr` retags a function-typed
  global rval as ptr-to-fn instead of dying on lvalue check.
- 00152 — `#line MACRO`: `#line` operands are now macro-expanded; the
  pre-expansion source line is captured for the delta math, and
  `pp-eval-cexpr` inherits cur-file/line-delta so `__LINE__` inside a
  following `#if` reflects the new mapping.
- 00162 — `int x[const 5]` / `int x[static 5]` / `int x[*]`: the array
  declarator now consumes type-qualifiers, `static`, and the VLA `*`
  inside `[…]` (C99 §6.7.5.2 fn-param syntax).

Plus three preparatory changes that don't flip a test on their own
but unblock attribute-heavy code: `parse-aggregate-spec` and
`eat-cv-quals!` eat `__attribute__` between tag and `{`, between `*`
and the next declarator piece; `parse-decl-cont` accepts a leading
`__attribute__`; and `parse-cast-or-unary` recognises a leading
attribute on the cast typename. `__builtin_expect(x, y)` is stubbed
as `(x)`.

## Next steps for bug hunting

Designated array init (`[N] = ...`), wide literals, and floats are
intentionally not supported. The remaining tractable bucket is small
and feature-shaped — statement expressions (213/214), bit-fields
(218), and the kitchen-sink initializer fixture (216) — each is a
substantial standalone feature, so spend time on them only if the
underlying capability is wanted for `tcc.c`.

The 60+ `#include` failures are gated on a single design call: either
land a minimal header search path or accept that these fixtures stay
as a TODO until we lift libc declarations into shipped headers.

## Layout

```
vendor/c-testsuite/
    LICENSE                # MIT (runner / scaffolding)
    TESTS-LICENSE          # case-by-case (per-test attribution upstream)
    README.md              # snapshot tag + integration notes
    single-exec/
        NNNNN.c
        NNNNN.c.expected   # exact stdout+stderr (often empty)
        NNNNN.c.tags       # portable / c89 / c99 / needs-libc / ...
```

Build artifacts live under `build/$ARCH/tests/cc-ext/$name`,
intermediates under `build/$ARCH/.work/tests/cc-ext/$name/`. Logs:
`cc.log` (cc.scm output), `p1pp.log` (P1pp assemble + hex2 link).
