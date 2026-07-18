// B4 dialect corpus: struct field shorthand `S { x }` (== `S { x: x }`), reading
// the same-named local. The checker uses it constantly when building tokens/
// nodes (`push_tok(Tok { kind, len, ... })`). Emit synthesizes a local load for
// each shorthand field. Also exercises returning a struct by value. Output via putb.
#[derive(Clone, Copy)]
struct Tok {
    kind: u64,
    len: u64,
}
fn make(kind: u64, len: u64) -> Tok {
    Tok { kind, len } // both fields shorthand, struct returned by value
}
fn main() {
    let kind: u64 = 65;
    let len: u64 = 66;
    let t = Tok { kind, len }; // shorthand from locals
    putb(t.kind); // A (65)
    putb(t.len); // B (66)
    let u = make(67, 68);
    putb(u.kind); // C (67)
    putb(u.len); // D (68)
    // mixed: one shorthand field, one explicit
    let kind2: u64 = 69;
    let v = Tok {
        kind: kind2,
        len: 70,
    };
    putb(v.kind); // E (69)
    putb(v.len); // F (70)
}
