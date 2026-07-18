
mod emit;

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::exit;

use subrust::apis::{HVAC_API, TEST_API};
use subrust::ast::*;
use subrust::check::*;
use subrust::diag::*;
use subrust::lex::{tok_label, T_EOF};
use subrust::machine::{Instance, INSTANCE_INIT};
use subrust::platform::{HostDef, Platform, SrErr, SR_OK};
use subrust::{Mem, CHK_INIT, EMPTY_API, MEM_INIT, NODE_NIL};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: subrust <lex|ast|check|run> <file.rs> [api: none|test|hvac]");
        exit(2);
    }
    let cmd = args[1].as_str();
    let path = args[2].as_str();
    let default_api = if cmd == "run" { "test" } else { "none" };
    let api: &'static HostDef = match args.get(3).map(|s| s.as_str()).unwrap_or(default_api) {
        "none" => &EMPTY_API,
        "test" => &TEST_API,
        "hvac" => &HVAC_API,
        "boot" => &subrust::apis::BOOT_API,
        other => {
            eprintln!("error: unknown api {other:?} (none|test|hvac)");
            exit(2);
        }
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            exit(2);
        }
    };

    let mut mem: Box<Mem> = Box::new(MEM_INIT);
    let mut out = String::new();

    match cmd {
        "lex" => {
            let ok = subrust::lex_source(&src, &mut mem);
            dump_tokens(&mut out, &src, &mem);
            finish(out, path, &src, &mem, ok);
        }
        "ast" => {
            let ok = subrust::parse_source(&src, &mut mem);
            if ok {
                dump_root(&mut out, &src, &mem);
            }
            finish(out, path, &src, &mem, ok);
        }
        "check" => {
            let mut chk: Box<Chk> = Box::new(CHK_INIT);
            let ok = subrust::check_source(&src, &mut mem, &mut chk, api);
            if ok {
                let _ = writeln!(out, "ok: {} function(s), {} struct(s), {} const(s)",
                    chk.fn_n, chk.struct_n, chk.const_n);
            }
            render_diags_ck(path, &src, &mem, Some((&chk, api)));
            let _ = std::io::stdout().write_all(out.as_bytes());
            if !ok {
                exit(1);
            }
        }
        "bench" => {
            let iters: u64 = args
                .get(4)
                .and_then(|s| s.parse().ok())
                .unwrap_or(20_000);
            bench(path, &src, &mut mem, api, args.get(3).map(|s| s.as_str()).unwrap_or("none"), iters);
        }
        "emit" => {
            let mut chk: Box<Chk> = Box::new(CHK_INIT);
            let ok = subrust::check_source(&src, &mut mem, &mut chk, api);
            if !ok {
                render_diags_ck(path, &src, &mem, Some((&chk, api)));
                exit(1);
            }
            match emit::emit_image(&src, &mem, &chk) {
                Ok(bytes) => {
                    let _ = std::io::stdout().write_all(&bytes);
                }
                Err(msg) => {
                    eprintln!("{msg}");
                    exit(1);
                }
            }
        }
        "run" => {
            let mut chk: Box<Chk> = Box::new(CHK_INIT);
            let ok = subrust::check_source(&src, &mut mem, &mut chk, api);
            if !ok {
                render_diags_ck(path, &src, &mem, Some((&chk, api)));
                exit(1);
            }
            let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
            let mut host = StdHost { chk: &chk };
            let e = subrust::call(&src, &mem, &chk, &mut inst, &mut host, "main", &[],
                                  100_000_000);
            if e != SR_OK {
                let d = Diag {
                    code: inst.trap_code,
                    span: Span { lo: inst.trap_lo, hi: inst.trap_hi },
                    a: 0,
                    b: 0,
                };
                render_one(path, &src, &mem, d, Some((&chk, api)));
                if inst.trap_code == E_T_ASSERT && inst.trap_msg != u32::MAX {
                    let b = chk.str_bytes(inst.trap_msg);
                    eprintln!("  message: {}", String::from_utf8_lossy(b));
                }
                exit(1);
            }
        }
        _ => {
            eprintln!("usage: subrust <lex|ast|check|run> <file.rs>");
            exit(2);
        }
    }
}

/// Canned HVAC host: fixed responses, no allocation, no string resolution —
/// approximates a free host so the interpreter itself is measured.
struct BenchHvacHost {
    calls: u64,
}

impl Platform for BenchHvacHost {
    fn host_call(&mut self, id: u16, _args: &[u64], ret: &mut [u64]) -> SrErr {
        self.calls += 1;
        match id {
            0 => {

                ret[0] = 30.5f64.to_bits();
                ret[1] = 5;
                ret[2] = 1;
            }
            2 => {

                ret[0] = 1;
                ret[1] = 0;
                ret[2] = 0;
                ret[3] = 0;
                ret[4] = 4;
                ret[5] = 1;
            }
            5 => {

                ret[0] = 3;
                ret[1] = 900;
            }
            _ => {}
        }
        SR_OK
    }
}

