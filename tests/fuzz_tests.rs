
mod common;

use std::fmt::Write as _;
use std::process::Command;

use subrust::apis::TEST_API;
use subrust::machine::{Instance, INSTANCE_INIT};
use subrust::platform::{Platform, SrErr, SR_OK};
use subrust::{Chk, CHK_INIT, MEM_INIT};

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1))
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Ty {
    I64,
    U32,
    U16,
    I16,
    U8,
    F64,
    Bool,
}

const NUMERIC: [Ty; 6] = [Ty::I64, Ty::U32, Ty::U16, Ty::I16, Ty::U8, Ty::F64];
const SCALARS: [Ty; 7] = [Ty::I64, Ty::U32, Ty::U16, Ty::I16, Ty::U8, Ty::F64, Ty::Bool];

fn ty_name(t: Ty) -> &'static str {
    match t {
        Ty::I64 => "i64",
        Ty::U32 => "u32",
        Ty::U16 => "u16",
        Ty::I16 => "i16",
        Ty::U8 => "u8",
        Ty::F64 => "f64",
        Ty::Bool => "bool",
    }
}

struct Var {
    name: String,
    ty: Ty,
    mutable: bool,
}

struct Case {
    id: usize,
    vars: Vec<Var>,
    arrays: Vec<String>,
    n_names: usize,
}

impl Case {
    fn fresh(&mut self, prefix: &str) -> String {
        self.n_names += 1;
        format!("c{}_{}{}", self.id, prefix, self.n_names)
    }
    fn muts_of(&self, t: Ty) -> Vec<usize> {
        (0..self.vars.len())
            .filter(|&i| self.vars[i].mutable && self.vars[i].ty == t)
            .collect()
    }
}

fn lit(r: &mut Rng, t: Ty) -> String {
    match t {
        Ty::I64 => {
            let v = r.below(2001) as i64 - 1000;
            if v < 0 {
                format!("({v}i64)")
            } else {
                format!("({v}i64)")
            }
        }
        Ty::U32 => format!("({}u32)", r.below(5000)),
        Ty::U16 => format!("({}u16)", r.below(3000)),
        Ty::I16 => {
            let v = r.below(4001) as i64 - 2000;
            format!("({v}i16)")
        }
        Ty::U8 => format!("({}u8)", r.below(16)),
        Ty::F64 => {
            let pool = [
                "0.0", "1.0", "(-1.5)", "0.25", "2.5", "3.5", "10.0", "0.1", "100.5", "(-3.75)",
            ];
            format!("({})", pool[r.below(pool.len() as u64) as usize])
        }
        Ty::Bool => {
            if r.chance(50) {
                "(true)".to_string()
            } else {
                "(false)".to_string()
            }
        }
    }
}

/// A variable of type t, or a literal if none exists.
fn var_or_lit(r: &mut Rng, c: &Case, t: Ty) -> String {
    let cands: Vec<usize> = (0..c.vars.len()).filter(|&i| c.vars[i].ty == t).collect();
    if cands.is_empty() || r.chance(35) {
        return lit(r, t);
    }
    let i = cands[r.below(cands.len() as u64) as usize];
    format!("({})", c.vars[i].name)
}

fn int_op(r: &mut Rng, unsigned_small: bool) -> &'static str {

    let pool: &[&str] = if unsigned_small {
        &["+", "&", "|", "^", "+", "&", "|", ">>", "-", "*", "/", "%"]
    } else {
        &["+", "-", "*", "&", "|", "^", "+", "-", ">>", "<<", "/", "%"]
    };
    pool[r.below(pool.len() as u64) as usize]
}

