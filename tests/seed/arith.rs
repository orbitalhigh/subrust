
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn nl() { putb(10); }
fn main() {
    put_u64(7 / 2); nl();
    put_u64(7 % 2); nl();
    put_u64(1 + 2 << 3); nl();
    put_u64(255 & 15 | 48 ^ 1); nl();
    put_u64(2 * 3 + 4 * 5); nl();
    put_u64((1 << 63) >> 63); nl();
    put_u64(18446744073709551615 % 10); nl();
}
