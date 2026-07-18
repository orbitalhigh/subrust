
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn nl() { putb(10); }
fn main() {
    let mut s: u64 = 0;
    let mut i: u64 = 1;
    while i <= 10 { s += i; i += 1; }
    put_u64(s); nl();

    let mut n: u64 = 0;
    loop { n += 1; if n == 7 { break; } }
    put_u64(n); nl();

    let mut t: u64 = 0;
    let mut j: u64 = 0;
    while j < 5 {
        j += 1;
        if j == 3 { continue; }
        t += j;
    }
    put_u64(t); nl();

    let mut hits: u64 = 0;
    let mut a: u64 = 0;
    while a < 3 {
        let mut b: u64 = 0;
        while b < 3 {
            if b > a { break; }
            hits += 1;
            b += 1;
        }
        a += 1;
    }
    put_u64(hits); nl();
}