/// Silent test host (print_* swallowed) for benching compute scripts.
struct NullHost {
    calls: u64,
}

impl Platform for NullHost {
    fn host_call(&mut self, _id: u16, _args: &[u64], _ret: &mut [u64]) -> SrErr {
        self.calls += 1;
        SR_OK
    }
}

fn bench(path: &str, src: &str, mem: &mut Mem, api: &HostDef, api_name: &str, iters: u64) {
    let mut chk: Box<Chk> = Box::new(CHK_INIT);
    let t0 = std::time::Instant::now();
    let ok = subrust::check_source(src, mem, &mut chk, api);
    let load = t0.elapsed();
    if !ok {
        render_diags_ck(path, src, mem, Some((&chk, api)));
        exit(1);
    }
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    const FUEL: u64 = 100_000_000;

    let hvac = api_name == "hvac";
    let entry = if hvac { "tick" } else { "main" };
    let mut argv: Vec<u64> = Vec::new();
    if hvac {

        argv.extend_from_slice(&[1_000_000, 1, 2026, 7, 16, 14, 0, 0]);
        argv.extend_from_slice(&[0, (-1i64) as u64, (-1i64) as u64]);
    }
    let mut mh = BenchHvacHost { calls: 0 };
    let mut nh = NullHost { calls: 0 };

    let mut run_once = |inst: &mut Instance| -> (SrErr, u64) {
        let host: &mut dyn Platform = if hvac { &mut mh } else { &mut nh };
        let e = subrust::call(src, mem, &chk, inst, host, entry, &argv, FUEL);
        (e, FUEL - inst.fuel)
    };

    let (e, steps) = run_once(&mut inst);
    if e != SR_OK {
        eprintln!("bench: {entry} failed: err {e}, trap {:#06x}", inst.trap_code);
        exit(1);
    }
    for _ in 0..100 {
        run_once(&mut inst);
    }

    let t1 = std::time::Instant::now();
    for _ in 0..iters {
        run_once(&mut inst);
    }
    let dt = t1.elapsed();

    let per_call_ns = dt.as_nanos() as f64 / iters as f64;
    let per_step_ns = per_call_ns / steps as f64;
    let hosts = if hvac { mh.calls } else { nh.calls };
    println!("load (lex+parse+check): {:.1} us", load.as_nanos() as f64 / 1000.0);
    println!(
        "{entry}: {iters} hot calls in {:.1} ms -> {:.2} us/call ({:.0} calls/s)",
        dt.as_secs_f64() * 1000.0,
        per_call_ns / 1000.0,
        1e9 / per_call_ns
    );
    println!(
        "steps/call: {steps} -> {:.1} ns/step; host calls total: {hosts}",
        per_step_ns
    );
}

/// The TEST_API host on real stdout (print_* by id order).
struct StdHost<'a> {
    chk: &'a Chk,
}

impl<'a> Platform for StdHost<'a> {
    fn host_call(&mut self, id: u16, args: &[u64], _ret: &mut [u64]) -> SrErr {
        match id {
            0 => println!("{}", args[0] as i64),
            1 | 2 => println!("{}", args[0]),
            3 => println!("{}", f64::from_bits(args[0])),
            4 => println!("{}", args[0] != 0),
            5 => {
                let b = self.chk.str_bytes(args[0] as u32);
                println!("{}", String::from_utf8_lossy(b));
            }
            _ => return 1,
        }
        SR_OK
    }
}

fn finish(out: String, path: &str, src: &str, mem: &Mem, ok: bool) {

    let _ = std::io::stdout().write_all(out.as_bytes());
    render_diags_ck(path, src, mem, None);
    if !ok {
        exit(1);
    }
}

fn render_diags_ck(path: &str, src: &str, mem: &Mem, ck: Option<(&Chk, &HostDef)>) {
    for i in 0..mem.diag_n {
        let d = mem.diags[i];
        render_one(path, src, mem, d, ck);
    }
    if mem.diag_lost > 0 {
        eprintln!("... and {} more errors not shown", mem.diag_lost);
    }
}

fn render_one(path: &str, src: &str, mem: &Mem, d: Diag, ck: Option<(&Chk, &HostDef)>) {
    let (line, col, line_text) = locate(src, d.span.lo);
    let msg = match ck {
        Some((chk, api)) => message_ck(src, mem, chk, api, d),
        None => message(src, d),
    };
    eprintln!("error[S{:04X}]: {}", d.code, msg);
    eprintln!("  --> {path}:{line}:{col}");
    let ln = format!("{line}");
    let pad = " ".repeat(ln.len());
    eprintln!("{pad} |");
    eprintln!("{ln} | {line_text}");
    let width = (d.span.hi.saturating_sub(d.span.lo)).max(1) as usize;
    let width = width.min(line_text.len().saturating_sub(col - 1).max(1));
    eprintln!("{pad} | {}{}", " ".repeat(col - 1), "^".repeat(width));
}

