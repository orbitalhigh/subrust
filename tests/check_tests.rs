
use subrust::apis::HVAC_API;
use subrust::check::*;
use subrust::diag::*;
use subrust::platform::EMPTY_API;
use subrust::{Chk, Mem, CHK_INIT, MEM_INIT};

fn try_check(src: &str) -> (bool, Box<Mem>, Box<Chk>) {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(src, &mut mem, &mut chk, &EMPTY_API);
    (ok, mem, chk)
}

fn check_ok(src: &str) -> (Box<Mem>, Box<Chk>) {
    let (ok, mem, chk) = try_check(src);
    if !ok && mem.diag_n > 0 {
        let d = mem.diags[0];
        panic!(
            "unexpected check error {:#06x} at {}..{} in {src:?}",
            d.code, d.span.lo, d.span.hi
        );
    }
    assert!(ok);
    (mem, chk)
}

fn check_err(src: &str) -> u16 {
    let (ok, mem, _) = try_check(src);
    assert!(!ok, "expected a check error for {src:?}");
    assert!(mem.diag_n > 0);
    mem.diags[0].code
}

#[test]
fn literal_adapts_to_annotation() {
    let (_, chk) = check_ok("fn f() -> i64 { let x: i64 = 1 + 2; x }");
    assert_eq!(chk.fns[0].ret, TY_I64);
}

#[test]
fn tuples() {

    check_ok("fn f(x: i64) -> i64 { let (a, b) = if x > 0 { (1i64, 2i64) } else { (3i64, 4i64) }; a + b }");
    check_ok("fn f() -> i64 { let (a, _, c) = (1u8, true, 9i64); a as i64 + c }");

    assert_eq!(check_err("fn f() { let (a, b) = (1i64, 2i64, 3i64); }"), E_TUPLE);

    assert_eq!(check_err("fn f() { let (a, b) = 5i64; }"), E_TUPLE);

    assert_eq!(
        check_err("fn f(c: bool) { let _ = if c { (1i64, 2i64) } else { (1i64, 2i64, 3i64) }; }"),
        E_TYPE_MISMATCH
    );
}

#[test]
fn let_integer_inference() {

    check_ok("fn g(v: i64) { }  fn f() { let x = 1; g(x); }");

    check_ok("fn f(s: &[u8]) { let mut i = 0; while i < s.len() { i += 1; } }");

    check_ok("fn f(a: &[i64]) -> i64 { let i = 1; a[i] }");

    check_ok("fn g(v: i64){} fn f(){ let x = 1; let y = x + 1; g(y); }");

    check_ok("fn f() { let x = 1; let y = x + 1; let _ = y; }");

    assert_eq!(check_err("fn g(v: i64){} fn f(){ let x=1; let a: i32 = x; g(x); }"), E_TYPE_MISMATCH);

    assert_eq!(check_err("fn g(a: i64, b: u8){} fn f(){ let x=1; g(x, x); }"), E_TYPE_MISMATCH);

    check_ok("fn f(s: &[u64]) -> u64 { let mut acc = 0; let mut i = 0;\n\
              while i < s.len() { acc += s[i]; i += 1; } acc }");

    assert_eq!(
        check_err("fn f(a: &[i64]) -> i64 { let mut i = 0; while i < 3 { i += 1; } a[i] }"),
        E_TYPE_MISMATCH
    );
}

#[test]
fn literal_adapts_across_operands() {

    check_ok("fn f(a: i64) -> bool { a > 4 * 3600 }");
    check_ok("fn f(a: i64) -> bool { 4 * 3600 < a }");
}

#[test]
fn literal_out_of_range() {
    assert_eq!(check_err("fn f() { let x: u8 = 300; }"), E_LIT_OUT_OF_RANGE);
    assert_eq!(check_err("fn f() { let x: i8 = -129; }"), E_LIT_OUT_OF_RANGE);
    check_ok("fn f() { let x: u8 = 255; let y: i8 = -128; }");
}

#[test]
fn min_literals_fold() {

    check_ok("fn f() { let x: i64 = -9223372036854775808; }");
    check_ok("fn f() { let x: i32 = -2147483648; }");
    assert_eq!(
        check_err("fn f() { let x: i32 = -2147483649; }"),
        E_LIT_OUT_OF_RANGE
    );
}

#[test]
fn suffixed_literals() {
    check_ok("fn f() { let x = 5i64; let y = x + 1; }");
    check_ok("fn f() { let a = [0i64; 4]; let v = a[0]; let _ = v; }");
    assert_eq!(check_err("fn f() { let x = 1i64 + 1i32; }"), E_TYPE_MISMATCH);
}

#[test]
fn index_needs_usize() {
    check_ok("fn f(a: [i64; 4]) -> i64 { a[3] }");
    check_ok("fn f(a: [i64; 4], i: usize) -> i64 { a[i] }");
    assert_eq!(
        check_err("fn f(a: [i64; 4], i: i32) -> i64 { a[i] }"),
        E_TYPE_MISMATCH
    );
}