fn expr(r: &mut Rng, c: &Case, t: Ty, depth: u32) -> String {
    if depth == 0 {
        return var_or_lit(r, c, t);
    }
    match t {
        Ty::Bool => match r.below(4) {
            0 => {

                let ot = NUMERIC[r.below(6) as usize];
                let ops = ["<", "<=", ">", ">=", "==", "!="];
                let op = ops[r.below(6) as usize];
                format!(
                    "(({}) {} ({}))",
                    expr(r, c, ot, depth - 1),
                    op,
                    expr(r, c, ot, depth - 1)
                )
            }
            1 => {
                let ops = ["&&", "||", "&", "|", "^"];
                let op = ops[r.below(5) as usize];
                format!(
                    "(({}) {} ({}))",
                    expr(r, c, Ty::Bool, depth - 1),
                    op,
                    expr(r, c, Ty::Bool, depth - 1)
                )
            }
            2 => format!("(!({}))", expr(r, c, Ty::Bool, depth - 1)),
            _ => var_or_lit(r, c, t),
        },
        Ty::F64 => match r.below(5) {
            0 | 1 => {
                let ops = ["+", "-", "*", "/", "%"];
                let op = ops[r.below(5) as usize];
                format!(
                    "(({}) {} ({}))",
                    expr(r, c, Ty::F64, depth - 1),
                    op,
                    expr(r, c, Ty::F64, depth - 1)
                )
            }
            2 => format!("(-({}))", expr(r, c, Ty::F64, depth - 1)),
            3 => {

                let ot = [Ty::I64, Ty::U32, Ty::U16, Ty::I16, Ty::U8][r.below(5) as usize];
                format!("(({}) as f64)", expr(r, c, ot, depth - 1))
            }
            _ => var_or_lit(r, c, t),
        },
        _ => {

            match r.below(8) {
                0 | 1 | 2 => {
                    let unsigned_small = t == Ty::U8 || t == Ty::U32;
                    let op = int_op(r, unsigned_small);
                    if op == "<<" || op == ">>" {

                        let amt = if r.chance(90) { r.below(7) } else { r.below(70) };
                        return format!("(({}) {} ({}u32))", expr(r, c, t, depth - 1), op, amt);
                    }
                    if op == "/" || op == "%" {

                        let d = expr(r, c, t, depth - 1);
                        let d = if r.chance(50) {
                            format!("(({}) | ({}))", d, one_of(t))
                        } else {
                            d
                        };
                        return format!("(({}) {} ({}))", expr(r, c, t, depth - 1), op, d);
                    }
                    format!(
                        "(({}) {} ({}))",
                        expr(r, c, t, depth - 1),
                        op,
                        expr(r, c, t, depth - 1)
                    )
                }
                3 => {

                    let from = SCALARS[r.below(7) as usize];
                    format!("(({}) as {})", expr(r, c, from, depth - 1), ty_name(t))
                }
                4 => {
                    if t == Ty::I64 {
                        format!("(-({}))", expr(r, c, t, depth - 1))
                    } else {
                        format!("(!({}))", expr(r, c, t, depth - 1))
                    }
                }
                5 => format!(
                    "(if ({}) {{ ({}) }} else {{ ({}) }})",
                    expr(r, c, Ty::Bool, depth - 1),
                    expr(r, c, t, depth - 1),
                    expr(r, c, t, depth - 1)
                ),
                6 => {
                    if t == Ty::I64 && !c.arrays.is_empty() && r.chance(60) {

                        let a = &c.arrays[r.below(c.arrays.len() as u64) as usize];
                        let idx = if r.chance(96) { r.below(4) } else { 4 };
                        format!("({a}[{idx}usize])")
                    } else if t == Ty::I64 {
                        format!(
                            "(match ({}) {{ 0 => ({}), 1 | 2 => ({}), _ => ({}) }})",
                            expr(r, c, Ty::I64, depth - 1),
                            expr(r, c, t, depth - 1),
                            expr(r, c, t, depth - 1),
                            expr(r, c, t, depth - 1)
                        )
                    } else {
                        var_or_lit(r, c, t)
                    }
                }
                _ => var_or_lit(r, c, t),
            }
        }
    }
}

fn one_of(t: Ty) -> &'static str {
    match t {
        Ty::I64 => "1i64",
        Ty::U32 => "1u32",
        Ty::U16 => "1u16",
        Ty::I16 => "1i16",
        Ty::U8 => "1u8",
        _ => "1i64",
    }
}

