// Reference for arith-core.P1pp verification: the
// SAME 24-byte-record protocol, computed with Rust checked/wrapping ops so
// its trap semantics are rustc's debug profile by construction. Diffing this
// against the P1pp program's stdout is diverse double-execution of SR-seed's
// u64 arithmetic core: two independent implementations (rustc codegen vs a
// hand-written P1pp/portable-ISA routine) must agree byte for byte.
use std::io::{Read, Write};

fn do_bin(op: u64, a: u64, b: u64) -> (u64, u64) {
    // returns (flag, result); flag==1 means the op traps (rustc debug panic)
    let r: Option<u64> = match op {
        1 => a.checked_add(b),
        2 => a.checked_sub(b),
        3 => a.checked_mul(b),
        4 => a.checked_div(b),
        5 => a.checked_rem(b),
        8 => Some(a & b),
        9 => Some(a | b),
        10 => Some(a ^ b),
        11 => if b >= 64 { None } else { Some(a << b) },
        12 => if b >= 64 { None } else { Some(a >> b) },
        13 => Some((a == b) as u64),
        14 => Some((a != b) as u64),
        15 => Some((a < b) as u64),
        16 => Some((a <= b) as u64),
        17 => Some((a > b) as u64),
        18 => Some((a >= b) as u64),
        _ => None,
    };
    match r {
        Some(v) => (0, v),
        None => (1, 0),
    }
}

fn main() {
    let mut inp = Vec::new();
    std::io::stdin().read_to_end(&mut inp).unwrap();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 24 <= inp.len() {
        let rd = |o: usize| -> u64 {
            let mut w = [0u8; 8];
            w.copy_from_slice(&inp[i + o..i + o + 8]);
            u64::from_le_bytes(w)
        };
        let (flag, res) = do_bin(rd(0), rd(8), rd(16));
        out.extend_from_slice(&flag.to_le_bytes());
        out.extend_from_slice(&res.to_le_bytes());
        i += 24;
    }
    std::io::stdout().write_all(&out).unwrap();
}