#[test]
fn operator_types() {
    check_ok("fn f(a: f64, b: f64) -> bool { a * 2.0 < b - 0.25 }");
    check_ok("fn f(a: bool, b: bool) -> bool { a && !b || a }");
    check_ok("fn f(a: u32) -> u32 { (a << 3) ^ (a >> 1) & a | 7 }");
    check_ok("fn f(a: i64) -> i64 { a << 2u32 }");
    assert_eq!(check_err("fn f(a: i64, b: f64) -> f64 { a + b }"), E_TYPE_MISMATCH);
    assert_eq!(check_err("fn f(a: bool) -> bool { a + a }"), E_BAD_OPERAND);
    assert_eq!(check_err("fn f(a: f64) -> f64 { a & a }"), E_BAD_OPERAND);
    assert_eq!(check_err("fn f(a: &str, b: &str) -> bool { a < b }"), E_BAD_OPERAND);
    check_ok("fn f(a: &str) -> bool { a == \"x\" }");
}

#[test]
fn negation_rules() {
    check_ok("fn f(a: i64) -> i64 { -a }");
    check_ok("fn f(a: f64) -> f64 { -a }");
    assert_eq!(check_err("fn f(a: u32) -> u32 { -a }"), E_NEG_UNSIGNED);
    assert_eq!(check_err("fn f() { let x: u32 = -1; }"), E_NEG_UNSIGNED);
}

#[test]
fn condition_must_be_bool() {
    assert_eq!(check_err("fn f(a: i64) { if a { } }"), E_TYPE_MISMATCH);
    assert_eq!(check_err("fn f(a: i64) { while a { } }"), E_TYPE_MISMATCH);
}

#[test]
fn casts() {
    check_ok("fn f(a: i64) -> f64 { a as f64 }");
    check_ok("fn f(a: f64) -> i32 { a as i32 }");
    check_ok("fn f(a: bool) -> i64 { a as i64 }");
    check_ok("fn f(a: u8) -> usize { a as usize }");
    assert_eq!(check_err("fn f(a: &str) -> i64 { a as i64 }"), E_BAD_CAST);
    assert_eq!(check_err("fn f(a: bool) -> f64 { a as f64 }"), E_BAD_CAST);
    assert_eq!(check_err("fn f(a: i64) -> bool { a as bool }"), E_BAD_CAST);
}

#[test]
fn shadowing_is_fine() {
    check_ok("fn f() -> i64 { let x = 1i64; let x = x + 1; x }");
}

#[test]
fn scope_ends() {
    assert_eq!(
        check_err("fn f() -> i64 { { let x = 1i64; } x }"),
        E_UNDEFINED
    );
}

#[test]
fn undefined_and_dup() {
    assert_eq!(check_err("fn f() -> i64 { y }"), E_UNDEFINED);
    assert_eq!(check_err("fn f() { } fn f() { }"), E_DUP_NAME);
    assert_eq!(
        check_err("const A: i64 = 1; fn A() { }"),
        E_DUP_NAME
    );
}

#[test]
fn assignment_mutability() {
    check_ok("fn f() { let mut x = 1; x = 2; x += 3; }");
    assert_eq!(check_err("fn f() { let x = 1; x = 2; }"), E_ASSIGN_IMMUTABLE);

    check_ok("#[derive(Clone, Copy)] struct S { a: i64 } fn f(mut s: S) { s.a = 3; }");
    assert_eq!(
        check_err("#[derive(Clone, Copy)] struct S { a: i64 } fn f(s: S) { s.a = 3; }"),
        E_ASSIGN_IMMUTABLE
    );

    check_ok("fn f(mut a: [bool; 4], i: usize) { a[i] = true; }");
    assert_eq!(
        check_err("fn f(a: [bool; 4]) { a[0] = true; }"),
        E_ASSIGN_IMMUTABLE
    );
}

#[test]
fn assign_targets() {
    assert_eq!(check_err("const A: i64 = 1; fn f() { A = 2; }"), E_ASSIGN_NOT_PLACE);
    assert_eq!(check_err("fn g() -> i64 { 1 } fn f() { g() = 2; }"), E_ASSIGN_NOT_PLACE);
}

#[test]
fn break_outside_loop() {
    assert_eq!(check_err("fn f() { break; }"), E_BREAK_OUTSIDE_LOOP);
    check_ok("fn f() { loop { break; } while true { continue; } }");
}

#[test]
fn struct_basics() {
    let (_, chk) = check_ok(
        "#[derive(Clone, Copy)] struct P { x: i64, y: i64 }
         fn f() -> i64 { let p = P { x: 1, y: 2 }; p.x + p.y }",
    );
    assert_eq!(chk.struct_n, 1);
    assert_eq!(chk.structs[0].size, 2);
}

#[test]
fn struct_requires_derive() {
    assert_eq!(check_err("struct P { x: i64 }"), E_MISSING_DERIVE);
    assert_eq!(
        check_err("#[derive(Clone)] struct P { x: i64 }"),
        E_MISSING_DERIVE
    );
}

#[test]
fn struct_literal_field_rules() {
    let pre = "#[derive(Clone, Copy)] struct P { x: i64, y: i64 } ";
    assert_eq!(
        check_err(&format!("{pre}fn f() -> P {{ P {{ x: 1 }} }}")),
        E_MISSING_FIELD
    );
    assert_eq!(
        check_err(&format!("{pre}fn f() -> P {{ P {{ x: 1, x: 2 }} }}")),
        E_DUP_FIELD
    );
    assert_eq!(
        check_err(&format!("{pre}fn f() -> P {{ P {{ x: 1, z: 2 }} }}")),
        E_UNKNOWN_FIELD
    );
    check_ok(&format!(
        "{pre}fn f() -> P {{ let x = 5i64; P {{ x, y: 2 }} }}"
    ));
}

