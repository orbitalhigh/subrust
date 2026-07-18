
use subrust::ast::*;
use subrust::diag::*;
use subrust::{Mem, MEM_INIT, NODE_NIL};

fn parse_ok(src: &str) -> Box<Mem> {
    let mut mem = Box::new(MEM_INIT);
    let ok = subrust::parse_source(src, &mut mem);
    if !ok && mem.diag_n > 0 {
        let d = mem.diags[0];
        panic!(
            "unexpected parse error {:#06x} at {}..{} in {src:?}",
            d.code, d.span.lo, d.span.hi
        );
    }
    assert!(ok);
    mem
}

fn parse_err(src: &str) -> (u16, Box<Mem>) {
    let mut mem = Box::new(MEM_INIT);
    let ok = subrust::parse_source(src, &mut mem);
    assert!(!ok, "expected a parse error for {src:?}");
    assert!(mem.diag_n > 0);
    (mem.diags[0].code, mem)
}

/// Body block of the item at root position `i` (must be an N_FN).
fn fn_body(mem: &Mem, i: u32) -> u32 {
    let mut it = mem.root_first;
    let mut k = 0;
    while k < i {
        it = mem.node(it).link;
        k += 1;
    }
    let n = mem.node(it);
    assert_eq!(n.kind, N_FN);
    n.e
}

/// nth statement of a block.
fn stmt(mem: &Mem, block: u32, i: u32) -> u32 {
    let b = mem.node(block);
    assert_eq!(b.kind, N_BLOCK);
    let mut it = b.b;
    let mut k = 0;
    while k < i {
        it = mem.node(it).link;
        k += 1;
    }
    it
}

fn tail(mem: &Mem, block: u32) -> u32 {
    let b = mem.node(block);
    assert_eq!(b.kind, N_BLOCK);
    b.e
}

#[test]
fn precedence_mul_over_add() {
    let mem = parse_ok("fn f() -> i32 { 1 + 2 * 3 }");
    let t = tail(&mem, fn_body(&mem, 0));
    let add = mem.node(t);
    assert_eq!(add.kind, N_BINARY);
    assert_eq!(add.x, OP_ADD);
    assert_eq!(mem.node(add.e).x, OP_MUL);
}

#[test]
fn precedence_and_over_or() {
    let mem = parse_ok("fn f(a: bool, b: bool, c: bool) -> bool { a || b && c }");
    let t = tail(&mem, fn_body(&mem, 0));
    let or = mem.node(t);
    assert_eq!(or.x, OP_OR);
    assert_eq!(mem.node(or.e).x, OP_AND);
}

#[test]
fn precedence_shift_below_add() {

    let mem = parse_ok("fn f() -> i32 { 1 + 2 << 3 }");
    let t = tail(&mem, fn_body(&mem, 0));
    let shl = mem.node(t);
    assert_eq!(shl.x, OP_SHL);
    assert_eq!(mem.node(shl.d).x, OP_ADD);
}

#[test]
fn cast_precedence() {

    let mem = parse_ok("fn f(x: i64, y: i32) -> i64 { x + y as i64 }");
    let t = tail(&mem, fn_body(&mem, 0));
    let add = mem.node(t);
    assert_eq!(add.x, OP_ADD);
    assert_eq!(mem.node(add.e).kind, N_CAST);

    let mem = parse_ok("fn g(x: i64) -> i64 { -x as i64 }");
    let t = tail(&mem, fn_body(&mem, 0));
    let cast = mem.node(t);
    assert_eq!(cast.kind, N_CAST);
    assert_eq!(mem.node(cast.d).kind, N_UNARY);

    let mem = parse_ok("fn h(p: &u64) -> u32 { *p as u32 }");
    let t = tail(&mem, fn_body(&mem, 0));
    let cast = mem.node(t);
    assert_eq!(cast.kind, N_CAST);
    assert_eq!(mem.node(cast.d).kind, N_DEREF);

    let mem = parse_ok("fn k() -> i64 { -1 as i64 }");
    let t = tail(&mem, fn_body(&mem, 0));
    let neg = mem.node(t);
    assert_eq!(neg.kind, N_UNARY);
    assert_eq!(mem.node(neg.e).kind, N_CAST);
}

