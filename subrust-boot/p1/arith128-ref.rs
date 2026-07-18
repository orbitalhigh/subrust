// Reference for arith128-core.P1pp: SR-seed's 128-bit (two-slot u64)
// arithmetic core, computed with Rust's checked/wrapping/saturating i128/u128
// ops so its trap semantics ARE rustc's debug profile by construction — the
// exact semantics of the checker's ce_bin128 / wrap_prim128 / sat_prim128 /
// un_op128 (subrust/src/check.rs). Diffing this against the P1pp program's
// stdout is diverse double-execution of the 128-bit core: rustc codegen vs a
// hand-written portable-ISA routine, byte for byte.
//
// Protocol: 40-byte records [op:u64, a_lo, a_hi, b_lo, b_hi] (LE) in;
//           24-byte records [flag:u64, r_lo, r_hi] out. flag==1 => the op
//           traps (overflow / div0 / shift>=128 / i128::MIN neg); on trap the
//           result is 0 (matching ce_bin128's `return 0`), so bytes compare
//           exactly. Bool ops put 0/1 in r_lo, r_hi=0, flag=0.
use std::io::{Read, Write};

fn do_bin128(op: u64, a: u128, b: u128) -> (u64, u128) {
    let sa = a as i128;
    let sb = b as i128;
    match op {
        1 => match a.checked_add(b) { Some(r) => (0, r), None => (1, 0) },
        2 => match a.checked_sub(b) { Some(r) => (0, r), None => (1, 0) },
        3 => match a.checked_mul(b) { Some(r) => (0, r), None => (1, 0) },
        4 => match sa.checked_add(sb) { Some(r) => (0, r as u128), None => (1, 0) },
        5 => match sa.checked_sub(sb) { Some(r) => (0, r as u128), None => (1, 0) },
        6 => match sa.checked_mul(sb) { Some(r) => (0, r as u128), None => (1, 0) },
        7 => if b == 0 { (1, 0) } else { (0, a / b) },
        8 => if b == 0 { (1, 0) } else { (0, a % b) },
        9 => match sa.checked_div(sb) { Some(r) => (0, r as u128), None => (1, 0) },
        10 => match sa.checked_rem(sb) { Some(r) => (0, r as u128), None => (1, 0) },
        11 => (0, a.wrapping_add(b)),
        12 => (0, a.wrapping_sub(b)),
        13 => (0, a.wrapping_mul(b)),
        14 => (0, a.wrapping_neg()),
        15 => if a == (i128::MIN as u128) { (1, 0) } else { (0, sa.wrapping_neg() as u128) },
        16 => (0, !a),
        17 => (0, a & b),
        18 => (0, a | b),
        19 => (0, a ^ b),
        20 => if b >= 128 { (1, 0) } else { (0, a << b) },
        21 => if b >= 128 { (1, 0) } else { (0, a >> b) },
        22 => if b >= 128 { (1, 0) } else { (0, (sa >> b) as u128) },
        23 => (0, a.wrapping_shl((b % 128) as u32)),
        24 => (0, a.rotate_left((b % 128) as u32)),
        25 => (0, a.rotate_right((b % 128) as u32)),
        26 => (0, a.saturating_add(b)),
        27 => (0, a.saturating_mul(b)),
        28 => (0, sa.saturating_add(sb) as u128),
        29 => (0, sa.saturating_mul(sb) as u128),
        30 => (0, (a == b) as u128),
        31 => (0, (a != b) as u128),
        32 => (0, (a < b) as u128),
        33 => (0, (a <= b) as u128),
        34 => (0, (a > b) as u128),
        35 => (0, (a >= b) as u128),
        36 => (0, (sa < sb) as u128),
        37 => (0, (sa <= sb) as u128),
        38 => (0, (sa > sb) as u128),
        39 => (0, (sa >= sb) as u128),
        _ => (1, 0),
    }
}

fn main() {
    let mut inp = Vec::new();
    std::io::stdin().read_to_end(&mut inp).unwrap();
    let mut out = Vec::new();
    let rd = |s: &[u8]| -> u64 { u64::from_le_bytes(s.try_into().unwrap()) };
    let mut i = 0;
    while i + 40 <= inp.len() {
        let op = rd(&inp[i..i + 8]);
        let a = (rd(&inp[i + 8..i + 16]) as u128) | ((rd(&inp[i + 16..i + 24]) as u128) << 64);
        let b = (rd(&inp[i + 24..i + 32]) as u128) | ((rd(&inp[i + 32..i + 40]) as u128) << 64);
        let (flag, r) = do_bin128(op, a, b);
        out.extend_from_slice(&flag.to_le_bytes());
        out.extend_from_slice(&((r & u64::MAX as u128) as u64).to_le_bytes());
        out.extend_from_slice(&((r >> 64) as u64).to_le_bytes());
        i += 40;
    }
    std::io::stdout().write_all(&out).unwrap();
}