#[test]
fn nested_structs_layout() {
    let (_, chk) = check_ok(
        "#[derive(Clone, Copy)] struct In { a: i64, b: i64 }
         #[derive(Clone, Copy)] struct Out { p: In, q: i64, r: [In; 2] }
         fn f(o: Out) -> i64 { o.r[1].b + o.p.a + o.q }",
    );
    assert_eq!(chk.structs[0].size, 2);
    assert_eq!(chk.structs[1].size, 7);
}

#[test]
fn recursive_struct_rejected() {
    assert_eq!(
        check_err("#[derive(Clone, Copy)] struct A { a: A }"),
        E_RECURSIVE_STRUCT
    );
}

#[test]
fn str_cannot_be_stored() {
    assert_eq!(
        check_err("#[derive(Clone, Copy)] struct S { name: &str }"),
        E_STR_FIELD
    );
}

#[test]
fn field_on_non_struct() {
    assert_eq!(check_err("fn f(a: i64) -> i64 { a.x }"), E_NOT_A_STRUCT);
}

#[test]
fn arrays() {
    check_ok("fn f() -> [bool; 4] { [true, false, true, false] }");
    check_ok("fn f() -> i64 { let a: [i64; 3] = [1, 2, 3]; a[2] }");
    check_ok("fn f() -> [i64; 4] { [0; 4] }");
    assert_eq!(
        check_err("fn f() { let a: [i64; 4] = [1, 2, 3]; }"),
        E_TYPE_MISMATCH
    );
    assert_eq!(check_err("fn f() { let a = []; }"), E_ANNOTATION_NEEDED);
    assert_eq!(check_err("fn f() { let a = [1, true]; }"), E_TYPE_MISMATCH);
    assert_eq!(check_err("fn f(a: i64) -> i64 { a[0] }"), E_NOT_AN_ARRAY);
}

#[test]
fn array_len_is_const() {
    check_ok("const N: usize = 4; fn f() -> [i64; N] { [7; N] }");
    assert_eq!(
        check_err("fn f(n: usize) { let a = [0i64; n]; }"),
        E_NOT_CONST
    );
}

#[test]
fn match_exhaustiveness() {
    check_ok("fn f(x: i64) -> i64 { match x { 0 => 1, _ => 2 } }");
    check_ok("fn f(b: bool) -> i64 { match b { true => 1, false => 2 } }");
    assert_eq!(
        check_err("fn f(x: i64) -> i64 { match x { 0 => 1, 1 => 2 } }"),
        E_NOT_EXHAUSTIVE
    );
    assert_eq!(
        check_err("fn f(b: bool) -> i64 { match b { true => 1 } }"),
        E_NOT_EXHAUSTIVE
    );
    assert_eq!(
        check_err("fn f(s: &str) -> i64 { match s { \"a\" => 1 } }"),
        E_NOT_EXHAUSTIVE
    );
}

#[test]
fn enums_field_less() {
    let e = "#[derive(Clone, Copy)] enum Mode { Idle, Heating, Purging }\n";

    check_ok(&format!("{e}fn f(m: Mode) -> u64 {{ match m {{ Mode::Idle => 1, Mode::Heating => 2, Mode::Purging => 3 }} }}"));

    check_ok(&format!("{e}fn f(m: Mode) -> u64 {{ match m {{ Mode::Idle => 1, _ => 2 }} }}"));

    assert_eq!(
        check_err(&format!("{e}fn f(m: Mode) -> u64 {{ match m {{ Mode::Idle => 1, Mode::Heating => 2 }} }}")),
        E_NOT_EXHAUSTIVE
    );

    assert_eq!(check_err(&format!("{e}fn f() {{ let _m = Mode::Nope; }}")), E_UNKNOWN_VARIANT);
    assert_eq!(
        check_err(&format!("{e}fn f(m: Mode) -> u64 {{ match m {{ Mode::Nope => 1, _ => 2 }} }}")),
        E_UNKNOWN_VARIANT
    );

    assert_eq!(
        check_err(&format!("{e}#[derive(Clone,Copy)] enum Other {{ A, B }}\nfn f(m: Mode) -> u64 {{ match m {{ Other::A => 1, _ => 2 }} }}")),
        E_PATTERN_TYPE
    );

    assert_eq!(
        check_err("enum Mode { A, B }\nfn f(m: Mode) -> u64 { match m { Mode::A => 1, Mode::B => 2 } }"),
        E_MISSING_DERIVE
    );

    assert_eq!(check_err(&format!("{e}#[derive(Clone,Copy)] enum Mode {{ X, Y }}")), E_DUP_NAME);
    assert_eq!(check_err("#[derive(Clone,Copy)] enum E { A, A }"), E_DUP_NAME);

    assert_eq!(check_err("#[derive(Clone,Copy)] enum E { A(u64), B }"), E_ENUM_PAYLOAD);
}

#[test]
fn asserts() {
    check_ok("fn main() { assert!(true); assert!(1 == 1, \"always\"); }");
    check_ok("fn f(a: u64, b: u64) { assert!(a < b); }");

    assert_eq!(check_err("fn main() { assert!(5); }"), E_TYPE_MISMATCH);

    assert_eq!(check_err("fn main() { let s = \"x\"; assert!(true, s); }"), E_ASSERT_MSG);
    assert_eq!(check_err("fn main() { assert!(true, \"x {}\", 1); }"), E_EXPECTED_TOKEN);

    assert_eq!(check_err("fn main() { foo!(1); }"), E_BAD_MACRO);
    assert_eq!(check_err("fn main() { println!(\"hi\"); }"), E_BAD_MACRO);
}

