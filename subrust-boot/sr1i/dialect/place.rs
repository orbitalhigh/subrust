// B4 dialect corpus: place expressions (deref-paths). Every assignable place —
// a struct field, an array element, a field/element reached through a reference —
// resolves to an absolute address; writes store through it, reads load from it.
// This is the machinery the self-hosted checker leans on hardest (it mutates
// node/struct fields and indexes tables everywhere). Output via putb.
#[derive(Clone, Copy)]
struct Point {
    x: u64,
    y: u64,
}
fn set_x(p: &mut Point, v: u64) {
    p.x = v; // field WRITE through &mut (RES_DEREF place)
}
fn get_y(p: &Point) -> u64 {
    p.y // field READ through &ref (RES_DEREF value)
}
fn bump(a: &mut [u64; 3], i: usize) {
    a[i] = a[i] + 1; // index READ and WRITE through a &mut ref
}
fn main() {
    let mut pt = Point { x: 10, y: 66 };
    pt.x = 65; // struct field WRITE on a local
    putb(pt.x); // A (65) — value-struct field read
    set_x(&mut pt, 90); // write pt.x = 90 through the ref
    putb(get_y(&pt)); // B (pt.y = 66) — field read through a ref
    let mut arr = [0u64; 3];
    arr[0] = 67; // array element WRITE (constant index)
    putb(arr[0]); // C (67)
    let mut i: usize = 1;
    arr[i] = 68; // array element WRITE (runtime index)
    putb(arr[i]); // D (68)
    arr[2] = 68;
    bump(&mut arr, 2); // arr[2] = 68 + 1 = 69 through a &mut ref
    putb(arr[2]); // E (69)
    putb(pt.x); // Z (90) — confirms set_x wrote through the ref
}
