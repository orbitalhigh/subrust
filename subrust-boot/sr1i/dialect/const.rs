// B4 dialect corpus: module consts — the #1 blocker found by the B4 probe (the
// self-hosted checker is saturated with `const`). Consts are fully resolved at
// check time: a scalar const lowers to its literal value; an aggregate (array/
// struct) const lowers to a multi-slot push from the val table. Also exercises a
// const used as a match pattern (N_PAT_CONST). Output via putb.
const LETTER_A: u64 = 65;
const OFFSET: u64 = 1;
const TABLE: [u64; 3] = [67, 68, 69];
const MODE: u64 = 2;

fn main() {
    // scalar const reference
    putb(LETTER_A); // A (65)
    // scalar const in arithmetic
    putb(LETTER_A + OFFSET); // B (66)
    // aggregate (array) const, indexed by a runtime value (forces a KCONST push)
    let i: usize = 0;
    putb(TABLE[i]); // C (TABLE[0] = 67)
    // a const as a match pattern (N_PAT_CONST)
    let m: u64 = 2;
    match m {
        MODE => putb(68), // D
        _ => putb(63),
    }
    let j: usize = 2;
    putb(TABLE[j]); // E (TABLE[2] = 69)
}
