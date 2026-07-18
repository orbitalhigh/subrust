
fn main() {
    let mut b: u64 = getb();
    while b != 18446744073709551615 {
        putb(b);
        b = getb();
    }
}
