// B4 dialect corpus: tuples. In subrust v0.1 tuples are deliberately minimal —
// the parser/checker reject tuple TYPES (so no tuple params/returns) and `.0`/
// `.1` field access (E_TUPLE). What IS supported is construct-and-destructure:
// `let (a, b) = (x, y);`. The tuple is laid out like an array of cells and the
// let stores it contiguously, binding each name to a sub-slot. Output via putb.
fn main() {
    // destructure a tuple literal
    let (a, b) = (65u64, 66u64);
    putb(a); // A
    putb(b); // B
    // destructure a tuple built from variables (parallel bind, swapped)
    let x = 67u64;
    let y = 68u64;
    let (p, q) = (y, x);
    putb(q); // C (67)
    putb(p); // D (68)
    // a three-element destructure
    let (i, j, k) = (69u64, 70u64, 71u64);
    putb(i); // E
    putb(j); // F
    putb(k); // G
}
