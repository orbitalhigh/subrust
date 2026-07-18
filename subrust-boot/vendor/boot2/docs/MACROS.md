# Macros (R7RS `syntax-rules`) for scheme1

Plan for adding hygienic `syntax-rules` macros to `scheme1.P1pp`.
Conforms to R7RS-small with the coverage limits noted in
[Scope](#scope).

## Architecture

Two-layer split:

- **P1 runtime** (in `scheme1.P1pp`) provides the *minimum* needed to
  recognize and dispatch macro bindings: a new heap-object type, three
  special forms (`define-syntax`, `let-syntax`, `letrec-syntax`), an
  expansion hook in `eval`, and a `gensym` primitive.
- **Scheme engine** (in `prelude.scm`) implements `syntax-rules` itself
  as a regular procedure that returns a transformer closure. Pattern
  matching, template rendering, ellipsis expansion, and gensym-based
  hygiene all live here.

Rationale: a hygienic matcher/renderer is ~300 lines of dense list
manipulation. Writing it in Scheme is straightforward and debuggable;
writing it in P1pp is neither. The runtime cost is one extra closure
call per macro use — negligible at the scale this interpreter
operates.

## Scope

Covered in v1:

- `define-syntax`, `let-syntax`, `letrec-syntax`.
- `syntax-rules` with literals list, single-depth ellipsis (`...`),
  underscore wildcard (`_`), and improper-tail patterns
  (`(p1 p2 ... . ptail)`).
- Hygiene: every template-introduced identifier (not a pattern
  variable, not a literal) is gensym-renamed per expansion so it
  cannot shadow a user binding at the use site.

Deferred (not v1):

- Multi-depth ellipsis (a pattern variable appearing under two or more
  `...`s, producing nested-list captures).
- Custom ellipsis identifier (`(syntax-rules ::: ...)`).
- `syntax-case` / `er-macro-transformer` / explicit-renaming variants.
- Macro-introduced top-level definitions (`define` inside an
  expansion). Internal `define` is already rejected by scheme1.

## Type additions

### `HDR.MACRO` heap object

Added to the existing `HDR` enum (currently
`{BV CLOSURE PRIM TD REC MV}`).

Layout (tagged HEAP, 16 bytes):

```
struct MACRO {
    hdr           ; HDR.MACRO
    transformer   ; tagged closure: (lambda (form) -> form)
}
```

The transformer is a Scheme closure built by `syntax-rules` at the
time `define-syntax` (or `let-syntax`/`letrec-syntax`) is evaluated.
It takes the entire macro form as a quoted datum (e.g. `'(my-when t
b1 b2)`) and returns a quoted datum that `eval` then re-evaluates.

Touchpoints for the GC tracer (when GC lands): trace the
`transformer` slot. For `equal?` / `write` / `display`: render as
`#<macro>`; not structurally comparable.

## Runtime changes

### Special-form dispatch

Three new entries in the `dispatch_form` table inside `eval`'s pair
branch:

```
%dispatch_form(&sym_define_syntax,  &.do_define_syntax)
%dispatch_form(&sym_let_syntax,     &.do_let_syntax)
%dispatch_form(&sym_letrec_syntax,  &.do_letrec_syntax)
```

Cached symbol slots and `intern_special_forms` updated alongside.

### `eval_define_syntax`

```
(define-syntax name transformer-expr)
```

1. Evaluate `transformer-expr` in the current env. Must yield a
   closure (`HDR.CLOSURE`) — error otherwise (`die(msg_bad_syntax)`).
2. `alloc_hdr(MACRO.SIZE, HDR.MACRO)`; store the closure in the
   transformer slot.
3. Bind the symbol's *global* slot to the tagged MACRO. (Same global
   binding mechanism as `define`.)
4. Return UNSPEC.

`alloc_hdr_main` is the right allocator: the macro outlives any
scratch-heap reset, just like `define-record-type`'s TD.

### `eval_let_syntax` / `eval_letrec_syntax`

```
(let-syntax    ((name transformer-expr) ...) body ...)
(letrec-syntax ((name transformer-expr) ...) body ...)
```

Both extend the **lexical** env with `(name . macro-obj)` pairs.
`let-syntax` evaluates each `transformer-expr` in the *outer* env;
`letrec-syntax` first installs all bindings as placeholders, then
evaluates each transformer in the new env (allowing mutual
recursion). Body evaluates in the extended env via `eval_body`.

The lexical env is the existing alist `((sym . val) ...)`. Macro
values are tagged HEAP objects, so they coexist with regular value
bindings without changing the alist shape.

### Macro dispatch in `eval`

After special-form dispatch in the pair branch, before the apply
path: when the head is a symbol, resolve it (lexical env first, then
global slot). If the resolved value is a `HDR.MACRO`:

1. Build the form: re-cons the head onto the args, *unevaluated* —
   essentially the original `expr`. Already in hand as `expr`.
2. Call the transformer closure with this form as its single
   argument. (Reuse `apply` with a one-element args list.)
