// B4 dialect corpus: primitive integer methods — wrapping_add/sub/mul/neg/shl and
// rotate_left/right, all width-aware (mask to the receiver's type). sr0i traps on
// overflow, so sr1i computes these with overflow-avoiding helpers. Saturating and
// f64-bit methods are deferred (they pair with signed arithmetic). Output via putb.
fn main() {
    // wrapping_add overflow at u8: 200 + 100 = 300 -> 44
    let a: u8 = 200;
    if a.wrapping_add(100) == 44 {
        putb(65); // A
    } else {
        putb(63);
    }
    // wrapping_sub underflow at u8: 10 - 20 -> 246
    let c: u8 = 10;
    if c.wrapping_sub(20) == 246 {
        putb(66); // B
    } else {
        putb(63);
    }
    // wrapping_mul at u32: 65536 * 65536 = 2^32 -> 0
    let e: u32 = 65536;
    if e.wrapping_mul(e) == 0 {
        putb(67); // C
    } else {
        putb(63);
    }
    // wrapping_mul at u64 (schoolbook path): 2^33 * 2^33 = 2^66 -> 0 mod 2^64
    let f: u64 = 8589934592; // 2^33
    if f.wrapping_mul(f) == 0 {
        putb(68); // D
    } else {
        putb(63);
    }
    // wrapping_neg at u8: -5 -> 251
    let g: u8 = 5;
    if g.wrapping_neg() == 251 {
        putb(69); // E
    } else {
        putb(63);
    }
    // wrapping_shl at u8: shift 9 is masked to 9 % 8 = 1, so 1 << 1 = 2
    let h: u8 = 1;
    if h.wrapping_shl(9) == 2 {
        putb(70); // F
    } else {
        putb(63);
    }
    // rotate_left at u8: 0b1000_0001 <<< 1 = 0b0000_0011 = 3
    let i: u8 = 129;
    if i.rotate_left(1) == 3 {
        putb(71); // G
    } else {
        putb(63);
    }
    // rotate_right at u8: 0b0000_0011 >>> 1 = 0b1000_0001 = 129
    let j: u8 = 3;
    if j.rotate_right(1) == 129 {
        putb(72); // H
    } else {
        putb(63);
    }
}
