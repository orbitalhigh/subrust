// B4 dialect corpus: structs (multi-slot values). Beyond SR-seed, so the
// reference is rustc-native (BOOT shims), not sr0i-direct. Output via putb.
#[derive(Clone, Copy)]
struct P {
    a: u64,
    b: u64,
}
fn main() {
    let p = P { a: 65, b: 66 };
    putb(p.a); // A
    putb(p.b); // B
    // construct from fields (field reads feeding a struct literal)
    let q = P { a: p.b, b: p.a };
    putb(q.a); // B
    putb(q.b); // A
    // field arithmetic into a new struct, then read back
    let s = P { a: p.a + 2, b: q.a + 2 };
    putb(s.a); // C
    putb(s.b); // D
    // a struct bound through another let (multi-slot copy)
    let t = s;
    putb(t.a); // C
    putb(t.b); // D
}
