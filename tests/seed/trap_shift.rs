
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn main() {
    putb(83);
    let s: u64 = 64;
    put_u64(1 << s);
}
