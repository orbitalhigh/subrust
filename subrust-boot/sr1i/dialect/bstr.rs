// B4 dialect corpus: byte-string literals b"..." — a &[u8] fat pointer
// [POOL_TAG|off, len] whose bytes live in the string pool, not frame memory.
// Exercises pool-backed indexing s[i], slice .len() over a pool slice, byte
// literals b'x', and indexing a literal directly. putb takes u64 while
// byte-slice elements are u8 (casts are deferred), so output is driven by
// u8 == b'x' comparisons — no widening needed.
fn main() {
    let s = b"ABC"; // &[u8] -> [POOL_TAG|off, 3]
    if s[0] == b'A' {
        putb(65); // A
    } else {
        putb(63);
    }
    if s[1] == b'B' {
        putb(66); // B
    } else {
        putb(63);
    }
    if s[2] == b'C' {
        putb(67); // C
    } else {
        putb(63);
    }
    if s.len() == 3 {
        putb(76); // L — .len() of a pool-backed slice
    } else {
        putb(63);
    }
    if b"Z"[0] == b'Z' {
        putb(90); // Z — index a byte-string literal directly
    } else {
        putb(63);
    }
}
