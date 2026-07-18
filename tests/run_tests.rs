
use subrust::apis::TEST_API;
use subrust::diag::*;
use subrust::machine::{Instance, INSTANCE_INIT};
use subrust::platform::{Platform, SrErr, SR_OK};
use subrust::{Chk, Mem, CHK_INIT, MEM_INIT};

const FUEL: u64 = 1_000_000;

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

/// Run `main()` of a program against TEST_API; return captured output.
fn run(src: &str) -> String {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(src, &mut mem, &mut chk, &TEST_API);
    if !ok && mem.diag_n > 0 {
        let d = mem.diags[0];
        panic!(
            "check failed {:#06x} at {}..{} in {src:?}",
            d.code, d.span.lo, d.span.hi
        );
    }
    assert!(ok);
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost {
        chk: &chk,
        out: String::new(),
    };
    let e = subrust::call(src, &mem, &chk, &mut inst, &mut host, "main", &[], FUEL);
    if e != SR_OK {
        panic!(
            "run failed: err {e}, trap {:#06x} at {}..{} in {src:?}",
            inst.trap_code, inst.trap_lo, inst.trap_hi
        );
    }
    host.out
}

/// Run `main()` and expect a trap with the given code.
fn run_trap(src: &str) -> u16 {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    assert!(subrust::check_source(src, &mut mem, &mut chk, &TEST_API));
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost {
        chk: &chk,
        out: String::new(),
    };
    let e = subrust::call(src, &mem, &chk, &mut inst, &mut host, "main", &[], FUEL);
    assert_ne!(e, SR_OK, "expected a trap for {src:?}");
    inst.trap_code
}

#[test]
fn arithmetic() {
    assert_eq!(run("fn main() { print_i64(2 + 3 * 4); }"), "14\n");
    assert_eq!(run("fn main() { print_i64((2 + 3) * 4); }"), "20\n");
    assert_eq!(run("fn main() { print_i64(7 / 2); print_i64(-7 / 2); print_i64(7 % 2); print_i64(-7 % 2); }"),
               "3\n-3\n1\n-1\n");
    assert_eq!(run("fn main() { print_i64(1 + 2 << 3); }"), "24\n");
    assert_eq!(run("fn main() { print_u64(0xFF_u64 & 0x0F | 0x30 ^ 0x01); }"), "63\n");
    assert_eq!(run("fn main() { print_f64(0.1 + 0.2); }"), "0.30000000000000004\n");
    assert_eq!(run("fn main() { print_f64(1.0 / 0.0); }"), "inf\n");
    assert_eq!(run("fn main() { print_bool(0.0 / 0.0 != 0.0 / 0.0); }"), "true\n");
}

#[test]
fn integer_widths_wrap_and_trap() {

    assert_eq!(run("fn main() { let x: u8 = 200; print_i64((x / 3) as i64); }"), "66\n");
    assert_eq!(run_trap("fn main() { let x: u8 = 200; let y = x + 100; }"), E_T_ARITH);
    assert_eq!(run_trap("fn main() { let x: i32 = 2147483647; let y = x + 1; }"), E_T_ARITH);
    assert_eq!(run_trap("fn main() { let a = 1; let b = 0; print_i64((a / b) as i64); }"), E_T_ARITH);
    assert_eq!(run_trap("fn main() { let x: i64 = -9223372036854775808; let y = -x; }"), E_T_ARITH);
    assert_eq!(run_trap("fn main() { let s = 64u32; print_u64((1u64 << s) as u64); }"), E_T_ARITH);
}

#[test]
fn negative_literal_folding() {
    assert_eq!(
        run("fn main() { print_i64(-9223372036854775808); }"),
        "-9223372036854775808\n"
    );
    assert_eq!(run("fn main() { print_i64(-(-1)); }"), "1\n");
}

#[test]
fn casts_runtime() {
    assert_eq!(run("fn main() { print_i64(300i64 as u8 as i64); }"), "44\n");
    assert_eq!(run("fn main() { print_i64((-1i64) as u8 as i64); }"), "255\n");
    assert_eq!(run("fn main() { print_i64(1e300 as i64); }"), "9223372036854775807\n");
    assert_eq!(run("fn main() { print_i64(-1.5 as i64); }"), "-1\n");
    assert_eq!(run("fn main() { print_i64((0.0 / 0.0) as i64); }"), "0\n");
    assert_eq!(run("fn main() { print_u64((-1.5) as u64); }"), "0\n");
    assert_eq!(run("fn main() { print_f64(true as i64 as f64); }"), "1\n");
}

