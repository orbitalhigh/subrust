#!/usr/bin/env python3
# R2/R3 for the CHECKER: adapt the REAL subrust lexer + parser + checker
# (src/lex.rs, src/parse.rs, src/check.rs) into one standalone subrust program
# (self/check.rs) that subrust itself checks and runs — lexing, parsing, then
# type-checking a sample on subrust.
#
# The lex()/parse()/check() logic is VERBATIM except:
#   - `///`/`//!` doc comments -> `//` (doc comments intentionally rejected)
#   - parse.rs's `tok_is` is dropped: check.rs's identical-signature `tok_is`
#     (which uses tok_bytes, a return-borrow slice) serves both, so no `&str`
#     slicing / tok_text is needed.
#   - check.rs's `bytes_eq` is dropped: the lexer's identical one serves.
#   - `#[derive(Clone, Copy)]` is added to `Chk` (the subset requires it; the
#     real one is heap-boxed and never copied).
# The host boundary is the "quarantined boundary": HostDef/HostStructDef
# /HostFnDef/HostField/HostTy use `&'static [..]`/`&str`, which subrust cannot
# hold in a struct. A fixed-array EMPTY shim replaces them (str names -> ids read
# by host_name -> b""), and check.rs's ~9 host-access sites are rewritten (.len()
# -> ._n, .name.as_bytes() -> host_name(..)). With an empty host those paths are
# dead but must type-check.
import re, os

HERE = os.path.dirname(os.path.abspath(__file__))


def read(name):
    return open(os.path.join(HERE, "..", "src", name)).read()


def strip_docs(s):
    return re.sub(r"^(\s*)//[/!]", r"\1//", s, flags=re.M)


def span(lines, start_pred, brace_from=None):
    start = next(i for i, l in enumerate(lines) if start_pred(l))
    if brace_from is None:
        return "\n".join(lines[start:])
    fn_start = next(i for i, l in enumerate(lines) if brace_from(l))
    depth = 0
    for i in range(fn_start, len(lines)):
        depth += lines[i].count("{") - lines[i].count("}")
        if depth == 0 and i > fn_start:
            return "\n".join(lines[start:i + 1])
    raise RuntimeError("unterminated block")


ast = read("ast.rs").splitlines()
lex = read("lex.rs").splitlines()
parse = read("parse.rs").splitlines()
check = read("check.rs").splitlines()
diag = read("diag.rs")

ecodes = "\n".join(re.findall(r"^pub const E_[A-Z_0-9]+: u16 = 0x[0-9A-Fa-f]+;", diag, re.M))

# AST constants + Node + nd()
ast_start = next(i for i, l in enumerate(ast) if l.startswith("pub const NODE_NIL"))
ast_end = next(i for i, l in enumerate(ast) if l.startswith("pub fn nd("))
adepth = 0
for i in range(ast_end, len(ast)):
    adepth += ast[i].count("{") - ast[i].count("}")
    if adepth == 0 and i > ast_end:
        ast_end = i
        break
ast_block = strip_docs("\n".join(ast[ast_start:ast_end + 1]))

# lexer body: `pub const T_EOF` .. end of lex()  (defines T_*, Tok, helpers,
# bytes_eq, lex())
lex_body = strip_docs(span(lex, lambda l: l.startswith("pub const T_EOF"),
                           lambda l: l.startswith("pub fn lex(src")))

# parser body: `pub const PARSE_DEPTH_CAP` .. EOF, minus its duplicate tok_is
parse_body = strip_docs(span(parse, lambda l: l.startswith("pub const PARSE_DEPTH_CAP")))
TOK_IS_PARSE = """fn tok_is(src: &str, mem: &Mem, ti: u32, s: &[u8]) -> bool {
    let t = mem.tok(ti);
    let b = tok_text(src, t).as_bytes();
    bytes_eq_pub(b, s)
}"""
assert TOK_IS_PARSE in parse_body, "parse tok_is drifted"
parse_body = parse_body.replace(TOK_IS_PARSE, "// (tok_is provided by the checker body)")