#[test]
fn chained_comparison_rejected() {
    let (code, _) = parse_err("fn f(a: i32, b: i32, c: i32) -> bool { a < b < c }");
    assert_eq!(code, E_CHAINED_COMPARISON);
}

#[test]
fn struct_literal_banned_in_condition_heads() {

    let mem = parse_ok("fn f(S: bool) { if S { } }");
    let body = fn_body(&mem, 0);
    assert_eq!(mem.node(body).c, 0);
    let t = tail(&mem, body);
    let iff = mem.node(t);
    assert_eq!(iff.kind, N_IF);
    assert_eq!(mem.node(iff.d).kind, N_NAME);

    let mem = parse_ok("fn g() -> bool { if (S { y: 1 }).y { true } else { false } }");
    let t = tail(&mem, fn_body(&mem, 0));
    assert_eq!(mem.node(t).kind, N_IF);

    parse_ok("fn h(S: bool) { while S { } }");
    parse_ok("fn i(S: i32) { match S { _ => 0 }; }");
}

#[test]
fn blocklike_statement_terminates_expression() {

    let mem = parse_ok("fn f() -> i32 { {} -1 }");
    let body = fn_body(&mem, 0);
    assert_eq!(mem.node(body).c, 1);
    let s = stmt(&mem, body, 0);
    assert_eq!(mem.node(s).kind, N_EXPR_STMT);
    let t = tail(&mem, body);
    let neg = mem.node(t);
    assert_eq!(neg.kind, N_UNARY);
    assert_eq!(neg.x, OP_NEG);
}

#[test]
fn let_forms() {
    let mem = parse_ok("fn f() { let a = 1; let mut b: i64 = 2; let _ = g(); }");
    let body = fn_body(&mem, 0);
    let a = mem.node(stmt(&mem, body, 0));
    assert_eq!(a.kind, N_LET);
    assert_eq!(a.x & FLAG_MUT, 0);
    assert_eq!(a.d, NODE_NIL);
    let b = mem.node(stmt(&mem, body, 1));
    assert_eq!(b.x & FLAG_MUT, FLAG_MUT);
    assert_ne!(b.d, NODE_NIL);
}

#[test]
fn let_requires_initializer() {

    let (code, _) = parse_err("fn f() { let x; }");
    assert_eq!(code, E_EXPECTED_TOKEN);
}

#[test]
fn assignments() {
    let mem = parse_ok("fn f(mut a: [i64; 4], i: usize) { a[i] = 3; a[0] += 2; s.x = 1; }");
    let body = fn_body(&mem, 0);

    let e0 = mem.node(stmt(&mem, body, 0));
    assert_eq!(e0.kind, N_EXPR_STMT);
    let s0 = mem.node(e0.e);
    assert_eq!(s0.kind, N_ASSIGN);
    assert_eq!(s0.x, 0);
    assert_eq!(mem.node(s0.d).kind, N_INDEX);
    let s1 = mem.node(mem.node(stmt(&mem, body, 1)).e);
    assert_eq!(s1.kind, N_ASSIGN);
    assert_eq!(s1.x, OP_ADD);
    let s2 = mem.node(mem.node(stmt(&mem, body, 2)).e);
    assert_eq!(mem.node(s2.d).kind, N_DOT);
}

#[test]
fn loops_and_ranges() {
    let mem = parse_ok("fn f(n: usize) { for i in 0..10 { } for j in 0..=n { } while true { break; } loop { continue; } }");
    let body = fn_body(&mem, 0);
    let f0 = mem.node(stmt(&mem, body, 0));
    assert_eq!(f0.kind, N_FOR);
    assert_eq!(f0.x & FLAG_INCLUSIVE, 0);
    let f1 = mem.node(stmt(&mem, body, 1));
    assert_eq!(f1.x & FLAG_INCLUSIVE, FLAG_INCLUSIVE);
}

