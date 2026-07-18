# cc.scm — Code Map

## Overview

`cc.scm` is a complete C compiler (7219 lines) written in Scheme (scheme1 dialect) that compiles C source to P1pp assembly. It implements a streaming pipeline: **lexer → preprocessor → parser → codegen**. Designed for minimal memory use with fixed pre-allocated buffers and a scratch/main heap discipline that resets per declaration. Targets the P1 64-bit RISC ISA via libp1pp macros; output is consumed directly by the M1pp expander and hex2++ assembler/linker (see [docs/M1PP.md](../docs/M1PP.md), [docs/HEX2pp.md](../docs/HEX2pp.md)).

---

## Structural Map

### Major Subsystems

| Subsystem | Lines | Role |
|-----------|-------|------|
| Utilities | 1–286 | Bytevector helpers, list/alist ops, output buffers, diagnostics, debug logging, name generation |
| Data Structures | 287–595 | Record type definitions, interned primitive ctypes, ctype predicates |
| Symbol Alphabets | ~530–~595 | Keyword and punctuator alists |
| Lexer | 596–1700 | Tokenizes C source; trigraph/splice, comments, escape sequences |
| Preprocessor | 1701–2540 | `#define`, `#if`, macro expansion with hide-sets; `pp-eval-cexpr` delegates to `parse-const-int` via `%pp-make-const-ps` |
| Code Generator | 2541–4280 | P1pp assembly emission, vstack, frame allocation, all operators and control flow |
| Parser | 4282–7180 | Recursive-descent + Pratt; declarations, statements, expressions; shared constant-expression evaluator |
| Main Driver | 7186–7219 | CLI parsing, file I/O, pipeline initialization |

---

## Key Data Structures

### Runtime state — the three structs that wire the pipeline together

`world` is shared between parser and codegen. `pstate` owns the token stream and drives parsing. `cg` owns all assembly emission state.

```
world
├── scope   (list of alist frames)  var/typedef/fn bindings
├── tags    (list of alist frames)  struct/union/enum tag bindings
├── str-pool (alist)                interned string literals → labels
└── tentatives (list)               file-scope tentative definitions

pstate
├── iter     (tok-iter)    pp-iter (lexer + preprocessor)
├── world    (shared)
├── loops    (stack)       break/continue targets (loop-ctx records)
├── fn-ctx   (fn-ctx|#f)  current function context
└── cg       (cg|#f)      codegen state (#f in pp const-expr context)

cg
├── text/data/bss (buf)        fixed-capacity output section buffers
├── vstack (list of opnd)      value stack for expression evaluation
├── frame-hi (fixnum)          next free frame byte offset
├── label-ctr (fixnum)         monotonic label counter
├── world (shared)
├── fn-meta (alist)            transient per-fn metadata (sret ptr, indirect slots, etc.)
├── fn-buf/prologue-buf (buf)  reused per-function; drained to text at fn-end
├── max-outgoing (fixnum)      maximum stack args staged in current fn
├── in-fn? (bool)              routes %cg-emit to fn-buf vs text
├── lib? (bool)                skip entry stub + ELF_end
└── str-prefix (bv)            namespace prefix for anonymous strings
```

### Leaf data records — passed through the pipeline as values

| Record | Fields | Purpose |
|--------|--------|---------|
| `loc` | file/line/col | Source location |
| `tok` | kind/value/loc/hide | Token with hide-set for macro expansion |
| `macro` | kind/params/body | Preprocessor macro definition |
| `ctype` | kind/size/align/ext + mutators | C type representation |
| `sym` | name/kind/storage/type/slot/defined? | Symbol table entry |
| `opnd` | kind/type/ext/lval? | Operand on the vstack |
| `loop-ctx` | kind/tag/has-continue? | Loop break/continue target info |
| `fn-ctx` | name/return-type/params/variadic? | Current function metadata |

---

## Compilation Flow