# checker body: `pub const TY_ERR` .. EOF
check_body = strip_docs(span(check, lambda l: l.startswith("pub const TY_ERR")))
# drop the duplicate bytes_eq (identical to the lexer's)
BYTES_EQ = """fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}"""
assert BYTES_EQ in check_body, "check bytes_eq drifted"
check_body = check_body.replace(BYTES_EQ, "// (bytes_eq provided by the lexer body)")
# drop str_span: a machine-only Chk method (returns a tuple, unused by check())
STR_SPAN = """    pub fn str_span(&self, idx: u32) -> (u64, u64) {
        let idx = idx as usize;
        if idx >= self.str_n {
            return (0, 0);
        }
        let e = self.strs[idx];
        (e.off as u64, e.len as u64)
    }"""
assert STR_SPAN in check_body, "str_span drifted"
check_body = check_body.replace(STR_SPAN, "    // (str_span: machine-only, dropped — tuples are out of subset)")
# `u128::from(x)` (integer widening `From`, not in the subset) == `x as u128`.
# These are in the 128-bit arithmetic, dead for an int-64 sample.
for frm, to in [
    ("u128::from(u64::MAX)", "(u64::MAX as u128)"),
    ("u128::from((1u64 << bits) - 1)", "(((1u64 << bits) - 1) as u128)"),
    ("u128::from(a & (m as u64))", "((a & (m as u64)) as u128)"),
    ("u128::from(b & (m as u64))", "((b & (m as u64)) as u128)"),
    ("u128::from(rm)", "(rm as u128)"),
]:
    assert frm in check_body, "u128::from site drifted: %r" % frm
    check_body = check_body.replace(frm, to)
# a binding match arm `other => other` (subrust has only `_`/literal/const
# patterns). The scrutinee is `inner[1]`, so `_ => inner[1]` is equivalent.
BIND_ARM = "            other => other,\n"
assert BIND_ARM in check_body, "binding arm drifted"
check_body = check_body.replace(BIND_ARM, "            _ => inner[1],\n")
# parse_f64 uses core's correctly-rounded float parser (core::str::from_utf8 +
# .parse::<f64>()) — a std boundary that can't self-host. Stub its tail; the
# sample has no float literals, so it is dead.
F64_TAIL = """    let s = core::str::from_utf8(&buf[..n]).unwrap_or(""); // residue-ok: core parser needed for L2 parity
    match s.parse::<f64>() { // residue-ok: core parser needed for L2 parity
        Ok(v) => v.to_bits(),
        Err(_) => {
            ndiag(mem, E_BAD_NUMBER, at, 0, 0);
            0
        }
    }"""
assert F64_TAIL in check_body, "parse_f64 tail drifted"
check_body = check_body.replace(F64_TAIL,
    "    let _ = n; // float parsing (core's parser) stubbed in the self-hosted checker\n"
    "    ndiag(mem, E_BAD_NUMBER, at, 0, 0);\n    0")
# subrust requires `let` initializers (no deferred init + definite-assignment
# analysis). The three declare-then-assign sites are each behaviour-equivalent to
# a pre-initialized `let mut` (the default is dead — every path assigns before
# use), so pre-initialize them with the right-typed zero value.
for decl, repl in [
    ("        let d;\n", "        let mut d = 0u64;\n"),
    ("        let out;\n", "        let mut out = 0u8;\n"),
    ("            let mode_place;\n", "            let mut mode_place = false;\n"),
    ("        let d: u128;\n", "        let mut d: u128 = 0;\n"),
    # `let mut i = 0; …; i = 2; …; while i < w.len()` — the bare-literal
    # assignment `i = 2` commits i to i32 under subrust's single-pass inference
    # (documented stricture); annotate so it stays usize like the later use wants.
    ("    let mut i = 0;\n    let mut hex = false;\n",
     "    let mut i: usize = 0;\n    let mut hex = false;\n"),
    # parse_f64 counters: `n >= 64` (compare vs literal) and `i` pin these to i32
    # before their usize index uses (`buf[n]`, `w[i]`); annotate them usize.
    ("    let mut buf = [0u8; 64];\n    let mut n = 0;\n    let mut i = 0;\n",
     "    let mut buf = [0u8; 64];\n    let mut n: usize = 0;\n    let mut i: usize = 0;\n"),
]:
    assert decl in check_body, "deferred-let site drifted: %r" % decl
    check_body = check_body.replace(decl, repl)
