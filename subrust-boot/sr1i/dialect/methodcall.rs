// B4 dialect corpus: user-defined method calls (impl blocks). The receiver is
// the implicit first argument — passed by address for &self/&mut self on a value
// receiver (RES_MPLACE), or by value otherwise. This is the checker's dominant
// call shape (mem.node(i), chk.rinfo(t), mem.diag(...) — dozens of methods).
// User methods reuse the ordinary call path (eval_call). Output via putb.
#[derive(Clone, Copy)]
struct Counter {
    n: u64,
}
impl Counter {
    fn get(&self) -> u64 {
        self.n // field read through &self
    }
    fn bump(&mut self, by: u64) {
        self.n = self.n + by; // field write through &mut self
    }
    fn plus(&self, k: u64) -> u64 {
        self.n + k
    }
}
fn main() {
    let mut c = Counter { n: 65 };
    putb(c.get()); // A (65) — &self method
    c.bump(1); // n = 66 — &mut self method mutates the receiver
    putb(c.get()); // B (66)
    putb(c.plus(1)); // C (66 + 1) — &self method with an arg
    c.bump(3); // n = 69
    putb(c.get()); // E (69)
    // a method result feeding another expression
    let v = c.plus(21); // 69 + 21 = 90
    putb(v); // Z (90)
}
