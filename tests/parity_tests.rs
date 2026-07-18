
use std::process::Command;

use subrust::apis::TEST_API;
use subrust::machine::{Instance, INSTANCE_INIT};
use subrust::platform::{Platform, SrErr, SR_OK};
use subrust::{Chk, CHK_INIT, MEM_INIT};

const SHIMS: &str = "
// ---- parity shims: the TEST_API host, compiled ----
fn print_i64(v: i64) { println!(\"{v}\") }
fn print_u64(v: u64) { println!(\"{v}\") }
fn print_usize(v: usize) { println!(\"{v}\") }
fn print_f64(v: f64) { println!(\"{v}\") }
fn print_bool(v: bool) { println!(\"{v}\") }
fn print_str(v: &str) { println!(\"{v}\") }
";

struct PrintHost<'a> {
    chk: &'a Chk,
    out: String,
}

impl<'a> Platform for PrintHost<'a> {
    fn host_call(&mut self, id: u16, args: &[u64], _ret: &mut [u64]) -> SrErr {
        match id {
            0 => self.out.push_str(&format!("{}\n", args[0] as i64)),
            1 | 2 => self.out.push_str(&format!("{}\n", args[0])),
            3 => self.out.push_str(&format!("{}\n", f64::from_bits(args[0]))),
            4 => self.out.push_str(&format!("{}\n", args[0] != 0)),
            5 => {
                let b = self.chk.str_bytes(args[0] as u32);
                self.out.push_str(std::str::from_utf8(b).unwrap_or("?"));
                self.out.push('\n');
            }
            _ => return 1,
        }
        SR_OK
    }
}

/// Interpret main(); returns (stdout, trapped).
fn interp(src: &str) -> (String, bool) {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(src, &mut mem, &mut chk, &TEST_API);
    if !ok && mem.diag_n > 0 {
        let d = mem.diags[0];
        panic!(
            "check failed {:#06x} at {}..{} in:\n{src}",
            d.code, d.span.lo, d.span.hi
        );
    }
    assert!(ok);
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost {
        chk: &chk,
        out: String::new(),
    };
    let e = subrust::call(src, &mem, &chk, &mut inst, &mut host, "main", &[], 10_000_000);
    (host.out, e != SR_OK)
}

/// Compile with rustc (debug-profile semantics) and run; (stdout, panicked).
fn compiled(name: &str, src: &str) -> (String, bool) {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let rs = dir.join(format!("parity_{name}.rs"));
    let bin = dir.join(format!("parity_{name}.bin"));
    let full = format!("{src}\n{SHIMS}");
    std::fs::write(&rs, full).expect("write case");
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
        .expect("run rustc");
    assert!(
        out.status.success(),
        "rustc rejected {name} (L1 violation!):\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&bin).output().expect("run case");
    (
        String::from_utf8_lossy(&run.stdout).to_string(),
        !run.status.success(),
    )
}

/// The parity assertion: identical stdout, identical trap/panic outcome.
fn parity(name: &str, src: &str) {
    let (iout, itrap) = interp(src);
    let (cout, ctrap) = compiled(name, src);
    assert_eq!(iout, cout, "stdout diverged on {name}");
    assert_eq!(itrap, ctrap, "trap outcome diverged on {name}");
}

#[test]
fn parity_lexer_suffixes_ops() {

    parity(
        "lexer_suffixes_ops",
        r#"
fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut i: usize = 0;
    while i < a.len() { if a[i] != b[i] { return false; } i += 1; }
    true
}
fn is_alpha(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') }
fn is_digit(c: u8) -> bool { c >= b'0' && c <= b'9' }
fn suffix_kind(s: &[u8]) -> u16 {
    if bytes_eq(s, b"i8") || bytes_eq(s, b"u8") || bytes_eq(s, b"i32")
        || bytes_eq(s, b"u32") || bytes_eq(s, b"i64") || bytes_eq(s, b"u64")
        || bytes_eq(s, b"usize") { return 2; }
    if bytes_eq(s, b"f64") { return 3; }
    if s.len() == 0 { return 2; }
    255
}
fn op3(a: u8, b: u8, c: u8) -> u16 {
    if a == b'<' && b == b'<' && c == b'=' { return 100; }
    if a == b'>' && b == b'>' && c == b'=' { return 101; }
    if a == b'.' && b == b'.' && c == b'=' { return 102; }
    0
}
fn main() {
    print_i64(suffix_kind(b"i64") as i64);
    print_i64(suffix_kind(b"f64") as i64);
    print_i64(suffix_kind(b"") as i64);
    print_i64(suffix_kind(b"q9") as i64);
    print_i64(op3(b'<', b'<', b'=') as i64);
    print_i64(op3(b'.', b'.', b'=') as i64);
    print_i64(op3(b'+', b'+', b'+') as i64);
    let src = [52u8, 50, 105, 54, 52];
    let mut j: usize = 0;
    while j < 5 && is_digit(src[j]) { j += 1; }
    let mut e: usize = j;
    while e < 5 && (is_alpha(src[e]) || is_digit(src[e])) { e += 1; }
    print_i64(suffix_kind(&src[j..e]) as i64);
}
"#,
    );
}

#[test]
fn parity_byte_patterns() {

    parity(
        "byte_patterns",
        r#"
fn punct(c: u8) -> u16 {
    match c {
        b'(' => 40,
        b')' => 41,
        b'{' | b'}' => 42,
        b'+' | b'-' | b'*' | b'/' => 60,
        b'\n' => 99,
        b'\\' => 92,
        _ => 255,
    }
}
fn main() {
    print_i64(punct(b'(') as i64);
    print_i64(punct(b'}') as i64);
    print_i64(punct(b'*') as i64);
    print_i64(punct(b'\n') as i64);
    print_i64(punct(b'\\') as i64);
    print_i64(punct(b'z') as i64);
}
"#,
    );
}

#[test]
fn parity_lexer_strings() {

    parity(
        "lexer_strings",
        r#"
const T_IDENT: u16 = 1;
const T_INT: u16 = 2;
const T_STR: u16 = 3;
const T_KW_FN: u16 = 10;
const CAP: usize = 256;
#[derive(Clone, Copy)]
struct Tok { kind: u16, pos: u32, len: u32 }
#[derive(Clone, Copy)]
struct Lex { toks: [Tok; CAP], n: usize, err: u16 }
impl Lex {
    fn push(&mut self, kind: u16, pos: usize, end: usize) {
        if self.n < CAP {
            self.toks[self.n] = Tok { kind: kind, pos: pos as u32, len: (end - pos) as u32 };
            self.n += 1;
        }
    }
    fn fail(&mut self, code: u16) { if self.err == 0 { self.err = code; } }
}
fn is_digit(c: u8) -> bool { c >= b'0' && c <= b'9' }
fn is_alpha(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') }
fn is_ident_start(c: u8) -> bool { c == b'_' || is_alpha(c) }
fn is_ident_char(c: u8) -> bool { c == b'_' || is_alpha(c) || is_digit(c) }
fn punct(c: u8) -> u16 {
    match c { b'(' => 40, b')' => 41, b'{' | b'}' => 42, b';' => 44, b',' => 45,
              b'+' | b'-' | b'*' | b'/' => 60, _ => 255 }
}
fn lex(src: &[u8], lx: &mut Lex) {
    let n: usize = src.len();
    let mut i: usize = 0;
    while i < n {
        let c = src[i];
        if c == b' ' || c == b'\n' || c == b'\t' { i += 1; continue; }
        if c == b'/' && i + 1 < n && src[i + 1] == b'*' {
            i += 2;
            let mut depth: u16 = 1;
            while i < n && depth > 0 {
                if i + 1 < n && src[i] == b'/' && src[i + 1] == b'*' { depth += 1; i += 2; }
                else if i + 1 < n && src[i] == b'*' && src[i + 1] == b'/' { depth -= 1; i += 2; }
                else { i += 1; }
            }
            if depth > 0 { lx.fail(1); }
            continue;
        }
        let start: usize = i;
        if c == b'"' {
            i += 1;
            while i < n && src[i] != b'"' {
                if src[i] == b'\\' && i + 1 < n { i += 2; } else { i += 1; }
            }
            if i < n { i += 1; } else { lx.fail(2); }
            lx.push(T_STR, start, i);
            continue;
        }
        if is_ident_start(c) {
            i += 1; while i < n && is_ident_char(src[i]) { i += 1; }
            let k = if src[start] == b'f' && i - start == 2 && src[start + 1] == b'n' { T_KW_FN } else { T_IDENT };
            lx.push(k, start, i);
            continue;
        }
        if is_digit(c) {
            while i < n && is_digit(src[i]) { i += 1; }
            lx.push(T_INT, start, i);
            continue;
        }
        i += 1;
        lx.push(punct(c), start, i);
    }
}
fn main() {
    let s = [
        102u8, 110, 32, 102, 40, 41, 32, 123, 32, 47, 42, 32, 99, 32, 47, 42, 32,
        110, 32, 42, 47, 32, 42, 47, 32, 34, 97, 92, 34, 98, 34, 32, 43, 32, 52, 50, 32, 125,
    ];
    let mut lx = Lex { toks: [Tok { kind: 0, pos: 0, len: 0 }; CAP], n: 0, err: 0 };
    lex(&s, &mut lx);
    print_usize(lx.n); print_i64(lx.err as i64);
    let mut i: usize = 0;
    while i < lx.n {
        let t = lx.toks[i];
        print_i64(t.kind as i64); print_i64(t.pos as i64); print_i64(t.len as i64);
        i += 1;
    }
}
"#,
    );
}

#[test]
fn parity_mini_lexer() {

    parity(
        "mini_lexer",
        r#"
const T_IDENT: u16 = 1;
const T_INT: u16 = 2;
const T_KW_FN: u16 = 10;
const T_KW_LET: u16 = 11;
const CAP: usize = 256;
#[derive(Clone, Copy)]
struct Tok { kind: u16, pos: u32, len: u32 }
#[derive(Clone, Copy)]
struct Lex { toks: [Tok; CAP], n: usize }
impl Lex {
    fn push(&mut self, kind: u16, pos: usize, end: usize) {
        if self.n < CAP {
            self.toks[self.n] = Tok { kind: kind, pos: pos as u32, len: (end - pos) as u32 };
            self.n += 1;
        }
    }
}
fn is_digit(c: u8) -> bool { c >= b'0' && c <= b'9' }
fn is_alpha(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') }
fn is_ident_start(c: u8) -> bool { c == b'_' || is_alpha(c) }
fn is_ident_char(c: u8) -> bool { c == b'_' || is_alpha(c) || is_digit(c) }
fn is_hex(c: u8) -> bool { is_digit(c) || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F') }
fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut i: usize = 0;
    while i < a.len() { if a[i] != b[i] { return false; } i += 1; }
    true
}
fn kw_lookup(w: &[u8]) -> u16 {
    if bytes_eq(w, b"fn") { return T_KW_FN; }
    if bytes_eq(w, b"let") { return T_KW_LET; }
    T_IDENT
}
fn lex(src: &[u8], lx: &mut Lex) {
    let n: usize = src.len();
    let mut i: usize = 0;
    while i < n {
        let c = src[i];
        if c == b' ' || c == b'\n' || c == b'\t' { i += 1; continue; }
        if c == b'/' && i + 1 < n && src[i + 1] == b'/' {
            i += 2; while i < n && src[i] != b'\n' { i += 1; } continue;
        }
        let start: usize = i;
        if is_ident_start(c) {
            i += 1; while i < n && is_ident_char(src[i]) { i += 1; }
            lx.push(kw_lookup(&src[start..i]), start, i); continue;
        }
        if is_digit(c) {
            if c == b'0' && i + 1 < n && src[i + 1] == b'x' {
                i += 2; while i < n && (is_hex(src[i]) || src[i] == b'_') { i += 1; }
            } else {
                while i < n && (is_digit(src[i]) || src[i] == b'_') { i += 1; }
            }
            lx.push(T_INT, start, i); continue;
        }
        let c1 = if i + 1 < n { src[i + 1] } else { 0 };
        if c == b'=' && c1 == b'=' { i += 2; lx.push(47, start, i); continue; }
        if c == b'<' && c1 == b'=' { i += 2; lx.push(52, start, i); continue; }
        i += 1;
        lx.push(c as u16, start, i);
    }
    lx.push(0, n, n);
}
fn main() {
    let s = [
        102u8, 110, 32, 97, 100, 100, 40, 97, 41, 32, 123, 32, 108, 101, 116, 32,
        120, 32, 61, 32, 48, 120, 70, 70, 59, 32, 114, 101, 116, 117, 114, 110, 32,
        97, 32, 61, 61, 32, 120, 32, 60, 61, 32, 49, 32, 125, 32, 47, 47, 32, 100,
        111, 110, 101,
    ];
    let mut lx = Lex { toks: [Tok { kind: 0, pos: 0, len: 0 }; CAP], n: 0 };
    lex(&s, &mut lx);
    print_usize(lx.n);
    let mut i: usize = 0;
    while i < lx.n {
        let t = lx.toks[i];
        print_i64(t.kind as i64); print_i64(t.pos as i64); print_i64(t.len as i64);
        i += 1;
    }
}
"#,
    );
}

#[test]
fn parity_integers() {
    parity(
        "integers",
        r#"
fn main() {
    print_i64(2 + 3 * 4 - 5);
    print_i64(7 / 2); print_i64(-7 / 2); print_i64(7 % 2); print_i64(-7 % 2);
    print_i64(1 + 2 << 3);
    print_u64(0xFF_u64 & 0x0F | 0x30 ^ 0x01);
    print_i64(-9223372036854775808);
    print_i64(-(-1));
    let x: u8 = 200; print_i64((x / 3) as i64);
    let s = 3u32; print_i64(1i64 << s);
    print_i64(!0i64); print_u64(!0u64);
    print_bool(1 < 2); print_bool(2u8 >= 3u8);
    let big: i64 = 9223372036854775807; print_i64(big);
}
"#,
    );
}

#[test]
fn parity_floats() {
    parity(
        "floats",
        r#"
fn main() {
    print_f64(0.1 + 0.2);
    print_f64(1.0 / 3.0);
    print_f64(1.0 / 0.0);
    print_f64(-1.0 / 0.0);
    print_f64(1e300 * 1e300);
    print_f64(2.5e-2);
    print_f64(-0.0);
    print_f64(36.0);
    print_bool(0.0 / 0.0 == 0.0 / 0.0);
    let t = 30.0; let target = 36.0;
    print_bool(t < target - 0.25);
    print_f64(1f64);
}
"#,
    );
}

#[test]
fn parity_casts() {
    parity(
        "casts",
        r#"
fn main() {
    print_i64(300i64 as u8 as i64);
    print_i64((-1i64) as u8 as i64);
    print_i64(1e300 as i64);
    print_u64((-1.5) as u64);
    print_i64(-1.5 as i64);
    print_i64((0.0 / 0.0) as i64);
    print_f64(true as i64 as f64);
    print_f64(-1i64 as f64);
    print_u64(18446744073709551615u64);
    print_f64(18446744073709551615u64 as f64);
    print_i64(255u8 as i8 as i64);
    print_usize(40000 as usize);
}
"#,
    );
}

#[test]
fn parity_control_flow() {
    parity(
        "control",
        r#"
fn collatz(mut n: i64) -> i64 {
    let mut steps = 0i64;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}
fn main() {
    print_i64(collatz(27));
    let mut s = 0i64;
    for i in 1i64..=10 { s += i; }
    print_i64(s);
    let mut c = 0i64;
    for i in 250u8..=255 { c += 1; }
    print_i64(c);
    for i in 5i64..2 { print_str("never"); }
    let mut n = 0i64;
    loop { n += 1; if n > 3 { break; } }
    print_i64(n);
    for i in 0i64..5 {
        if i == 2 { continue; }
        print_i64(i);
    }
    match 42i64 { 0 => print_str("zero"), 1 | 2 => print_str("small"), _ => print_str("big") }
    let cmd = "spa high";
    let v = match cmd { "spa low" => 20.0, "spa high" => 36.0, _ => 0.0 };
    print_f64(v);
}
"#,
    );
}

#[test]
fn parity_aggregates() {
    parity(
        "aggregates",
        r#"
#[derive(Clone, Copy)]
struct P { x: i64, y: i64 }
#[derive(Clone, Copy)]
struct Seg { a: P, b: P }
fn e(v: i64) -> i64 { print_i64(v); v }
fn mid(s: Seg) -> P { P { x: (s.a.x + s.b.x) / 2, y: (s.a.y + s.b.y) / 2 } }
fn t() -> bool { print_str("t"); true }
fn f() -> bool { print_str("f"); false }
fn main() {
    let s = Seg { a: P { x: 0, y: 10 }, b: P { x: 4, y: 2 } };
    let m = mid(s);
    print_i64(m.x); print_i64(m.y);
    let y = 7i64;
    let p = P { y, x: e(1) };
    print_i64(p.x); print_i64(p.y);
    let mut a = [0i64; 4];
    a[2] = 9; a[2] += 1;
    print_i64(a[2]);
    let mut mtx = [[0i64; 3]; 2];
    mtx[1][2] = 42;
    print_i64(mtx[1][2]); print_i64(mtx[0][0]);
    print_bool(f() && t());
    print_bool(t() || f());
    let arr = [3i64, 1, 4, 1, 5];
    let mut sum = 0i64;
    for i in 0usize..5 { sum += arr[i]; }
    print_i64(sum);
    let strs = "x\ty";
    print_str(strs);
    print_bool(strs == "x\ty");
}
"#,
    );
}

#[test]
fn parity_consts_and_fns() {
    parity(
        "consts",
        r#"
const SPA_HIGH: f64 = 36.0;
const D: i64 = 4 * 3600;
const N: usize = 4;
fn fib(n: i64) -> i64 { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }
fn main() {
    print_f64(SPA_HIGH);
    print_i64(D);
    let a = [7i64; N];
    print_i64(a[N - 1]);
    print_i64(fib(15));
}
"#,
    );
}

#[test]
fn parity_inference() {
    parity(
        "inference",
        r#"
fn main() {
    let c = true;
    let x = if c { 1 } else { 2 };
    print_i64(x as i64);
    let m = match 7i64 { 0 => 10, _ => 20 };
    print_i64(m as i64);
    print_u64(((200 + 100) as u8) as u64);
    print_u64(((if c { 300 } else { 2 }) as u8) as u64);
    print_u64(((match 5i64 { 0 => 300, _ => 2 }) as u8) as u64);
    print_i64((match 5i64 { 0 => 300i64, _ => 999 }) as i64);
    let y: i64 = if c { 1 } else { 2 };
    print_i64(y);
}
"#,
    );
}

#[test]
fn parity_assoc_consts() {
    parity(
        "assoc_consts",
        r#"
fn main() {
    print_u64(u8::MAX as u64); print_i64(i8::MIN as i64); print_i64(i8::MAX as i64);
    print_u64(u16::MAX as u64); print_i64(i16::MIN as i64);
    print_u64(u32::MAX as u64); print_i64(i32::MIN as i64);
    print_u64(u64::MAX); print_i64(i64::MIN); print_i64(i64::MAX);
    print_u64(usize::MAX as u64); print_i64(isize::MIN as i64);
    let x: i32 = 5; print_bool(x < i32::MAX);
}
"#,
    );
}

#[test]
fn parity_refs() {
    parity(
        "refs",
        r#"
#[derive(Clone, Copy)]
struct P { x: i64, y: i64 }
fn get(p: &i64) -> i64 { *p }
fn sum(a: &i64, b: &i64) -> i64 { *a + *b }
fn fst(p: &P) -> i64 { (*p).x }
fn dbl(p: &i64) -> i64 { *p * 2 }
fn du(p: &u64) -> u64 { *p + 1 }
fn main() {
    let x: i64 = 42;
    print_i64(get(&x));
    let y: i64 = 100;
    print_i64(sum(&x, &y));
    let s = P { x: 7, y: 99 };
    print_i64(fst(&s));
    let a = [10i64, 20, 30, 40];
    print_i64(dbl(&a[3]));
    let z: u64 = 5;
    print_u64(du(&z));
    let m: u128 = 18446744073709551616u128;
    print_u64((refu128(&m) & 0xFFFFFFFFFFFFFFFFu128) as u64);
}
fn refu128(p: &u128) -> u128 { *p + 1u128 }
"#,
    );
}

#[test]
fn parity_refs_mut() {
    parity(
        "refs_mut",
        r#"
#[derive(Clone, Copy)]
struct P { x: i64, y: i64 }
fn inc(p: &mut i64) { *p = *p + 1; }
fn addv(p: &mut i64, v: i64) { *p += v; }
fn zero(p: &mut u64) { *p = 0; }
fn setx(p: &mut P) { (*p).x = 99; (*p).y += 1; }
fn setr(p: &mut i64) { *p = 20; }
fn swap(a: &mut i64, b: &mut i64) { let t = *a; *a = *b; *b = t; }
fn inc128(p: &mut u128) { *p = *p + 1u128; }
fn main() {
    let mut x: i64 = 5;
    inc(&mut x); print_i64(x);
    addv(&mut x, 10); print_i64(x);
    let mut u: u64 = 77;
    zero(&mut u); print_u64(u);
    let mut s = P { x: 1, y: 2 };
    setx(&mut s); print_i64(s.x); print_i64(s.y);
    let mut a = [10i64, 11, 12, 13];
    setr(&mut a[2]); print_i64(a[2]); print_i64(a[0]);
    let mut p: i64 = 3;
    let mut q: i64 = 8;
    swap(&mut p, &mut q); print_i64(p); print_i64(q);
    let mut big: u128 = 18446744073709551615u128;
    inc128(&mut big);
    print_u64((big >> 64) as u64); print_u64((big & 0xFFFFFFFFFFFFFFFFu128) as u64);
}
"#,
    );
}

#[test]
fn parity_refs_mut_trap() {

    parity(
        "refs_mut_trap",
        r#"
fn boom(p: &mut u8) { *p += 200; }
fn main() {
    print_i64(1);
    let mut x: u8 = 100;
    boom(&mut x);
    print_i64(2);
}
"#,
    );
}

#[test]
fn parity_f64_bits() {

    parity(
        "f64_bits",
        r#"
fn main() {
    let x: f64 = 1.5;
    print_u64(x.to_bits());
    let y: f64 = f64::from_bits(4609434218613702656u64);
    print_f64(y);
    print_bool(x.is_nan());
    let nan: f64 = f64::from_bits(9221120237041090560u64);
    print_bool(nan.is_nan());
    let z: f64 = 0.0;
    print_bool((z / z).is_nan());
    print_u64(f64::from_bits(x.to_bits()).to_bits());
    let inf: f64 = 1.0 / 0.0;
    print_bool(inf.is_nan());
    print_u64((2.5e-2f64).to_bits());
    print_f64(f64::from_bits(0u64));
}
"#,
    );
}

#[test]
fn parity_saturating() {

    parity(
        "saturating",
        r#"
fn main() {
    let a: u8 = 250;
    print_i64(a.saturating_add(10) as i64);
    print_i64(a.saturating_mul(2) as i64);
    let b: i8 = 100;
    print_i64(b.saturating_add(100) as i64);
    print_i64(b.saturating_mul(2) as i64);
    let c: i8 = -100;
    print_i64(c.saturating_add(-100) as i64);
    print_i64(c.saturating_mul(2) as i64);
    let d: u32 = 4000000000;
    print_u64(d.saturating_add(1000000000) as u64);
    let e: i64 = 9000000000000000000;
    print_i64(e.saturating_add(1000000000000000000));
    let f: u128 = 340282366920938463463374607431768211455u128;
    print_u64((f.saturating_add(1u128) >> 64) as u64);
    let g: i128 = 170141183460469231731687303715884105720i128;
    print_i64((g.saturating_mul(2i128) >> 100) as i64);
    let h: i8 = 5;
    print_i64(h.saturating_add(3) as i64);
    print_i64(h.saturating_mul(4) as i64);
}
"#,
    );
}

#[test]
fn parity_index_deref() {

    parity(
        "index_deref",
        r#"
fn get(a: &[i64; 4], i: usize) -> i64 { a[i] }
fn set(a: &mut [i64; 4], i: usize, v: i64) { a[i] = v; }
fn addto(a: &mut [i64; 4]) { a[0] = a[1] + a[2] * a[3]; a[1] += a[0]; }
fn sum(a: &[i64; 4]) -> i64 { let mut t: i64 = 0; let mut i: usize = 0; while i < 4 { t += a[i]; i += 1; } t }
fn main() {
    let mut a = [10i64, 20, 30, 40];
    print_i64(get(&a, 2));
    set(&mut a, 0, 99);
    print_i64(a[0]);
    addto(&mut a);
    print_i64(a[0]); print_i64(a[1]);
    print_i64(sum(&a));
}
"#,
    );
}

#[test]
fn parity_rotate() {

    parity(
        "rotate",
        r#"
fn main() {
    print_u64(0x80000000u32.rotate_left(1) as u64);
    print_u64(1u32.rotate_right(1) as u64);
    print_i64(0x12u8.rotate_left(4) as i64);
    print_i64(0xABCDu16.rotate_left(8) as i64);
    print_u64(0x0123456789ABCDEFu64.rotate_left(16));
    print_u64(0x0123456789ABCDEFu64.rotate_right(16));
    print_u64((0x1u128 << 127).rotate_left(1) as u64);
    print_i64((-1i32).rotate_left(5) as i64);
    let x: u32 = 0xDEADBEEF;
    print_u64(x.rotate_left(0) as u64);
    print_u64(x.rotate_left(32) as u64);
}
"#,
    );
}

#[test]
fn parity_chacha_quarter_round() {

    parity(
        "chacha_qr",
        r#"
fn quarter_round(a: usize, b: usize, c: usize, d: usize, state: &mut [u32; 16]) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}
fn main() {
    let mut s = [0u32; 16];
    s[0] = 0x11111111u32; s[1] = 0x01020304u32; s[2] = 0x9b8d6f43u32; s[3] = 0x01234567u32;
    quarter_round(0, 1, 2, 3, &mut s);
    print_u64(s[0] as u64); print_u64(s[1] as u64); print_u64(s[2] as u64); print_u64(s[3] as u64);
}
"#,
    );
}

#[test]
fn parity_wrapping() {

    parity(
        "wrapping",
        r#"
fn main() {
    let a: u8 = 250;
    print_i64(a.wrapping_add(10) as i64);
    print_i64(a.wrapping_mul(2) as i64);
    print_i64(a.wrapping_sub(255) as i64);
    let b: i8 = -128;
    print_i64(b.wrapping_neg() as i64);
    print_i64(b.wrapping_sub(1) as i64);
    print_i64(b.wrapping_mul(3) as i64);
    let c: u32 = 4294967295;
    print_u64(c.wrapping_add(2) as u64);
    let d: i64 = 1;
    print_i64(d.wrapping_shl(63));
    print_i64(d.wrapping_shl(64));
    let e: u128 = 340282366920938463463374607431768211455u128;
    print_u64(e.wrapping_add(5u128) as u64);
    print_u64((e.wrapping_mul(2u128) & 0xFFFFFFFFFFFFFFFFu128) as u64);
    let g: i128 = -170141183460469231731687303715884105728i128;
    print_i64(g.wrapping_neg() as i64);
    let f: u64 = 100;
    print_u64(f.wrapping_sub(200));
    print_u64(7u64.wrapping_shl(61));
}
"#,
    );
}

#[test]
fn parity_subslice() {

    parity(
        "subslice",
        r#"
fn sum(s: &[i64]) -> i64 {
    let mut t: i64 = 0; let mut i: usize = 0;
    while i < s.len() { t += s[i]; i += 1; } t
}
fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut i: usize = 0;
    while i < a.len() { if a[i] != b[i] { return false; } i += 1; }
    true
}
fn kw(w: &[u8]) -> u16 { if eq(w, b"fn") { return 10; } if eq(w, b"let") { return 11; } 1 }
fn resum(s: &[i64]) -> i64 { sum(&s[1..3]) }  // sub-slice of a slice param
fn main() {
    let a = [10i64, 20, 30, 40, 50];
    print_i64(sum(&a[1..4]));
    print_i64(sum(&a[0..2]));
    print_usize((&a[2..5]).len());
    print_i64(sum(&a[3..3]));
    print_i64(resum(&a[0..5]));
    let src = [102u8, 110, 108, 101, 116];
    print_i64(kw(&src[0..2]) as i64);
    print_i64(kw(&src[2..5]) as i64);
    print_i64(kw(&b"letx"[0..3]) as i64);
}
"#,
    );
}

#[test]
fn parity_subslice_oob() {

    parity(
        "subslice_oob",
        r#"
fn sum(s: &[i64]) -> i64 {
    let mut t: i64 = 0; let mut i: usize = 0;
    while i < s.len() { t += s[i]; i += 1; } t
}
fn main() {
    let a = [1i64, 2, 3];
    print_i64(sum(&a[0..2]));
    print_i64(sum(&a[1..9]));
    print_i64(777);
}
"#,
    );
}

#[test]
fn parity_pub() {

    parity(
        "pub",
        r#"
pub const K: i64 = 42;
#[derive(Clone, Copy)]
pub struct P { pub x: i64, y: i64 }
pub fn get(p: &P) -> i64 { p.x + p.y }
fn main() { let p = P { x: K, y: 8 }; print_i64(get(&p)); }
"#,
    );
}

#[test]
fn parity_cast_precedence() {

    parity(
        "cast_precedence",
        r#"
fn dc(p: &u64) -> u32 { *p as u32 }
fn main() {
    let x: i32 = 5;
    print_i64(-x as u8 as i64);
    print_i64(!x as u8 as i64);
    let y: u64 = 4000000000;
    print_u64(dc(&y) as u64);
    print_i64(-1 as i64);
    print_i64(-1.9 as i64);
    print_i64((-1i64) as u8 as i64);
    let mut i: usize = 300;
    let r = &mut i;
    print_u64(*r as u32 as u64);
}
"#,
    );
}

#[test]
fn parity_assign_expr() {

    parity(
        "assign_expr",
        r#"
fn punct(c: u8) -> u16 {
    let mut kind: u16 = 0;
    let mut adv: usize = 1;
    match c {
        b'(' => kind = 40,
        b')' => kind = 41,
        b'{' | b'}' => kind = 42,
        b':' => { if adv == 1 { kind = 50; adv = 2; } else { kind = 51; } }
        _ => kind = 255,
    }
    kind + (adv as u16)
}
fn classify(x: i64) -> i64 {
    let mut r: i64 = 0;
    if x < 0 { r = -1 } else if x == 0 { r = 0 } else { r = 1 }
    r
}
fn main() {
    print_i64(punct(b'(') as i64);
    print_i64(punct(b'}') as i64);
    print_i64(punct(b':') as i64);
    print_i64(punct(b'z') as i64);
    print_i64(classify(-9)); print_i64(classify(0)); print_i64(classify(42));
    let mut x: i64 = 5;
    x += 3; x = x * 2;
    print_i64(x);
}
"#,
    );
}

#[test]
fn parity_recursive_parser() {

    parity(
        "recursive_parser",
        r#"
const CAP: usize = 128;
#[derive(Clone, Copy)]
struct Node { kind: u16, val: i64, lhs: u32, rhs: u32 }
#[derive(Clone, Copy)]
struct P { nodes: [Node; CAP], nn: usize, pos: usize }
impl P {
    fn add(&mut self, kind: u16, val: i64, lhs: u32, rhs: u32) -> u32 {
        let id = self.nn as u32;
        self.nodes[self.nn] = Node { kind: kind, val: val, lhs: lhs, rhs: rhs };
        self.nn += 1;
        id
    }
}
fn skip_ws(src: &[u8], p: &mut P) { while p.pos < src.len() && src[p.pos] == b' ' { p.pos += 1; } }
fn parse_num(src: &[u8], p: &mut P) -> i64 {
    let mut v: i64 = 0;
    while p.pos < src.len() && src[p.pos] >= b'0' && src[p.pos] <= b'9' {
        v = v * 10 + (src[p.pos] - b'0') as i64; p.pos += 1;
    }
    v
}
fn parse_factor(src: &[u8], p: &mut P) -> u32 {
    skip_ws(src, p);
    if p.pos < src.len() && src[p.pos] == b'(' {
        p.pos += 1;
        let e = parse_expr(src, p);
        skip_ws(src, p);
        if p.pos < src.len() && src[p.pos] == b')' { p.pos += 1; }
        return e;
    }
    let v = parse_num(src, p);
    p.add(0, v, 0, 0)
}
fn parse_term(src: &[u8], p: &mut P) -> u32 {
    let mut lhs = parse_factor(src, p);
    loop {
        skip_ws(src, p);
        if p.pos < src.len() && src[p.pos] == b'*' {
            p.pos += 1; let rhs = parse_factor(src, p); lhs = p.add(3, 0, lhs, rhs);
        } else if p.pos < src.len() && src[p.pos] == b'/' {
            p.pos += 1; let rhs = parse_factor(src, p); lhs = p.add(4, 0, lhs, rhs);
        } else { return lhs; }
    }
}
fn parse_expr(src: &[u8], p: &mut P) -> u32 {
    let mut lhs = parse_term(src, p);
    loop {
        skip_ws(src, p);
        if p.pos < src.len() && src[p.pos] == b'+' {
            p.pos += 1; let rhs = parse_term(src, p); lhs = p.add(1, 0, lhs, rhs);
        } else if p.pos < src.len() && src[p.pos] == b'-' {
            p.pos += 1; let rhs = parse_term(src, p); lhs = p.add(2, 0, lhs, rhs);
        } else { return lhs; }
    }
}
fn eval(p: &P, id: u32) -> i64 {
    let nd = p.nodes[id as usize];
    match nd.kind {
        0 => nd.val,
        1 => eval(p, nd.lhs) + eval(p, nd.rhs),
        2 => eval(p, nd.lhs) - eval(p, nd.rhs),
        3 => eval(p, nd.lhs) * eval(p, nd.rhs),
        _ => { let d = eval(p, nd.rhs); eval(p, nd.lhs) / d }
    }
}
fn calc(src: &[u8]) -> i64 {
    let mut p = P { nodes: [Node { kind: 0, val: 0, lhs: 0, rhs: 0 }; CAP], nn: 0, pos: 0 };
    let root = parse_expr(src, &mut p);
    eval(&p, root)
}
fn main() {
    let s = [b'1', b' ', b'+', b' ', b'2', b' ', b'*', b' ', b'3', b' ', b'-',
             b' ', b'(', b'4', b' ', b'+', b' ', b'5', b')', b' ', b'/', b' ', b'3'];
    print_i64(calc(&s));
    let t = [b'2', b'*', b'(', b'3', b'+', b'4', b')', b'*', b'5'];
    print_i64(calc(&t));
    let u = [b'1', b'0', b'0', b'-', b'6', b'/', b'2', b'/', b'3'];
    print_i64(calc(&u));
}
"#,
    );
}

#[test]
fn parity_core_architecture() {

    parity(
        "core_architecture",
        r#"
#[derive(Clone, Copy)]
struct Tok { kind: u16, pos: u32 }
#[derive(Clone, Copy)]
struct Mem { toks: [Tok; 8], n: usize }
impl Mem {
    fn node(&self, i: usize) -> Tok { self.toks[i] }
    fn push(&mut self, k: u16, p: u32) { self.toks[self.n] = Tok { kind: k, pos: p }; self.n += 1; }
    fn push2(&mut self, k: u16) { self.push(k, 0); self.push(k, 1); }
    fn kind_of(&self, i: usize) -> u16 { self.node(i).kind }
}
fn classify(m: &Mem, i: usize) -> i64 {
    match m.node(i).kind {
        1 => 100,
        2 => 200,
        _ => 0,
    }
}
fn main() {
    let mut m = Mem { toks: [Tok { kind: 0, pos: 0 }; 8], n: 0 };
    m.push(1, 10);
    m.push2(2);
    print_usize(m.n);
    print_i64(m.node(0).kind as i64);
    print_i64(m.node(1).pos as i64);
    print_i64(m.kind_of(2) as i64);
    print_i64(classify(&m, 0));
    print_i64(classify(&m, 1));
    print_i64(classify(&m, 2));
}
"#,
    );
}

#[test]
fn parity_lexer_helpers() {

    parity(
        "lexer_helpers",
        r#"
fn is_digit(c: u8) -> bool { c >= b'0' && c <= b'9' }
fn is_hex(c: u8) -> bool {
    is_digit(c) || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')
}
fn is_alpha(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') }
fn is_ident_start(c: u8) -> bool { c == b'_' || is_alpha(c) }
fn is_ident_char(c: u8) -> bool { c == b'_' || is_alpha(c) || is_digit(c) }
fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut i: usize = 0;
    while i < a.len() {
        if a[i] != b[i] { return false; }
        i += 1;
    }
    true
}
fn kw_lookup(w: &[u8]) -> u16 {
    if bytes_eq(w, b"fn") { return 10; }
    if bytes_eq(w, b"let") { return 11; }
    if bytes_eq(w, b"return") { return 31; }
    1
}
fn main() {
    print_bool(is_digit(b'7')); print_bool(is_digit(b'x'));
    print_bool(is_hex(b'f')); print_bool(is_hex(b'g'));
    print_bool(is_ident_start(b'_')); print_bool(is_ident_start(b'9'));
    print_bool(is_ident_char(b'a')); print_bool(is_ident_char(b'-'));
    let fnbytes = [102u8, 110];
    print_i64(kw_lookup(&fnbytes) as i64);
    print_i64(kw_lookup(b"return") as i64);
    print_i64(b'A' as i64); print_i64(b'\n' as i64);
    print_i64(b'\\' as i64); print_i64(b'\'' as i64); print_i64(b'0' as i64);
}
"#,
    );
}

#[test]
fn parity_bytestr() {

    parity(
        "bytestr",
        r#"
fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut i: usize = 0;
    while i < a.len() {
        if a[i] != b[i] { return false; }
        i += 1;
    }
    true
}
fn kw(w: &[u8]) -> u16 {
    if bytes_eq(w, b"fn") { return 10; }
    if bytes_eq(w, b"let") { return 11; }
    if bytes_eq(w, b"return") { return 31; }
    1
}
fn main() {
    let a = [102u8, 110];
    let b = [108u8, 101, 116];
    let c = [120u8, 121];
    print_i64(kw(&a) as i64);
    print_i64(kw(&b) as i64);
    print_i64(kw(&c) as i64);
    print_bool(bytes_eq(b"fn", b"fn"));
    print_bool(bytes_eq(b"fn", b"let"));
    print_i64(b"hello".len() as i64);
    print_i64(b"abc"[1] as i64);
    print_i64(b"a\nb"[1] as i64);
    print_bool(bytes_eq(b"", b""));
}
"#,
    );
}

#[test]
fn parity_return() {

    parity(
        "return",
        r#"
fn classify(x: i64) -> i64 {
    if x < 0 { return -1; }
    if x == 0 { return 0; }
    1
}
fn sum_until(limit: i64) -> i64 {
    let mut t: i64 = 0;
    let mut i: i64 = 0;
    while i < 100 {
        if t > limit { return t; }
        t += i;
        i += 1;
    }
    t
}
fn pick(k: u16) -> i64 {
    match k {
        1 => return 10,
        2 => 20,
        _ => return -5,
    }
}
fn clamp(x: i64) -> i64 {
    let y = if x > 10 { return 10 } else { x };
    y
}
fn early(v: i64) {
    if v == 42 { return; }
    print_i64(v);
}
fn main() {
    print_i64(classify(-7)); print_i64(classify(0)); print_i64(classify(99));
    print_i64(sum_until(20));
    print_i64(pick(1)); print_i64(pick(2)); print_i64(pick(9));
    print_i64(clamp(3)); print_i64(clamp(50));
    early(42); early(7);
}
"#,
    );
}

#[test]
fn parity_slices() {

    parity(
        "slices",
        r#"
fn sum(s: &[i64]) -> i64 {
    let mut t: i64 = 0;
    let mut i: usize = 0;
    while i < s.len() { t += s[i]; i += 1; }
    t
}
fn dot(a: &[i64], b: &[i64]) -> i64 {
    let mut t: i64 = 0;
    let mut i: usize = 0;
    while i < a.len() { t += a[i] * b[i]; i += 1; }
    t
}
fn nth(s: &[u64], i: usize) -> u64 { s[i] }
fn main() {
    let a = [10i64, 20, 30, 40];
    print_i64(sum(&a));
    print_usize(a.len());
    let b = [2i64, 2, 2, 2];
    print_i64(dot(&a, &b));
    let u = [5u64, 6, 7];
    print_u64(nth(&u, 2));
    print_usize(u.len());
    let one = [99i64];
    print_i64(sum(&one));
}
"#,
    );
}

#[test]
fn parity_slices_mut() {

    parity(
        "slices_mut",
        r#"
fn fill(s: &mut [i64], v: i64) {
    let mut i: usize = 0;
    while i < s.len() { s[i] = v; i += 1; }
}
fn scale(s: &mut [i64], k: i64) {
    let mut i: usize = 0;
    while i < s.len() { s[i] *= k; i += 1; }
}
fn set(s: &mut [u64], i: usize, v: u64) { s[i] = v; }
fn sum(s: &[i64]) -> i64 {
    let mut t: i64 = 0; let mut i: usize = 0;
    while i < s.len() { t += s[i]; i += 1; } t
}
fn main() {
    let mut a = [1i64, 2, 3, 4];
    scale(&mut a, 10);
    print_i64(sum(&a));
    fill(&mut a, 7);
    print_i64(sum(&a));
    print_i64(a[0]); print_i64(a[3]);
    let mut b = [0u64, 0, 0];
    set(&mut b, 1, 99);
    print_u64(b[1]);
}
"#,
    );
}

#[test]
fn parity_slices_mut_oob() {

    parity(
        "slices_mut_oob",
        r#"
fn set(s: &mut [i64], i: usize, v: i64) { s[i] = v; }
fn main() {
    let mut a = [1i64, 2, 3];
    set(&mut a, 0, 10);
    print_i64(a[0]);
    set(&mut a, 9, 10);
    print_i64(777);
}
"#,
    );
}

#[test]
fn parity_slices_oob() {

    parity(
        "slices_oob",
        r#"
fn nth(s: &[i64], i: usize) -> i64 { s[i] }
fn main() {
    let a = [1i64, 2, 3];
    print_i64(nth(&a, 0));
    print_i64(nth(&a, 5));
    print_i64(999);
}
"#,
    );
}

#[test]
fn parity_const_patterns() {

    parity(
        "const_patterns",
        r#"
const N_FN: u16 = 1;
const N_STRUCT: u16 = 2;
const N_CONST: u16 = 3;
const LO: i64 = -5;
const HELLO: &str = "hi";
const YES: bool = true;
fn name(k: u16) -> i64 {
    match k {
        N_FN => 10,
        N_STRUCT | N_CONST => 20,
        _ => 0,
    }
}
fn signed(x: i64) -> i64 {
    match x {
        LO => 100,
        0 => 1,
        _ => -1,
    }
}
fn greet(s: &str) -> i64 { match s { HELLO => 1, _ => 2 } }
fn flag(b: bool) -> i64 { match b { YES => 7, _ => 8 } }
fn main() {
    print_i64(name(1)); print_i64(name(2)); print_i64(name(3)); print_i64(name(9));
    print_i64(signed(-5)); print_i64(signed(0)); print_i64(signed(42));
    print_i64(greet("hi")); print_i64(greet("bye"));
    print_i64(flag(true)); print_i64(flag(false));
}
"#,
    );
}

#[test]
fn parity_methods() {
    parity(
        "methods",
        r#"
#[derive(Clone, Copy)]
struct Counter { n: i64 }
impl Counter {
    fn get(&self) -> i64 { self.n }
    fn add(&mut self, v: i64) { self.n = self.n + v; }
    fn bump(&mut self) { self.n += 1; }
    fn sum_with(&self, other: i64) -> i64 { self.n + other }
    fn doubled(self) -> i64 { self.n * 2 }
}
#[derive(Clone, Copy)]
struct Point { x: i64, y: i64 }
impl Point {
    fn manhattan(&self) -> i64 { self.x + self.y }
    fn shift(&mut self, dx: i64, dy: i64) { self.x += dx; self.y += dy; }
    fn scale(&mut self, k: i64) { self.x *= k; self.y *= k; }
}
fn use_ref(c: &mut Counter) { c.add(10); c.bump(); }
fn read_ref(c: &Counter) -> i64 { c.get() }
fn main() {
    let mut c = Counter { n: 5 };
    c.bump();
    print_i64(c.get());          // 6
    c.add(4);
    print_i64(c.get());          // 10
    print_i64(c.sum_with(100));  // 110
    print_i64(c.doubled());      // 20 (self by value)
    use_ref(&mut c);             // +10 +1 => 21
    print_i64(c.get());          // 21
    print_i64(read_ref(&c));     // 21
    let mut p = Point { x: 1, y: 2 };
    p.shift(3, 4);
    print_i64(p.manhattan());    // 10
    p.scale(2);
    print_i64(p.x); print_i64(p.y); // 8 12
    // method result feeding another call, chained mutation
    let mut d = Counter { n: 0 };
    d.add(c.get());
    print_i64(d.get());          // 21
}
"#,
    );
}

#[test]
fn parity_methods_nested() {

    parity(
        "methods_nested",
        r#"
#[derive(Clone, Copy)]
struct C { n: i64 }
impl C {
    fn get(&self) -> i64 { self.n }
    fn triple(&self) -> i64 { self.get() * 3 }
    fn incr_by_get(&mut self) { self.n += self.get(); }
}
#[derive(Clone, Copy)]
struct Inner { v: i64 }
#[derive(Clone, Copy)]
struct Outer { a: Inner, b: i64 }
impl Inner { fn val(&self) -> i64 { self.v } fn set(&mut self, x: i64) { self.v = x; } }
impl Outer {
    fn total(&self) -> i64 { self.a.val() + self.b }
    fn bump_inner(&mut self, x: i64) { self.a.set(x); }
}
fn main() {
    let mut c = C { n: 4 };
    print_i64(c.triple());
    c.incr_by_get();
    print_i64(c.get());
    let mut o = Outer { a: Inner { v: 10 }, b: 5 };
    print_i64(o.total());
    o.bump_inner(100);
    print_i64(o.total());
    print_i64(o.a.val());
}
"#,
    );
}

#[test]
fn parity_int128() {
    parity(
        "int128",
        r#"
fn hi(x: u128) -> u64 { (x >> 64) as u64 }
fn lo(x: u128) -> u64 { (x & 0xFFFFFFFFFFFFFFFFu128) as u64 }
fn main() {
    let a: u128 = 18446744073709551616u128;
    let b: u128 = a * 1000u128 + 12345u128;
    print_u64(hi(b)); print_u64(lo(b));
    let c: u128 = 340282366920938463463374607431768211455u128;
    print_u64(hi(c)); print_u64(lo(c));
    print_u64(hi(u128::MAX)); print_u64(lo(u128::MAX));
    let d: i128 = -170141183460469231731687303715884105728i128;
    print_i64((d >> 100) as i64);
    print_i64((i128::MIN >> 120) as i64);
    let e: u128 = b / 7u128; print_u64(lo(e));
    print_u64(lo(b % 1000000u128));
    print_bool(a < b); print_bool(c > b); print_bool(a == a);
    let g: u128 = 5u128; print_u64((g << 100u32 >> 100u32) as u64);
    print_u64((u64::MAX as u128 * 3u128 & 0xFFFFFFFFFFFFFFFFu128) as u64);
    print_i64((123456789012345678901234567890u128 % 1000000007u128) as i64);
    print_u64((1000u64 as i128 as u128 & 0xFFFFu128) as u64);
    print_i64(((-5i128) as i64));
}
"#,
    );
}

#[test]
fn parity_trap_overflow() {

    parity(
        "trap_overflow",
        r#"
fn main() {
    print_str("before");
    let x: i32 = 2147483647;
    let y = x + 1;
    print_i64(y as i64);
}
"#,
    );
}

#[test]
fn parity_trap_div_zero() {
    parity(
        "trap_div",
        r#"
fn main() {
    print_i64(1);
    let d = 0i64;
    print_i64(10 / d);
}
"#,
    );
}

#[test]
fn parity_trap_oob() {
    parity(
        "trap_oob",
        r#"
fn main() {
    let a = [1i64, 2, 3];
    let i = 3usize;
    print_str("indexing");
    print_i64(a[i]);
}
"#,
    );
}

#[test]
fn parity_trap_shift() {
    parity(
        "trap_shift",
        r#"
fn main() {
    print_str("shifting");
    let s = 64u32;
    print_u64(1u64 << s);
}
"#,
    );
}

#[test]
fn parity_const_aggregates() {

    parity(
        "const_aggregates",
        r#"
#[derive(Clone, Copy)]
struct Tok { kind: u16, a: u32, b: u32 }
const TOK_NONE: Tok = Tok { kind: 0, a: 0, b: 0 };
const PRIMES: [u32; 5] = [2, 3, 5, 7, 11];
const ZEROS: [i64; 4] = [0; 4];

#[derive(Clone, Copy)]
struct Cfg { limits: [u32; 3], base: i64 }
const BASE: i64 = 100;
const CFG: Cfg = Cfg { limits: [10, 20, 30], base: BASE };

fn main() {
    // bind the whole struct const to a local, then read a field
    let t = TOK_NONE;
    print_u64(t.kind as u64);
    print_u64(t.a as u64);
    // sum an array const by runtime index
    let mut s: u64 = 0;
    let mut i: usize = 0;
    while i < 5 {
        s += PRIMES[i] as u64;
        i += 1;
    }
    print_u64(s);
    // array-repeat const
    print_i64(ZEROS[0] + ZEROS[3]);
    // field access directly on a const rvalue, then index its array field
    print_u64(CFG.limits[0] as u64);
    print_u64(CFG.limits[2] as u64);
    print_i64(CFG.base);
    // index a const array rvalue directly
    print_u64(PRIMES[4] as u64);
}
"#,
    );
}

#[test]
fn parity_const_array_in_fn() {

    parity(
        "const_array_in_fn",
        r#"
const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];

fn digit(n: usize) -> u8 {
    if n < 10 { DIGITS[n] } else { b'?' }
}

fn main() {
    let mut i: usize = 0;
    while i < 12 {
        print_i64(digit(i) as i64);
        i += 1;
    }
}
"#,
    );
}

#[test]
fn parity_if_intlit_defer() {

    parity(
        "if_intlit_defer",
        r#"
fn scan(w: &[u8], hex: bool) -> usize {
    let mut i = if hex { 2 } else { 0 };   // inferred usize from `i < w.len()`
    let mut n: usize = 0;
    while i < w.len() { n += 1; i += 1; }
    n
}
fn main() {
    let a = [1u8, 2, 3, 4, 5];
    print_usize(scan(&a, false));
    print_usize(scan(&a, true));
    // unconstrained → i32 default (both branches), still matches rustc
    let x = if true { 10 } else { 20 };
    print_i64((x + 5) as i64);
    // pinned wide by a later u64 use
    let y = if false { 1 } else { 4000000000 };
    print_u64(y);
}
"#,
    );
}

#[test]
fn parity_open_slice() {

    parity(
        "open_slice",
        r#"
fn tail_sum(s: &[i64], from: usize) -> i64 {
    let w = &s[from..];
    let mut t: i64 = 0;
    let mut i: usize = 0;
    while i < w.len() { t += w[i]; i += 1; }
    t
}
fn main() {
    let a = [1i64, 2, 3, 4, 5];
    print_i64(tail_sum(&a, 0));
    print_i64(tail_sum(&a, 2));
    print_i64(tail_sum(&a, 5));   // empty tail
    // open slice of an array reference, then index
    let b = [10u8, 20, 30];
    let w = &b[1..];
    print_usize(w.len());
    print_i64(w[1] as i64);
}
"#,
    );
}

#[test]
fn parity_tuples() {

    parity(
        "tuples",
        r#"
fn classify(x: i64) -> i64 {
    let (kind, mag) = if x < 0 {
        (0i64, 0 - x)
    } else if x == 0 {
        (1i64, 0)
    } else {
        (2i64, x)
    };
    kind * 1000 + mag
}
fn main() {
    print_i64(classify(-5));
    print_i64(classify(0));
    print_i64(classify(42));
    // a 3-tuple with mixed element types
    let (a, b, c) = (7u8, true, 100i64);
    print_i64(a as i64);
    print_bool(b);
    print_i64(c);
    // a mutable element, then reassigned
    let (mut n, s) = (0i64, 5i64);
    n += s;
    n += s;
    print_i64(n);
    // wildcards in a tuple pattern
    let (_, keep, _) = (1i64, 2i64, 3i64);
    print_i64(keep);
}
"#,
    );
}

#[test]
fn parity_pub_fn_impl() {

    parity(
        "pub_fn_impl",
        r#"
#[derive(Clone, Copy)]
struct Counter { n: i64 }
impl Counter {
    pub fn get(&self) -> i64 { self.n }
    pub fn bump(&mut self) { self.n += 1; }
    fn twice(&self) -> i64 { self.n * 2 }
}
fn main() {
    let mut c = Counter { n: 10 };
    c.bump();
    c.bump();
    print_i64(c.get());
    print_i64(c.twice());
}
"#,
    );
}

#[test]
fn parity_checker_frame_budget() {

    parity(
        "checker_frame_budget",
        r#"
#[derive(Clone, Copy)] struct SInfo { a: u32, b: u32, c: u32, d: u32, e: u32, host: u32 }
#[derive(Clone, Copy)] struct AInfo { elem: u16, len: u32, size: u32, x: u32 }
#[derive(Clone, Copy)] struct RInfo { pointee: u16, mutable: u16 }
#[derive(Clone, Copy)] struct CInfo { name: u32, node: u32, ty: u16, state: u16, bits: u64 }
#[derive(Clone, Copy)] struct FInfo { name: u32, node: u32, fp: u32, pn: u32, ret: u16, frame: u32, st: u32, sy: u16 }
#[derive(Clone, Copy)] struct StrEntry { off: u32, len: u32 }
#[derive(Clone, Copy)]
struct Chk {
    ty: [u16; 24], res: [u32; 24],
    structs: [SInfo; 2], arrs: [AInfo; 2], refs: [RInfo; 2], slices: [RInfo; 2],
    consts: [CInfo; 2], fns: [FInfo; 4], vals: [u64; 16],
    strs: [StrEntry; 4], str_pool: [u8; 32], locals: [CInfo; 8],
    node_n: usize, struct_n: usize, fn_n: usize,
}
#[derive(Clone, Copy)] struct Tok { kind: u16, len: u16, pos: u32 }
#[derive(Clone, Copy)] struct Node { kind: u16, x: u16, a: u32, b: u32, c: u32, d: u32, e: u32, link: u32, lo: u32, hi: u32 }
#[derive(Clone, Copy)]
struct Mem { toks: [Tok; 24], nodes: [Node; 24], tok_n: usize, node_n: usize }
fn main() {
    let z6 = SInfo { a: 0, b: 0, c: 0, d: 0, e: 0, host: 0 };
    let za = AInfo { elem: 0, len: 0, size: 0, x: 0 };
    let zr = RInfo { pointee: 0, mutable: 0 };
    let zc = CInfo { name: 0, node: 0, ty: 0, state: 0, bits: 0 };
    let zf = FInfo { name: 0, node: 0, fp: 0, pn: 0, ret: 0, frame: 0, st: 0, sy: 0 };
    let zs = StrEntry { off: 0, len: 0 };
    let mut chk = Chk {
        ty: [0u16; 24], res: [0u32; 24],
        structs: [z6; 2], arrs: [za; 2], refs: [zr; 2], slices: [zr; 2],
        consts: [zc; 2], fns: [zf; 4], vals: [0u64; 16],
        strs: [zs; 4], str_pool: [0u8; 32], locals: [zc; 8],
        node_n: 0, struct_n: 0, fn_n: 0,
    };
    let zt = Tok { kind: 0, len: 0, pos: 0 };
    let zn = Node { kind: 0, x: 0, a: 0, b: 0, c: 0, d: 0, e: 0, link: 0, lo: 0, hi: 0 };
    let mut mem = Mem { toks: [zt; 24], nodes: [zn; 24], tok_n: 0, node_n: 0 };
    // touch both, as the checker would, so nothing is optimized to nothing
    chk.ty[0] = 7; chk.res[0] = 42; chk.vals[3] = 1000; chk.str_pool[5] = 65;
    chk.fns[0].ret = 9; chk.node_n = 12;
    mem.toks[0].kind = 3; mem.nodes[1].a = 99; mem.tok_n = 24;
    let mut sum: u64 = 0;
    sum += chk.ty[0] as u64;
    sum += chk.res[0] as u64;
    sum += chk.vals[3];
    sum += chk.str_pool[5] as u64;
    sum += chk.fns[0].ret as u64;
    sum += chk.node_n as u64;
    sum += mem.toks[0].kind as u64;
    sum += mem.nodes[1].a as u64;
    sum += mem.tok_n as u64;
    print_u64(sum);
}
"#,
    );
}

#[test]
fn parity_empty_slice() {

    parity(
        "empty_slice",
        r#"
fn head(s: &[u8], n: usize) -> &[u8] {
    if n <= s.len() { &s[0..n] } else { &[] }
}
fn sum(s: &[i64]) -> i64 {
    let mut t: i64 = 0;
    let mut i: usize = 0;
    while i < s.len() { t += s[i]; i += 1; }
    t
}
fn main() {
    let a = [10u8, 20, 30];
    print_usize(head(&a, 2).len());
    print_usize(head(&a, 9).len());   // fallback → empty
    print_i64(head(&a, 2)[1] as i64);
    // an empty slice sums to 0 and has length 0
    let e: &[i64] = &[];
    print_usize(e.len());
    print_i64(sum(e));
}
"#,
    );
}

#[test]
fn parity_return_borrow() {

    parity(
        "return_borrow",
        r#"
fn window(s: &[i64], lo: usize, hi: usize) -> &[i64] {
    if lo <= hi && hi <= s.len() { &s[lo..hi] } else { s }
}
fn whole(s: &[u8]) -> &[u8] { s }
fn nth_byte(bs: &[u8], i: usize) -> u8 {
    let w = whole(bs);
    w[i]
}
fn main() {
    let a = [10i64, 20, 30, 40, 50];
    let mid = window(&a, 1, 4);
    print_i64(mid[0]);
    print_i64(mid[2]);
    print_usize(mid.len());
    // out-of-range → the fallback returns the whole slice
    let all = window(&a, 9, 1);
    print_usize(all.len());
    // borrow a byte-string parameter through a helper
    print_i64(nth_byte(b"hello", 1) as i64);
    print_i64(nth_byte(b"hello", 4) as i64);
}
"#,
    );
}

#[test]
fn parity_reborrow() {

    parity(
        "reborrow",
        r#"
#[derive(Clone, Copy)]
struct P { i: i64 }
fn peek(p: &P) -> i64 { p.i }
fn step(p: &mut P) -> i64 {
    // p is &mut P here; peek wants &P
    let cur = peek(p);
    p.i += 1;
    cur
}
fn sum3(s: &[i64]) -> i64 { s[0] + s[1] + s[2] }
fn drive(a: &mut [i64]) -> i64 {
    a[0] = 10;
    // a is &mut [i64]; sum3 wants &[i64]
    sum3(a)
}
fn main() {
    let mut p = P { i: 5 };
    print_i64(step(&mut p));
    print_i64(step(&mut p));
    print_i64(peek(&p));
    let mut xs = [1i64, 2, 3];
    print_i64(drive(&mut xs));
}
"#,
    );
}

#[test]
fn parity_let_inference() {

    parity(
        "let_inference",
        r#"
fn sum(s: &[u64]) -> u64 {
    let mut total = 0;       // inferred u64 from `total += s[i]`
    let mut i = 0;           // inferred usize from `i < s.len()`
    while i < s.len() {
        total += s[i];
        i += 1;
    }
    total
}
fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut i = 0;           // inferred usize
    while i < a.len() {
        if a[i] != b[i] { return false; }
        i += 1;
    }
    true
}
fn main() {
    let a = [10u64, 20, 30, 4000000000];   // last elem needs > i32
    print_u64(sum(&a));
    // a literal inferred wide enough to hold a big value (would overflow i32)
    let big = 4000000000;                  // inferred u64 via print_u64
    print_u64(big);
    print_bool(eq(b"lexer", b"lexer"));
    print_bool(eq(b"lex", b"lexer"));
    // counter reused as an index into a const table; the bound is a usize
    // (`.len()`), which is what pins the counter — as in the real lexer
    let table = [7i64, 8, 9];
    let n = table.len();
    let mut j = 0;          // inferred usize from `j < n`
    let mut acc = 0;        // inferred i64 from `acc += table[j]`
    while j < n {
        acc += table[j];
        j += 1;
    }
    print_i64(acc);
}
"#,
    );
}

#[test]
fn parity_str_bytes() {

    parity(
        "str_bytes",
        r#"
fn count_dots(src: &str) -> usize {
    let b = src.as_bytes();
    let mut n: usize = 0;
    let mut i: usize = 0;
    while i < b.len() {
        if b[i] == b'.' { n += 1; }
        i += 1;
    }
    n
}
fn eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut i: usize = 0;
    while i < a.len() { if a[i] != b[i] { return false; } i += 1; }
    true
}
fn main() {
    // .len() on a literal and on a parameter path
    print_usize("hello".len());
    print_usize("".len());
    print_usize(count_dots("a.b.c.d"));
    // .as_bytes(): index and first/last byte
    let s = "Rustc";
    let b = s.as_bytes();
    print_usize(b.len());
    print_i64(b[0] as i64);
    print_i64(b[4] as i64);
    // chained: .as_bytes().len() must equal .len()
    print_bool("worldly".as_bytes().len() == "worldly".len());
    // compare a str's bytes against a byte-string literal
    print_bool(eq_bytes("abc".as_bytes(), b"abc"));
    print_bool(eq_bytes("abc".as_bytes(), b"abd"));
}
"#,
    );
}