fn gen_case(r: &mut Rng, id: usize) -> String {
    let mut c = Case {
        id,
        vars: Vec::new(),
        arrays: Vec::new(),
        n_names: 0,
    };
    let mut body = String::new();
    let _ = writeln!(body, "    print_str(\"== case {id}\");");

    let n_stmts = 4 + r.below(5);
    for _ in 0..n_stmts {
        match r.below(10) {

            0..=4 => {
                let t = SCALARS[r.below(7) as usize];
                let name = c.fresh("v");
                let mutable = r.chance(60);
                let d = 1 + r.below(3) as u32;
                let e = expr(r, &c, t, d);
                let m = if mutable { "mut " } else { "" };
                let _ = writeln!(body, "    let {m}{name}: {} = {e};", ty_name(t));
                c.vars.push(Var {
                    name,
                    ty: t,
                    mutable,
                });
            }

            5 => {
                let name = c.fresh("a");
                let e0 = expr(r, &c, Ty::I64, 1);
                let e1 = expr(r, &c, Ty::I64, 1);
                let e2 = expr(r, &c, Ty::I64, 1);
                let e3 = expr(r, &c, Ty::I64, 1);
                let _ = writeln!(body, "    let {name}: [i64; 4] = [{e0}, {e1}, {e2}, {e3}];");
                c.arrays.push(name);
            }

            6 | 7 => {
                let t = SCALARS[r.below(7) as usize];
                let muts = c.muts_of(t);
                if muts.is_empty() {
                    continue;
                }
                let v = &c.vars[muts[r.below(muts.len() as u64) as usize]].name;
                let (op, val) = match t {
                    Ty::Bool => {
                        let ops = ["&=", "|=", "^="];
                        (ops[r.below(3) as usize], expr(r, &c, t, 2))
                    }
                    Ty::F64 => {
                        let ops = ["+=", "-=", "*=", "/="];
                        (ops[r.below(4) as usize], expr(r, &c, t, 2))
                    }
                    _ => {
                        if r.chance(15) {
                            ("<<=", format!("({}u32)", r.below(6)))
                        } else {
                            let ops = ["+=", "-=", "*=", "&=", "|=", "^="];
                            (ops[r.below(6) as usize], expr(r, &c, t, 2))
                        }
                    }
                };
                let _ = writeln!(body, "    {v} {op} {val};");
            }

            8 => {
                let t = SCALARS[r.below(7) as usize];
                let muts = c.muts_of(t);
                if muts.is_empty() {
                    continue;
                }
                let v = &c.vars[muts[r.below(muts.len() as u64) as usize]].name;
                let cond = expr(r, &c, Ty::Bool, 2);
                let val = expr(r, &c, t, 2);
                let alt = expr(r, &c, t, 1);
                if r.chance(50) {
                    let _ = writeln!(body, "    if {cond} {{ {v} = {val}; }}");
                } else {
                    let _ = writeln!(
                        body,
                        "    if {cond} {{ {v} = {val}; }} else {{ {v} = {alt}; }}"
                    );
                }
            }

            _ => {
                let muts = c.muts_of(Ty::I64);
                if muts.is_empty() {
                    continue;
                }
                let v = c.vars[muts[r.below(muts.len() as u64) as usize]].name.clone();
                let iv = c.fresh("i");
                let hi = r.below(9);
                let inclusive = if r.chance(30) { "=" } else { "" };
                let _ = writeln!(
                    body,
                    "    for {iv} in 0i64..{inclusive}{hi}i64 {{ {v} += (({iv}) & (7i64)); }}"
                );
            }
        }
    }

    for v in &c.vars {
        let p = match v.ty {
            Ty::I64 => format!("    print_i64(({}));", v.name),
            Ty::I16 => format!("    print_i64((({}) as i64));", v.name),
            Ty::U32 | Ty::U16 | Ty::U8 => format!("    print_u64((({}) as u64));", v.name),
            Ty::F64 => format!("    print_f64(({}));", v.name),
            Ty::Bool => format!("    print_bool(({}));", v.name),
        };
        let _ = writeln!(body, "{p}");
    }
    for a in &c.arrays {
        let _ = writeln!(body, "    print_i64(({a}[0usize]) + ({a}[3usize]));");
    }

    format!("fn case_{id}() {{\n{body}}}\n")
}