#[test]
fn range_outside_for_rejected() {
    let (code, _) = parse_err("fn f() { 0..4; }");
    assert_eq!(code, E_RANGE_HERE);
}

#[test]
fn break_with_value_rejected() {
    let (code, _) = parse_err("fn f() { loop { break 1; } }");
    assert_eq!(code, E_EXPECTED_TOKEN);
}

#[test]
fn match_arms() {
    let mem = parse_ok(
        "fn f(x: i32) -> i32 { match x { 0 => 1, 1 | 2 => { 3 } -4 => 5, _ => 6, } }",
    );
    let t = tail(&mem, fn_body(&mem, 0));
    let m = mem.node(t);
    assert_eq!(m.kind, N_MATCH);
    assert_eq!(m.c, 4);

    let arm1 = mem.node(mem.node(m.b).link);
    assert_eq!(arm1.kind, N_ARM);
    assert_eq!(arm1.c, 2);

    let arm2 = mem.node(mem.node(mem.node(m.b).link).link);
    assert_eq!(mem.node(arm2.b).x, 1);
}

#[test]
fn match_on_strings() {
    parse_ok("fn f(c: &str) -> bool { match c { \"on\" => true, \"off\" => false, _ => false, } }");
}

#[test]
fn match_comma_rules() {

    let (code, _) = parse_err("fn f(x: i32) -> i32 { match x { 0 => 1 1 => 2, _ => 3 } }");
    assert_eq!(code, E_EXPECTED_TOKEN);

    parse_ok("fn g(x: i32) -> i32 { match x { 0 => { 1 } _ => 2, } }");
}

#[test]
fn float_patterns_rejected() {
    let (code, _) = parse_err("fn f(x: f64) -> i32 { match x { 1.0 => 1, _ => 0 } }");
    assert_eq!(code, E_EXPECTED_PATTERN);
}

#[test]
fn struct_and_derives() {
    let mem = parse_ok("#[derive(Clone, Copy)] struct S { a: i64, b: [bool; 4] }");
    let s = mem.node(mem.root_first);
    assert_eq!(s.kind, N_STRUCT);
    assert_eq!(s.x, DERIVE_CLONE | DERIVE_COPY);
    assert_eq!(s.c, 2);
}

#[test]
fn bad_derive_rejected() {
    let (code, _) = parse_err("#[derive(Debug)] struct S { a: i64 }");
    assert_eq!(code, E_BAD_DERIVE);
}

#[test]
fn attr_on_fn_rejected() {
    let (code, _) = parse_err("#[derive(Clone, Copy)] fn f() { }");
    assert_eq!(code, E_BAD_ATTR);
}

#[test]
fn use_item() {
    let mem = parse_ok("use example_api::zones::*;");
    let u = mem.node(mem.root_first);
    assert_eq!(u.kind, N_USE);
    assert_eq!(u.c, 2);
    assert_eq!(u.x, 1);
}

#[test]
fn const_item() {
    let mem = parse_ok("const HIGH: f64 = 36.0;");
    assert_eq!(mem.node(mem.root_first).kind, N_CONST);
}

#[test]
fn item_recovery_continues() {

    let mut mem = Box::new(MEM_INIT);
    let ok = subrust::parse_source("trait E { } fn ok() { }", &mut mem);
    assert!(!ok);
    assert_eq!(mem.diags[0].code, E_RESERVED_KEYWORD);
    assert_eq!(mem.root_n, 1);
}

#[test]
fn tuples() {

    parse_ok("fn f() { let x = (1, 2); let _ = x; }");
    parse_ok("fn f() { let (a, b) = (1, 2); let _ = a + b; }");
    parse_ok("fn f() { let (mut a, b, _) = (1, 2, 3); a += b; }");

    let (code, _) = parse_err("fn f() { x.0; }");
    assert_eq!(code, E_TUPLE);
    let (code, _) = parse_err("struct S(i64);");
    assert_eq!(code, E_TUPLE);
}

