
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn main() {
    let mut x: u64 = 1;
    let mut i: u64 = 0;
    while i < 100000 {
        x = (x * 31 + 7) % 1000000007;
        i += 1;
    }
    put_u64(x);
    putb(10);
}