3. Tail-call `eval` on the returned form in the *current* env.

Pseudocode (in P1 terms inside `eval`):

```
:.maybe_macro
%hdr_type(t0, resolved)
%bine(t0, %HDR.MACRO, &.apply_normal, t1)
%heap_ld(a0, resolved, %MACRO.transformer)   ; closure
%ldl(a1, expr)                                ; the form (quoted)
%li(a2, %imm_val(%IMM.NIL))
%call(&cons)                                  ; args = (form)
%mov(a1, a0)
%heap_ld(a0, ..., transformer)
%call(&apply)
%mov(a0, ...)                                 ; expanded form
%ldl(a1, env)
%tail(&eval)
```

Ordering: macro check happens *after* the static special-form
dispatch table, so user code cannot redefine `if`, `lambda`, etc.
via `define-syntax`. (That's a deliberate restriction; lifting it
would require turning every special form into a default macro
binding that the user can override.)

### `gensym` primitive

```
(gensym)        ; -> fresh symbol, e.g. g.0, g.1, ...
(gensym "tag")  ; -> g.tag.N
```

Implementation: a process-global counter `gensym_counter` in BSS.
Builds a name by `format`-ing `g.<n>` (or `g.<tag>.<n>`) into a
scratch buffer, then calls `intern` so the symbol gets a stable
slot in the symtab. Because `intern` is keyed on the byte string,
distinct counters always produce distinct symbols.

The interned name can never collide with user identifiers because
user identifiers cannot contain `.` followed by a digit at the
exact pattern produced — *except* that scheme1 *does* allow `.` in
identifiers. Mitigation: prefix with a byte that the reader rejects
in user identifiers but allows when interning programmatically.
Cleanest: pick a leading byte outside the reader's identifier
charset (e.g. `\x01`) so user code cannot construct a colliding
name through the reader. The byte is invisible in `display` output
unless the user explicitly writes the symbol — acceptable.

Added to `prim_table`: `gensym`.

## Scheme engine (`syntax-rules` in prelude.scm)

`syntax-rules` is a regular procedure. Its result is a closure of
the form `(lambda (form) ...)`.

```
(syntax-rules literals rule ...)
  ;; literals = list of identifiers
  ;; rule     = (pattern template) where pattern is (head . sub-pattern)
```

Top-level call shape after macro expansion of the special form:

```
(define-syntax foo (syntax-rules (lit) ((foo p) t) ...))
```

`define-syntax`'s argument is just the `(syntax-rules ...)`
expression. Eval evaluates that expression, getting a closure, then
wraps it in HDR.MACRO. So `syntax-rules` must be available in the
prelude before any macro is defined.

### Algorithm — pattern matching

`(match-pattern pat form literals)` returns either `#f` (no match)
or an alist of `(pattern-var . captured-form-or-list)`.

Pattern dispatch:

| Pattern shape           | Match rule                                                |
|-------------------------|-----------------------------------------------------------|
| `_`                     | Match anything; no binding.                               |
| `<id>` ∈ literals       | Match iff `form` is the same symbol (`eq?`).              |
| `<id>` (otherwise)      | Match anything; bind `id` → `form`.                       |
| `()`                    | Match iff `form` is `'()`.                                |
| `(p_head . p_rest)`     | See ellipsis / improper / proper rules below.             |
| `<literal-datum>`       | Match by `equal?` (numbers, strings, booleans, chars).    |

Pair-pattern cases:

1. **Ellipsis**: pattern is `(p ... . p_tail)` where `p ...` is the
   ellipsis element. Greedily consume as many leading elements of
   `form` as match `p`, collecting per-pattern-var captures into
   parallel lists. Then match the remaining tail against `p_tail`.
2. **Improper tail without ellipsis**: pattern is `(p1 ... . p_tail)`
   (dot before a non-ellipsis tail). Match each `p_i` against
   `(list-ref form i)`, then match `p_tail` against the dotted rest.
3. **Proper list**: lengths must match exactly; element-wise
   recursion.

### Ellipsis capture shape

When `p` (under `...`) contains pattern vars `v1, v2, ...`, after
consuming `n` elements, each `v_i` is bound to a *list of length n*
of the values captured at that position. (Empty list when `n = 0`.)

Each pattern-var binding is annotated with its **depth**: 0 for a
plain capture, 1 for ellipsis-captured. Depth-1 only in v1.

### Algorithm — template rendering

`(render template bindings rename-map)` walks the template and
produces an output form.

- Symbol that is a depth-0 pattern var → substitute its captured
  form.
- Symbol that is a depth-1 pattern var **outside** of an ellipsis
  context → error (arity mismatch).
- `(t ... . t_tail)`: for each ellipsis-relevant pattern var
  appearing in `t` (the sub-template before `...`), look up its
  list of captures. All such lists must have equal length `n`
  (else error). Render `t` `n` times, each time with depth-1 vars
  shadowed by their `i`-th element. Splice the results into the
  output, then continue with `t_tail`.
