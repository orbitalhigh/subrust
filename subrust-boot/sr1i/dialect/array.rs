// B4 dialect corpus: arrays — literals, repeat `[x; N]`, and bounds-checked
// indexing (constant and by a runtime counter). Output via putb.
fn main() {
    let a = [65u64, 66, 67, 68]; // A B C D
    putb(a[0]); // A
    putb(a[3]); // D
    // index by a runtime counter
    let mut i: usize = 0;
    while i < 4 {
        putb(a[i]); // A B C D
        i += 1;
    }
    // repeat literal
    let z = [90u64; 3]; // Z Z Z
    putb(z[0]);
    putb(z[1]);
    putb(z[2]);
    // array built from indexed elements of another array
    let b = [a[1], a[2]]; // B C
    putb(b[0]);
    putb(b[1]);
}
