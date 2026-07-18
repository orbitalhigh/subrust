# Building tcc-0.9.26 from this repo

Describes the host-side flatten step that produces `tcc.flat.c` —
the single-file C bytestream that the boot pipeline (`boot3.sh` →
`boot4.sh`) feeds to cc.scm to bootstrap tcc-0.9.26.

[bootprep/stage1-flatten.sh](../bootprep/stage1-flatten.sh) runs on
the host (macOS or Linux). It flattens upstream `tcc.c` into a single
`tcc.flat.c` bytestream using only the host C preprocessor — no
M2-Planet, Mes Scheme, or MesCC anywhere.

This is the upstream half of the [CC.md](CC.md) story: once cc.scm
ingests `tcc.flat.c`, the rest of the pipeline (`boot3` → `boot4`)
is in-tree.

## Inputs

| Path | Contents |
|------|----------|
| `../lb-work/distfiles/tcc-0.9.26.tar.gz`             | tcc-0.9.26-1147-gee75a10c source (janneke's bootstrap-friendly fork; same artifact live-bootstrap consumes) |
| `../lb-work/distfiles/mes-0.27.1.tar.gz`             | GNU Mes 0.27.1 — used **only** for its bundled minimal libc sources and headers, not for any Scheme runtime |
| `../live-bootstrap/steps/tcc-0.9.26/simple-patches/` | Two file-open reorder patches applied before the flatten step |
| `../mes/include/`                                    | Same Mes headers as the tarball — used at flatten time so we don't pull in host glibc/musl |

The three scripts sit on top of these inputs; they require nothing
else from the host besides `tar`, `awk`, a host `cc`, and `podman`.

## Pipeline overview

```
tcc-0.9.26-1147-gee75a10c.tar.gz                   live-bootstrap source
        │
        │   stage1-flatten.sh                      (host)
        │     • unpack
        │     • apply 2 simple-patches
        │     • host cc -E -nostdinc with mes headers + tcc-mes defines
        ▼
build/$ARCH/vendor/tcc/tcc.flat.c          ~600 KB single-file C
```

`tcc.flat.c` is a portable artifact; downstream the boot pipeline
(`boot3.sh` → `boot4.sh`) feeds it to cc.scm to produce a working
tcc-0.9.26.

## Stage 1 — flatten tcc.c into tcc.flat.c

`bootprep/stage1-flatten.sh --arch X86_64`

Mirrors the live-bootstrap `tcc-mes` invocation
([steps/tcc-0.9.26/pass1.kaem:60–87](../../live-bootstrap/steps/tcc-0.9.26/pass1.kaem))
minus the actual compile. The host preprocessor expands every `#include`
in `tcc.c` (which uses `ONE_SOURCE=1` to fold `libtcc.c`, `tcctools.c`,
and the per-arch backends in via `#include`) and inlines all the Mes-
bundled standard headers.

Stage 1 deliberately stays on the host — it is just text manipulation
and the `tcc.flat.c` it produces is consumed identically downstream
regardless of where stage 1 ran.

### Sub-steps

1. **Unpack** `tcc-0.9.26.tar.gz` into `build/amd64/vendor/tcc/`.
2. **Apply simple-patches**: `remove-fileopen.before/.after` then
   `addback-fileopen.before/.after` against `tcctools.c`. Implemented
   as an `awk` literal-block replacer (live-bootstrap's `simple-patch`
   is a trivial before/after substitution).
3. **Empty config.h shims**: live-bootstrap creates two empty
   `config.h` files via `catm`. We do the same — one in
   `$TCC_PKG/config.h`, one in `mes-overlay/mes/config.h` for the
   `<mes/config.h>` reach the Mes stdio.h does.
4. **Host preprocess**: `cc -E -nostdinc` with the Mes headers as the
   sole `-I` set, plus the same `-D` set live-bootstrap passes:

   ```
   -D BOOTSTRAP=1
   -D HAVE_LONG_LONG=1
   -D inline=
   -D ONE_SOURCE=1
   -D TCC_TARGET_X86_64=1
   -D __linux__=1
   -D __x86_64__=1            # mescc would inject these; we mirror them
   -D CONFIG_TCC_*="..."      # exactly the live-bootstrap paths
   ```

   Output: a single ~600 KB C bytestream, no remaining `#include`s,
   no preprocessor directives at all.

5. **(Optional, --verify)** host `cc -c tcc.flat.c -> tcc.flat.o`. On
   macOS this produces a Mach-O .o; the verify is purely a "does the
   source compile" check. Failure here means the flatten step is wrong.


## What this unlocks for the scheme1 cc

The interface for the slot scheme CC fills:

- **Input**: `tcc.flat.c` produced by stage 1.
- **Output**: a working ELF capable of compiling the same `tests/cc`
  fixtures the regular `cc` suite covers.

`make tcc-boot2 ARCH=aarch64` runs that path end-to-end:
`cc.scm + tcc.flat.c → tcc-boot2`, linking against a `cc.scm`-built
`libc.flat.c` instead of mes libc. The `tcc-cc` acceptance suite
(`make test SUITE=tcc-cc`) shows full parity with the gcc-built
control on aarch64 and amd64.

## Reproducibility

```
bootprep/stage1-flatten.sh --arch X86_64
```

Artifacts in `build/$ARCH/vendor/tcc/`:

| File          | Size   | Built by | What it is                         |
|---------------|--------|----------|------------------------------------|
| `tcc.flat.c`  | ~600KB | host cc  | flattened single-source tcc-0.9.26 |

`build/` is in `.gitignore`; nothing tracked outside the scripts
themselves.

## Issues / bugs

### tcc 0.9.26 SEGV on large concatenated TU

When ~22+ mes libc files are catm'd into one TU and the chain hits a
file with non-trivial inline asm (the trigger we found was
`linux/x86_64-mes-gcc/_exit.c`), tcc-0.9.26 crashes mid-compile.
Below that threshold all combinations work. Each individual file
compiles fine.

The interaction is some accumulator state inside tcc — symbol table,
hash chain, or similar — that overflows or hits a corrupted state
when the TU grows large enough.

**Workaround**: compile each `.c` separately, then `ar` together.
The boot pipeline does this for the mes libc / musl per-file
sweeps. Bonus: avoids redundant header re-parses, faster overall.

**Confirmed in canonical live-bootstrap.** The
[`tools/diag-livebootstrap-qemu.sh`](../tools/diag-livebootstrap-qemu.sh)
diagnostic runs upstream live-bootstrap's amd64 pass1 chain inside
the same busybox + linux/amd64 QEMU we use, and its mescc-built
`tcc-mes` SEGVs at exactly this step (`tcc-mes -c unified-libc.c`
with `assert fail: 0` then SIGSEGV). The per-file workaround is
load-bearing for any tcc-0.9.26-on-QEMU build, not specific to our
path.

## Known limitations (riscv64)

aarch64 and amd64 are at full self-host parity (cc.scm path matches
the gcc-built control on every fixture). riscv64 has two real open
items, both rooted in tcc's riscv64 backend rather than in cc.scm
or the P1 pipeline.

### riscv64: u32 narrowing leaves dirty upper bits

`tests/cc/335-ternary-merge-arith-conv` fails on riscv64 in both
`tcc-cc[stage2]` and `tcc-cc[stage3]` (identical behavior — the
fixed-point property holds, the bug is in tcc's RISC-V codegen, not
in cc.scm or the P1 pipeline). aarch64 and amd64 are green.

The proximate trigger is in `riscv64-gen.c::load()`:

```c
func3 = size == 1 ? 0 : size == 2 ? 1 : size == 4 ? 2 : 3;
if (size < 4 && !is_float(sv->type.t) && (sv->type.t & VT_UNSIGNED))
    func3 |= 4;          // promotes lb→lbu, lh→lhu, but skips lw→lwu
```

The `func3 |= 4` promotion to LWU is gated on `size < 4`, so a 4-byte
unsigned load uses LW (sign-extending) instead of LWU (zero-extending).
`gen_cast` to `VT_INT|VT_UNSIGNED` from a wider source emits no
narrowing — it relies on the use-time load to truncate, but with LW
the high u32 bits of the source leak through. `(u32)x` where `x` is
`u64` with bit 31 set then evaluates to `0xFFFFFFFFFFFFFFFF`. This
same bug is present in upstream tcc mob.

**Why the one-line patch isn't enough.** Widening the gate to
`size <= 4` (so 4-byte unsigned loads use LWU) regresses
`017-int-arith` and `128-cast-signedness`. They were passing because
two compensating bugs canceled out: stock tcc on riscv64 also
sign-extends unsigned 32-bit immediate constants (`LUI`/`ADDI` with a
bit-31-set value), so a comparison between an `unsigned int`
variable (loaded with sign-extending LW) and an `unsigned int`
constant (loaded with sign-extending LUI/ADDI) had matching dirty
upper bits and `BEQ` saw them as equal. Fixing only the load breaks
that join, because the compare path also lies — `BEQ` is a 64-bit
instruction but C semantics require 32-bit width for `unsigned int ==
unsigned int`.

**Full fix shape.** Three coupled pieces: (1) load — emit LWU for
unsigned 4-byte loads; (2) immediate — clear bits 32–63 when
materializing an unsigned 32-bit constant with bit 31 set; (3)
compare — eagerly canonicalize 32-bit-typed values into zero-extended
or sign-extended form (per `VT_UNSIGNED`) after every op that can
leave the upper half dirty. Pieces 2 and 3 overlap: if values are
canonicalized at every produce site, the load fix becomes one of many
sites that need to do it. This is what gcc/clang's RISC-V backends
do, and it's beyond the scope of the literal-block `simple-patches`
mechanism — file upstream or write a real canonicalization pass.

For now: known limitation, document, move on. The scalar codegen
elsewhere on riscv64 is fine — only u32 narrowing of a wider source
trips it.

### riscv64: tcc0 → tcc1 is not a fixed point (cc.scm behavioral bug)

`boot3.sh` + `boot4.sh` produce four staged compilers:

- `tcc0` = tcc-source compiled by cc.scm     (boot3 output)
- `tcc1` = tcc-source compiled by tcc0       (boot4)
- `tcc2` = tcc-source compiled by tcc1       (boot4)
- `tcc3` = tcc-source compiled by tcc2       (boot4)

The fixed-point check is **`tcc2 == tcc3`** (asserted at the end of
`boot4.sh`, verified on aarch64, amd64, riscv64). On riscv64 the
weaker `tcc1 == tcc2` does *not* hold: `tcc0(tcc.flat.c)` produces
a 616100-byte `.o` while `tcc1(tcc.flat.c)` and `tcc2(tcc.flat.c)`
produce a byte-identical 615892-byte `.o` — 208 bytes larger from
tcc0 (200 in `.text` + 8 ripple in symtab/reloc offsets). amd64 and
aarch64 satisfy `tcc1 == tcc2`; only riscv64 diverges.

This is a **bug to investigate**, not just a "fatter code"
observation. cc.scm should be a *faithful* (semantics-preserving)
compiler — slower or larger output is acceptable, but tcc0 and tcc1
must produce byte-identical output when run on the same source.
That they don't on riscv64 means cc.scm's translation of tcc.flat.c
into tcc0 changed what tcc0 *does at runtime*, not just how it's
encoded. We don't care about peephole optimizations being missed; we
do care that tcc0 makes different codegen decisions than tcc1
makes.

#### What's known

The visible symptom: tcc0 emits 4 RISCV codegen patterns differently
than tcc1 does:

| Source pattern | tcc0 emits | tcc1 emits | Δ |
|---|---|---|---|
| `x = x - imm` (i32) | `addiw t,zero,imm; addw rd,rs,t` | `addiw rd,rs,imm` | +4 B |
| `x = x & imm` | `addiw t,zero,imm; and rd,rs,t` | `andi rd,rs,imm` | +4 B |
| zero-ext after `sext.w` | `sext.w r,r; slli r,r,0x20; srli r,r,0x20` | `sext.w r,r` | +8 B |
| `x == 0xFFFFFFFF` (i32) | `addiw t,zero,-1; slli/srli; beq x,t,L` | `addi x,x,1; beqz x,L` | +8 B |

These are decision points in `riscv64-gen.c` (immediate-folding,
zero-ext elision). Same source code, same input C, but the running
tcc0 takes the slow branch where the running tcc1 takes the fast
one — even though both are compiled from the same `tcc.flat.c`.

#### Hypothesis to test

cc.scm likely miscompiles an integer comparison or bit-test inside
the immediate-fits-in-instruction guard in `riscv64-gen.c`. Most of
the missed patterns share the shape `if (small_int_fits) { fold } else
{ materialize }`. If cc.scm gets the predicate wrong (e.g. signed vs.
unsigned compare, or wrong branch on a particular bit pattern), tcc0
falls into the materialize path on inputs where tcc1 takes the fold
path.

#### Repro / starting point

```sh
# In the riscv64 container with boot3+boot4 outputs present:
$TCC0 -nostdlib -c -o /tmp/flat-tcc0.o tcc.flat.c
$TCC1 -nostdlib -c -o /tmp/flat-tcc1.o tcc.flat.c
# wc -c /tmp/flat-tcc0.o /tmp/flat-tcc1.o   →  616100 vs 615892
# objdump -d both, normalize addresses, diff to find divergent functions
```

The first divergent function in disassembly is `tal_free_impl` — a
small refcount-decrement that hits the "x = x - 1" pattern. Good
starting point because the function is short and the source path is
narrow.

Until this is fixed, tcc1 is the "shake-out" stage and tcc2 is the
canonical compiler.

## AT-series patches (post-bootstrap uniformity)

These patches go beyond the bootstrap-stub patches in
`vendor/tcc/patches/` and exist to remove per-arch
workarounds in seed-kernel and the build pipeline. They live in the
same patch directory and are listed in `bootprep/stage1-flatten.sh`'s
`apply_our_patch` block.

### AT.2 — native PT_NOTE for PVH boot

Stock tcc 0.9.26 tags every assembler-created section as
`SHT_PROGBITS` and emits only `PT_LOAD` phdrs for static EXEs.
QEMU's PVH `-kernel` path on amd64 scans `PT_NOTE` phdrs for the
Xen 18 note that names the 32-bit entry; without one it errors out
("Error loading uncompressed kernel without PVH ELF Note"). The
old workaround was a post-link host tool, `elf-pvh-note.c`, that
rewrote the ELF after tcc finished. AT.2 replaces it with two
patches in `tccelf.c`:

- `note-section-sht-note` — `find_section()` creates `.note*`
  sections as `SHT_NOTE` instead of `SHT_PROGBITS`.
- `pt-note-phdr` — `elf_output_file()` bumps `phnum` by one when at
  least one `SHT_NOTE+SHF_ALLOC` section exists, then fills the
  reserved phdr slot with a `PT_NOTE` covering
  `[min(sh_offset), max(sh_offset+sh_size))` over those sections.
  The bump is gated on actual presence so arches with no note
  sections (aarch64, riscv64) keep the same phnum and produce
  byte-identical output.
- `load-obj-accept-sht-note` — `tcc_load_object_file()`'s
  accepted-section-type list adds `SHT_NOTE`. Without this, .o
  files emitted by the patched `find_section` (which now produces
  `SHT_NOTE` for `.note*`) get silently skipped during the link;
  the subsequent `.rela.note.*` merge then derefs a NULL
  `sm_table[].s` for the (skipped) note section. Strict pair with
  `note-section-sht-note`.

After AT.2 the post-link `elf-pvh-note.c` tool, the amd64-only
branch in `boot6.sh`, and the amd64 fixup block in
`boot6-gen-runscm.sh` are all gone. The amd64 kernel is emitted
ready-to-boot by tcc3.
