// B4 dialect corpus: &str — a 1-slot string id whose byte span lives in the
// pool. Exercises .len() (RES_STR_LEN via the STRS table), .as_bytes()
// (RES_STR_BYTES -> [POOL_TAG|off, len]) then pool-backed indexing, and passing
// a &str to a function. putb takes u64 while bytes are u8 and casts are
// deferred, so output is driven by u8 == b'x' / usize comparisons.
fn slen(s: &str) -> usize {
    s.len()
}
fn main() {
    let s = "ABCD";
    let b = s.as_bytes(); // &[u8] -> [POOL_TAG|off, 4]
    if b[0] == b'A' {
        putb(65); // A
    } else {
        putb(63);
    }
    if b[3] == b'D' {
        putb(68); // D
    } else {
        putb(63);
    }
    if s.len() == 4 {
        putb(76); // L — &str .len() via the STRS table
    } else {
        putb(63);
    }
    if slen("XYZ") == 3 {
        putb(77); // M — &str passed to a function, then .len()
    } else {
        putb(63);
    }
    if "hello".as_bytes()[1] == b'e' {
        putb(69); // E — .as_bytes() on a literal, then index
    } else {
        putb(63);
    }
}