/// 1-based line/col and the line's text for a byte offset.
fn locate(src: &str, pos: u32) -> (usize, usize, String) {
    let pos = (pos as usize).min(src.len());
    let before = &src[..pos];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col = pos - line_start + 1;
    let line_end = src[pos..]
        .find('\n')
        .map(|i| pos + i)
        .unwrap_or(src.len());
    (line, col, src[line_start..line_end].to_string())
}

/// Compose the message: base text plus expected/found token labels where the
/// code carries them.
fn message(src: &str, d: Diag) -> String {
    match d.code {
        E_EXPECTED_TOKEN => format!(
            "expected {}, found {}",
            tok_label(d.a as u16),
            tok_label(d.b as u16)
        ),
        E_EXPECTED_ITEM => format!(
            "expected an item (`fn`, `struct`, `const` or `use`), found {}",
            tok_label(d.b as u16)
        ),
        E_EXPECTED_EXPR => format!("expected an expression, found {}", tok_label(d.b as u16)),
        E_EXPECTED_TYPE => format!("expected a type, found {}", tok_label(d.b as u16)),
        E_EXPECTED_PATTERN => format!(
            "expected a pattern (integer, string, `true`, `false` or `_`), found {}",
            tok_label(d.b as u16)
        ),
        E_RESERVED_KEYWORD => {
            let lo = d.span.lo as usize;
            let hi = (d.span.hi as usize).min(src.len());
            let word = if lo < hi { &src[lo..hi] } else { "?" };
            format!("`{word}` is a reserved Rust keyword and not in the subrust subset")
        }
        _ => diag_text(d.code).to_string(),
    }
}

/// Check-phase messages, with type names rendered from the checker tables.
fn message_ck(src: &str, mem: &Mem, chk: &Chk, api: &HostDef, d: Diag) -> String {
    let tl = |t: u32| ty_label(src, mem, chk, api, t as u16);
    match d.code {
        E_TYPE_MISMATCH => format!("type mismatch: expected `{}`, found `{}`", tl(d.a), tl(d.b)),
        E_ARG_COUNT => format!("wrong number of arguments: expected {}, found {}", d.a, d.b),
        E_BAD_CAST => format!("cannot cast `{}` to `{}`", tl(d.a), tl(d.b)),
        E_NOT_A_STRUCT => format!("field access on non-struct value of type `{}`", tl(d.b)),
        E_NOT_AN_ARRAY => format!("indexing a non-array value of type `{}`", tl(d.b)),
        E_BAD_OPERAND => format!(
            "operator `{}` not supported for type `{}`",
            op_name(d.a as u16),
            tl(d.b)
        ),
        E_MISSING_FIELD => format!(
            "struct literal is missing fields: expected {}, found {}",
            d.a, d.b
        ),
        E_PATTERN_TYPE => format!(
            "pattern type does not match: the value being matched is `{}`",
            tl(d.a)
        ),
        _ => message(src, d),
    }
}

fn ty_label(src: &str, mem: &Mem, chk: &Chk, api: &HostDef, t: u16) -> String {
    match t {
        TY_ERR => "{error}".to_string(),
        TY_UNIT => "()".to_string(),
        TY_BOOL => "bool".to_string(),
        TY_I8 => "i8".to_string(),
        TY_U8 => "u8".to_string(),
        TY_I16 => "i16".to_string(),
        TY_U16 => "u16".to_string(),
        TY_ISIZE => "isize".to_string(),
        TY_I32 => "i32".to_string(),
        TY_U32 => "u32".to_string(),
        TY_I64 => "i64".to_string(),
        TY_U64 => "u64".to_string(),
        TY_USIZE => "usize".to_string(),
        TY_F64 => "f64".to_string(),
        TY_STR => "&str".to_string(),
        TY_INTLIT => "{integer}".to_string(),
        _ => {
            if ty_is_struct(t) {
                let s = chk.sinfo(t);
                if s.host > 0 {
                    return api.structs[(s.host - 1) as usize].name.to_string();
                }
                return tok_text(src, mem.tok(s.name_tok)).to_string();
            }
            if ty_is_arr(t) {
                let a = chk.ainfo(t);
                return format!("[{}; {}]", ty_label(src, mem, chk, api, a.elem), a.len);
            }
            if ty_is_enum(t) {
                return tok_text(src, mem.tok(chk.einfo(t).name_tok)).to_string();
            }
            format!("ty#{t:04x}")
        }
    }
}

