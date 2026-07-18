// rustc reference for the f_* IEEE-f64 intrinsics — the exact BOOT_SHIMS
// bodies from subrust/tests/common/mod.rs. Reads the same 17-byte-record
// protocol as f_amd64.s and writes 8-byte-LE results, so the two are
// differentially comparable byte-for-byte.
use std::io::{Read, Write};

fn f_add(a: u64, b: u64) -> u64 { (f64::from_bits(a) + f64::from_bits(b)).to_bits() }
fn f_sub(a: u64, b: u64) -> u64 { (f64::from_bits(a) - f64::from_bits(b)).to_bits() }
fn f_mul(a: u64, b: u64) -> u64 { (f64::from_bits(a) * f64::from_bits(b)).to_bits() }
fn f_div(a: u64, b: u64) -> u64 { (f64::from_bits(a) / f64::from_bits(b)).to_bits() }
fn f_rem(a: u64, b: u64) -> u64 { (f64::from_bits(a) % f64::from_bits(b)).to_bits() }
fn f_lt(a: u64, b: u64) -> bool { f64::from_bits(a) < f64::from_bits(b) }
fn f_eq(a: u64, b: u64) -> bool { f64::from_bits(a) == f64::from_bits(b) }
fn f_from_i(a: u64) -> u64 { ((a as i64) as f64).to_bits() }
fn f_to_i(a: u64) -> u64 { (f64::from_bits(a) as i64) as u64 }

fn main() {
    let mut inp = Vec::new();
    std::io::stdin().read_to_end(&mut inp).unwrap();
    let out = std::io::stdout();
    let mut o = out.lock();
    let mut i = 0;
    while i + 17 <= inp.len() {
        let op = inp[i];
        let a = u64::from_le_bytes(inp[i + 1..i + 9].try_into().unwrap());
        let b = u64::from_le_bytes(inp[i + 9..i + 17].try_into().unwrap());
        let r = match op {
            0 => f_add(a, b),
            1 => f_sub(a, b),
            2 => f_mul(a, b),
            3 => f_div(a, b),
            4 => f_rem(a, b),
            5 => f_lt(a, b) as u64,
            6 => f_eq(a, b) as u64,
            7 => f_from_i(a),
            8 => f_to_i(a),
            _ => 0,
        };
        o.write_all(&r.to_le_bytes()).unwrap();
        i += 17;
    }
}
