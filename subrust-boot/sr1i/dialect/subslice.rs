// B4 dialect corpus: sub-slicing a slice, `&s[lo..hi]` / `&s[lo..]`. The base is
// a fat pointer [addr, len] (dynamic len read at runtime), and the result is
// [addr + lo*es, hi - lo]. The checker sub-slices its byte buffer constantly
// (`&b[start..i]` when scanning tokens). Also covers POOL_TAG byte-string
// sub-slices (the offset stays tagged). Output via putb.
fn slice_get(s: &[u64], i: usize) -> u64 {
    let sub = &s[1..]; // open-ended sub-slice of a slice parameter
    sub[i]
}
fn main() {
    let a = [65u64, 66, 67, 68, 69]; // A B C D E
    let s = &a[0..5]; // full slice
    let mid = &s[1..4]; // sub-slice of a slice: [66, 67, 68]
    putb(mid[0]); // B (66)
    putb(mid[2]); // D (68)
    let tail = &s[2..]; // open-ended sub-slice: [67, 68, 69]
    putb(tail[0]); // C (67)
    putb(slice_get(s, 3)); // (&s[1..])[3] = s[4] = 69 (E)
    // byte-string (POOL_TAG) sub-slice: the tagged offset advances by lo
    let bytes = b"pqA";
    let bsub = &bytes[2..3]; // ['A']
    if bsub[0] == b'A' {
        putb(65); // A
    } else {
        putb(63);
    }
}