#[test]
fn match_pattern_types() {
    assert_eq!(
        check_err("fn f(s: &str) -> i64 { match s { 1 => 1, _ => 2 } }"),
        E_PATTERN_TYPE
    );
    assert_eq!(
        check_err("fn f(x: u8) -> i64 { match x { 300 => 1, _ => 2 } }"),
        E_LIT_OUT_OF_RANGE
    );
    assert_eq!(
        check_err("fn f(x: u8) -> i64 { match x { -1 => 1, _ => 2 } }"),
        E_NEG_UNSIGNED
    );
    check_ok("fn f(x: i8) -> i64 { match x { -128 => 1, _ => 2 } }");
}

#[test]
fn match_const_patterns() {

    check_ok("const A: u16 = 1; const B: u16 = 2;\n\
              fn f(k: u16) -> i64 { match k { A => 1, B | 3 => 2, _ => 0 } }");
    check_ok("const HI: &str = \"hi\";\n\
              fn f(s: &str) -> i64 { match s { HI => 1, _ => 0 } }");

    assert_eq!(
        check_err("const A: u16 = 1;\nfn f(k: u16) -> i64 { match k { A => 1 } }"),
        E_NOT_EXHAUSTIVE
    );

    assert_eq!(
        check_err("const A: u8 = 1;\nfn f(k: u16) -> i64 { match k { A => 1, _ => 0 } }"),
        E_PATTERN_TYPE
    );

    assert_eq!(
        check_err("fn f(k: u16) -> i64 { match k { NOPE => 1, _ => 0 } }"),
        E_UNDEFINED
    );

    check_ok("fn f(c: u8) -> i64 { match c { b'(' => 1, b'{' | b'}' => 2, _ => 0 } }");

    assert_eq!(
        check_err("fn f(c: u16) -> i64 { match c { b'(' => 1, _ => 0 } }"),
        E_PATTERN_TYPE
    );
}

#[test]
fn wrapping_methods() {

    check_ok("fn f(a: u8, b: u8) -> u8 { a.wrapping_add(b) }");
    check_ok("fn f(a: i64) -> i64 { a.wrapping_neg() }");
    check_ok("fn f(a: u32, n: u32) -> u32 { a.wrapping_shl(n) }");
    check_ok("fn f(a: i128, b: i128) -> i128 { a.wrapping_mul(b) }");
    check_ok("fn f(a: u8, b: u8) -> u8 { a.saturating_add(b) }");
    check_ok("fn f(a: i64, b: i64) -> i64 { a.saturating_mul(b) }");
    check_ok("fn f(x: u32, n: u32) -> u32 { x.rotate_left(n) }");
    check_ok("fn f(x: u8) -> u8 { x.rotate_right(3) }");
    check_ok("fn f(x: u128) -> u128 { x.rotate_left(40) }");

    check_ok("fn f(a: &[i64; 4]) -> i64 { a[0] }");
    check_ok("fn f(a: &mut [i64; 4]) { a[0] = 1; a[1] += a[0]; }");

    assert_eq!(check_err("fn f(a: &[i64; 4]) { a[0] = 1; }"), E_ASSIGN_IMMUTABLE);

    assert_eq!(check_err("fn f(p: &i64) -> i64 { p[0] }"), E_NOT_AN_ARRAY);

    check_ok("fn f(x: f64) -> u64 { x.to_bits() }");
    check_ok("fn f(x: f64) -> bool { x.is_nan() }");
    check_ok("fn f(b: u64) -> f64 { f64::from_bits(b) }");
    assert_eq!(check_err("fn f(x: f64) -> u64 { x.to_bits(1) }"), E_ARG_COUNT);
    assert_eq!(check_err("fn f(x: f64) -> u64 { x.frobnicate() }"), E_UNKNOWN_METHOD);
    assert_eq!(check_err("fn f() -> f64 { f64::from_bits(1.5) }"), E_TYPE_MISMATCH);
    assert_eq!(check_err("fn f() -> f64 { f64::nonsense(1u64) }"), E_BAD_PATH);

    assert_eq!(check_err("fn f(a: u8) -> u8 { a.wrapping_add() }"), E_ARG_COUNT);
    assert_eq!(check_err("fn f(a: u8) -> u8 { a.wrapping_neg(1) }"), E_ARG_COUNT);
    assert_eq!(check_err("fn f(a: u8) -> u8 { a.frobnicate(1) }"), E_UNKNOWN_METHOD);
    assert_eq!(check_err("fn f(a: bool) -> bool { a.wrapping_add(a) }"), E_NOT_A_STRUCT);

    assert_eq!(check_err("fn f(a: u8, b: u16) -> u8 { a.wrapping_add(b) }"), E_TYPE_MISMATCH);
}

#[test]
fn diverging_loops() {

    check_ok("fn f() -> i64 { loop {} }");
    check_ok("fn f() -> i64 { loop { return 1; } }");
    check_ok("fn f(c: bool) -> i64 { loop { if c { return 1; } } }");
    check_ok("fn f() { loop { break; } }");

    check_ok("fn f(c: bool) -> i64 { loop { while c { break; } return 1; } }");

    assert_eq!(check_err("fn f() -> i64 { loop { break; } }"), E_TYPE_MISMATCH);
    assert_eq!(check_err("fn f(c: bool) -> i64 { loop { if c { break; } return 1; } }"), E_TYPE_MISMATCH);
}

