
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn nl() { putb(10); }
fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
fn pw(b: u64, e: u64) -> u64 {
    if e == 0 { 1 } else { b * pw(b, e - 1) }
}
fn main() {
    put_u64(fib(20)); nl();
    put_u64(gcd(1071, 462)); nl();
    put_u64(pw(3, 7)); nl();
}
