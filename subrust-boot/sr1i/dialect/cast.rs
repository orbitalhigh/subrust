// B4 dialect corpus: `as` integer casts (mask_to semantics) — widening,
// narrowing (truncate low bits), same-width identity, and signed narrowing with
// sign-extension. f64/128-bit casts are not SR-seed. This also unblocks the
// natural `putb(byte as u64)` form (putb wants u64, byte-slice elems are u8).
fn main() {
    // widening u8 -> u64: index a byte string and print the element directly
    let s = b"A";
    putb(s[0] as u64); // A (65)
    // narrowing u32 -> u8 (low 8 bits), then widen for putb
    let n: u32 = 322; // 256 + 66
    putb((n as u8) as u64); // B (66)
    // i32 -> u8 narrowing (unsigned low byte)
    let m: i32 = 323; // 256 + 67
    putb((m as u8) as u64); // C (67)
    // usize -> u64 (same width, identity)
    let u: usize = 68;
    putb(u as u64); // D (68)
    // signed narrowing sign-extension: 200 as i8 = -56, bits 0xFFFF_FFFF_FFFF_FFC8
    let k: i32 = 200;
    let bits: u64 = 0xFFFF_FFFF_FFFF_FFC8;
    if ((k as i8) as i64) as u64 == bits {
        putb(69); // E — sign extension correct
    } else {
        putb(63);
    }
}