#[test]
fn returns() {

    check_ok("fn f(x: i64) -> i64 { if x < 0 { return 0; } x }");
    check_ok("fn f() -> i64 { return 5; }");
    check_ok("fn f(x: i64) { if x == 0 { return; } }");

    check_ok("fn f(x: i64) -> i64 { let y: i64 = if x > 0 { return x } else { 0 }; y }");
    check_ok("fn f(k: u16) -> i64 { match k { 1 => return 10, _ => 0 } }");
    check_ok("fn f(x: i64) -> i64 { if x > 0 { return x } else { return -x } }");

    assert_eq!(check_err("fn f() -> i64 { return true; }"), E_TYPE_MISMATCH);

    assert_eq!(check_err("fn f() -> i64 { return; }"), E_TYPE_MISMATCH);

    assert_eq!(check_err("fn f() -> u8 { return 300; }"), E_LIT_OUT_OF_RANGE);
}

#[test]
fn match_arm_types_unify() {
    assert_eq!(
        check_err("fn f(x: i64) -> i64 { match x { 0 => 1i64, _ => true } }"),
        E_TYPE_MISMATCH
    );
}

#[test]
fn if_as_value_needs_else() {
    check_ok("fn f(c: bool) -> i64 { if c { 1 } else { 2 } }");
    assert_eq!(
        check_err("fn f(c: bool) -> i64 { let x: i64 = if c { 1 }; x }"),
        E_NO_ELSE
    );

    check_ok("fn f(c: bool) { if c { } }");
}

#[test]
fn branch_types_unify() {
    assert_eq!(
        check_err("fn f(c: bool) -> i64 { if c { 1i64 } else { true } }"),
        E_TYPE_MISMATCH
    );
}

#[test]
fn calls() {
    check_ok("fn g(a: i64, b: bool) -> i64 { if b { a } else { 0 } } fn f() -> i64 { g(5, true) }");
    assert_eq!(
        check_err("fn g(a: i64) { } fn f() { g() }"),
        E_ARG_COUNT
    );
    assert_eq!(
        check_err("fn g(a: i64) { } fn f() { g(true) }"),
        E_TYPE_MISMATCH
    );
    assert_eq!(check_err("fn f() { g(1) }"), E_UNKNOWN_FN);
    assert_eq!(check_err("fn g() { } fn f() { let x = g; }"), E_FN_AS_VALUE);
}

#[test]
fn return_type_enforced() {
    assert_eq!(check_err("fn f() -> i64 { }"), E_TYPE_MISMATCH);
    assert_eq!(check_err("fn f() -> i64 { true }"), E_TYPE_MISMATCH);
    check_ok("fn f() { }");

    check_ok("fn f() -> i64 { g() } fn g() -> i64 { 1 }");
}

#[test]
fn recursion_is_allowed() {
    check_ok("fn fib(n: i64) -> i64 { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }");
}

#[test]
fn consts() {
    let (_, chk) = check_ok("const HIGH: f64 = 36.0; const D: i64 = 4 * 3600; fn f() -> f64 { HIGH }");
    assert_eq!(chk.consts[0].ty, TY_F64);
    assert_eq!(chk.consts[1].bits as i64, 14400);
}

#[test]
fn const_references_const() {
    let (_, chk) = check_ok("const A: i64 = 2; const B: i64 = A * 3; fn f() -> i64 { B }");
    assert_eq!(chk.consts[1].bits as i64, 6);
}

#[test]
fn const_cycle() {
    assert_eq!(
        check_err("const A: i64 = B; const B: i64 = A; fn f() { }"),
        E_CONST_CYCLE
    );
}

#[test]
fn const_overflow_is_compile_error() {
    assert_eq!(
        check_err("const A: i32 = 2147483647 + 1; fn f() { }"),
        E_CONST_OVERFLOW
    );
    assert_eq!(
        check_err("const A: i32 = 1 / 0; fn f() { }"),
        E_CONST_OVERFLOW
    );
}

#[test]
fn const_aggregates_ok() {

    check_ok("#[derive(Clone, Copy)] struct P { x: i64 } const A: P = P { x: 1 }; fn f() { }");
    check_ok("const A: [u8; 3] = [1, 2, 3]; fn f() { }");
    check_ok("const A: [i64; 4] = [7; 4]; fn f() { }");

    check_ok(
        "#[derive(Clone, Copy)] struct P { v: [u8; 2], n: i64 } \
         const N: i64 = 5; const A: P = P { v: [1, 2], n: N }; fn f() { }",
    );
}

#[test]
fn const_ref_still_rejected() {

    assert_eq!(check_err("const A: &i64 = &1; fn f() { }"), E_CONST_TYPE);
}

#[test]
fn const_not_const() {
    assert_eq!(
        check_err("fn g() -> i64 { 1 } const A: i64 = g(); fn f() { }"),
        E_NOT_CONST
    );
}

#[test]
fn for_loops() {
    check_ok("fn f() -> i64 { let mut s = 0i64; for i in 0i64..10 { s = s + i; } s }");
    check_ok("fn f(n: usize) { for _ in 0..n { } }");

    assert_eq!(
        check_err("fn f() { for i in 0..4 { i = 2; } }"),
        E_ASSIGN_IMMUTABLE
    );

    assert_eq!(
        check_err("fn f(a: i64, b: u32) { for _ in a..b { } }"),
        E_TYPE_MISMATCH
    );
    assert_eq!(check_err("fn f() { for x in 0.0..4.0 { } }"), E_TYPE_MISMATCH);
}

