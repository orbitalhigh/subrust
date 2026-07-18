// B4 dialect corpus: `&place` in general — the empty slice `&[]` (a [0,0] fat
// pointer), and thin references to a struct field (`&s.a`) or an array element
// (`&arr[i]`), read back through the reference. These build on the paddr address
// model. Output via putb.
#[derive(Clone, Copy)]
struct P {
    a: u64,
    b: u64,
}
fn read(p: &u64) -> u64 {
    *p
}
fn main() {
    // &[] empty slice -> length 0
    let empty: &[u64] = &[];
    if empty.len() == 0 {
        putb(65); // A
    } else {
        putb(63);
    }
    // &s.field -> thin ref, read through it
    let s = P { a: 66, b: 67 };
    putb(read(&s.a)); // B (66)
    putb(read(&s.b)); // C (67)
    // &arr[i] -> thin ref to an element (constant and runtime index)
    let arr = [68u64, 69, 70];
    putb(read(&arr[0])); // D (68)
    let i: usize = 2;
    putb(read(&arr[i])); // F (70)
}
