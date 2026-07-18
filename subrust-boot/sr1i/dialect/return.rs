// B4 dialect corpus: early `return`. sr1i is a recursive tree-walker, so return
// is a signal (R_SIG=3) that propagates up through blocks/loops to the enclosing
// call; the value rides a RETVAL buffer so block/loop stack resets can't clobber
// it. The self-hosted checker is full of early returns (guards, error bail-outs).
// Output via putb.
fn classify(n: u64) -> u64 {
    if n == 0 {
        return 90; // early return with a value (Z)
    }
    if n < 10 {
        return 65; // A
    }
    66 // B — tail expression, no return
}
fn first_ge(arr: &[u64; 4], threshold: u64) -> u64 {
    let mut i: usize = 0;
    while i < 4 {
        if arr[i] >= threshold {
            return arr[i]; // return from inside a loop
        }
        i += 1;
    }
    0
}
fn maybe_putb(flag: bool) {
    if !flag {
        return; // bare early return from a unit function
    }
    putb(68); // D
}
fn main() {
    putb(classify(0)); // Z (90)
    putb(classify(5)); // A (65)
    putb(classify(50)); // B (66)
    let table = [10u64, 20, 67, 40];
    putb(first_ge(&table, 67)); // C (67) — found mid-loop
    maybe_putb(false); // prints nothing (bare return)
    maybe_putb(true); // D (68)
}