#[test]
fn use_rejected() {
    assert_eq!(check_err("use example_api::*; fn f() { }"), E_USE_UNSUPPORTED);
}

#[test]
fn unknown_type() {
    assert_eq!(check_err("fn f(a: Foo) { }"), E_UNKNOWN_TYPE);
}

#[test]
fn expr_stmt_any_type_ok() {

    check_ok("fn g() -> i64 { 1 } fn f() { g(); 1 + 2; }");
}

#[test]
fn frame_layout_slots() {
    let (_, chk) = check_ok(
        "#[derive(Clone, Copy)] struct P { x: i64, y: i64 }
         fn f(a: i64, p: P) -> i64 { let b = a; let q = p; q.y + b }",
    );

    assert_eq!(chk.fns[0].frame, 6);
}

#[test]
fn cast_hint_topology() {

    check_ok("fn f(c: bool) -> u8 { (if c { 300 } else { 2 }) as u8 }");
    check_ok("fn f(y: i64) -> u8 { (match y { 0 => 300, _ => 2 }) as u8 }");
    check_ok("fn f() -> u8 { (200 + 100) as u8 }");
    assert_eq!(check_err("fn f() -> u8 { ({ 300 }) as u8 }"), E_LIT_OUT_OF_RANGE);
    assert_eq!(check_err("fn f() -> i8 { (-(300)) as i8 }"), E_LIT_OUT_OF_RANGE);

    assert_eq!(
        check_err("fn f(c: bool) { let x: u8 = if c { 300 } else { 2 }; }"),
        E_LIT_OUT_OF_RANGE
    );

    check_ok("fn f(y: i64) -> u8 { (match y { 0 => 300i64, _ => 2 }) as u8 }");
}

#[test]
fn branch_literal_unification() {

    check_ok("fn f(c: bool) { let x = if c { 1 } else { 2 }; let _ = x + 1i32; }");
    check_ok("fn f(c: bool) -> i64 { if c { 1 } else { 2i64 } }");
    check_ok("fn f(y: i64) -> i64 { match y { 0 => 1, 1 => 2i64, _ => 3 } }");
    assert_eq!(
        check_err("fn f(c: bool) { let x = if c { 1 } else { 2.5 }; }"),
        E_TYPE_MISMATCH
    );
}

#[test]
fn host_fn_arg_types_enforced() {
    let src = "fn f() { sensor(42); }";
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(src, &mut mem, &mut chk, &HVAC_API);
    assert!(!ok);
    assert_eq!(mem.diags[0].code, E_TYPE_MISMATCH);
}

#[test]
fn host_name_collision() {
    let src = "fn sensor(a: i64) -> i64 { a } fn f() { }";
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(src, &mut mem, &mut chk, &HVAC_API);
    assert!(!ok);
    assert_eq!(mem.diags[0].code, E_DUP_NAME);
}

#[test]
fn assoc_const_paths() {

    check_ok("fn f() -> u64 { u64::MAX }");
    check_ok("fn f() -> i64 { i64::MIN }");
    check_ok("fn f() -> u8 { u8::MAX }");
    check_ok("fn f() -> isize { isize::MIN }");
    check_ok("fn f(a: i32) -> bool { a < i32::MAX }");

    assert_eq!(check_err("fn f() -> u64 { usize::MAX }"), E_TYPE_MISMATCH);

    assert_eq!(check_err("fn f() -> i64 { i64::FOO }"), E_BAD_PATH);
    assert_eq!(check_err("fn f() { let x = Foo::MAX; }"), E_BAD_PATH);

    check_ok("fn f() -> i128 { i128::MAX }");
    check_ok("fn f() -> u128 { u128::MAX }");

    assert_eq!(check_err("fn f() { let x = f64::MAX; }"), E_BAD_PATH);
}

#[test]
fn int128_types() {
    check_ok("fn f() -> u128 { 340282366920938463463374607431768211455u128 }");
    check_ok("fn f() -> i128 { -170141183460469231731687303715884105728i128 }");
    check_ok("fn f(a: u128, b: u128) -> u128 { a * b + 1u128 }");
    check_ok("fn f(a: i128, b: i128) -> bool { a < b }");
    check_ok("fn f(a: u128) -> u64 { (a >> 64) as u64 }");
    check_ok("fn f(a: u64) -> u128 { a as u128 }");
    check_ok("fn f(a: u128) -> u128 { a << 100u32 }");

    assert_eq!(
        check_err("fn f() -> u128 { 340282366920938463463374607431768211456u128 }"),
        E_LIT_OUT_OF_RANGE
    );

    assert_eq!(check_err("const A: u128 = 1u128; fn f() { }"), E_NOT_CONST);

    assert_eq!(check_err("fn f(a: u128, b: u64) -> u128 { a + b }"), E_TYPE_MISMATCH);
}