fn gen_batch(seed: u64, n_cases: usize) -> String {
    let mut r = Rng::new(seed);
    let mut src = String::new();
    for id in 0..n_cases {
        src.push_str(&gen_case(&mut r, id));
        src.push('\n');
    }
    src.push_str("fn main() {\n");
    for id in 0..n_cases {
        let _ = writeln!(src, "    case_{id}();");
    }
    src.push_str("}\n");
    src
}

const SHIMS: &str = "
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

fn interp(src: &str) -> (String, bool) {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(src, &mut mem, &mut chk, &TEST_API);
    if !ok {
        let d = if mem.diag_n > 0 { mem.diags[0] } else { subrust::diag::DIAG_NONE };
        panic!(
            "fuzz program failed to check: {:#06x} at {}..{}\n----\n{src}",
            d.code, d.span.lo, d.span.hi
        );
    }
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost {
        chk: &chk,
        out: String::new(),
    };
    let e = subrust::call(src, &mem, &chk, &mut inst, &mut host, "main", &[], 100_000_000);
    (host.out, e != SR_OK)
}

fn compiled(name: &str, src: &str) -> (String, bool) {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let rs = dir.join(format!("fuzz_{name}.rs"));
    let bin = dir.join(format!("fuzz_{name}.bin"));
    std::fs::write(&rs, format!("{src}\n{SHIMS}")).expect("write");
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
        "rustc rejected fuzz batch {name} (L1 violation!):\n{}\n----\n{src}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&bin).output().expect("run");
    (
        String::from_utf8_lossy(&run.stdout).to_string(),
        !run.status.success(),
    )
}

fn fuzz_batch(seed: u64, n_cases: usize) {
    let src = gen_batch(seed, n_cases);
    let (iout, itrap) = interp(&src);
    let (cout, ctrap) = compiled(&format!("s{seed}"), &src);
    if iout != cout || itrap != ctrap {

        let mut line = 0;
        let a: Vec<&str> = iout.lines().collect();
        let b: Vec<&str> = cout.lines().collect();
        while line < a.len() && line < b.len() && a[line] == b[line] {
            line += 1;
        }
        panic!(
            "DIVERGENCE seed {seed}: interp trap={itrap} rustc trap={ctrap}, first diff at output line {line}:\n  interp: {:?}\n  rustc:  {:?}\n---- program ----\n{src}",
            a.get(line),
            b.get(line)
        );
    }
}

/// Fast net, runs in the default suite (3 rustc invocations).
#[test]
fn fuzz_smoke() {
    for seed in 1..=3u64 {
        fuzz_batch(seed, 8);
    }
}