fn dump_tokens(out: &mut String, src: &str, mem: &Mem) {
    for i in 0..mem.tok_n {
        let t = mem.toks[i];
        if t.kind == T_EOF {
            let _ = writeln!(out, "{i:5}  eof");
            continue;
        }
        let text = tok_text(src, t);
        let text = if text.len() > 32 { &text[..32] } else { text };
        let _ = writeln!(
            out,
            "{i:5}  {:<18} {:>6}+{:<4} {}",
            tok_label(t.kind),
            t.pos,
            t.len,
            text
        );
    }
}

fn dump_root(out: &mut String, src: &str, mem: &Mem) {
    let mut it = mem.root_first;
    while it != NODE_NIL {
        dump_node(out, src, mem, it, 0);
        it = mem.node(it).link;
    }
}

fn name_of(src: &str, mem: &Mem, ti: u32) -> String {
    tok_text(src, mem.tok(ti)).to_string()
}

fn dump_node(out: &mut String, src: &str, mem: &Mem, idx: u32, depth: usize) {
    if idx == NODE_NIL {
        return;
    }
    let n = mem.node(idx);
    let ind = "  ".repeat(depth);
    let label = node_name(n.kind);
    let info = match n.kind {
        N_FN | N_STRUCT | N_CONST | N_PARAM | N_FIELD | N_NAME | N_CALL | N_DOT | N_LET
        | N_FOR | N_STRUCT_LIT | N_FIELD_INIT | N_USE_SEG | N_TY_NAME => {
            format!(" {}", name_of(src, mem, n.a))
        }
        N_LIT_INT | N_LIT_FLOAT | N_LIT_STR | N_PAT_INT | N_PAT_STR => {
            format!(" {}", name_of(src, mem, n.a))
        }
        N_LIT_BOOL | N_PAT_BOOL => format!(" {}", if n.x == 1 { "true" } else { "false" }),
        N_UNARY | N_BINARY => format!(" {}", op_name(n.x)),
        N_ASSIGN => {
            if n.x == 0 {
                " =".to_string()
            } else {
                format!(" {}=", op_name(n.x))
            }
        }
        _ => String::new(),
    };
    let muts = if (n.kind == N_LET || n.kind == N_PARAM) && n.x & FLAG_MUT != 0 {
        " mut"
    } else {
        ""
    };
    let _ = writeln!(out, "{ind}{label}{muts}{info}");

    match n.kind {
        N_FN => {
            dump_list(out, src, mem, n.b, depth + 1);
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_PARAM | N_FIELD => dump_node(out, src, mem, n.e, depth + 1),
        N_STRUCT => dump_list(out, src, mem, n.b, depth + 1),
        N_CONST => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_USE => dump_list(out, src, mem, n.b, depth + 1),
        N_TY_ARRAY => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_CALL => dump_list(out, src, mem, n.b, depth + 1),
        N_DOT => dump_node(out, src, mem, n.d, depth + 1),
        N_INDEX | N_CAST | N_ARRAY_REPEAT => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_UNARY => dump_node(out, src, mem, n.e, depth + 1),
        N_BINARY | N_ASSIGN => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_STRUCT_LIT => dump_list(out, src, mem, n.b, depth + 1),
        N_FIELD_INIT => dump_node(out, src, mem, n.e, depth + 1),
        N_ARRAY_LIT => dump_list(out, src, mem, n.b, depth + 1),
        N_IF => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
            dump_node(out, src, mem, n.b, depth + 1);
        }
        N_MATCH => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_list(out, src, mem, n.b, depth + 1);
        }
        N_ARM => {
            dump_list(out, src, mem, n.b, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_BLOCK => {
            dump_list(out, src, mem, n.b, depth + 1);
            if n.e != NODE_NIL {
                let _ = writeln!(out, "{ind}  (tail)");
                dump_node(out, src, mem, n.e, depth + 2);
            }
        }
        N_LET => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_EXPR_STMT => dump_node(out, src, mem, n.e, depth + 1),
        N_WHILE => {
            dump_node(out, src, mem, n.d, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        N_LOOP => dump_node(out, src, mem, n.e, depth + 1),
        N_FOR => {
            dump_node(out, src, mem, n.b, depth + 1);
            dump_node(out, src, mem, n.c, depth + 1);
            dump_node(out, src, mem, n.e, depth + 1);
        }
        _ => {}
    }
}

fn dump_list(out: &mut String, src: &str, mem: &Mem, first: u32, depth: usize) {
    let mut it = first;
    while it != NODE_NIL {
        dump_node(out, src, mem, it, depth);
        it = mem.node(it).link;
    }
}