# structs must be Copy in the subset (the real Chk/CeErr are never copied)
check_body = check_body.replace("pub struct Chk {", "#[derive(Clone, Copy)]\npub struct Chk {")
check_body = check_body.replace("pub struct CeErr {", "#[derive(Clone, Copy)]\npub struct CeErr {")
# host-boundary rewrites (empty fixed-array shim)
check_body = check_body.replace("host.structs.len()", "host.struct_n")
check_body = check_body.replace("host.fns.len()", "host.fn_n")
check_body = check_body.replace(".fields.len()", ".field_n")
for recv in ["host.structs[(s.host - 1) as usize]", "host.fns[i]", "hs.fields[f]"]:
    check_body = check_body.replace(recv + ".name.as_bytes()", "host_name(" + recv + ".name)")
check_body = check_body.replace("t.sname.as_bytes()", "host_name(t.sname)")

SCAFFOLD = """// GENERATED by self/generate_check.py — do not edit by hand.
// R2/R3 self-host: the REAL subrust lexer + parser + checker, adapted to run ON
// subrust: lex, parse, then type-check a sample and report the result.
const SRC_MAX: usize = 16777216;
const FRAME_MAX: u32 = 1024;
// tiny pools: the interpreted Chk + Mem are stack locals in main(), so together
// they must fit subrust's 1024-slot frame (verified by parity_checker_frame_budget)
const CAP_TOKS: usize = 32;
const CAP_NODES: usize = 28;
const CAP_DIAGS: usize = 6;
const CAP_STRUCTS: usize = 2;
const CAP_ENUMS: usize = 2;
const CAP_ARRS: usize = 2;
const CAP_REFS: usize = 2;
const CAP_SLICES: usize = 2;
const CAP_TUPLES: usize = 4;
const CAP_CONSTS: usize = 2;
const CAP_FNS: usize = 4;
const CAP_LOCALS: usize = 8;
const CAP_STRS: usize = 4;
const CAP_STR_POOL: usize = 32;
const CAP_VALS: usize = 16;
"""

HOST_SHIM = """
// ---- host-API boundary shim (empty; the quarantined boundary) ------
const HT_STRUCT: u16 = 0xFFFD;
const HT_ARR: u16 = 0xFFFE;
const HCAP_FIELDS: usize = 1;
const HCAP_PARAMS: usize = 1;
const HCAP_HSTRUCTS: usize = 1;
const HCAP_HFNS: usize = 1;
#[derive(Clone, Copy)]
struct HostTy { kind: u16, sname: u32, elem: u16, len: u32 }
#[derive(Clone, Copy)]
struct HostField { name: u32, ty: HostTy }
#[derive(Clone, Copy)]
struct HostStructDef { name: u32, fields: [HostField; HCAP_FIELDS], field_n: usize }
#[derive(Clone, Copy)]
struct HostFnDef { name: u32, params: [HostTy; HCAP_PARAMS], param_n: usize, ret: HostTy }
#[derive(Clone, Copy)]
struct HostDef {
    structs: [HostStructDef; HCAP_HSTRUCTS], struct_n: usize,
    fns: [HostFnDef; HCAP_HFNS], fn_n: usize,
}
// (host_ty_size is defined by the checker body)
// empty host: no names, so this is dead but must type-check
fn host_name(id: u32) -> &[u8] { let _ = id; b"" }
const H_TY0: HostTy = HostTy { kind: 0, sname: 0, elem: 0, len: 0 };
const H_FIELD0: HostField = HostField { name: 0, ty: H_TY0 };
const H_STRUCT0: HostStructDef = HostStructDef { name: 0, fields: [H_FIELD0; HCAP_FIELDS], field_n: 0 };
const H_FN0: HostFnDef = HostFnDef { name: 0, params: [H_TY0; HCAP_PARAMS], param_n: 0, ret: H_TY0 };
const EMPTY_HOST: HostDef = HostDef {
    structs: [H_STRUCT0; HCAP_HSTRUCTS], struct_n: 0,
    fns: [H_FN0; HCAP_HFNS], fn_n: 0,
};
"""