#[test]
fn short_circuit() {
    let src = "
        fn t() -> bool { print_str(\"t\"); true }
        fn f() -> bool { print_str(\"f\"); false }
        fn main() {
            print_bool(f() && t());
            print_bool(t() || f());
        }";
    assert_eq!(run(src), "f\nfalse\nt\ntrue\n");
}

#[test]
fn loops() {
    assert_eq!(
        run("fn main() { let mut s = 0i64; for i in 1i64..=10 { s += i; } print_i64(s); }"),
        "55\n"
    );
    assert_eq!(
        run("fn main() { let mut s = 0i64; for i in 0i64..5 { if i == 2 { continue; } s += i; } print_i64(s); }"),
        "8\n"
    );
    assert_eq!(
        run("fn main() { let mut n = 0i64; while true { n += 1; if n == 7 { break; } } print_i64(n); }"),
        "7\n"
    );
    assert_eq!(
        run("fn main() { let mut n = 0i64; loop { n += 1; if n > 3 { break; } } print_i64(n); }"),
        "4\n"
    );

    assert_eq!(
        run("fn main() { let mut c = 0i64; for i in 250u8..=255 { c += 1; } print_i64(c); }"),
        "6\n"
    );

    assert_eq!(
        run("fn main() { let mut c = 0i64; for i in 5i64..2 { c += 1; } print_i64(c); }"),
        "0\n"
    );
}

#[test]
fn nested_loops_break() {
    assert_eq!(
        run("fn main() {
                 let mut hits = 0i64;
                 for i in 0i64..3 {
                     for j in 0i64..3 {
                         if j > i { break; }
                         hits += 1;
                     }
                 }
                 print_i64(hits);
             }"),
        "6\n"
    );
}

#[test]
fn if_and_match_values() {
    assert_eq!(
        run("fn main() { let x = if 3 > 2 { 10i64 } else { 20 }; print_i64(x); }"),
        "10\n"
    );
    let src = "
        fn name(x: i64) -> &str {
            match x { 0 => \"zero\", 1 | 2 => \"small\", -1 => \"neg\", _ => \"big\" }
        }
        fn main() {
            print_str(name(0)); print_str(name(2)); print_str(name(-1)); print_str(name(9));
        }";
    assert_eq!(run(src), "zero\nsmall\nneg\nbig\n");
    let src2 = "
        fn main() {
            let cmd = \"spa high\";
            let v = match cmd { \"spa low\" => 20.0, \"spa high\" => 36.0, _ => 0.0 };
            print_f64(v);
        }";
    assert_eq!(run(src2), "36\n");
}

#[test]
fn enums_field_less() {

    let src = "
        #[derive(Clone, Copy)]
        enum Mode { Idle, Heating, Purging }
        fn step(m: Mode) -> u64 {
            match m { Mode::Idle => 10, Mode::Heating => 20, Mode::Purging => 30 }
        }
        fn next(m: Mode) -> Mode {
            match m { Mode::Idle => Mode::Heating, Mode::Heating => Mode::Purging, Mode::Purging => Mode::Idle }
        }
        fn main() {
            let mut cur = Mode::Idle;
            let mut i: u64 = 0;
            while i < 4 {
                cur = next(cur);
                print_u64(step(cur));
                i = i + 1;
            }
            // wildcard arm also makes a match exhaustive
            let label: u64 = match cur { Mode::Idle => 1, _ => 2 };
            print_u64(label);
        }";
    assert_eq!(run(src), "20\n30\n10\n20\n2\n");

    let field = "
        #[derive(Clone, Copy)] enum Mode { A, B, C }
        #[derive(Clone, Copy)] struct S { m: Mode, x: u64 }
        fn pick(s: S) -> u64 { match s.m { Mode::A => 1, Mode::B => 2, Mode::C => s.x } }
        fn main() {
            let s = S { m: Mode::C, x: 7 };
            print_u64(pick(s));
            let a = S { m: Mode::A, x: 99 };
            print_u64(pick(a));
        }";
    assert_eq!(run(field), "7\n1\n");
}

#[test]
fn asserts() {

    assert_eq!(run("fn main() { assert!(1 + 1 == 2); print_u64(7); }"), "7\n");
    assert_eq!(
        run("fn main() { let n: u64 = 5; assert!(n > 3, \"n too small\"); print_u64(n); }"),
        "5\n"
    );

    assert_eq!(run_trap("fn main() { print_u64(1); assert!(1 > 2); print_u64(2); }"), E_T_ASSERT);
    assert_eq!(run_trap("fn main() { assert!(false, \"unreachable state\"); }"), E_T_ASSERT);
}

#[test]
fn functions_and_recursion() {
    assert_eq!(
        run("fn fib(n: i64) -> i64 { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }
             fn main() { print_i64(fib(20)); }"),
        "6765\n"
    );

    assert_eq!(
        run_trap("fn deep(n: i64) -> i64 { deep(n + 1) } fn main() { print_i64(deep(0)); }"),
        E_T_STACK
    );
}

#[test]
fn fuel_runs_out() {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let src = "fn main() { loop { } }";
    assert!(subrust::check_source(src, &mut mem, &mut chk, &TEST_API));
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost { chk: &chk, out: String::new() };
    let e = subrust::call(src, &mem, &chk, &mut inst, &mut host, "main", &[], 10_000);
    assert_ne!(e, SR_OK);
    assert_eq!(inst.trap_code, E_T_FUEL);
}

#[test]
fn structs_roundtrip() {
    let src = "
        #[derive(Clone, Copy)]
        struct P { x: i64, y: i64 }
        #[derive(Clone, Copy)]
        struct Seg { a: P, b: P }
        fn mid(s: Seg) -> P { P { x: (s.a.x + s.b.x) / 2, y: (s.a.y + s.b.y) / 2 } }
        fn main() {
            let s = Seg { a: P { x: 0, y: 10 }, b: P { x: 4, y: 2 } };
            let m = mid(s);
            print_i64(m.x); print_i64(m.y);
        }";
    assert_eq!(run(src), "2\n6\n");
}

#[test]
fn struct_shorthand_and_order() {

    let src = "
        #[derive(Clone, Copy)]
        struct P { x: i64, y: i64 }
        fn e(v: i64) -> i64 { print_i64(v); v }
        fn main() {
            let y = 7i64;
            let p = P { y, x: e(1) };
            print_i64(p.x); print_i64(p.y);
        }";
    assert_eq!(run(src), "1\n1\n7\n");
}

#[test]
fn arrays_runtime() {
    assert_eq!(
        run("fn main() { let a = [3i64, 1, 4, 1, 5]; let mut s = 0i64; for i in 0usize..5 { s += a[i]; } print_i64(s); }"),
        "14\n"
    );
    assert_eq!(
        run("fn main() { let mut a = [0i64; 4]; a[2] = 9; a[2] += 1; print_i64(a[2] + a[0]); }"),
        "10\n"
    );

    assert_eq!(
        run("fn main() {
                 let mut m = [[0i64; 3]; 2];
                 m[1][2] = 42;
                 let i = 1usize; let j = 2usize;
                 print_i64(m[i][j]); print_i64(m[0][2]);
             }"),
        "42\n0\n"
    );
    assert_eq!(run_trap("fn main() { let a = [1i64; 4]; let i = 4usize; print_i64(a[i]); }"), E_T_OOB);

    assert_eq!(
        run("fn pick(v: [bool; 4], i: usize) -> bool { v[i] }
             fn main() { print_bool(pick([true, false, true, false], 2)); }"),
        "true\n"
    );
}

#[test]
fn branch_literals_decode() {

    assert_eq!(
        run("fn main() { let x = if true { 1 } else { 2 }; print_i64((x) as i64); }"),
        "1\n"
    );
    assert_eq!(
        run("fn main() { let x = match 7i64 { 0 => 10, _ => 20 }; print_i64((x) as i64); }"),
        "20\n"
    );
}

#[test]
fn cast_hint_runtime() {

    assert_eq!(run("fn main() { print_u64(((200 + 100) as u8) as u64); }"), "44\n");
    assert_eq!(
        run("fn main() { print_u64(((if true { 300 } else { 2 }) as u8) as u64); }"),
        "44\n"
    );
    assert_eq!(
        run("fn main() { print_u64(((match 5i64 { 0 => (1i64), _ => (2i64) }) as u8) as u64); }"),
        "2\n"
    );
}

#[test]
fn strings_runtime() {
    assert_eq!(
        run("fn main() { let a = \"x\\ty\"; print_str(a); print_bool(a == \"x\\ty\"); print_bool(a != \"z\"); }"),
        "x\ty\ntrue\ntrue\n"
    );
}

#[test]
fn consts_runtime() {
    assert_eq!(
        run("const HIGH: f64 = 36.0; const D: i64 = 4 * 3600;
             fn main() { print_f64(HIGH); print_i64(D); }"),
        "36\n14400\n"
    );
}

#[test]
fn assignment_order() {

    let src = "
        fn r() -> i64 { print_str(\"rhs\"); 5 }
        fn ix() -> usize { print_str(\"idx\"); 1 }
        fn main() {
            let mut a = [0i64; 2];
            a[ix()] = r();
            print_i64(a[1]);
        }";
    assert_eq!(run(src), "rhs\nidx\n5\n");
}
