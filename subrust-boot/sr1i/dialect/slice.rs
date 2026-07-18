// B4 dialect corpus: slices — &arr[lo..hi], &arr[lo..], &arr[0..] (whole),
// slice .len() (usize), slice indexing, and passing a slice fat pointer
// [addr,len] to a function (2-slot param + read through it). Output via putb.
fn head(s: &[u64]) -> u64 {
    s[0]
}
fn len_is_3(s: &[u64]) -> u64 {
    // exercises slice .len() (usize) in a usize comparison; returns B or X
    if s.len() == 3 {
        66
    } else {
        88
    }
}
fn main() {
    let a = [65u64, 66, 67, 68, 69]; // A B C D E
    let s = &a[1..4]; // [66,67,68] = B C D
    putb(s[0]); // B
    putb(s[1]); // C
    putb(s[2]); // D
    putb(len_is_3(s)); // B — .len() through a fn-param slice
    let t = &a[2..]; // open-ended: [67,68,69] = C D E
    putb(t[0]); // C
    putb(head(t)); // C — index through a fn-param slice
    putb(len_is_3(t)); // B
    let full = &a[0..]; // whole array as a slice
    putb(full[4]); // E
    if full.len() == 5 {
        putb(65); // A
    } else {
        putb(88);
    }
}