#[test]
fn references() {

    check_ok("fn f(p: &i64) -> i64 { *p } fn main() { let x: i64 = 1; let _ = f(&x); }");
    check_ok("fn f(a: &i64, b: &u64) -> i64 { *a } fn g() { }");
    check_ok("#[derive(Clone, Copy)] struct S { x: i64 } fn f(p: &S) -> i64 { (*p).x }");
    check_ok("fn f(p: &mut i64) { } fn main() { let mut x: i64 = 1; f(&mut x); }");

    check_ok("fn main() { let x: i64 = 1; let r: &i64 = &x; let _ = *r; }");

    check_ok("fn f(p: &i64) -> &i64 { p }");
    assert_eq!(check_err("fn f() -> &i64 { let x: i64 = 1; &x }"), E_REF_ESCAPES);

    assert_eq!(check_err("#[derive(Clone, Copy)] struct S { p: &i64 }"), E_REF_ESCAPES);
    assert_eq!(check_err("fn f() { let a: [&i64; 2] = [&1i64, &2i64]; }"), E_REF_ESCAPES);

    assert_eq!(check_err("fn g(p: &mut i64) {} fn main() { let x: i64 = 1; g(&mut x); }"), E_REF_MUT_NEEDED);
    assert_eq!(check_err("fn main() { let x: i64 = 1; let y = *x; }"), E_DEREF_NON_REF);

    assert_eq!(check_err("fn f(p: & &i64) { }"), E_REF_ESCAPES);

    check_ok("fn inc(p: &mut i64) { *p = *p + 1; }");
    check_ok("fn add(p: &mut i64, v: i64) { *p += v; }");
    check_ok("#[derive(Clone, Copy)] struct S { x: i64 } fn f(p: &mut S) { (*p).x = 5; }");
    check_ok("fn z(p: &mut u64) { *p = 0u64; }");

    assert_eq!(check_err("fn f(p: &i64) { *p = 1; }"), E_ASSIGN_IMMUTABLE);
    assert_eq!(check_err("fn f(p: &i64) { *p += 1; }"), E_ASSIGN_IMMUTABLE);
    assert_eq!(check_err("#[derive(Clone, Copy)] struct S { x: i64 } fn f(p: &S) { (*p).x = 5; }"), E_ASSIGN_IMMUTABLE);

    assert_eq!(check_err("fn main() { let mut x: i64 = 1; *x = 2; }"), E_DEREF_NON_REF);
}

#[test]
fn methods() {

    check_ok("#[derive(Clone, Copy)] struct C { n: i64 }\n\
              impl C { fn get(&self) -> i64 { self.n } fn add(&mut self, v: i64) { self.n += v; } }\n\
              fn main() { let mut c = C { n: 1 }; c.add(2); let _ = c.get(); }");
    check_ok("#[derive(Clone, Copy)] struct C { n: i64 }\n\
              impl C { fn dbl(self) -> i64 { self.n * 2 } }\n\
              fn main() { let c = C { n: 3 }; let _ = c.dbl(); }");

    check_ok("#[derive(Clone, Copy)] struct C { n: i64 }\n\
              impl C { fn get(&self) -> i64 { self.n } fn bump(&mut self) { self.n += 1; } }\n\
              fn f(c: &mut C) { c.bump(); } fn g(c: &C) -> i64 { c.get() }");

    assert_eq!(check_err("#[derive(Clone, Copy)] struct C { n: i64 }\n\
                          impl C { fn get(&self) -> i64 { self.n } }\n\
                          fn main() { let c = C { n: 1 }; let _ = c.nope(); }"), E_UNKNOWN_METHOD);

    assert_eq!(check_err("#[derive(Clone, Copy)] struct C { n: i64 }\n\
                          impl C { fn set(&mut self) { self.n = 0; } }\n\
                          fn main() { let c = C { n: 1 }; c.set(); }"), E_REF_MUT_NEEDED);

    assert_eq!(check_err("#[derive(Clone, Copy)] struct C { n: i64 }\n\
                          impl C { fn bad(&self) { self.n = 0; } }"), E_ASSIGN_IMMUTABLE);

    assert_eq!(check_err("fn main() { let x: i64 = 1; let _ = x.foo(); }"), E_UNKNOWN_METHOD);
    assert_eq!(check_err("fn main() { let b: bool = true; let _ = b.foo(); }"), E_NOT_A_STRUCT);

    assert_eq!(check_err("#[derive(Clone, Copy)] struct C { n: i64 }\n\
                          impl C { fn a(&self) -> i64 { 0 } fn a(&self) -> i64 { 1 } }"), E_DUP_NAME);

    assert_eq!(check_err("impl Nope { fn a(&self) {} }"), E_UNKNOWN_TYPE);

    assert_eq!(check_err("#[derive(Clone, Copy)] struct C { n: i64 }\n\
                          impl C { fn a(&self, v: i64) -> i64 { v } }\n\
                          fn main() { let c = C { n: 1 }; let _ = c.a(); }"), E_ARG_COUNT);

    assert_eq!(check_err("#[derive(Clone, Copy)] struct C { n: i64 }\n\
                          impl C { fn a() -> i64 { 0 } }"), E_BAD_RECEIVER);

    check_ok("#[derive(Clone, Copy)] struct C { n: i64 }\n\
              fn get() -> i64 { 0 }\n\
              impl C { fn get(&self) -> i64 { self.n } }\n\
              fn main() { let c = C { n: 1 }; let _ = c.get(); let _ = get(); }");
}