```
Source file
  → lex-iter (make-lex-iter)            — streaming tokenizer
  → pp-iter  (make-pp-iter)             — macro expansion + directives
  → parse-translation-unit (pstate/cg)  — recursive descent + Pratt
      per-decl: call-with-scratch-cycle — scratch heap reset per declaration
      function bodies: cg-fn-begin → parse-fn-body → cg-fn-end
  → cg-finish                           — tentatives → .bss, entry stub, combine sections
  → write output file
```

**Per-function code path:**
1. `cg-fn-begin` — emit param spills, sret setup, allocate prologue-buf
2. `parse-fn-body` — emits P1pp directly into fn-buf via cg ops
3. `cg-fn-end` — drain prologue-buf + fn-buf into text, emit `:.ret` label and ret block, wrap in `%fn(<name>, <frame>, { ... })`

**`#if` constant-expression path:**
`pp-eval-cexpr` → resolve `defined`, macro-expand, idents→0 → `%pp-make-const-ps` (minimal pstate, empty scope, no cg) → `parse-const-int` (shared with parser)

---

## Line Map

| Lines | Description |
|-------|-------------|
| **1–116** | Bytevector primitives: `bv=`, `bv-prefix?`, `bv-slice`, `bv-cat`, `bv->fixnum`; list/alist utilities: `alist-ref`, `alist-update`, `any`, `every`, `count`; integer helpers: `min3`, `align-up` |
| **117–122** | `%BUF-CAP-*` — buffer pre-allocation constants (TEXT 8MiB, DATA 2MiB, BSS 2MiB, FN 256KiB, PROLOGUE 16KiB) |
| **124–215** | Output buffer system: `buf` record, `buf-push!`, `buf-flush`, `buf-reset!`, `buf-drain!` — fixed-capacity, no growth |
| **216–286** | Diagnostics: `die` with loc formatting, `slurp-fd`, `write-bv-fd`; debug logging: `debug-log-on!/off!`, `trace-emit` flags; fresh name generator: `make-namer` |
| **287–528** | Record type definitions: `loc`, `tok`, `macro`, `ctype`, `sym`, `opnd`, `loop-ctx`, `fn-ctx`, `world`, `pstate`, `cg`; interned primitive ctypes (`%t-void`, `%t-i8`…`%t-u64`, `%t-bool`, `%t-flt`, `%t-dbl`, `%t-ldbl`); ctype predicates: `%ctype-ptr?`, `%ctype-pointee`, `%ctype-unsigned?`, `%ctype-arith?`, `%ctype-fp?`; ctype accessors |
| **530–595** | `%keyword-alist` — storage/qualifiers/type specifiers/statements/operators/reserved; `%punct-alist` — punctuators longest-first, digraphs |
| **596–660** | Lexer byte-class predicates: `%digit?`, `%hex?`, `%alpha?`, `%ident-start?`, `%ident-cont?`, `%hspace?`, `%newline?`; `%lex-scratch` buffer |
| **661–790** | Logical byte access: `%lex-peek` with trigraph translation + line splice |
| **791–940** | Comment stripping: `%skip-ws-and-comments`, `%skip-line-comment`, `%skip-block-comment` |
| **941–1090** | Byte-run scanners: `%scan-while`, `%fill-while-bv`, `%accum-int-while`, `%accum-octal-bounded` |
| **1091–1290** | Token readers: `lex-read-ident`, `%lex-read-number` (hex/octal/decimal), `%lex-read-string` (with escapes), `lex-read-char` |
| **1291–1370** | `%lex-read-punct` with longest-match bucketing; `%punct-buckets` |
| **1371–1700** | `lex-iter` streaming token source: `make-lex-iter`, `%lex-iter-pull` with heap-rewind discipline; `list-iter` wrapper; `lex-tokenize` test driver |
| **1701–1820** | Preprocessor state (`pp-state`), token classification helpers (`%pp-eof?`, `%pp-nl?`, `%pp-hash?`, etc.) |
| **1821–1920** | Built-in macros: `__FILE__`, `__LINE__`, `__STDC__`, `__LISPCC__`, `__DATE__`, `__TIME__`, `__STDC_VERSION__`, `__STDC_HOSTED__`, `__VA_ARGS__` |
| **1921–2020** | Streaming pp-iter: `make-pp-iter`, `%pp-iter-pull` with out-buf stashing |
| **2021–2120** | Upstream helpers: `%pp-pull-upstream`, `%pp-peek-upstream`, `%pp-unshift-upstream!`, `%pp-collect-line-stream`, `%pp-collect-args-stream` |
| **2121–2270** | Directive dispatch: `%pp-dispatch-step`, `%pp-dispatch-directive` → `%pp-do-define`, `%pp-do-undef`, `%pp-do-if/ifdef/ifndef/elif/else/endif` with cond-stack |
| **2271–2370** | Directives: `%pp-do-error`, `%pp-do-line`, `%pp-do-pragma`, `%pp-do-include` |
| **2371–2430** | Macro expansion: `%pp-emit-expanded`, `%pp-apply-macro`, `%pp-prepare-body`, `%pp-collect-args`, `%pp-bind-args` (variadic), `%pp-substitute` (`#param` stringize, `##` paste) |
| **2431–2540** | Paste operator: `%pp-paste-tokens`; string fusion: `%pp-maybe-fuse-str`; `#if` evaluator: `%pp-make-const-ps` (IO adapter wrapping token list as minimal pstate), `pp-eval-cexpr`, `%pp-resolve-defined`, `%pp-expand-line`, `%pp-idents-as-zero` |
| **2541–2640** | CG emission primitives: `%cg-emit-buf`, `%cg-emit`, `%cg-emit-many`, `%cg-fresh-label`, `%n` (number→bv) |
| **2641–2745** | CG metadata: `%cg-fn-set!/%cg-fn-get`; register/label helpers: `%cg-reg→bv`, `%cg-emit-li`, `%cg-emit-la`, slot-expr (`(+ %<fn>__SO N)` so the slot offset resolves through the per-fn `__SO` macro at M1pp time) |
| **2745–2810** | Load/store emission: `%cg-emit-ld/st`, sub-byte width helpers; `%cg-emit-sext`; `%cg-canonicalize` (kind-driven sext/zext that puts a register back in canonical 64-bit form for its ctype); `%cg-emit-{ld,st}-bv` (width-dispatch core for the typed/slot-typed load+store family) |
| **2810–2860** | `%cg-emit-{ld,st}-{slot-,}typed` thin wrappers calling the `-bv` core; `%cg-spill-reg` |
| **2860–3020** | Operand loading: `%cg-load-opnd-into` (imm/frame/global) — re-canonicalizes a frame rval against its type kind on load via `%cg-canonicalize`; vstack ops: `cg-push/pop/top/depth/dup`, snapshot/rewind for sizeof |
| **3020–3170** | Materialize: `cg-push-imm`, `cg-push-string` (with intern), `cg-push-sym` (fn/enum/var/param), `cg-push-deref` (indirect-slot tracking) |
| **3171–3360** | Aggregate access: `cg-push-field` with `%cg-find-field` (anonymous-member-aware lookup, shared with parser's offsetof), `cg-decay-array`; address/deref: `%cg-emit-addr-of` (handles lval-indirect-frame, direct-frame for both lval and rval, and global), `cg-copy-struct`, `cg-assign-struct`, `cg-take-addr`, `cg-load` |
| **3361–3530** | Type conversions: `cg-cast` (bool/ptr/widening/narrowing — calls `%cg-canonicalize` on narrow targets), `cg-promote`, `cg-arith-conv` |
| **3530–3700** | Operators: `cg-binop` (pointer arithmetic scaling, comparison; uses `%cg-canonicalize` for narrow-typed binop results), `cg-unop` (neg/bnot/lnot), `cg-assign` (type coercion), post-inc/dec |
| **3700–3830** | Function calls: `cg-call` (sret >16B struct return, arg staging a0–a3 + stack, variadic) |
| **3830–3910** | Return: `cg-return` (void/scalar/struct via `%b(&.ret)` to the per-fn dotted local label); conditional: `cg-if`, `cg-ifelse`, `cg-ifelse-merge` (ternary/`&&`/`||`); `%cg-merge-arith-type` (C11 §6.5.15 result type for ternary merge) |
| **3910–4020** | Loop control flow: `cg-loop` (opens nested `.scope` with `:.top`/`:.end`), `cg-break` / `cg-continue` (bare `%break`/`%continue` resolved by hex2++'s innermost-out scope walk); switch: `cg-switch-begin/case/default/end` (dotted case labels and dispatch table inside the switch's `.scope`) |
| **3950–4040** | Variadic: `cg-va-start`, `cg-va-arg` (ap-lvalue store/load through `%cg-emit-addr-of`), `cg-va-end`; labels/goto: `cg-emit-label`, `cg-goto` — user C labels emit as `cc__<fn>__user_<name>` global names so `goto` survives nested loop/switch scopes |
| **4040–4280** | Globals/data: `cg-emit-global` (prefixes `.align <ctype-align>` for both `.data` and `.bss`), `cg-emit-extern`, tentatives, `cg-intern-string` (string pool with `.align 8` framing), `%cg-bv->hex-lines` (bare-hex chunked output for hex2++); frame: `cg-alloc-slot`; lifecycle: `cg-init`, `cg-fn-begin/v`, `cg-fn-end` (wraps body in `%fn(name, frame, { … })`), `cg-finish` |
| **4282–4400** | Scope/tag ops: `scope-enter/leave`, `scope-bind/lookup`, `tag-bind/lookup`, `typedef?` |
| **4400–4500** | Type compatibility: `ctype-compat?`, `%fn-ctype-compat?`, `%fn-params-compat?`; symbol merge: `sym-merge` (linkage inheritance) |
| **4500–4600** | Type constructors: `%mk-ptr`, `%mk-arr`, `%mk-fn`; qualifier handling: `eat-cv-quals!`, `skip-gnu-attribute!`, `eat-gnu-attributes!` |
| **4600–4660** | Declaration specifiers: `parse-decl-spec` (storage/type/signedness), `resolve-base` |
| **4660–4720** | Aggregate parsing: `parse-aggregate-spec` (struct/union forward + complete), `parse-struct-fields` (union offset=0), `complete-agg!` (size/align/fields), `parse-enum-spec` |
| **4660–4720** | Const-expr value helpers: `%const-trunc`, `%const-arith-conv`, `%const-arith-conv-type`, `%const-promote`, `%const-bool?` |
| **4720–4760** | `%punct-scan` — generic top-level token scanner (paren/bracket depth + optional ternary-`?` depth) parameterised by stop predicate; `%const-skip-dead-arm` (unevaluated arm of `?:`); `%const-skip-{land,lor}-rhs` (short-circuit && / ||) |
| **4760–4810** | Const-expr binary-level infrastructure: `%const-binl` (generic left-associative loop), `%const-arith-op`, `%const-div-op`, `%const-cmp-op`, `%const-shift-op` |
| **4690–5120** | Constant expression evaluator: `parse-const-expr` → `parse-const-cond` (ternary) → binary levels via `%const-binl` (lor/land/bor/bxor/band/eq/rel/add/shift/mul) → `parse-const-cast` → `parse-const-unary` (sizeof, &, prefix ops) → `parse-const-primary` (INT/CHAR/paren/enum-const); `%const-sizeof-expr` (cg snapshot/rewind; guards against pp context) |
| **4970–4985** | `%tok-decl-start?` — single canonical "does TOK begin a type-name?" predicate. Used by `%const-tok-is-decl?`, `%const-paren-is-cast?`, `token-is-decl?`, `stmt-starts-decl?` (which adds storage classes), and `parse-cast-or-unary` (which adds `__attribute__`) |
| **4940–5120** | offsetof support: `%const-parse-addrof-postfix`, `%const-parse-addrof-primary` — recognizes `&((T*)0)->field` chains; reuses `%cg-find-field` |
| **5120–5290** | `parse-const-int`; declarators: `parse-declarator`, `parse-decl-cont`, `parse-decl-suf-cont`, `parse-fn-params` |
| **5290–5320** | Phase 3 promotion: `%promote-pending-completions`, `rewrite-pending-completions!`, `promote-roots!`, `promote-iter-buffers!` (main/scratch boundary) |
| **5294–5420** | Translation unit: `parse-translation-unit` with `call-with-scratch-cycle` per decl; `parse-decl-or-fn` |
| **5426–5705** | Declarations/definitions: `handle-decl` (typedef/fn/var/static/file-scope/block-scope with tentatives) |
| **5706–5770** | Initializer support helpers: `%init-drop-thru-field` (designator drop), `%global-init-elem` / `%local-init-elem` (brace-vs-elision element/field dispatch shared by the four `parse-init-*-list/mode` walkers) |
| **5816–6065** | Global initializers: `parse-init-global` (string/brace/scalar with inferred-length arrays), `%parse-init-array-list` with element promotion, `%parse-init-struct-list` with designated designators and padding |
| **6070–6290** | Local initializers: `parse-init-local-aggregate` (string/brace), `%parse-init-local-array-list`, `%parse-init-local-struct-list` (zero-pass); compound literals as frame lvalues |
| **6296–6320** | Function body: `parse-fn-body`, `%parse-fn-body-inner` (param binding, scope enter/leave) |
| **6316–6610** | Statements: `parse-stmt` dispatch, `parse-cstmt`, `parse-if-stmt`, `parse-while-stmt`, `parse-do-stmt` (`.scope` with `:.body` / `:.top` for `continue`-to-cond semantics), `parse-for-stmt` (`.scope` with deferred condition/step), `parse-switch-stmt`, `parse-case-stmt`, `parse-default-stmt`, `parse-return-stmt`, `parse-goto-stmt`, `parse-labelled-stmt`, `parse-expr-stmt`, `parse-local-decl` |
| **6613–6655** | `%binop-bp` — Pratt binding power table (comma=1, assign=4, `\|\|`=10, `&&`=20, bitwise=30–50, relational=60, shift=70, add=80, mul=90) |
| **6656–6900** | Expression parser: `parse-expr` (`expr-bp(0)`), `parse-expr-bp` (Pratt climbing), `parse-binary-rhs` (comma/assign/compound-assign/ternary/logical/bitwise) |
| **6903–7065** | Unary/cast/postfix: `parse-unary` (prefix ops, sizeof), `parse-cast-or-unary` (paren disambiguation via `%tok-decl-start?` + `__attribute__` check), `parse-compound-literal`, `parse-postfix` (`[]`/call/`.`/`->`/post-inc/post-dec) |
| **6965–7180** | Call parsing: `call-fn-type`, `parse-call-args` (param casting, variadic promotion); builtins: `parse-builtin-va-start/va-arg/va-end`; primary: `parse-primary` (literals/idents/strings/parens/enum-consts); rvalue: `rval!`, `rval-not-fn!` |
| **7186–7219** | Driver: `%cc-slurp`, `%cc-write`, CLI flag parsing (`--cc-debug`, `--cc-trace-emit`, `--lib=PFX`), `%cc-initial-defines` (CCSCM sentinel), `cc-main` (pipeline init + `parse-translation-unit` + `cg-finish` + write) |

---

## Notable Design Choices

- **Streaming pipeline** — no materialized token list; each stage pulls one token at a time
- **Fixed buffers** — pre-allocated per section (text/data/bss); no growth; tuned by `%BUF-CAP-*`
- **Heap discipline** — scratch heap reset at declaration boundaries via `call-with-scratch-cycle`; live roots deep-copied to main heap before reset
- **Vstack-based codegen** — expression evaluation pushes/pops `opnd` records; values optionally spilled to frame slots
- **Macro hide-sets** — `tok` carries hide set to prevent recursive expansion (C11 §6.10.3.4)
- **Shared constant-expression evaluator** — `parse-const-*` serves both the parser (typed, with sizeof/cast/offsetof) and the preprocessor `#if` evaluator (`%pp-make-const-ps` wraps a token list as a minimal pstate with empty scope and `ps-cg = #f`); `%const-binl` is the generic left-associative binary-level pattern, fed by combiners (`%const-arith-op`, `%const-div-op`, `%const-cmp-op`, `%const-shift-op`) for every level from `||` down to `*` / `/` / `%`
- **Sign-extension discipline** — narrow types (i8/i16/i32) stored as canonical 64-bit forms via shli/sari; widening casts are relabel-only. `%cg-canonicalize` centralises kind-driven sext/zext and is called from `%cg-load-opnd-into` (frame-rval load), `cg-cast` (narrowing), and `cg-binop` (narrow-typed result), so a relabel-only cast (e.g. via `cg-arith-conv`) reads correctly downstream.
- **Sret (struct return)** — structs >16B use indirect result: caller passes pointer in `a0`
- **Variadic ABI** — 16 contiguous 8-byte slots; args 0–3 from `a`-regs, 4+ from `LDARG`. `cg-va-start` / `cg-va-arg` route ap-lvalue stores/loads through `%cg-emit-addr-of`.
- **Tentative definitions** — collected in `world-tentatives`; emitted as `.bss` only if no full definition appears by TU end
- **FP softening** — float/double types parsed and sized per SysV ABI but all FP ops emit integer bitpattern operations
- **M1pp + hex2++ output** — bodies are wrapped in libp1pp's `%fn(name, frame, { … })`, which opens a hex2++ `.scope` and emits `%enter`/`%eret`. Compiler-internal labels (`:.ret`, loop `:.top`/`:.end`, switch `:.lbl_N`) are dotted scope-locals resolved by hex2++'s innermost-out scope walk; `%break` / `%continue` resolve through the same walk to the nearest enclosing scoped loop. User C labels use `cc__<fn>__user_<name>` global mangling so `goto` is unaffected by nested scopes (C labels have function scope, not block).
- **Alignment via `.align`** — `cg-emit-global` emits `.align <ctype-align>` before every `.data` or `.bss` symbol; `cg-intern-string` brackets each pooled string with `.align 8` so a non-multiple-of-4 string doesn't misalign the next instruction on aarch64. Intra-struct field padding is inline zero bytes — offsets are constant relative to the aligned struct start, so a `.align` directive there would be redundant.
- **Bare-hex string emission** — string pool and `(label-ref . LBL)` initializer pieces emit as bare hex chunks (≤64 bytes / 128 hex chars per line) consumed directly by hex2++.
- **Ternary common type** — `cg-ifelse-merge` runs `%cg-merge-arith-type` over both arms after they emit; the result `opnd` carries the C11 §6.5.15 common type. The slot stores the raw 8-byte payload; `%cg-load-opnd-into` re-canonicalizes against whichever common type was picked. `&&`/`||` callers pre-cast both arms to `%t-i32`, so the merge is a no-op for them.
- **Single type-name predicate** — every "does this token start a type-name?" check runs through `%tok-decl-start?` (`%const-tok-is-decl?`, `%const-paren-is-cast?`, `token-is-decl?`, the cast-or-unary disambiguator, and `stmt-starts-decl?` which adds storage classes).
- **Shared bracket scanner** — `%punct-scan` is the one paren/bracket-depth walker, parameterised by stop predicate and an optional ternary-`?` tracking flag. All const-expr "skip dead arm" / "skip short-circuited rhs" helpers route through it.
- **One core ld/st helper** — `%cg-emit-{ld,st}-bv` is the shared body behind both the slot-typed (base = `sp`, off rendered through `%cg-slot-expr`) and typed (explicit base register, raw int off via `%n`) variants. Width dispatch lives in one place; the four wrappers are 1-line trampolines.
- **Aggregate-init dispatch** — `%global-init-elem` and `%local-init-elem`, each parameterised by an `elide?` flag, drive the brace-vs-elision element/field decision for all four `parse-init-*-list/mode` walkers.
