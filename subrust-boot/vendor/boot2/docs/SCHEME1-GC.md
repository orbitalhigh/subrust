# scheme1 GC

Spec for adding a Cheney-style semispace copying garbage collector to
`scheme1/scheme1.P1pp`. Replaces the main heap's bump-only allocator
with a moving collector; leaves the scratch heap and the
`heap-mark` / `heap-rewind!` family alone.

## Goals & non-goals

Goals:
- Reclaim unreachable main-heap objects automatically. Programs no
  longer depend on `heap-mark` / `heap-rewind!` discipline to stay
  within the 256 MiB main-heap reservation.
- Preserve every observable semantic of the existing interpreter
  (`eq?`, mutation through `set-car!` / `set-cdr!` / `bytevector-u8-set!`,
  closure capture, record identity).
- Keep the scratch-heap, `heap-mark`, `heap-rewind!`, `use-scratch-heap!`
  surface working unchanged. `cc.scm` must run with no source change.
- Roots are unambiguous: every Scheme value live across an allocation
  point lives in a place the collector explicitly knows how to find.

Non-goals:
- Generational, incremental, or concurrent collection.
- Collection of the scratch heap.
- Reducing the worst-case pause time. (Stop-the-world is fine.)
- Collection of the symbol table (it stays append-only and pinned;
  see [Roots](#roots)).
- Finalizers, weak references, ephemerons.

## Algorithm

Stop-the-world Cheney semispace copy. The 256 MiB main heap is split
into two equal **semispaces** of 128 MiB each. At any moment one is
the active *from-space* (allocations bump into it) and the other is
the idle *to-space*.

A collection:
1. Swap the role of the two spaces. The old from-space becomes the
   new to-space's source; the old to-space becomes the new
   from-space (initially empty, `next == start`).
2. **Forward roots.** For each root word holding a tagged pointer
   into the old from-space, copy the pointee into the new
   from-space and overwrite the root with the new tagged pointer.
3. **Scan.** Walk the new from-space from the start to the
   advancing `next` pointer with a cursor `scan`. For each object
   between `scan` and `next`, forward each of its tagged-pointer
   fields. `next` advances as new objects are appended;
   `scan == next` is the termination condition.
4. The old from-space is now garbage in bulk. No per-object sweep.

Forwarding is in-place: when an object is copied, its old header
word is overwritten with a forwarding tag carrying the new address
(see [Forwarding](#forwarding)). A second visit to the same object
sees the forwarding tag and just reuses the recorded new address.

Why Cheney over mark-sweep-with-free-list (the original ask): the
existing allocator is bump-and-pointer. Cheney keeps it that way
post-collection (no free list, no fragmentation), and the collector
itself is short — copy + scan, no separate mark and sweep passes.

## Heap layout

Current main-heap region (`HEAP_CAP_BYTES = 0x10000000`, 256 MiB)
is replaced by two regions of 128 MiB each in BSS, sized via two new
constants:

```
%macro SEMISPACE_CAP_BYTES() 0x08000000 %endm   # 128 MiB
%macro SCHEME_STACK_CAP_BYTES() 0x00100000 %endm   # 1 MiB; see Roots
```

New global pointer slots (replacing `heap_buf_ptr` /
`heap_next` / `heap_end`):

| Slot                | Meaning                                            |
|---------------------|----------------------------------------------------|
| `space_a_ptr`       | base of semispace A (set by `init_arenas`)         |
| `space_b_ptr`       | base of semispace B (set by `init_arenas`)         |
| `from_space_start`  | base of the active from-space                      |
| `from_space_end`    | end of the active from-space                       |
| `from_space_next`   | bump pointer into active from-space                |
| `to_space_start`    | base of the idle to-space                          |
| `to_space_end`      | end of the idle to-space                           |

`current_heap_next_ptr` / `current_heap_end_ptr` continue to exist
unchanged. When the current heap is "main," they point at
`from_space_next` / `from_space_end`. When the current heap is
"scratch," they point at the scratch slots (unchanged from today).
After a collection, the slots `from_space_next` /
`from_space_end` are updated to refer to the *new* from-space,
and `current_heap_next_ptr` is repointed if main is current.

The scratch heap, the symtab arena, and the readbuf are unaffected.

## Roots

Roots are split into three closed sets:

### 1. Symbol table

Every `SYMENT.global_val` slot in the symbol-table BSS array is a
potential root. Iteration is bounded by `symtab_count`. The table
itself never moves; entries are pinned and only their `global_val`
field is forwarded.

### 2. Scheme value stack

A new dedicated stack, allocated in BSS (`SCHEME_STACK_CAP_BYTES`),
that holds **every Scheme value live across a call site**. The P1
call stack (`sp`) continues to hold raw machine state — return
addresses, raw integer locals, scratch — and is *not* scanned.

Two new globals:
- `scheme_stack_base`  — set at init to the start of the stack region.
- `scheme_sp`          — current top; grows upward.

A frame on the Scheme stack is a contiguous array of N tagged
values. A function reserves N slots on entry, accesses them by
displacement off `scheme_sp`-at-entry (saved as the function's
"sfp"), and releases them on exit. New macros:

| Macro                  | Effect                                                    |
|------------------------|-----------------------------------------------------------|
| `%senter(n)`           | bump `scheme_sp` by `n*8`; save old in current P1 frame   |
| `%sleave(n)`           | drop `n*8` from `scheme_sp`                                |
| `%sst(reg, slot)`      | store `reg` into Scheme-frame `slot`                      |
| `%sld(reg, slot)`      | load Scheme-frame `slot` into `reg`                       |
| `%spush(reg)`          | push `reg` onto Scheme stack (1-slot bump)                |
| `%spop(reg)`           | pop one slot into `reg`                                    |

A new `%fn3(name, scheme_locals, raw_locals, body)` form parallels
`%fn2`: it builds two frames simultaneously, one on the P1 stack
(raw locals — cursors, byte counts, pointers into BSS) and one on
the Scheme value stack (every tagged Scheme value). The collector
walks only the Scheme stack.

Functions that hold zero Scheme values across calls keep using
`%fn2` / `%fn` and need no Scheme-stack frame at all (e.g. `memcpy`,
`strlen`, low-level syscall trampolines, the writer's byte
emitters).

`scheme_sp` itself is the only stack-side root pointer the GC needs;
the live region is `[scheme_stack_base, scheme_sp)`.

### 3. In-flight argument-passing registers

At every potential GC point — that is, at every allocation —
arguments / temporaries already-live in `a0`–`a3` and `t0`–`t3` may
hold tagged values. The protocol is: **never call into the
allocator with a live tagged value held only in a register**. Either
spill to the Scheme frame first, or pass it through `a0` / `a1` to
the allocator (which the allocator itself preserves across
collection by treating its inputs as roots — see
[Allocation hooks](#allocation-hooks)).

This is a hard discipline; violating it produces use-after-free that
silently survives until the next collection. We mitigate by:
- Treating `a0` and `a1` of the allocator entry points (`cons`,
  `alloc_hdr`, `alloc_bytes`) as additional roots during a
  collection that fires from inside the allocator. They get
  forwarded along with everything else and the allocator returns
  with the updated values.
- Forbidding any other register from holding a tagged value across
  a `%call` to a function that may allocate. (Audit pass; this is
  already mostly true because of the existing spill discipline.)

## Object headers and tracing

Every heap object — without exception — begins with an 8-byte
header word whose low byte is one of the `HDR` enum values. After
this change there are no headerless allocations: the existing
`alloc_bytes` is replaced by a header-emitting variant.

### New tags

```
%enum HDR { BV CLOSURE PRIM TD REC MV RAW FWD }
```

- `HDR.RAW` — opaque byte buffer with no internal tagged refs.
  Used for BV data, symtab name copies (when those move into the
  GC heap; they currently live in main but are pinned — see
  [Pinned allocations](#pinned-allocations)), and any other raw
  payload that needs to participate in the parsable-heap walk.
  Header word: `(raw_size_bytes << 8) | HDR.RAW`.
- `HDR.FWD` — forwarding sentinel (only valid in from-space during
  a collection). Header word: `(new_addr << 8) | HDR.FWD`. Because
  every heap object is 8-byte aligned, `new_addr` shifted left 8
  loses no information for any address representable in our memory
  layout.

### Per-type trace and size

The collector dispatches on `hdr.low_byte` and produces (a) the
total size in bytes of the allocation including the 8-byte header,
and (b) a list of slot offsets containing tagged pointer fields.

| HDR         | Total size                              | Pointer-bearing slots                     |
|-------------|-----------------------------------------|-------------------------------------------|
| `BV`        | 24 (hdr, len, cap), data via `BV.data`  | `BV.data` (points to a `RAW` block)       |
| `CLOSURE`   | 32                                      | `CLOSURE.params`, `CLOSURE.body`, `CLOSURE.env` |
| `PRIM`      | 24                                      | `PRIM.data` (only if entry is a parameterized prim — see below) |
| `TD`        | 32                                      | `TD.name`, `TD.fields`                    |
| `REC`       | `16 + nfields*8` (read `nfields` from `td`) | `td` slot + each field slot           |
| `MV`        | `8 + count*8` (read `count` from header high bytes) | each value slot                |
| `RAW`       | `8 + align8(size)` (read `size` from header high bytes) | none                       |
| `FWD`       | (must not be visited; precondition violation if seen during scan) | n/a |

PAIRs are **not** in this table because they are tagged with
`TAG.PAIR`, not `TAG.HEAP`. Their layout is fixed:
`[car | cdr]`, no header byte. PAIR copy is special-cased: 16
bytes, two tagged-pointer slots (car at offset 0, cdr at +8).

Because PAIRs have no header byte, the collector also can't store
a forwarding sentinel at offset 0 the same way it does for HEAP
objects. Instead we use the convention: for a forwarded PAIR,
overwrite the **car** slot with `(new_addr << 3) | TAG.PAIR` (a
self-tagged forward) and set the **cdr** slot to a sentinel
`IMM.UNBOUND`. To detect: a from-space pair is forwarded iff its
cdr slot equals the `UNBOUND` immediate. (We pick `UNBOUND`
because it is an immediate that user code never legitimately stores
into a cdr — it is reserved for "symbol unbound" lookups.)

For PRIM, the `data` slot is *only* a tagged pointer when the prim
is a parameterized prim (record accessor / mutator / ctor /
predicate). For plain prims it's zero. Trace logic: read the slot;
if `tagof != TAG.FIXNUM && tagof != 0`, treat as a pointer.
Equivalent and simpler: always trace `PRIM.data` — fixnum-tagged
words and zero will fail the tag check inside the forwarding
routine and pass through unchanged.

### Allocation hooks

`cons` and `alloc_hdr` and `alloc_bytes` (which now emits a
`RAW` header) gain an OOM check that triggers a collection
instead of aborting:

```
:cons
  load from_space_next, from_space_end
  if next + 16 <= end: bump, write, return
  else:
    save a0, a1 to a known location (Scheme stack push)
    call gc_collect
    pop a0, a1 (now forwarded if they were heap pointers)
    retry: this time it must succeed, else abort with msg_heap_full
```

The save-restore around `gc_collect` is exactly the in-flight
register protocol: the allocator's inputs are spilled onto the
Scheme stack so they participate as roots during the collection.

## Forwarding

`forward(tagged_ptr)` is the core operation, called by both root
forwarding and the scan loop:

```
case tagof(p):
  FIXNUM, IMM, SYM:    return p unchanged   # not a heap ref
  PAIR:
    raw = p - 1
    if cdr(raw) == imm_val(UNBOUND):        # already forwarded
        return car(raw)                      # holds new tagged ptr
    new = bump to-space by 16
    new[0] = car(raw); new[8] = cdr(raw)
    car(raw) = (new << 3) | TAG.PAIR        # write forward
    cdr(raw) = imm_val(UNBOUND)              # forward marker
    return car(raw)
  HEAP:
    raw = p - 3
    hdr = ld(raw, 0)
    if (hdr & 0xff) == HDR.FWD:
        return ((hdr >> 8) | TAG.HEAP-untagged-arith)   # extract new
    size = size_of(hdr)
    new = bump to-space by size
    memcpy(new, raw, size)
    st(raw, 0, (new << 8) | HDR.FWD)
    return new + 3
```

The scan loop walks to-space byte-by-byte using `size_of` to skip
over each copied object, calling `forward` on each tagged-pointer
slot listed by the per-type tracer.

## Pinned allocations

Some interpreter-owned allocations must not move:
- Symtab name buffers (`alloc_bytes_main` from `intern`).
- TD objects that hold field-name lists (`alloc_hdr_main` /
  `cons_main` from `eval_define_record_type`).
- Pre-allocated MACRO objects, special-form name strings, etc.

Today these live in the main heap, distinguished from user
allocations only by their use of the `*_main` allocator suffix.
After GC introduction, they need to live in a region the collector
**doesn't** sweep. Two options:

1. **Move pinned allocations to a separate "perm" region.** Rename
   `alloc_*_main` to `alloc_*_perm`, point them at a third BSS arena
   (perm) sized for ~1 MiB. Collector treats perm as a root region
   (scans every word, forwards heap pointers) but never moves perm
   objects. Cleaner.
2. **Keep pinned allocations in from-space and special-case them
   during copy.** Tag a "pinned" bit in the header; collector copies
   the *contents* into to-space conceptually but actually leaves
   them in place and just forwards their internal references.
   Complicates the moving invariant.

Recommend option 1. Perm region is small, write-once, scan-only.
The handful of `*_main` call sites in the interpreter all become
`*_perm`.

## Collection lifecycle

Init (called from `heap_init`):
1. Reserve both semispaces and the perm region in BSS.
2. `from_space = space_a`, `to_space = space_b`,
   `from_space_next = from_space_start`.
3. `scheme_sp = scheme_stack_base`.

Trigger:
- Only on alloc-fail in the from-space. No proactive trigger, no
  threshold-based trigger.
- A collection that fails to free enough space for the pending
  allocation aborts with `msg_heap_full`. (No heap growth.)

Collection body (`gc_collect`):
1. Swap from/to roles. Set `to_space_next = to_space_start`.
2. Forward all roots:
   - Walk `[scheme_stack_base, scheme_sp)`: for each word, replace
     in place with `forward(word)`.
   - Walk `[symtab[0], symtab[symtab_count])`: for each entry, replace
     `global_val` with `forward(global_val)`.
   - Walk perm region's tagged-pointer fields (via the same
     header-driven trace dispatch).
   - Forward the allocator's spilled `a0`/`a1` if a collection
     fired from inside an allocator.
3. Scan: cursor walks new from-space until it catches `next`.
   For each object, forward each pointer-bearing slot per the
   per-type trace.
4. Optionally zero the old from-space (debug only — helps catch
   stale pointers; off by default for speed).

Post-conditions:
- All live objects copied to the new from-space; all forwarding
  pointers consumed.
- `from_space_next` reflects the new occupancy.
- The old from-space is logically free.

## Test posture

A debug build flag (compile-time `%macro GC_DEBUG_FILL_FROM() %endm`)
that, when defined, fills the old from-space with a poison byte
after collection. Any surviving raw pointer to a copied object will
read poisoned bytes on its next dereference and crash visibly.

Test scaffolding (under `tests/scheme1/`):
- `gc-stress.scm`: allocate ~10 MiB of garbage in a loop with a
  small live set; assert the heap doesn't grow.
- `gc-identity.scm`: pre-collection vs post-collection `eq?` on
  pairs / records / closures should keep its answer.
- `gc-mutation.scm`: `set-car!` on a pair forwarded mid-collection
  is observed correctly.
- `gc-cons-main.scm`: pinned objects survive collections without
  changing identity.
- `gc-during-bv.scm`: trigger collection during `bv-grow`; verify
  the BV's `RAW` data block is correctly forwarded.
- `gc-during-parser.scm`: large input program forces collection
  during parsing; verify reader-built lists end up correct.
- `gc-during-cc.scm`: run `cc.scm` on a small input end-to-end
  with an artificially shrunken from-space (e.g. 4 MiB) so
  collection is exercised at every phase. Output must be
  byte-identical to the un-shrunken run.

## Migration plan

The change is large enough to land in stages. Each stage is
independently testable; stages 1–4 are pure refactors that change
no observable behavior.

1. **Add the perm region.** Introduce `*_perm` allocators and
   migrate every `*_main` call site to `*_perm`. The "main" name
   is freed up to mean "GC-managed" later. No semantic change.
2. **Add HDR.RAW.** Replace `alloc_bytes` with a variant that
   emits a `RAW` header. Update `bv_alloc`, `bv_grow`, and the
   single other `alloc_bytes` call site (string allocator) to
   accept the new offset of the data buffer (now `+8` past the
   raw allocation start). Heap is now parsable.
3. **Add the Scheme value stack.** Build the BSS region, the
   `scheme_sp` global, and the `%senter` / `%sleave` / `%sst` /
   `%sld` / `%spush` / `%spop` / `%fn3` macros. No callers yet.
4. **Migrate frames.** Convert every `%fn2` whose locals hold
   tagged Scheme values to `%fn3`, splitting locals into Scheme
   vs raw. Order: leaves first (`bind_params`, `eval_args`,
   `apply_build_args`), then `eval` and the `eval_*` family,
   then primitives, then the parser's value-producing leaves
   (`parse_atom`, `parse_string`, `parse_char`, `parse_list`,
   `parse_u8_body`, `parse_one`), then the writer
   (`write_to_bv`, `write_pair_to_bv`, `value_to_bv`).
5. **Wire the collector.** Add `gc_collect`, `forward`,
   `size_of`, and the per-type trace dispatch. Hook into
   `cons` / `alloc_hdr` / `alloc_bytes` OOM paths.
6. **Shrink the heap and validate.** Cut the from-space from
   128 MiB to 4 MiB temporarily and run the full
   `tests/scheme1/` and `tests/cc-pp/` suites. Restore.

After stage 4 the interpreter still works exactly as today (Scheme
values just live in a different stack), so each stage can be
landed and tested independently. Stage 5 is where new behavior
appears.

## Open items

- **Pinned cons cells.** `cons_main` produces PAIRs in main; with
  GC, those PAIRs must instead live in perm. Need to confirm
  every existing `cons_main` caller is fine with that.
- **Multi-value packs as roots in mid-flight.** `prim_call_with_values`
  and friends pass MV-packs through registers. Audit to ensure no
  MV-pack is held only in a register across an allocation.
- **`apply` with deep arg lists.** `apply_build_args` already builds
  the arg list incrementally; verify that the head/tail pointers
  are spilled to the Scheme stack across each `cons`.
- **Statistics.** A `(gc-stats)` primitive returning collection
  count, bytes copied, last pause length. Useful for the test
  scaffolding and for `cc.scm` self-instrumentation. Defer until
  after stage 5 lands.