#[test]
fn reborrow_mut_to_shared() {

    check_ok("#[derive(Clone, Copy)] struct P { i: i64 }\n\
              fn peek(p: &P) -> i64 { p.i }\n\
              fn f(p: &mut P) -> i64 { peek(p) }");

    check_ok("fn sum(s: &[i64]) -> i64 { s[0] }\n\
              fn f(a: &mut [i64]) -> i64 { sum(a) }");

    assert_eq!(
        check_err("#[derive(Clone, Copy)] struct P { i: i64 }\n\
                   fn poke(p: &mut P) { p.i = 0; }\n\
                   fn f(p: &P) { poke(p); }"),
        E_TYPE_MISMATCH
    );
}

#[test]
fn slices() {

    check_ok("fn f(s: &[i64]) -> i64 { s[0] }\n\
              fn main() { let a = [1i64, 2, 3]; let _ = f(&a); }");
    check_ok("fn f(s: &[u64]) -> usize { s.len() }\n\
              fn main() { let a = [1u64, 2]; let _ = f(&a); let _ = a.len(); }");
    check_ok("fn sum(s: &[i64]) -> i64 { let mut t: i64 = 0; let mut i: usize = 0;\n\
              while i < s.len() { t += s[i]; i += 1; } t }");

    check_ok("fn f(s: &[i64]) -> i64 { let w = s; w[0] }");

    check_ok("fn f(s: &[i64]) -> &[i64] { s }");
    check_ok("fn f(s: &[i64]) -> &[i64] { &s[1..3] }");
    check_ok("fn f(s: &[u8]) -> &[u8] { if s.len() > 2 { &s[0..1] } else { s } }");

    check_ok("fn f(s: &[u8]) -> &[u8] { if s.len() > 2 { &s[0..1] } else { &[] } }");
    check_ok("fn f() { let e: &[i64] = &[]; let _ = e.len(); }");

    assert_eq!(check_err("fn f() { let e = &[]; let _ = e; }"), E_ANNOTATION_NEEDED);

    check_ok("fn f(s: &[i64]) -> &[i64] { let w = s; w }");

    assert_eq!(check_err("fn f() -> &[i64] { let a = [1i64, 2, 3]; &a[0..2] }"), E_REF_ESCAPES);
    assert_eq!(check_err("#[derive(Clone, Copy)] struct S { s: &[i64] }"), E_REF_ESCAPES);

    assert_eq!(check_err("fn f(s: &[i64]) -> i64 { s.first() }"), E_UNKNOWN_METHOD);

    assert_eq!(check_err("fn f(s: &[i64]) { s[0] = 1; }"), E_ASSIGN_IMMUTABLE);
    check_ok("fn f(s: &mut [i64]) { s[0] = 1; s[1] += 2; }");

    assert_eq!(check_err("fn f(s: &[i64]) -> i64 { s[0] }\n\
                          fn main() { let x: i64 = 5; let _ = f(&x); }"), E_TYPE_MISMATCH);

    check_ok("fn f(s: &[i64]) -> i64 { s[0] }\n\
              fn main() { let a = [1i64, 2, 3]; let _ = f(&a[1..3]); }");
    check_ok("fn g(s: &[u8]) -> usize { s.len() }\n\
              fn f(s: &[u8]) -> usize { g(&s[1..2]) }");
    check_ok("fn h(x: &mut [i64]) {} fn f(s: &mut [i64]) { h(&mut s[0..1]); }");

    assert_eq!(check_err("fn f(s: &[i64]) { let x = s[0..1]; }"), E_SUBSLICE_REF);

    assert_eq!(check_err("fn h(x: &mut [i64]) {} fn f(s: &[i64]) { h(&mut s[0..1]); }"), E_REF_MUT_NEEDED);

    check_ok("fn eat(s: &[u8]) -> usize { s.len() }\n\
              fn main() { let _ = eat(b\"hello\"); let _ = b\"x\"[0]; }");
    check_ok("fn f() -> u8 { b\"abc\"[1] }");

    check_ok("fn f() -> u8 { b'A' }");
    check_ok("fn f(c: u8) -> bool { c == b'0' || c == b'9' }");
    check_ok("fn f() -> i64 { b'\\n' as i64 }");
    assert_eq!(check_err("fn f() -> i32 { b'A' }"), E_TYPE_MISMATCH);

    assert_eq!(check_err("#[derive(Clone, Copy)] struct S { b: &[u8] }"), E_REF_ESCAPES);

    assert_eq!(check_err("fn f(s: &[i64]) -> i64 { s[0] }\n\
                          fn main() { let _ = f(b\"hi\"); }"), E_TYPE_MISMATCH);
}

#[test]
fn str_byte_access() {

    check_ok("fn f(s: &str) -> usize { s.len() }");
    check_ok("fn f(s: &str) -> u8 { s.as_bytes()[0] }");
    check_ok("fn f(s: &str) -> usize { s.as_bytes().len() }");
    check_ok("fn scan(s: &str) -> usize {\n\
              let b = s.as_bytes(); let mut i: usize = 0;\n\
              while i < b.len() { i += 1; } i }");
    check_ok("fn main() { let _ = \"hi\".len(); let _ = \"hi\".as_bytes()[0]; }");

    assert_eq!(
        check_err("fn f(s: &[i64]) -> i64 { s[0] }\n\
                   fn g(t: &str) -> i64 { f(t.as_bytes()) }"),
        E_TYPE_MISMATCH
    );

    assert_eq!(check_err("fn f(s: &str) -> usize { s.trim() }"), E_UNKNOWN_METHOD);
    assert_eq!(check_err("fn f(s: &str) -> usize { s.len(1) }"), E_ARG_COUNT);
}