#[test]
fn method_syntax_parses() {

    parse_ok("fn f(x: i64) { x.abs(); }");
    parse_ok("#[derive(Clone, Copy)] struct S { n: i64 }\n\
              impl S { fn get(&self) -> i64 { self.n } fn set(&mut self, v: i64) { self.n = v; } }");
    parse_ok("fn f() { a.b().c(1, 2); }");

    parse_ok("#[derive(Clone, Copy)] struct S { n: i64 }\n\
              impl S { pub fn get(&self) -> i64 { self.n } fn priv2(&self) -> i64 { 0 } }");
}

#[test]
fn return_parses() {

    parse_ok("fn f() -> i32 { return 1; }");
    parse_ok("fn f() { return; }");
    parse_ok("fn f(x: i64) -> i64 { if x < 0 { return -x; } x }");
    parse_ok("fn f() -> i64 { return 1 + 2 * 3; }");
}

#[test]
fn struct_update_rejected() {
    let (code, _) = parse_err("fn f(a: S) -> S { S { x: 1, ..a } }");
    assert_eq!(code, E_STRUCT_UPDATE);
}

#[test]
fn call_target_must_be_a_name() {

    parse_ok("fn f() { (g)(); }");

    let (code, _) = parse_err("fn f() { 1(); }");
    assert_eq!(code, E_CALL_NOT_NAME);
    let (code, _) = parse_err("fn f(a: [i64; 2]) { a[0](); }");
    assert_eq!(code, E_CALL_NOT_NAME);
}

#[test]
fn depth_cap() {
    let mut src = String::from("fn f() { let x = ");
    for _ in 0..80 {
        src.push('(');
    }
    src.push('1');
    for _ in 0..80 {
        src.push(')');
    }
    src.push_str("; }");
    let (code, _) = parse_err(&src);
    assert_eq!(code, E_TOO_DEEP);
}

#[test]
fn if_as_expression() {
    let mem = parse_ok("fn f(c: bool) -> i32 { let x = if c { 1 } else { 2 }; x }");
    let l = mem.node(stmt(&mem, fn_body(&mem, 0), 0));
    assert_eq!(l.kind, N_LET);
    assert_eq!(mem.node(l.e).kind, N_IF);
}

#[test]
fn arrays_and_indexing() {
    let mem = parse_ok("fn f() -> i64 { let a = [1, 2, 3,]; let b = [0i64; 4]; a[2] + b[0] }");
    let body = fn_body(&mem, 0);
    let a = mem.node(stmt(&mem, body, 0));
    let alit = mem.node(a.e);
    assert_eq!(alit.kind, N_ARRAY_LIT);
    assert_eq!(alit.c, 3);
    let b = mem.node(stmt(&mem, body, 1));
    assert_eq!(mem.node(b.e).kind, N_ARRAY_REPEAT);
}

#[test]
fn struct_literals_and_shorthand() {
    let mem = parse_ok("fn f(x: i64) -> P { P { x, y: 2 } }");
    let t = tail(&mem, fn_body(&mem, 0));
    let sl = mem.node(t);
    assert_eq!(sl.kind, N_STRUCT_LIT);
    assert_eq!(sl.c, 2);
    let f0 = mem.node(sl.b);
    assert_eq!(f0.e, NODE_NIL);
    let f1 = mem.node(f0.link);
    assert_ne!(f1.e, NODE_NIL);
}

#[test]
fn unit_and_parens() {
    parse_ok("fn f() { let u = (); let v = (1 + 2) * 3; }");
}

#[test]
fn calls() {
    let mem = parse_ok("fn f() { g(); h(1, \"two\", true,); }");
    let body = fn_body(&mem, 0);
    let c = mem.node(mem.node(stmt(&mem, body, 1)).e);
    assert_eq!(c.kind, N_CALL);
    assert_eq!(c.c, 3);
}
