// B4 dialect corpus: references — &local, *p read, *p write, and passing refs
// to functions (read through &T, write through &mut T). Output via putb.
fn set(p: &mut u64, v: u64) {
    *p = v;
}
fn get(p: &u64) -> u64 {
    *p
}
fn main() {
    let mut x: u64 = 65;
    putb(get(&x)); // A (read through &u64)
    set(&mut x, 66); // write through &mut u64
    putb(x); // B
    putb(get(&x)); // B
    // a reference bound to a local, written through *r
    let mut y: u64 = 90;
    let r = &mut y;
    *r = 91;
    putb(y); // [ (91)
    // read through a &-local
    let s = &y;
    putb(*s); // [
}