- Symbol that is a literal or free identifier → look up in
  `rename-map`; emit the renamed symbol if present, else emit
  unchanged.
- Pair (non-ellipsis) → recurse into car and cdr.
- Other (literal datum, `'()`) → emit unchanged.

### Hygiene: identifier renaming

Before each expansion, walk the template and collect every symbol
that is **not**:
- a pattern variable in `bindings`, or
- a literal in the rule's literals list, or
- a reference to a variable visible at the macro's *use site* via
  the standard scope chain (we approximate this as: any symbol not
  introduced by the template is fine to leave alone).

In a single-global-env Scheme, the simpler rule "rename every
template-only identifier that binds a name" suffices: rename
identifiers that appear in *binding positions* of constructs the
template introduces (`lambda` formals, `let` bound names, `define`
names). Identifiers in operator/operand positions don't need
renaming because they reference globals or pattern-bound values.

Implementation: walk the template once, collect a set of
"binding-position" identifiers (this requires a small table of
which forms bind names: `lambda`, `let`, `let*`, `letrec`,
`let-values`, `let*-values`, `do`, `define`, `define-record-type`).
For each, allocate a fresh `gensym` name and substitute every
occurrence in the template (binding *and* references) consistently.

This is conservative — it sometimes renames identifiers that
wouldn't have collided — but that's fine, it's still hygienic.

### Putting it together — the transformer closure

```
(define (syntax-rules literals . rules)
  (lambda (form)
    (let loop ((rs rules))
      (cond
        ((null? rs)
         (error "no syntax-rules pattern matched" form))
        (else
         (let* ((rule (car rs))
                (pat (car rule))
                (tpl (cadr rule))
                (b   (match-pattern pat form literals)))
           (if b
               (let ((rmap (build-rename-map tpl literals b)))
                 (render tpl b rmap))
               (loop (cdr rs)))))))))
```

`match-pattern`, `build-rename-map`, and `render` are the three
core helpers; together ~300 lines.

## Files / line budget

| Location                              | Add        |
|---------------------------------------|------------|
| `scheme1.P1pp` HDR enum               | +1 line    |
| `scheme1.P1pp` MACRO struct           | +3 lines   |
| `scheme1.P1pp` `intern_special_forms` | ~8 lines   |
| `scheme1.P1pp` eval dispatch + macro hook | ~30 lines  |
| `scheme1.P1pp` `eval_define_syntax`   | ~25 lines  |
| `scheme1.P1pp` `eval_let_syntax`      | ~50 lines  |
| `scheme1.P1pp` `eval_letrec_syntax`   | ~50 lines  |
| `scheme1.P1pp` `prim_gensym_entry`    | ~25 lines  |
| `scheme1.P1pp` writer `#<macro>` case | ~5 lines   |
| `scheme1.P1pp` prim_table + names     | +3 lines   |
| `prelude.scm` `syntax-rules` engine   | ~350 lines |

Total: ~200 lines P1pp, ~350 lines Scheme.

## Testing

Add a `tests/scheme1/13x-macro-*.scm` series. Minimum coverage:

- `130-macro-basic.scm` — `(define-syntax my-when ...)`, simple
  fixed-arity expansion.
- `131-macro-ellipsis.scm` — `my-when` with `body ...`, `my-list`
  (`(x ...)` → `(list x ...)`), zero-element case.
- `132-macro-let.scm` — re-implement `let` via `syntax-rules` with
  inner-shape pattern `((name val) ...)`. Verify against builtin.
- `133-macro-tail.scm` — improper-tail pattern, e.g. `(_ x . rest)`.
- `134-macro-literals.scm` — literal `else`-style identifier in a
  cond-shaped macro.
- `135-macro-hygiene.scm` — macro that introduces a binding that
  would shadow a user var; verify the user var still resolves.
  Classic test: `(define-syntax swap! ...)` using a temp.
- `136-let-syntax.scm` — local `let-syntax`, including a body that
  references a same-named global value — must shadow correctly and
  un-shadow outside the body.
- `137-letrec-syntax.scm` — two mutually-recursive transformers
  (rare in practice but spec-required).
- `138-no-match.scm` — fall-through error path.

Each fixture is a normal scheme1 test (`.scm` + `.expected-exit`),
runnable via `tests/boot-run-scheme1.sh`.

## Open caveats

- **Top-level `define` inside expansions is rejected.** scheme1
  rejects internal `define`, and that check fires *after* macro
  expansion. A macro that expands to `(begin (define x 1) ...)` at
  the top level works; the same expansion inside a `lambda` body
  does not. Document, don't fix.
- **Macros cannot override built-in special forms.** Dispatch
  checks special-form symbols before macro lookup. Lifting this
  would require representing the special forms as default
  bindings.
- **No source-location tracking.** Errors from inside expansions
  point at the macro implementation, not the use site. Consistent
  with scheme1's existing error story.
- **`equal?` on macros is reference equality only.** Two
  `syntax-rules` expressions compiled separately are distinct
  closures. Not specified by R7RS as comparable.
