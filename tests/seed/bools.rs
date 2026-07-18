
fn putbool(b: bool) { if b { putb(84); } else { putb(70); } }
fn t() -> bool { putb(116); true }
fn f() -> bool { putb(102); false }
fn main() {
    putbool(1 < 2);
    putbool(2 >= 3);
    putbool(f() && t());
    putbool(t() || f());
    putbool(true ^ false);
    putbool(true & false);
    putbool(!false);
    putb(10);
}