MEM = """
#[derive(Clone, Copy)]
struct Mem {
    toks: [Tok; CAP_TOKS], tok_n: usize,
    nodes: [Node; CAP_NODES], node_n: usize,
    diags: [Diag; CAP_DIAGS], diag_n: usize,
    diag_lost: u32,
    overflow: bool,
    root_first: u32, root_n: u32,
}
impl Mem {
    fn reset(&mut self) {
        self.tok_n = 0; self.node_n = 0; self.diag_n = 0; self.diag_lost = 0;
        self.overflow = false; self.root_first = NODE_NIL; self.root_n = 0;
    }
    fn diag(&mut self, code: u16, lo: u32, hi: u32, a: u32, b: u32) {
        if self.diag_n < CAP_DIAGS {
            self.diags[self.diag_n] = Diag { code: code, lo: lo, hi: hi, a: a, b: b };
            self.diag_n += 1;
        } else { self.diag_lost += 1; }
    }
    fn push_tok(&mut self, t: Tok) {
        if self.tok_n >= CAP_TOKS {
            if !self.overflow { self.overflow = true; self.diag(E_TOO_MANY_TOKENS, t.pos, t.pos, 0, 0); }
            return;
        }
        self.toks[self.tok_n] = t; self.tok_n += 1;
    }
    fn tok(&self, i: u32) -> Tok {
        let i = i as usize;
        if i < self.tok_n { self.toks[i] } else { TOK_NONE }
    }
    fn push_node(&mut self, n: Node) -> u32 {
        if self.node_n >= CAP_NODES {
            if !self.overflow { self.overflow = true; self.diag(E_TOO_MANY_NODES, n.lo, n.hi, 0, 0); }
            return NODE_NIL;
        }
        self.nodes[self.node_n] = n; self.node_n += 1;
        (self.node_n - 1) as u32
    }
    fn node(&self, i: u32) -> Node {
        let i = i as usize;
        if i < self.node_n { self.nodes[i] } else { NODE_NONE }
    }
    fn set_link(&mut self, i: u32, link: u32) {
        let i = i as usize;
        if i < self.node_n { self.nodes[i].link = link; }
    }
}
#[derive(Clone, Copy)]
struct Diag { code: u16, lo: u32, hi: u32, a: u32, b: u32 }
fn main() {
    let src = "fn f(x: i64) -> i64 { x }";
    let mut m = Mem {
        toks: [TOK_NONE; CAP_TOKS], tok_n: 0,
        nodes: [NODE_NONE; CAP_NODES], node_n: 0,
        diags: [Diag { code: 0, lo: 0, hi: 0, a: 0, b: 0 }; CAP_DIAGS], diag_n: 0,
        diag_lost: 0, overflow: false, root_first: NODE_NIL, root_n: 0,
    };
    let lexed = lex(src, &mut m);
    let parsed = parse(src, &mut m);
    let mut chk = CHK_INIT;
    let host = EMPTY_HOST; // a const is not a place; bind it to reference it
    let ok = check(src, &mut m, &mut chk, &host);
    print_bool(lexed);
    print_bool(parsed);
    print_bool(ok);
    print_usize(m.diag_n);
    print_i64(m.root_n as i64);
}
"""

out = (SCAFFOLD + ecodes + "\n\n" + ast_block + "\n" + HOST_SHIM + "\n"
       + lex_body + "\n" + parse_body + "\n" + check_body + "\n" + MEM)
open(os.path.join(HERE, "check.rs"), "w").write(out)
print("wrote self/check.rs (%d bytes)" % len(out))
