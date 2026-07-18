
fn put_u64(v: u64) {
    if v >= 10 { put_u64(v / 10); }
    putb(48 + v % 10);
}
fn nl() { putb(10); }
fn putbool(b: bool) { if b { putb(84); } else { putb(70); } }
fn main() {
    put_u64(f_add(f_from_i(1), f_div(f_from_i(1), f_from_i(2)))); nl();
    put_u64(f_to_i(f_mul(f_from_i(3), f_from_i(4)))); nl();
    putbool(f_lt(f_div(f_from_i(0), f_from_i(0)), f_from_i(1)));
    putbool(f_eq(f_div(f_from_i(0), f_from_i(0)), f_div(f_from_i(0), f_from_i(0))));
    nl();
    put_u64(f_to_i(f_div(f_from_i(1), f_from_i(0)))); nl();
    put_u64(f_to_i(f_sub(f_from_i(0), f_div(f_from_i(1), f_from_i(0))))); nl();
    put_u64(f_rem(f_from_i(7), f_from_i(2))); nl();
    put_u64(f_from_i(1) ^ (1 << 63)); nl();
}
