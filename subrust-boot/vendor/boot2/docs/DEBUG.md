# DEBUG

Debugging facilities for the cc / P1pp / M1pp / hex2pp pipeline. Each
section: what the tool does, how to turn it on, what you get back.

## Tracepoints (`%trace` / `--cc-trace-emit`)

Per-function-entry stderr probe. With `CC_TRACE_EMIT=1` set, cc.scm
injects a `%trace(&LBL, LEN)` between each function's prologue and
body. At runtime each entry prints one line:

```
[trace @601a34 main]
```

The hex is the runtime address of the trace site. The trailing word is
the mangled function name, interned through cc's regular string pool.

Build + run:

```sh
CC_TRACE_EMIT=1 sh scripts/run-tests.sh --suite cc --arch aarch64 007-call-with-args
# stderr:
# [trace @6019fc main]
# [trace @6018fc g]
```

For a built ELF outside the test runner:

```sh
make tcc-boot2 ARCH=aarch64 CC_TRACE_EMIT=1
./build/aarch64/tcc-boot2/tcc-boot2 -version 2>trace.log
```

Cost: register save/restore traffic plus one call per traced function.
Off by default; the `%trace` macro itself lives in
[P1/P1pp.P1pp](../P1/P1pp.P1pp) (§Tracepoint) and can also be invoked
manually — drop a `%trace(&label, len)` into any `combined.M1pp`
snapshot under `build/$ARCH/.work/<src>/`, re-run the M1pp/hex2pp
stages, and bisect by stderr position. `%trace` preserves the exposed
P1 registers (`a0..a3`, `t0..t2`, `s0..s3`) by borrowing temporary
stack space, so it is safe to add inside an active `%fn` body after
the function prologue. The borrowed area includes the backend's
standard frame prefix, so trace saves stay below the caller's frame.

To map an address back to its function, see the lookup tool below.

## Address → label lookup (`m1-symbols.py lookup`)

Resolves a runtime address (e.g. from a `%trace` line) to its
enclosing function. Reads the label map straight out of `expanded.hex2pp`
(or legacy `prog.hex2`) and finds the largest label address `<= target`,
skipping M1pp's mangled macro-locals (`:@name` → `:name__N`) so a trace address
resolves to the *function* containing it, not the trace's own
`:@here`.

```sh
# Pass the ELF; the tool reads <ELF>.workdir to find expanded.hex2pp.
tools/m1-symbols.py lookup --elf build/aarch64/tests/cc/007-call-with-args 0x6019fc 0x6018fc
# 0x6019fc   main+0x24
# 0x6018fc   g+0x2c

# Pipe the trace log through it.
./build/aarch64/tests/cc/007-call-with-args 2>&1 \
  | grep -oE '@[0-9a-f]+' | tr -d @ \
  | tools/m1-symbols.py lookup --elf build/aarch64/tests/cc/007-call-with-args
```

Other input modes: `--hex2 <expanded.hex2pp|prog.hex2>` (skip the
sidecar lookup) or `--map <file>` (use a pre-built map from
`m1-symbols.py map`). Pass
`--include-macro-locals` to see the closest label even when it's a
`name__N` artifact — useful when you want to know which trace site
fired vs. which function it sits in.

Output is `0xADDR\tLABEL+0xN`, tab-separated, one per line.

## Disassembly (`disasm-elf.sh`)

llvm-objdump wrapper that handles two quirks of our seed ELF: oversized
ph_memsz (truncated to ph_filesz on a temp copy) and the absent section
table (replaced by labels injected from `expanded.hex2pp`, or legacy
`prog.hex2`). The output has
real `<funcname>:` headers and `<PT_LOAD#0+0xNNN>` xrefs rewritten to
`<label+offset>`:

```sh
tools/disasm-elf.sh build/aarch64/tests/cc/007-call-with-args
# 0000000000601a34 <main>:
#   601a34: ...
#   601a40: ldr w17, ... <libp1pp__trace+0x0>
```

Defaults to `-d` (text). Pass `-D` for data + text. `--start-address`
defaults to `e_entry` (skipping the on-disk ELF header bytes); override
with your own `--start-address=` to see the header. The
`<elf>.workdir` sidecar must exist for label annotation; it's written
automatically by `boot-build-p1*.sh`. Set `NO_LABELS=1` to disable
annotation.

## cc.scm phase tracing (`CC_DEBUG=1`)

cc.scm has a sticky `(debug-log ...)` channel for between-phase heap
usage. Toggle with `CC_DEBUG=1` (boot-build-cc.sh) or `--cc-debug`
(direct invocation). One stderr line per phase:

```sh
CC_DEBUG=1 sh scripts/run-tests.sh --suite cc --arch aarch64 007-call-with-args
# [cc] lex ... heap=...
# [cc] pp ... heap=...
# ...
```

Most useful when bisecting an OOM or watching where parse memory
balloons. Independent of `CC_TRACE_EMIT` — combine freely.

## Pipeline intermediates (`build/$ARCH/.work/`)

Every P1pp/P1 build leaves its intermediates next to the ELF:

```
build/$ARCH/.work/<src-path>/
    combined.M1pp     # backend + frontend + libp1pp + user TU, catm'd
    expanded.hex2pp   # M1pp output, ready for hex2pp
    linked.hex2pp     # ELF header + expanded.hex2pp
    cc.log            # cc.scm stderr (if --cc-debug or trace-emit)
    p1pp.log          # M1pp/hex2pp stderr
```

Each ELF also gets a one-line `<elf>.workdir` sidecar pointing at this
directory — that's how `disasm-elf.sh` and `m1-symbols.py lookup
--elf` find `expanded.hex2pp` (or legacy `prog.hex2` for raw-P1
builds). On a failing build the runner prints the
partial-intermediates path; on a passing build the files stay around
for inspection.

To re-run a single intermediate stage by hand: edit the file in place
and invoke the next tool directly (`build/$ARCH/M1pp/M1pp
combined.M1pp expanded.hex2pp`, then `build/$ARCH/hex2pp/hex2pp
-B 0x600000 linked.hex2pp out`). Useful for poking `%trace` calls into
`combined.M1pp` without recompiling cc.scm.

## End-to-end debugging recipe

A typical "binary segfaults, where?" loop:

```sh
# 1. Build with traces.
CC_TRACE_EMIT=1 make build/aarch64/tests/cc/myprog ARCH=aarch64

# 2. Run; capture trace.
./build/aarch64/tests/cc/myprog 2>trace.log; echo "exit=$?"

# 3. Last trace line shows the last function entered before the crash.
tail -1 trace.log
# [trace @601b40 parse_decl]

# 4. (Optional) Resolve all addresses to functions for context.
grep -oE '@[0-9a-f]+' trace.log | tr -d @ \
  | tools/m1-symbols.py lookup --elf build/aarch64/tests/cc/myprog

# 5. Inspect the disassembly around the crash site.
tools/disasm-elf.sh build/aarch64/tests/cc/myprog \
  | grep -B 2 -A 20 '<parse_decl>:'
```
