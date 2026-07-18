
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn main() {
    putb(68);
    let z: u64 = 0;
    put_u64(10 / z);
}