/// Wide sweep, run deliberately: `cargo test fuzz_soak -- --ignored`.
/// Seed count via SUBRUST_FUZZ_SEEDS (default 40) when hunting.
#[test]
#[ignore]
fn fuzz_soak() {
    let n: u64 = std::env::var("SUBRUST_FUZZ_SEEDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);
    for seed in 1..=n {
        fuzz_batch(seed, 12);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum STy {
    U64,
    Bool,
}

fn seed_leaf(r: &mut Rng, vars: &[(String, STy)], ty: STy) -> String {
    let cands: Vec<&String> = vars.iter().filter(|v| v.1 == ty).map(|v| &v.0).collect();
    if ty == STy::Bool {
        if cands.is_empty() || r.chance(40) {
            return if r.chance(50) { "(true)".into() } else { "(false)".into() };
        }
        return format!("({})", cands[r.below(cands.len() as u64) as usize]);
    }
    if cands.is_empty() || r.chance(40) {
        format!("({}u64)", r.below(4096))
    } else {
        format!("({})", cands[r.below(cands.len() as u64) as usize])
    }
}

fn seed_expr(r: &mut Rng, vars: &[(String, STy)], ty: STy, depth: u32) -> String {
    if depth == 0 {
        return seed_leaf(r, vars, ty);
    }
    if ty == STy::Bool {
        return match r.below(3) {
            0 => {
                let ops = ["<", "<=", ">", ">=", "==", "!="];
                let op = ops[r.below(6) as usize];
                format!(
                    "(({}) {} ({}))",
                    seed_expr(r, vars, STy::U64, depth - 1),
                    op,
                    seed_expr(r, vars, STy::U64, depth - 1)
                )
            }
            1 => {
                let ops = ["&&", "||", "&", "|", "^"];
                let op = ops[r.below(5) as usize];
                format!(
                    "(({}) {} ({}))",
                    seed_expr(r, vars, STy::Bool, depth - 1),
                    op,
                    seed_expr(r, vars, STy::Bool, depth - 1)
                )
            }
            _ => format!("(!({}))", seed_expr(r, vars, STy::Bool, depth - 1)),
        };
    }
    match r.below(8) {
        0 | 1 | 2 => {
            let pool = ["+", "&", "|", "^", "+", "&", ">>", "-", "*", "/", "%"];
            let op = pool[r.below(pool.len() as u64) as usize];
            if op == ">>" {
                let amt = if r.chance(92) { r.below(8) } else { r.below(70) };
                return format!("(({}) >> ({}u64))", seed_expr(r, vars, ty, depth - 1), amt);
            }
            if op == "/" || op == "%" {
                let d = seed_expr(r, vars, ty, depth - 1);
                let d = if r.chance(60) {
                    format!("(({}) | (1u64))", d)
                } else {
                    d
                };
                return format!("(({}) {} ({}))", seed_expr(r, vars, ty, depth - 1), op, d);
            }
            if op == "*" {
                return format!(
                    "((({}) & (65535u64)) * (({}) & (4095u64)))",
                    seed_expr(r, vars, ty, depth - 1),
                    seed_expr(r, vars, ty, depth - 1)
                );
            }
            format!(
                "(({}) {} ({}))",
                seed_expr(r, vars, ty, depth - 1),
                op,
                seed_expr(r, vars, ty, depth - 1)
            )
        }
        3 => format!(
            "(if ({}) {{ ({}) }} else {{ ({}) }})",
            seed_expr(r, vars, STy::Bool, depth - 1),
            seed_expr(r, vars, ty, depth - 1),
            seed_expr(r, vars, ty, depth - 1)
        ),
        4 => {
            let a = if r.chance(97) {
                r.below(256)
            } else {
                (1u64 << 20) + r.below(4)
            };
            format!("(ld({a}u64))")
        }
        5 => {
            let ops = ["f_add", "f_sub", "f_mul", "f_div"];
            let op = ops[r.below(4) as usize];
            format!(
                "({op}(f_from_i({}), f_from_i({})))",
                seed_leaf(r, vars, STy::U64),
                seed_leaf(r, vars, STy::U64)
            )
        }
        _ => seed_leaf(r, vars, ty),
    }
}

fn gen_seed_case(r: &mut Rng, id: usize) -> String {
    let mut vars: Vec<(String, STy)> = Vec::new();
    let mut muts: Vec<usize> = Vec::new();
    let mut stores: Vec<u64> = Vec::new();
    let mut body = String::new();
    let n = 4 + r.below(5);
    for k in 0..n {
        match r.below(8) {
            0..=3 => {
                let ty = if r.chance(75) { STy::U64 } else { STy::Bool };
                let name = format!("c{id}_v{k}");
                let d = 1 + r.below(3) as u32;
                let e = seed_expr(r, &vars, ty, d);
                let mutable = r.chance(60);
                let m = if mutable { "mut " } else { "" };
                let t = if ty == STy::U64 { "u64" } else { "bool" };
                let _ = writeln!(body, "    let {m}{name}: {t} = {e};");
                if mutable {
                    muts.push(vars.len());
                }
                vars.push((name, ty));
            }
            4 | 5 => {
                if muts.is_empty() {
                    continue;
                }
                let vi = muts[r.below(muts.len() as u64) as usize];
                let name = vars[vi].0.clone();
                let ty = vars[vi].1;
                if ty == STy::U64 {
                    let ops = ["+=", "-=", "*=", "&=", "|=", "^="];
                    let op = ops[r.below(6) as usize];
                    let e = seed_expr(r, &vars, STy::U64, 2);
                    let _ = writeln!(body, "    {name} {op} (({e}) & (65535u64));");
                } else {
                    let ops = ["&=", "|=", "^="];
                    let op = ops[r.below(3) as usize];
                    let e = seed_expr(r, &vars, STy::Bool, 2);
                    let _ = writeln!(body, "    {name} {op} {e};");
                }
            }
            6 => {
                let a = id as u64 * 64 + r.below(32);
                let e = seed_expr(r, &vars, STy::U64, 2);
                let _ = writeln!(body, "    st({a}u64, {e});");
                stores.push(a);
            }
            _ => {
                let i = format!("c{id}_i{k}");
                let acc = format!("c{id}_a{k}");
                let hi = r.below(8);
                let e = seed_expr(r, &vars, STy::U64, 2);
                let _ = writeln!(body, "    let mut {acc}: u64 = 0;");
                let _ = writeln!(body, "    let mut {i}: u64 = 0;");
                let _ = writeln!(
                    body,
                    "    while {i} < {hi}u64 {{ {acc} += (({e}) & (4095u64)); {i} += 1; }}"
                );
                vars.push((acc, STy::U64));
                vars.push((i, STy::U64));
            }
        }
    }
    for (name, ty) in &vars {
        if *ty == STy::U64 {
            let _ = writeln!(body, "    putw(({name}));");
        } else {
            let _ = writeln!(body, "    if {name} {{ putb(84); }} else {{ putb(70); }}");
        }
    }
    for a in &stores {
        let _ = writeln!(body, "    putw(ld({a}u64));");
    }
    format!("fn case_{id}() {{\n{body}}}\n")
}

fn gen_seed_batch(seed: u64, n_cases: usize) -> String {
    let mut r = Rng::new(seed ^ 0x5EED_5EED);
    let mut src = String::from(
        "fn putw(v: u64) {\n    putb(v >> 56); putb(v >> 48); putb(v >> 40); putb(v >> 32);\n    putb(v >> 24); putb(v >> 16); putb(v >> 8); putb(v);\n}\n\n",
    );
    for id in 0..n_cases {
        src.push_str(&gen_seed_case(&mut r, id));
        src.push('\n');
    }
    src.push_str("fn main() {\n");
    for id in 0..n_cases {
        let _ = writeln!(src, "    case_{id}();");
    }
    src.push_str("}\n");
    src
}

fn seed_fuzz_batch(seed: u64, n_cases: usize) {
    let src = gen_seed_batch(seed, n_cases);
    let (iout, itrap) = common::interp_boot(&src, &[]);
    let (cout, ctrap) = common::compiled_boot(&format!("fz{seed}"), &src, &[]);
    if iout != cout || itrap != ctrap {
        panic!(
            "SEED DIVERGENCE seed {seed}: interp trap={itrap} rustc trap={ctrap}\n interp: {:02x?}\n rustc:  {:02x?}\n---- program ----\n{src}",
            &iout[..iout.len().min(64)],
            &cout[..cout.len().min(64)]
        );
    }
}

#[test]
fn seed_fuzz_smoke() {
    for seed in 1..=3u64 {
        seed_fuzz_batch(seed, 6);
    }
}

/// Wider SR-seed sweep: `cargo test seed_fuzz_soak -- --ignored`
#[test]
#[ignore]
fn seed_fuzz_soak() {
    let n: u64 = std::env::var("SUBRUST_FUZZ_SEEDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);
    for seed in 1..=n {
        seed_fuzz_batch(seed, 10);
    }
}
