
use std::io::Write as _;
use std::process::{Command, Stdio};

use subrust::apis::{BOOT_API, BOOT_WORDS};
use subrust::machine::{Instance, INSTANCE_INIT};
use subrust::platform::{Platform, SrErr, SR_OK};
use subrust::{CHK_INIT, MEM_INIT};

/// The BOOT_API host: flat word memory, byte input, byte output.
pub struct BootHost {
    pub mem: Vec<u64>,
    pub inp: Vec<u8>,
    pub pos: usize,
    pub out: Vec<u8>,
}

impl BootHost {
    pub fn new(input: &[u8]) -> BootHost {
        BootHost {
            mem: vec![0u64; BOOT_WORDS],
            inp: input.to_vec(),
            pos: 0,
            out: Vec::new(),
        }
    }
}

impl Platform for BootHost {
    fn host_call(&mut self, id: u16, args: &[u64], ret: &mut [u64]) -> SrErr {
        match id {
            0 => {
                let a = args[0] as usize;
                if a >= BOOT_WORDS {
                    return 1;
                }
                ret[0] = self.mem[a];
            }
            1 => {
                let a = args[0] as usize;
                if a >= BOOT_WORDS {
                    return 1;
                }
                self.mem[a] = args[1];
            }
            2 => {
                ret[0] = if self.pos < self.inp.len() {
                    let b = self.inp[self.pos] as u64;
                    self.pos += 1;
                    b
                } else {
                    u64::MAX
                };
            }
            3 => self.out.push(args[0] as u8),
            4 => ret[0] = (f64::from_bits(args[0]) + f64::from_bits(args[1])).to_bits(),
            5 => ret[0] = (f64::from_bits(args[0]) - f64::from_bits(args[1])).to_bits(),
            6 => ret[0] = (f64::from_bits(args[0]) * f64::from_bits(args[1])).to_bits(),
            7 => ret[0] = (f64::from_bits(args[0]) / f64::from_bits(args[1])).to_bits(),
            8 => ret[0] = (f64::from_bits(args[0]) % f64::from_bits(args[1])).to_bits(),
            9 => ret[0] = (f64::from_bits(args[0]) < f64::from_bits(args[1])) as u64,
            10 => ret[0] = (f64::from_bits(args[0]) == f64::from_bits(args[1])) as u64,
            11 => ret[0] = ((args[0] as i64) as f64).to_bits(),
            12 => ret[0] = ((f64::from_bits(args[0]) as i64) as u64),
            _ => return 1,
        }
        SR_OK
    }
}

/// The same 13 functions for rustc-compiled runs. Must match BootHost
/// exactly; putb flushes per byte so trap prefixes compare.
pub const BOOT_SHIMS: &str = r#"
// ---- BOOT_API shims (see subrust-boot/SR-SEED.md) ----
use std::io::{Read as _, Write as _};
use std::sync::{LazyLock, Mutex};
static MEM: LazyLock<Mutex<Vec<u64>>> = LazyLock::new(|| Mutex::new(vec![0u64; 1 << 20]));
static INP: LazyLock<Mutex<(Vec<u8>, usize)>> = LazyLock::new(|| {
    let mut v = Vec::new();
    std::io::stdin().read_to_end(&mut v).unwrap();
    Mutex::new((v, 0))
});
fn ld(a: u64) -> u64 { MEM.lock().unwrap()[a as usize] }
fn st(a: u64, v: u64) { MEM.lock().unwrap()[a as usize] = v; }
fn getb() -> u64 {
    let mut g = INP.lock().unwrap();
    if g.1 < g.0.len() { let b = g.0[g.1] as u64; g.1 += 1; b } else { u64::MAX }
}
fn putb(b: u64) {
    let mut o = std::io::stdout();
    o.write_all(&[b as u8]).unwrap();
    o.flush().unwrap();
}
fn f_add(a: u64, b: u64) -> u64 { (f64::from_bits(a) + f64::from_bits(b)).to_bits() }
fn f_sub(a: u64, b: u64) -> u64 { (f64::from_bits(a) - f64::from_bits(b)).to_bits() }
fn f_mul(a: u64, b: u64) -> u64 { (f64::from_bits(a) * f64::from_bits(b)).to_bits() }
fn f_div(a: u64, b: u64) -> u64 { (f64::from_bits(a) / f64::from_bits(b)).to_bits() }
fn f_rem(a: u64, b: u64) -> u64 { (f64::from_bits(a) % f64::from_bits(b)).to_bits() }
fn f_lt(a: u64, b: u64) -> bool { f64::from_bits(a) < f64::from_bits(b) }
fn f_eq(a: u64, b: u64) -> bool { f64::from_bits(a) == f64::from_bits(b) }
fn f_from_i(a: u64) -> u64 { ((a as i64) as f64).to_bits() }
fn f_to_i(a: u64) -> u64 { (f64::from_bits(a) as i64) as u64 }
"#;

/// Interpret main() under subrust+BOOT_API. Returns (output, trapped).
pub fn interp_boot(src: &str, input: &[u8]) -> (Vec<u8>, bool) {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(src, &mut mem, &mut chk, &BOOT_API);
    if !ok {
        let d = if mem.diag_n > 0 { mem.diags[0] } else { subrust::diag::DIAG_NONE };
        panic!(
            "SR-seed program failed to check: {:#06x} at {}..{}\n----\n{src}",
            d.code, d.span.lo, d.span.hi
        );
    }
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = BootHost::new(input);
    let e = subrust::call(src, &mem, &chk, &mut inst, &mut host, "main", &[], 500_000_000);
    (host.out, e != SR_OK)
}

/// Compile with rustc (parity flags) and run with `input` piped to stdin.
/// Returns (stdout bytes, panicked).
pub fn compiled_boot(name: &str, src: &str, input: &[u8]) -> (Vec<u8>, bool) {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let rs = dir.join(format!("boot_{name}.rs"));
    let bin = dir.join(format!("boot_{name}.bin"));
    std::fs::write(&rs, format!("{src}\n{BOOT_SHIMS}")).expect("write");
    let out = Command::new("rustc")
        .args([
            "--edition",
            "2021",
            "-O",
            "-C",
            "overflow-checks=on",
            "-C",
            "debug-assertions=on",
            "-A",
            "warnings",
            "-A",
            "arithmetic_overflow",
            "-A",
            "unconditional_panic",
        ])
        .arg(&rs)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("rustc");
    assert!(
        out.status.success(),
        "rustc rejected {name} (L1 violation!):\n{}\n----\n{src}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut child = Command::new(&bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn");
    child.stdin.take().unwrap().write_all(input).expect("stdin");
    let run = child.wait_with_output().expect("run");
    (run.stdout, !run.status.success())
}

/// The differential assertion for one SR-seed program.
pub fn boot_diff(name: &str, src: &str, input: &[u8]) -> (Vec<u8>, bool) {
    let (iout, itrap) = interp_boot(src, input);
    let (cout, ctrap) = compiled_boot(name, src, input);
    assert_eq!(
        iout, cout,
        "stdout diverged on {name}\n interp: {:?}\n rustc:  {:?}",
        String::from_utf8_lossy(&iout),
        String::from_utf8_lossy(&cout)
    );
    assert_eq!(itrap, ctrap, "trap outcome diverged on {name}");
    (iout, itrap)
}
