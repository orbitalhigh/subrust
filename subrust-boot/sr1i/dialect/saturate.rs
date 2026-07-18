// B4 dialect corpus: unsigned saturating_add / saturating_mul, clamped to the
// type's max. sr0i traps on overflow, so sr1i detects the overflow and clamps
// instead. The checker uses `size_of(elem).saturating_mul(len)` to bound array
// sizes. (Signed saturating + f64 bit methods remain deferred.) Output via putb.
fn main() {
    // u8 saturating_add overflow -> 255
    let a: u8 = 200;
    if a.saturating_add(100) == 255 {
        putb(65); // A
    } else {
        putb(63);
    }
    // u32 saturating_mul overflow -> u32::MAX
    let c: u32 = 100000;
    if c.saturating_mul(100000) == 4294967295 {
        putb(66); // B (10^10 clamps)
    } else {
        putb(63);
    }
    // u64 saturating_add overflow -> u64::MAX
    let d: u64 = 18446744073709551610;
    if d.saturating_add(100) == 18446744073709551615 {
        putb(67); // C
    } else {
        putb(63);
    }
    // in-range saturating_add (no clamp): 60 + 8 = 68
    let e: u8 = 60;
    if e.saturating_add(8) == 68 {
        putb(68); // D
    } else {
        putb(63);
    }
    // u64 saturating_mul overflow -> u64::MAX
    let f: u64 = 10000000000;
    if f.saturating_mul(10000000000) == 18446744073709551615 {
        putb(69); // E (10^20 clamps)
    } else {
        putb(63);
    }
}
