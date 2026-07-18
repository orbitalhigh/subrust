// B4 dialect corpus: type-associated path constants (`u32::MAX`, `u8::MAX`,
// `usize::MAX`, …). The checker folds these to a 1-slot value at check time, so
// they emit exactly like an integer literal. check.rs uses `u32::MAX` as a
// sentinel. Output via putb.
fn main() {
    let x: u32 = 4294967295;
    if x == u32::MAX {
        putb(65); // A
    } else {
        putb(63);
    }
    let big: u64 = 18446744073709551615;
    if big == u64::MAX {
        putb(66); // B
    } else {
        putb(63);
    }
    let b: u8 = 200;
    if b < u8::MAX {
        putb(67); // C (200 < 255)
    } else {
        putb(63);
    }
    let n: usize = 5;
    if n < usize::MAX {
        putb(68); // D
    } else {
        putb(63);
    }
}
