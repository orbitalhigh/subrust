
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn nl() { putb(10); }
fn main() {
    st(0, 0);
    st(1, 1);
    let mut i: u64 = 2;
    while i <= 92 {
        st(i, ld(i - 1) + ld(i - 2));
        i += 1;
    }
    put_u64(ld(90)); nl();
    let mut x: u64 = 0;
    let mut j: u64 = 0;
    while j <= 92 {
        x ^= ld(j);
        j += 1;
    }
    put_u64(x); nl();
}
