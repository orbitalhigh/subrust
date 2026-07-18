// B4 dialect corpus (trap): out-of-bounds array index must trap, with the
// output prefix (bytes 1,2,3) still emitted before the trap.
fn main() {
    let a = [1u64, 2, 3];
    let mut i: usize = 0;
    while i <= 3 {
        putb(a[i]); // a[3] is out of bounds (len 3) → trap
        i += 1;
    }
}
