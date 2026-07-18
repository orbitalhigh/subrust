# hex2++

A small, byte-oriented assembler/linker that takes hex source with labels and
references and emits a flat binary. Implemented in P1; used by `cc.scm` and
the P1 backends as the final stage of the `M1pp → hex2++` toolchain. M1pp
output feeds hex2++ directly — there is no intermediate macro/hex stage.

## Invocation

```
hex2++ [-B ADDR]        # base address
       [-E | -e]        # big-endian | little-endian (default: little)
       [-b]             # binary digit mode (default: hex)
       [-N]             # non-executable output
       IN OUT
```

`IN` and `OUT` are positional: a single input file and a single output file.
To assemble several sources together, concatenate them upstream (e.g. with
`catm`) and pass the combined file as `IN`. Output is one flat binary
written from `Base_Address` upward. Unless `-N` is set and the output is a
regular file, the output is `chmod 0750`'d.

There is no per-target configuration. Any target-specific encoding (RISC-V
bitfield-scattered immediates, native branch displacements, etc.) is the
responsibility of the upstream M1pp layer, which packs full instruction
words at expansion time. hex2++ sees only contiguous-byte values.

## Lexical structure

- **Whitespace** — space, tab, newline; separates tokens, otherwise ignored.
- **Comments** — `#` or `;` to end of line.
- **Byte mode** — chosen once at invocation:
  - `HEX` (default): two hex digits → one byte. Digits `0-9 a-f A-F`.
  - `BINARY` (`-b`): eight binary digits → one byte. Digits `0 1`.

Bytes within a token may be separated by whitespace freely; only digit count
matters.

Active characters:

```
0-9 a-f A-F  hex digits (HEX mode)
0-1          binary digits (BINARY mode)
:            label definition
. (+kw)      directive (.align, .fill, .scope, .endscope, .ptrsize)
! @ $ ~ % &  label reference
- >          label arithmetic in references (synonyms)
# ;          line comment
ws           token separator
```

## Labels

```
:NAME       define label NAME at the current emit position (ip)
```

Label names are tokens terminated by whitespace or `-`. Labels may be
referenced before they are defined; forward references resolve in pass 2.

The label namespace is global except that names beginning with `.` *inside
a `.scope`* are local to that scope. The leading character of the token
disambiguates labels from directives: `:.NAME` is a label definition,
`&.NAME` / `%.NAME` / etc. are label references, and a bare `.NAME` (no
leading `:` or sigil, at statement position) is a directive. Directive
names are therefore reserved only at statement position, and remain
available as label tokens when prefixed with `:` or a sigil.

```
.scope
  :.L1
  ...
  &.L1
.endscope
```

- `.scope` directives nest. A dotted reference inside a scope resolves to
  the nearest enclosing definition, so an inner scope shadows an outer one
  with the same local name.
- Non-dotted labels defined inside a `.scope` remain global.
- Dot-prefixed labels outside any `.scope` are ordinary global labels;
  the leading `.` is just part of the name.

## Label references

A reference is a single sigil character followed by a label expression:

| Sigil | Width    | Form | Range                  |
|-------|----------|------|------------------------|
| `!`   | 1 B      | rel  | `-128..127`            |
| `@`   | 2 B      | rel  | `-32768..32767`        |
| `$`   | 2 B      | abs  | `0..65535`             |
| `~`   | 3 B      | rel  | `-2^23..2^23-1`        |
| `%`   | ptrsize  | rel  | unchecked              |
| `&`   | ptrsize  | abs  | unchecked              |

The width of `%` and `&` is set by [`.ptrsize`](#ptrsize-n) — 4 bytes by
default, 8 for 64-bit pointer targets.

- "rel" emits `target - base`, where `base` is `ip` immediately after the
  reference's bytes are accounted for.
- "abs" emits the target's absolute address (which includes `Base_Address`).
- Multi-byte values are emitted little-endian unless `--big-endian` is set.

The label expression takes one of two forms:

```
SIGIL LABEL                    # plain reference
SIGIL LABEL - OTHER            # emit target(LABEL) - target(OTHER)
SIGIL LABEL > OTHER            # synonym for `LABEL - OTHER`
```

The `LABEL - OTHER` form overrides the default base with another label, and
applies uniformly to all sigils. `>` is accepted as an alias for `-` so
hex2 inputs that use the relative-base override syntax assemble unchanged;
both produce identical bytes. Both labels must be defined somewhere in the
input. Range checks apply identically to plain and arithmetic forms.

Only one subtraction per reference; no addition, nesting, or
parenthesization.

Examples:

```
# jump table entries
:jt
  &case0-jt   &case1-jt   &case2-jt

# string length prefix (string bytes themselves come from the
# upstream M1pp layer, which decodes a bare `"hello"` into the
# five hex bytes shown here)
:s_begin
  68 65 6c 6c 6f
:s_end
  &s_end-s_begin
```

## Directives

### `.align N [PATTERN]`

```
.align N            # pad to N-byte boundary with zero bytes
.align N PATTERN    # pad with the given byte/word pattern
```

- `N` is a positive power-of-two decimal integer.
- `PATTERN`, if present, is a hex byte or hex word literal in the current
  byte mode (e.g. `00`, `90`, `d503201f`). The pattern is repeated and
  rotated as needed to fill the gap.
- If `ip` is already aligned, no bytes are emitted.

The pad pattern is supplied by whichever upstream layer knows the target
(typically a per-backend M1pp macro). hex2++ stays target-neutral.

### `.fill N B`

```
.fill N B           # emit N copies of byte B
```

- `N` is a non-negative decimal integer.
- `B` is one byte literal in the current byte mode.

### `.scope` / `.endscope`

See [Labels](#labels).

### `.ptrsize N`

```
.ptrsize 4          # default
.ptrsize 8          # 64-bit pointer targets
```

Sets the byte width of the `&` and `%` sigils. `N` must be `4` or `8`.

`.ptrsize` is whole-invocation: the first occurrence seen across all
inputs binds the width for the entire run, and any subsequent
`.ptrsize` must specify the same value or it is an error. If no
`.ptrsize` directive appears, the width defaults to `4`.

## Implementation outline

Two passes:

- **Pass 1** — read every input file, advancing `ip` and recording label
  definitions. `.align` and `.fill` advance `ip` deterministically;
  `.scope` / `.endscope` push and pop the scope stack, assigning each
  open scope a fresh id.
- **Pass 2** — re-read, emit bytes, resolve references. Scope ids are
  assigned in the same order, so pass-1 definitions and pass-2
  references see identical ids.

The label table carries `(name, target_ip, scope_id)` entries. Lookup for a
dotted name walks the scope stack innermost-out and returns the first match;
lookup for a non-dotted name ignores scope.

Both labels in `LABEL-OTHER` have known addresses by the start of pass 2, so
the subtraction is a single operation at emit time. No third pass is
required.
