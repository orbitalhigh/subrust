
use crate::ast::*;
use crate::diag::*;
use crate::lex::*;

/// Static nesting cap: bounds parser native recursion (PLAN.md §7.1).
pub const PARSE_DEPTH_CAP: u16 = 64;

/// Parser cursor state. Plain Copy struct passed by `&mut`.
#[derive(Clone, Copy)]
pub struct P {
    pub i: u32,
    pub depth: u16,
    pub no_struct: u16,
}

fn tk(mem: &Mem, p: &P) -> u16 {
    mem.tok(p.i).kind
}

fn tk1(mem: &Mem, p: &P) -> u16 {
    mem.tok(p.i + 1).kind
}

fn cur(mem: &Mem, p: &P) -> Tok {
    mem.tok(p.i)
}

fn bump(p: &mut P) {
    p.i += 1;
}

fn at(mem: &Mem, p: &P, k: u16) -> bool {
    tk(mem, p) == k
}

fn accept(mem: &Mem, p: &mut P, k: u16) -> bool {
    if tk(mem, p) == k {
        bump(p);
        return true;
    }
    false
}

fn expect(mem: &mut Mem, p: &mut P, k: u16) -> bool {
    if accept(mem, p, k) {
        return true;
    }
    let t = cur(mem, p);
    mem.diag(
        E_EXPECTED_TOKEN,
        t.pos,
        t.pos + t.len as u32,
        k as u32,
        t.kind as u32,
    );
    false
}

/// Expect an identifier; returns its token index or NODE_NIL.
/// `allow_underscore` admits `_` (let/for bindings).
fn expect_ident(mem: &mut Mem, p: &mut P, allow_underscore: bool) -> u32 {
    let k = tk(mem, p);
    if k == T_IDENT || (allow_underscore && k == T_UNDERSCORE) {
        let i = p.i;
        bump(p);
        return i;
    }
    let t = cur(mem, p);
    if k == T_KW_RESERVED {
        mem.diag(E_RESERVED_KEYWORD, t.pos, t.pos + t.len as u32, 0, 0);
    } else {
        mem.diag(
            E_EXPECTED_TOKEN,
            t.pos,
            t.pos + t.len as u32,
            T_IDENT as u32,
            k as u32,
        );
    }
    NODE_NIL
}

/// End offset of the last consumed token (for parent spans).
fn prev_end(mem: &Mem, p: &P) -> u32 {
    if p.i == 0 {
        return 0;
    }
    let t = mem.tok(p.i - 1);
    t.pos + t.len as u32
}

fn enter(mem: &mut Mem, p: &mut P) -> bool {
    if p.depth >= PARSE_DEPTH_CAP {
        let t = cur(mem, p);
        mem.diag(E_TOO_DEEP, t.pos, t.pos + t.len as u32, 0, 0);
        return false;
    }
    p.depth += 1;
    true
}

fn leave(p: &mut P) {
    if p.depth > 0 {
        p.depth -= 1;
    }
}

fn ns_set(p: &mut P, v: u16) -> u16 {
    let old = p.no_struct;
    p.no_struct = v;
    old
}

/// Append `el` to a linked list tracked by (first, last, count).
fn chain_push(mem: &mut Mem, first: &mut u32, last: &mut u32, count: &mut u32, el: u32) {
    if el == NODE_NIL {
        return;
    }
    if *first == NODE_NIL {
        *first = el;
    } else {
        mem.set_link(*last, el);
    }
    *last = el;
    *count += 1;
}

fn tok_is(src: &str, mem: &Mem, ti: u32, s: &[u8]) -> bool {
    let t = mem.tok(ti);
    let b = tok_text(src, t).as_bytes();
    bytes_eq_pub(b, s)
}

fn bytes_eq_pub(a: &[u8], b: &[u8]) -> bool {
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
}

const PREC_OR: u8 = 1;
const PREC_AND: u8 = 2;
const PREC_CMP: u8 = 3;
const PREC_BOR: u8 = 4;
const PREC_BXOR: u8 = 5;
const PREC_BAND: u8 = 6;
const PREC_SHIFT: u8 = 7;
const PREC_ADD: u8 = 8;
const PREC_MUL: u8 = 9;

fn bin_op(k: u16) -> u16 {
    match k {
        T_PIPEPIPE => OP_OR,
        T_AMPAMP => OP_AND,
        T_EQEQ => OP_EQ,
        T_NE => OP_NE,
        T_LT => OP_LT,
        T_LE => OP_LE,
        T_GT => OP_GT,
        T_GE => OP_GE,
        T_PIPE => OP_BOR,
        T_CARET => OP_BXOR,
        T_AMP => OP_BAND,
        T_SHL => OP_SHL,
        T_SHR => OP_SHR,
        T_PLUS => OP_ADD,
        T_MINUS => OP_SUB,
        T_STAR => OP_MUL,
        T_SLASH => OP_DIV,
        T_PERCENT => OP_REM,
        _ => 0,
    }
}

fn bin_prec(k: u16) -> u8 {
    match k {
        T_PIPEPIPE => PREC_OR,
        T_AMPAMP => PREC_AND,
        T_EQEQ | T_NE | T_LT | T_LE | T_GT | T_GE => PREC_CMP,
        T_PIPE => PREC_BOR,
        T_CARET => PREC_BXOR,
        T_AMP => PREC_BAND,
        T_SHL | T_SHR => PREC_SHIFT,
        T_PLUS | T_MINUS => PREC_ADD,
        T_STAR | T_SLASH | T_PERCENT => PREC_MUL,
        _ => 0,
    }
}

fn is_cmp(k: u16) -> bool {
    bin_prec(k) == PREC_CMP
}

/// Compound-assignment token -> binary op; T_EQ -> 0 (plain); else 0xFFFF.
fn assign_op(k: u16) -> u16 {
    match k {
        T_EQ => 0,
        T_PLUSEQ => OP_ADD,
        T_MINUSEQ => OP_SUB,
        T_STAREQ => OP_MUL,
        T_SLASHEQ => OP_DIV,
        T_PERCENTEQ => OP_REM,
        T_AMPEQ => OP_BAND,
        T_PIPEEQ => OP_BOR,
        T_CARETEQ => OP_BXOR,
        T_SHLEQ => OP_SHL,
        T_SHREQ => OP_SHR,
        _ => 0xFFFF,
    }
}

fn is_assign(k: u16) -> bool {
    assign_op(k) != 0xFFFF
}

/// Parse the token stream in `mem` (produced by `lex`) into the node pool.
/// Top-level items end up chained from `mem.root_first`. Returns true if
/// there were no errors.
pub fn parse(src: &str, mem: &mut Mem) -> bool {
    let mut p = P {
        i: 0,
        depth: 0,
        no_struct: 0,
    };
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    while !at(mem, &p, T_EOF) {
        let it = p_item(src, mem, &mut p);
        if it == NODE_NIL {
            sync_item(mem, &mut p);
            continue;
        }
        chain_push(mem, &mut first, &mut last, &mut count, it);
    }
    mem.root_first = first;
    mem.root_n = count;
    mem.diag_n == 0 && !mem.overflow
}

/// Skip to the next plausible item start at brace depth 0.
fn sync_item(mem: &Mem, p: &mut P) {
    if !at(mem, p, T_EOF) {
        bump(p);
    }
    let mut depth: u32 = 0;
    loop {
        let k = tk(mem, p);
        if k == T_EOF {
            return;
        }
        if k == T_LBRACE {
            depth += 1;
        } else if k == T_RBRACE {
            if depth > 0 {
                depth -= 1;
            }
        } else if depth == 0
            && (k == T_KW_FN
                || k == T_KW_STRUCT
                || k == T_KW_IMPL
                || k == T_KW_CONST
                || k == T_KW_USE
                || k == T_POUND)
        {
            return;
        }
        bump(p);
    }
}

/// Skip to just after the next `;` (or to a `}`) at brace depth 0.
fn sync_stmt(mem: &Mem, p: &mut P) {
    if !at(mem, p, T_EOF) && !at(mem, p, T_RBRACE) && !at(mem, p, T_SEMI) {
        bump(p);
    }
    let mut depth: u32 = 0;
    loop {
        let k = tk(mem, p);
        if k == T_EOF {
            return;
        }
        if k == T_LBRACE {
            depth += 1;
        } else if k == T_RBRACE {
            if depth == 0 {
                return;
            }
            depth -= 1;
        } else if k == T_SEMI && depth == 0 {
            bump(p);
            return;
        }
        bump(p);
    }
}

fn p_item(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let mut derives: u16 = 0;
    let mut has_attr = false;
    while at(mem, p, T_POUND) {
        has_attr = true;
        if !p_attr(src, mem, p, &mut derives) {
            return NODE_NIL;
        }
    }
    let _ = accept(mem, p, T_KW_PUB);
    let k = tk(mem, p);
    if k == T_KW_STRUCT {
        return p_struct(src, mem, p, derives);
    }
    if k == T_KW_ENUM {
        return p_enum(src, mem, p, derives);
    }
    if has_attr {
        let t = cur(mem, p);
        mem.diag(E_BAD_ATTR, t.pos, t.pos + t.len as u32, 0, 0);
        return NODE_NIL;
    }
    if k == T_KW_FN {
        return p_fn(src, mem, p, NODE_NIL);
    }
    if k == T_KW_IMPL {
        return p_impl(src, mem, p);
    }
    if k == T_KW_CONST {
        return p_const(src, mem, p);
    }
    if k == T_KW_USE {
        return p_use(mem, p);
    }
    let t = cur(mem, p);
    if k == T_KW_RESERVED {
        mem.diag(E_RESERVED_KEYWORD, t.pos, t.pos + t.len as u32, 0, 0);
    } else {
        mem.diag(E_EXPECTED_ITEM, t.pos, t.pos + t.len as u32, 0, k as u32);
    }
    NODE_NIL
}

/// `#[derive(Clone, Copy)]` — the only attribute in the subset.
fn p_attr(src: &str, mem: &mut Mem, p: &mut P, derives: &mut u16) -> bool {
    let pound = cur(mem, p);
    bump(p);
    if at(mem, p, T_BANG) {
        mem.diag(E_BAD_ATTR, pound.pos, pound.pos + 2, 0, 0);
        return false;
    }
    if !expect(mem, p, T_LBRACK) {
        return false;
    }
    let name = expect_ident(mem, p, false);
    if name == NODE_NIL {
        return false;
    }
    if !tok_is(src, mem, name, b"derive") {
        let t = mem.tok(name);
        mem.diag(E_BAD_ATTR, t.pos, t.pos + t.len as u32, 0, 0);
        return false;
    }
    if !expect(mem, p, T_LPAREN) {
        return false;
    }
    while !at(mem, p, T_RPAREN) {
        let d = expect_ident(mem, p, false);
        if d == NODE_NIL {
            return false;
        }
        if tok_is(src, mem, d, b"Clone") {
            *derives |= DERIVE_CLONE;
        } else if tok_is(src, mem, d, b"Copy") {
            *derives |= DERIVE_COPY;
        } else {
            let t = mem.tok(d);
            mem.diag(E_BAD_DERIVE, t.pos, t.pos + t.len as u32, 0, 0);
            return false;
        }
        if !accept(mem, p, T_COMMA) && !at(mem, p, T_RPAREN) {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_COMMA as u32,
                t.kind as u32,
            );
            return false;
        }
    }
    bump(p);
    expect(mem, p, T_RBRACK)
}

/// `fn name(params) -> ret { body }`. `self_tok` is the `impl` type-name token
/// when parsing a method (NODE_NIL for a free function); a method must open
/// with a `self`/`&self`/`&mut self`/`mut self` receiver.
fn p_fn(src: &str, mem: &mut Mem, p: &mut P, self_tok: u32) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let name = expect_ident(mem, p, false);
    if name == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_LPAREN) {
        return NODE_NIL;
    }
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    if self_tok != NODE_NIL {
        let recv = p_receiver(mem, p, self_tok);
        if recv == NODE_NIL {
            return NODE_NIL;
        }
        chain_push(mem, &mut first, &mut last, &mut count, recv);
        if !accept(mem, p, T_COMMA) && !at(mem, p, T_RPAREN) {
            let t = cur(mem, p);
            mem.diag(E_EXPECTED_TOKEN, t.pos, t.pos + t.len as u32, T_COMMA as u32, t.kind as u32);
            return NODE_NIL;
        }
    }
    while !at(mem, p, T_RPAREN) {
        let plo = cur(mem, p).pos;
        let mutf = if accept(mem, p, T_KW_MUT) { FLAG_MUT } else { 0 };
        let pname = expect_ident(mem, p, true);
        if pname == NODE_NIL {
            return NODE_NIL;
        }
        if !expect(mem, p, T_COLON) {
            return NODE_NIL;
        }
        let ty = p_type(src, mem, p);
        if ty == NODE_NIL {
            return NODE_NIL;
        }
        let mut n = nd(N_PARAM, plo, prev_end(mem, p));
        n.a = pname;
        n.x = mutf;
        n.e = ty;
        let param = mem.push_node(n);
        chain_push(mem, &mut first, &mut last, &mut count, param);
        if !accept(mem, p, T_COMMA) && !at(mem, p, T_RPAREN) {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_COMMA as u32,
                t.kind as u32,
            );
            return NODE_NIL;
        }
    }
    bump(p);
    let mut ret = NODE_NIL;
    if accept(mem, p, T_ARROW) {
        ret = p_type(src, mem, p);
        if ret == NODE_NIL {
            return NODE_NIL;
        }
    }
    let body = p_block(src, mem, p);
    if body == NODE_NIL {
        return NODE_NIL;
    }
    let mut n = nd(N_FN, lo, prev_end(mem, p));
    n.a = name;
    n.b = first;
    n.c = count;
    n.d = ret;
    n.e = body;
    mem.push_node(n)
}

/// Method receiver: `self`, `mut self`, `&self`, `&mut self`. Builds an
/// N_PARAM named `self` whose type is `Self` / `&Self` / `&mut Self`, where
/// `Self` is the `impl` type named by `self_tok`.
fn p_receiver(mem: &mut Mem, p: &mut P, self_tok: u32) -> u32 {
    let lo = cur(mem, p).pos;
    let is_ref = at(mem, p, T_AMP);
    if is_ref {
        bump(p);
    }
    let refmut = if is_ref && accept(mem, p, T_KW_MUT) { FLAG_MUT } else { 0 };
    let valmut = if !is_ref && accept(mem, p, T_KW_MUT) { FLAG_MUT } else { 0 };
    if !at(mem, p, T_KW_SELF) {
        let t = cur(mem, p);
        mem.diag(E_BAD_RECEIVER, t.pos, t.pos + t.len as u32, 0, 0);
        return NODE_NIL;
    }
    let self_name = p.i;
    bump(p);
    let mut tn = nd(N_TY_NAME, lo, prev_end(mem, p));
    tn.a = self_tok;
    let mut ty = mem.push_node(tn);
    if is_ref {
        let mut tr = nd(N_TY_REF, lo, prev_end(mem, p));
        tr.x = refmut;
        tr.d = ty;
        ty = mem.push_node(tr);
    }
    let mut pn = nd(N_PARAM, lo, prev_end(mem, p));
    pn.a = self_name;
    pn.x = valmut;
    pn.e = ty;
    mem.push_node(pn)
}

/// `impl Type { fn method(self-receiver, ...) ... }` — inherent methods.
fn p_impl(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    if !at(mem, p, T_IDENT) {
        let t = cur(mem, p);
        mem.diag(E_EXPECTED_TYPE, t.pos, t.pos + t.len as u32, 0, t.kind as u32);
        return NODE_NIL;
    }
    let type_tok = p.i;
    bump(p);
    if !expect(mem, p, T_LBRACE) {
        return NODE_NIL;
    }
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    while !at(mem, p, T_RBRACE) {
        if at(mem, p, T_EOF) {
            let t = cur(mem, p);
            mem.diag(E_EXPECTED_TOKEN, t.pos, t.pos + t.len as u32, T_RBRACE as u32, t.kind as u32);
            return NODE_NIL;
        }
        let _ = accept(mem, p, T_KW_PUB);
        if !at(mem, p, T_KW_FN) {
            let t = cur(mem, p);
            mem.diag(E_EXPECTED_ITEM, t.pos, t.pos + t.len as u32, 0, t.kind as u32);
            return NODE_NIL;
        }
        let m = p_fn(src, mem, p, type_tok);
        if m == NODE_NIL {
            return NODE_NIL;
        }
        chain_push(mem, &mut first, &mut last, &mut count, m);
    }
    bump(p);
    let mut n = nd(N_IMPL, lo, prev_end(mem, p));
    n.a = type_tok;
    n.b = first;
    n.c = count;
    mem.push_node(n)
}

/// `struct Name { field: Type, ... }` with derive flags from the attribute.
fn p_struct(src: &str, mem: &mut Mem, p: &mut P, derives: u16) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let name = expect_ident(mem, p, false);
    if name == NODE_NIL {
        return NODE_NIL;
    }
    if at(mem, p, T_LPAREN) || at(mem, p, T_SEMI) {
        let t = cur(mem, p);
        mem.diag(E_TUPLE, t.pos, t.pos + t.len as u32, 0, 0);
        return NODE_NIL;
    }
    if !expect(mem, p, T_LBRACE) {
        return NODE_NIL;
    }
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    while !at(mem, p, T_RBRACE) {
        let flo = cur(mem, p).pos;
        let _ = accept(mem, p, T_KW_PUB);
        let fname = expect_ident(mem, p, false);
        if fname == NODE_NIL {
            return NODE_NIL;
        }
        if !expect(mem, p, T_COLON) {
            return NODE_NIL;
        }
        let ty = p_type(src, mem, p);
        if ty == NODE_NIL {
            return NODE_NIL;
        }
        let mut n = nd(N_FIELD, flo, prev_end(mem, p));
        n.a = fname;
        n.e = ty;
        let f = mem.push_node(n);
        chain_push(mem, &mut first, &mut last, &mut count, f);
        if !accept(mem, p, T_COMMA) && !at(mem, p, T_RBRACE) {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_COMMA as u32,
                t.kind as u32,
            );
            return NODE_NIL;
        }
    }
    bump(p);
    let mut n = nd(N_STRUCT, lo, prev_end(mem, p));
    n.a = name;
    n.b = first;
    n.c = count;
    n.x = derives;
    mem.push_node(n)
}

/// `enum Name { V1, V2(Type), ... }` — an N_VARIANT chain. A variant's optional
/// single payload `(Type)` is stored in `e` (field-less when NIL). Tuple/struct
/// variants beyond a single positional payload are not in the subset.
fn p_enum(src: &str, mem: &mut Mem, p: &mut P, derives: u16) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let name = expect_ident(mem, p, false);
    if name == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_LBRACE) {
        return NODE_NIL;
    }
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    while !at(mem, p, T_RBRACE) {
        let vlo = cur(mem, p).pos;
        let vname = expect_ident(mem, p, false);
        if vname == NODE_NIL {
            return NODE_NIL;
        }
        let mut payload = NODE_NIL;
        if accept(mem, p, T_LPAREN) {
            payload = p_type(src, mem, p);
            if payload == NODE_NIL {
                return NODE_NIL;
            }
            if !expect(mem, p, T_RPAREN) {
                return NODE_NIL;
            }
        }
        let mut v = nd(N_VARIANT, vlo, prev_end(mem, p));
        v.a = vname;
        v.e = payload;
        let vn = mem.push_node(v);
        chain_push(mem, &mut first, &mut last, &mut count, vn);
        if !accept(mem, p, T_COMMA) && !at(mem, p, T_RBRACE) {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_COMMA as u32,
                t.kind as u32,
            );
            return NODE_NIL;
        }
    }
    bump(p);
    let mut n = nd(N_ENUM, lo, prev_end(mem, p));
    n.a = name;
    n.b = first;
    n.c = count;
    n.x = derives;
    mem.push_node(n)
}

/// Recognized built-in macros. Only `assert!(cond)` and `assert!(cond, "literal")`
/// are in the subset (PLAN.md §6.2 — the one macro exception). The condition is any
/// bool expression; the optional message must be a plain string literal (no format
/// arguments). Parsed at primary-expression position when an ident is followed by `!`.
fn p_macro(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    let name = p.i;
    bump(p);
    bump(p);
    if !tok_is(src, mem, name, b"assert") {
        let t = mem.tok(name);
        mem.diag(E_BAD_MACRO, t.pos, t.pos + t.len as u32, 0, 0);
        return NODE_NIL;
    }
    if !expect(mem, p, T_LPAREN) {
        return NODE_NIL;
    }
    let sv = ns_set(p, 0);
    let cond = p_expr(src, mem, p, 1);
    if cond == NODE_NIL {
        ns_set(p, sv);
        return NODE_NIL;
    }
    let mut msg = NODE_NIL;
    if accept(mem, p, T_COMMA) {
        if at(mem, p, T_STR) {
            msg = p.i;
            bump(p);
        } else if !at(mem, p, T_RPAREN) {
            let t = cur(mem, p);
            mem.diag(E_ASSERT_MSG, t.pos, t.pos + t.len as u32, 0, 0);
            ns_set(p, sv);
            return NODE_NIL;
        }
    }
    ns_set(p, sv);
    if !expect(mem, p, T_RPAREN) {
        return NODE_NIL;
    }
    let mut n = nd(N_ASSERT, lo, prev_end(mem, p));
    n.c = cond;
    n.a = msg;
    mem.push_node(n)
}

/// `const NAME: Type = expr;`
fn p_const(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let name = expect_ident(mem, p, false);
    if name == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_COLON) {
        return NODE_NIL;
    }
    let ty = p_type(src, mem, p);
    if ty == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_EQ) {
        return NODE_NIL;
    }
    let value = p_expr(src, mem, p, 1);
    if value == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_SEMI) {
        return NODE_NIL;
    }
    let mut n = nd(N_CONST, lo, prev_end(mem, p));
    n.a = name;
    n.d = ty;
    n.e = value;
    mem.push_node(n)
}

/// `use seg::seg::*;` — validated against the host prelude by the checker.
fn p_use(mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    let mut star: u16 = 0;
    let seg = expect_ident(mem, p, false);
    if seg == NODE_NIL {
        return NODE_NIL;
    }
    let mut n = nd(N_USE_SEG, mem.tok(seg).pos, prev_end(mem, p));
    n.a = seg;
    let s = mem.push_node(n);
    chain_push(mem, &mut first, &mut last, &mut count, s);
    while accept(mem, p, T_COLONCOLON) {
        if at(mem, p, T_STAR) {
            bump(p);
            star = 1;
            break;
        }
        let seg = expect_ident(mem, p, false);
        if seg == NODE_NIL {
            return NODE_NIL;
        }
        let mut n = nd(N_USE_SEG, mem.tok(seg).pos, prev_end(mem, p));
        n.a = seg;
        let s = mem.push_node(n);
        chain_push(mem, &mut first, &mut last, &mut count, s);
    }
    if !expect(mem, p, T_SEMI) {
        return NODE_NIL;
    }
    let mut n = nd(N_USE, lo, prev_end(mem, p));
    n.b = first;
    n.c = count;
    n.x = star;
    mem.push_node(n)
}

fn p_type(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = type_inner(src, mem, p);
    leave(p);
    r
}

fn type_inner(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let t = cur(mem, p);
    let lo = t.pos;
    let k = t.kind;
    if k == T_IDENT {
        bump(p);
        let mut n = nd(N_TY_NAME, lo, prev_end(mem, p));
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    if k == T_AMP {
        bump(p);

        if at(mem, p, T_IDENT) && tok_is(src, mem, p.i, b"str") {
            bump(p);
            return mem.push_node(nd(N_TY_STR, lo, prev_end(mem, p)));
        }
        let mutf = if accept(mem, p, T_KW_MUT) { FLAG_MUT } else { 0 };

        if at(mem, p, T_LBRACK) {
            bump(p);
            let elem = p_type(src, mem, p);
            if elem == NODE_NIL {
                return NODE_NIL;
            }
            if at(mem, p, T_SEMI) {

                bump(p);
                let len = p_expr(src, mem, p, 1);
                if len == NODE_NIL {
                    return NODE_NIL;
                }
                if !expect(mem, p, T_RBRACK) {
                    return NODE_NIL;
                }
                let mut an = nd(N_TY_ARRAY, lo, prev_end(mem, p));
                an.d = elem;
                an.e = len;
                let arr = mem.push_node(an);
                let mut n = nd(N_TY_REF, lo, prev_end(mem, p));
                n.x = mutf;
                n.d = arr;
                return mem.push_node(n);
            }
            if !expect(mem, p, T_RBRACK) {
                return NODE_NIL;
            }
            let mut n = nd(N_TY_SLICE, lo, prev_end(mem, p));
            n.x = mutf;
            n.d = elem;
            return mem.push_node(n);
        }
        let pointee = p_type(src, mem, p);
        if pointee == NODE_NIL {
            return NODE_NIL;
        }
        let mut n = nd(N_TY_REF, lo, prev_end(mem, p));
        n.x = mutf;
        n.d = pointee;
        return mem.push_node(n);
    }
    if k == T_LBRACK {
        bump(p);
        let elem = p_type(src, mem, p);
        if elem == NODE_NIL {
            return NODE_NIL;
        }
        if !expect(mem, p, T_SEMI) {
            return NODE_NIL;
        }
        let len = p_expr(src, mem, p, 1);
        if len == NODE_NIL {
            return NODE_NIL;
        }
        if !expect(mem, p, T_RBRACK) {
            return NODE_NIL;
        }
        let mut n = nd(N_TY_ARRAY, lo, prev_end(mem, p));
        n.d = elem;
        n.e = len;
        return mem.push_node(n);
    }
    if k == T_LPAREN {
        bump(p);
        if !expect(mem, p, T_RPAREN) {

            let t = cur(mem, p);
            mem.diag(E_TUPLE, t.pos, t.pos + t.len as u32, 0, 0);
            return NODE_NIL;
        }
        return mem.push_node(nd(N_TY_UNIT, lo, prev_end(mem, p)));
    }
    if k == T_KW_RESERVED {
        mem.diag(E_RESERVED_KEYWORD, t.pos, t.pos + t.len as u32, 0, 0);
        return NODE_NIL;
    }
    mem.diag(E_EXPECTED_TYPE, t.pos, t.pos + t.len as u32, 0, k as u32);
    NODE_NIL
}

fn p_block(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = block_inner(src, mem, p);
    leave(p);
    r
}

fn block_inner(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    if !expect(mem, p, T_LBRACE) {
        return NODE_NIL;
    }
    let sv = ns_set(p, 0);
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    let mut tail = NODE_NIL;
    loop {
        let k = tk(mem, p);
        if k == T_RBRACE {
            break;
        }
        if k == T_EOF {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_RBRACE as u32,
                k as u32,
            );
            break;
        }
        if k == T_SEMI {
            bump(p);
            continue;
        }
        if k == T_KW_LET {
            let s = p_let(src, mem, p);
            if s == NODE_NIL {
                sync_stmt(mem, p);
                continue;
            }
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        if k == T_KW_BREAK || k == T_KW_CONTINUE {
            let t = cur(mem, p);
            bump(p);
            let kind = if k == T_KW_BREAK { N_BREAK } else { N_CONTINUE };

            if !at(mem, p, T_SEMI) && !at(mem, p, T_RBRACE) {
                let u = cur(mem, p);
                mem.diag(
                    E_EXPECTED_TOKEN,
                    u.pos,
                    u.pos + u.len as u32,
                    T_SEMI as u32,
                    u.kind as u32,
                );
                sync_stmt(mem, p);
                continue;
            }
            accept(mem, p, T_SEMI);
            let s = mem.push_node(nd(kind, t.pos, prev_end(mem, p)));
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        if k == T_KW_WHILE {
            let wlo = cur(mem, p).pos;
            bump(p);
            let svc = ns_set(p, 1);
            let cond = p_expr(src, mem, p, 1);
            ns_set(p, svc);
            if cond == NODE_NIL {
                sync_stmt(mem, p);
                continue;
            }
            let body = p_block(src, mem, p);
            if body == NODE_NIL {
                sync_stmt(mem, p);
                continue;
            }
            let mut n = nd(N_WHILE, wlo, prev_end(mem, p));
            n.d = cond;
            n.e = body;
            let s = mem.push_node(n);
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        if k == T_KW_LOOP {
            let llo = cur(mem, p).pos;
            bump(p);
            let body = p_block(src, mem, p);
            if body == NODE_NIL {
                sync_stmt(mem, p);
                continue;
            }
            let mut n = nd(N_LOOP, llo, prev_end(mem, p));
            n.e = body;
            let s = mem.push_node(n);
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        if k == T_KW_FOR {
            let s = p_for(src, mem, p);
            if s == NODE_NIL {
                sync_stmt(mem, p);
                continue;
            }
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        if k == T_KW_RESERVED {
            let t = cur(mem, p);
            mem.diag(E_RESERVED_KEYWORD, t.pos, t.pos + t.len as u32, 0, 0);
            sync_stmt(mem, p);
            continue;
        }

        let blocklike = k == T_KW_IF || k == T_KW_MATCH || k == T_LBRACE;
        let e = if k == T_KW_IF {
            p_if(src, mem, p)
        } else if k == T_KW_MATCH {
            p_match(src, mem, p)
        } else if k == T_LBRACE {
            p_block(src, mem, p)
        } else {
            p_expr(src, mem, p, 1)
        };
        if e == NODE_NIL {
            sync_stmt(mem, p);
            continue;
        }
        if blocklike {

            if at(mem, p, T_RBRACE) {
                tail = e;
                break;
            }
            let en = mem.node(e);
            let mut n = nd(N_EXPR_STMT, en.lo, en.hi);
            n.e = e;
            let s = mem.push_node(n);
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        let nk = tk(mem, p);
        if is_assign(nk) {
            let op = assign_op(nk);
            bump(p);
            let rhs = p_expr(src, mem, p, 1);
            if rhs == NODE_NIL {
                sync_stmt(mem, p);
                continue;
            }
            if !expect(mem, p, T_SEMI) {
                sync_stmt(mem, p);
                continue;
            }
            let en = mem.node(e);
            let mut n = nd(N_ASSIGN, en.lo, prev_end(mem, p));
            n.x = op;
            n.d = e;
            n.e = rhs;
            let s = mem.push_node(n);
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        if nk == T_SEMI {
            bump(p);
            let en = mem.node(e);
            let mut n = nd(N_EXPR_STMT, en.lo, prev_end(mem, p));
            n.e = e;
            let s = mem.push_node(n);
            chain_push(mem, &mut first, &mut last, &mut count, s);
            continue;
        }
        if nk == T_RBRACE {
            tail = e;
            break;
        }
        let t = cur(mem, p);
        if nk == T_DOTDOT || nk == T_DOTDOTEQ {
            mem.diag(E_RANGE_HERE, t.pos, t.pos + t.len as u32, 0, 0);
        } else {
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_SEMI as u32,
                nk as u32,
            );
        }
        sync_stmt(mem, p);
    }
    expect(mem, p, T_RBRACE);
    ns_set(p, sv);
    let mut n = nd(N_BLOCK, lo, prev_end(mem, p));
    n.b = first;
    n.c = count;
    n.e = tail;
    mem.push_node(n)
}

/// `let [mut] name [: ty] = expr;` (initializer required — stricter than Rust)
fn p_let(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);

    if at(mem, p, T_LPAREN) {
        bump(p);
        let mut first = NODE_NIL;
        let mut last = NODE_NIL;
        let mut count: u32 = 0;
        while !at(mem, p, T_RPAREN) {
            let em = if accept(mem, p, T_KW_MUT) { FLAG_MUT } else { 0 };
            let nm = expect_ident(mem, p, true);
            if nm == NODE_NIL {
                return NODE_NIL;
            }
            let mut pn = nd(N_NAME, lo, prev_end(mem, p));
            pn.a = nm;
            pn.x = em;
            let node = mem.push_node(pn);
            if first == NODE_NIL {
                first = node;
            } else {
                mem.set_link(last, node);
            }
            last = node;
            count += 1;
            if !accept(mem, p, T_COMMA) && !at(mem, p, T_RPAREN) {
                let t = cur(mem, p);
                mem.diag(E_EXPECTED_TOKEN, t.pos, t.pos + t.len as u32, T_RPAREN as u32, t.kind as u32);
                return NODE_NIL;
            }
        }
        bump(p);
        if !expect(mem, p, T_EQ) {
            return NODE_NIL;
        }
        let init = p_expr(src, mem, p, 1);
        if init == NODE_NIL {
            return NODE_NIL;
        }
        if !expect(mem, p, T_SEMI) {
            return NODE_NIL;
        }
        let mut n = nd(N_LET, lo, prev_end(mem, p));
        n.x = FLAG_TUPLE;
        n.b = first;
        n.c = count;
        n.e = init;
        return mem.push_node(n);
    }
    let mutf = if accept(mem, p, T_KW_MUT) { FLAG_MUT } else { 0 };
    let name = expect_ident(mem, p, true);
    if name == NODE_NIL {
        return NODE_NIL;
    }
    let mut ty = NODE_NIL;
    if accept(mem, p, T_COLON) {
        ty = p_type(src, mem, p);
        if ty == NODE_NIL {
            return NODE_NIL;
        }
    }
    if !expect(mem, p, T_EQ) {
        return NODE_NIL;
    }
    let init = p_expr(src, mem, p, 1);
    if init == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_SEMI) {
        return NODE_NIL;
    }
    let mut n = nd(N_LET, lo, prev_end(mem, p));
    n.a = name;
    n.x = mutf;
    n.d = ty;
    n.e = init;
    mem.push_node(n)
}

/// `for var in lo..hi { body }` / `..=` — ranges exist only here in v0.1.
fn p_for(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let var = expect_ident(mem, p, true);
    if var == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_KW_IN) {
        return NODE_NIL;
    }
    let sv = ns_set(p, 1);
    let lo_e = p_expr(src, mem, p, 1);
    if lo_e == NODE_NIL {
        ns_set(p, sv);
        return NODE_NIL;
    }
    let incl = if accept(mem, p, T_DOTDOTEQ) {
        FLAG_INCLUSIVE
    } else if accept(mem, p, T_DOTDOT) {
        0
    } else {
        let t = cur(mem, p);
        mem.diag(
            E_EXPECTED_TOKEN,
            t.pos,
            t.pos + t.len as u32,
            T_DOTDOT as u32,
            t.kind as u32,
        );
        ns_set(p, sv);
        return NODE_NIL;
    };
    let hi_e = p_expr(src, mem, p, 1);
    ns_set(p, sv);
    if hi_e == NODE_NIL {
        return NODE_NIL;
    }
    let body = p_block(src, mem, p);
    if body == NODE_NIL {
        return NODE_NIL;
    }
    let mut n = nd(N_FOR, lo, prev_end(mem, p));
    n.a = var;
    n.x = incl;
    n.b = lo_e;
    n.c = hi_e;
    n.e = body;
    mem.push_node(n)
}

fn p_expr(src: &str, mem: &mut Mem, p: &mut P, min: u8) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = expr_inner(src, mem, p, min);
    leave(p);
    r
}

fn expr_inner(src: &str, mem: &mut Mem, p: &mut P, min: u8) -> u32 {
    let mut lhs = p_cast(src, mem, p);
    if lhs == NODE_NIL {
        return NODE_NIL;
    }
    loop {
        let k = tk(mem, p);
        let op = bin_op(k);
        let prec = bin_prec(k);
        if op == 0 || prec < min {
            break;
        }
        bump(p);
        let rhs = p_expr(src, mem, p, prec + 1);
        if rhs == NODE_NIL {
            return NODE_NIL;
        }

        if prec == PREC_CMP && is_cmp(tk(mem, p)) {
            let t = cur(mem, p);
            mem.diag(E_CHAINED_COMPARISON, t.pos, t.pos + t.len as u32, 0, 0);
            return NODE_NIL;
        }
        let l = mem.node(lhs);
        let r = mem.node(rhs);
        let mut n = nd(N_BINARY, l.lo, r.hi);
        n.x = op;
        n.d = lhs;
        n.e = rhs;
        lhs = mem.push_node(n);
        if lhs == NODE_NIL {
            return NODE_NIL;
        }
    }

    if min <= 1 {
        let ak = tk(mem, p);
        if is_assign(ak) {
            let op = assign_op(ak);
            bump(p);
            let rhs = p_expr(src, mem, p, 1);
            if rhs == NODE_NIL {
                return NODE_NIL;
            }
            let l = mem.node(lhs);
            let r = mem.node(rhs);
            let mut n = nd(N_ASSIGN, l.lo, r.hi);
            n.x = op;
            n.d = lhs;
            n.e = rhs;
            lhs = mem.push_node(n);
            if lhs == NODE_NIL {
                return NODE_NIL;
            }
        }
    }
    lhs
}

/// Unary expression, then `as` casts. Unary prefixes (`- ! * &`) bind TIGHTER
/// than `as`, as in rustc: `*p as u32` is `(*p) as u32`, `-x as u8` is
/// `(-x) as u8`. The one exception is a negation applied to a bare *literal*:
/// `-1 as u8` is `-(1 as u8)` (rustc E0600 — it never yields `255`).
fn p_cast(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let mut e = p_unary(src, mem, p);
    if e == NODE_NIL {
        return NODE_NIL;
    }
    while at(mem, p, T_KW_AS) {
        bump(p);
        let ty = p_type(src, mem, p);
        if ty == NODE_NIL {
            return NODE_NIL;
        }
        let en = mem.node(e);
        let mut n = nd(N_CAST, en.lo, prev_end(mem, p));
        n.d = e;
        n.e = ty;
        e = mem.push_node(n);
        if e == NODE_NIL {
            return NODE_NIL;
        }
    }
    e
}

fn p_unary(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = unary_inner(src, mem, p);
    leave(p);
    r
}

fn unary_inner(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let t = cur(mem, p);
    let k = t.kind;
    if k == T_MINUS {
        bump(p);

        let nk = tk(mem, p);
        let operand = if nk == T_INT || nk == T_FLOAT {
            let mut inner = p_postfix(src, mem, p);
            if inner == NODE_NIL {
                return NODE_NIL;
            }
            while at(mem, p, T_KW_AS) {
                bump(p);
                let ty = p_type(src, mem, p);
                if ty == NODE_NIL {
                    return NODE_NIL;
                }
                let cn = mem.node(inner);
                let mut c = nd(N_CAST, cn.lo, prev_end(mem, p));
                c.d = inner;
                c.e = ty;
                inner = mem.push_node(c);
                if inner == NODE_NIL {
                    return NODE_NIL;
                }
            }
            inner
        } else {
            p_unary(src, mem, p)
        };
        if operand == NODE_NIL {
            return NODE_NIL;
        }
        let on = mem.node(operand);
        let mut n = nd(N_UNARY, t.pos, on.hi);
        n.x = OP_NEG;
        n.e = operand;
        return mem.push_node(n);
    }
    if k == T_BANG {
        bump(p);
        let operand = p_unary(src, mem, p);
        if operand == NODE_NIL {
            return NODE_NIL;
        }
        let on = mem.node(operand);
        let mut n = nd(N_UNARY, t.pos, on.hi);
        n.x = OP_NOT;
        n.e = operand;
        return mem.push_node(n);
    }

    if k == T_AMP {
        bump(p);
        let mutf = if accept(mem, p, T_KW_MUT) { FLAG_MUT } else { 0 };
        let operand = p_unary(src, mem, p);
        if operand == NODE_NIL {
            return NODE_NIL;
        }
        let on = mem.node(operand);
        let mut n = nd(N_REFOF, t.pos, on.hi);
        n.x = mutf;
        n.e = operand;
        return mem.push_node(n);
    }
    if k == T_STAR {
        bump(p);
        let operand = p_unary(src, mem, p);
        if operand == NODE_NIL {
            return NODE_NIL;
        }
        let on = mem.node(operand);
        let mut n = nd(N_DEREF, t.pos, on.hi);
        n.e = operand;
        return mem.push_node(n);
    }
    p_postfix(src, mem, p)
}

fn p_postfix(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let mut e = p_primary(src, mem, p);
    if e == NODE_NIL {
        return NODE_NIL;
    }
    loop {
        let k = tk(mem, p);
        if k == T_LPAREN {
            let en = mem.node(e);
            if en.kind != N_NAME {
                let t = cur(mem, p);
                mem.diag(E_CALL_NOT_NAME, t.pos, t.pos + t.len as u32, 0, 0);
                return NODE_NIL;
            }
            bump(p);
            let sv = ns_set(p, 0);
            let mut first = NODE_NIL;
            let mut last = NODE_NIL;
            let mut count: u32 = 0;
            while !at(mem, p, T_RPAREN) {
                let arg = p_expr(src, mem, p, 1);
                if arg == NODE_NIL {
                    ns_set(p, sv);
                    return NODE_NIL;
                }
                chain_push(mem, &mut first, &mut last, &mut count, arg);
                if !accept(mem, p, T_COMMA) && !at(mem, p, T_RPAREN) {
                    let t = cur(mem, p);
                    mem.diag(
                        E_EXPECTED_TOKEN,
                        t.pos,
                        t.pos + t.len as u32,
                        T_COMMA as u32,
                        t.kind as u32,
                    );
                    ns_set(p, sv);
                    return NODE_NIL;
                }
            }
            bump(p);
            ns_set(p, sv);
            let mut n = nd(N_CALL, en.lo, prev_end(mem, p));
            n.a = en.a;
            n.b = first;
            n.c = count;
            e = mem.push_node(n);
            if e == NODE_NIL {
                return NODE_NIL;
            }
            continue;
        }
        if k == T_LBRACK {
            bump(p);
            let sv = ns_set(p, 0);
            let idx = p_expr(src, mem, p, 1);
            ns_set(p, sv);
            if idx == NODE_NIL {
                return NODE_NIL;
            }
            let en = mem.node(e);

            if at(mem, p, T_DOTDOT) {
                bump(p);

                let hi = if at(mem, p, T_RBRACK) {
                    NODE_NIL
                } else {
                    let sv2 = ns_set(p, 0);
                    let h = p_expr(src, mem, p, 1);
                    ns_set(p, sv2);
                    if h == NODE_NIL {
                        return NODE_NIL;
                    }
                    h
                };
                if !expect(mem, p, T_RBRACK) {
                    return NODE_NIL;
                }
                let mut n = nd(N_SLICE, en.lo, prev_end(mem, p));
                n.d = e;
                n.b = idx;
                n.c = hi;
                e = mem.push_node(n);
                if e == NODE_NIL {
                    return NODE_NIL;
                }
                continue;
            }
            if !expect(mem, p, T_RBRACK) {
                return NODE_NIL;
            }
            let mut n = nd(N_INDEX, en.lo, prev_end(mem, p));
            n.d = e;
            n.e = idx;
            e = mem.push_node(n);
            if e == NODE_NIL {
                return NODE_NIL;
            }
            continue;
        }
        if k == T_DOT {
            bump(p);
            let fk = tk(mem, p);
            if fk == T_INT {
                let t = cur(mem, p);
                mem.diag(E_TUPLE, t.pos, t.pos + t.len as u32, 0, 0);
                return NODE_NIL;
            }
            let field = expect_ident(mem, p, false);
            if field == NODE_NIL {
                return NODE_NIL;
            }
            let en = mem.node(e);
            if at(mem, p, T_LPAREN) {

                bump(p);
                let sv = ns_set(p, 0);
                let mut first = NODE_NIL;
                let mut last = NODE_NIL;
                let mut count: u32 = 0;
                while !at(mem, p, T_RPAREN) {
                    let arg = p_expr(src, mem, p, 1);
                    if arg == NODE_NIL {
                        ns_set(p, sv);
                        return NODE_NIL;
                    }
                    chain_push(mem, &mut first, &mut last, &mut count, arg);
                    if !accept(mem, p, T_COMMA) && !at(mem, p, T_RPAREN) {
                        let t = cur(mem, p);
                        mem.diag(E_EXPECTED_TOKEN, t.pos, t.pos + t.len as u32, T_COMMA as u32, t.kind as u32);
                        ns_set(p, sv);
                        return NODE_NIL;
                    }
                }
                bump(p);
                ns_set(p, sv);
                let mut n = nd(N_METHOD, en.lo, prev_end(mem, p));
                n.d = e;
                n.a = field;
                n.b = first;
                n.c = count;
                e = mem.push_node(n);
                if e == NODE_NIL {
                    return NODE_NIL;
                }
                continue;
            }
            let mut n = nd(N_DOT, en.lo, prev_end(mem, p));
            n.d = e;
            n.a = field;
            e = mem.push_node(n);
            if e == NODE_NIL {
                return NODE_NIL;
            }
            continue;
        }
        break;
    }
    e
}

fn p_primary(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let t = cur(mem, p);
    let k = t.kind;
    let lo = t.pos;
    let hi = t.pos + t.len as u32;

    if k == T_INT || k == T_FLOAT || k == T_STR || k == T_BSTR || k == T_BYTE {
        bump(p);
        let kind = if k == T_INT {
            N_LIT_INT
        } else if k == T_FLOAT {
            N_LIT_FLOAT
        } else if k == T_BSTR {
            N_LIT_BSTR
        } else if k == T_BYTE {
            N_LIT_BYTE
        } else {
            N_LIT_STR
        };
        let mut n = nd(kind, lo, hi);
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    if k == T_KW_TRUE || k == T_KW_FALSE {
        bump(p);
        let mut n = nd(N_LIT_BOOL, lo, hi);
        n.x = if k == T_KW_TRUE { 1 } else { 0 };
        return mem.push_node(n);
    }
    if k == T_KW_SELF {

        bump(p);
        let mut n = nd(N_NAME, lo, hi);
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    if k == T_KW_RETURN {
        bump(p);

        let k2 = tk(mem, p);
        let has_val = !(k2 == T_SEMI || k2 == T_RBRACE || k2 == T_RPAREN
            || k2 == T_RBRACK || k2 == T_COMMA || k2 == T_EOF);
        let mut n = nd(N_RETURN, lo, prev_end(mem, p));
        if has_val {
            let v = p_expr(src, mem, p, 1);
            if v == NODE_NIL {
                return NODE_NIL;
            }
            n.e = v;
            n.hi = mem.node(v).hi;
        }
        return mem.push_node(n);
    }
    if k == T_IDENT {

        if tk1(mem, p) == T_BANG {
            return p_macro(src, mem, p);
        }

        if tk1(mem, p) == T_LBRACE && p.no_struct == 0 {
            return p_struct_lit(src, mem, p);
        }

        if tk1(mem, p) == T_COLONCOLON {
            let ty_tok = p.i;
            bump(p);
            bump(p);
            let member = expect_ident(mem, p, false);
            if member == NODE_NIL {
                return NODE_NIL;
            }

            if at(mem, p, T_LPAREN) {
                bump(p);
                let sv = ns_set(p, 0);
                let mut first = NODE_NIL;
                let mut last = NODE_NIL;
                let mut count: u32 = 0;
                while !at(mem, p, T_RPAREN) {
                    let arg = p_expr(src, mem, p, 1);
                    if arg == NODE_NIL {
                        ns_set(p, sv);
                        return NODE_NIL;
                    }
                    chain_push(mem, &mut first, &mut last, &mut count, arg);
                    if !accept(mem, p, T_COMMA) && !at(mem, p, T_RPAREN) {
                        let t = cur(mem, p);
                        mem.diag(E_EXPECTED_TOKEN, t.pos, t.pos + t.len as u32, T_COMMA as u32, t.kind as u32);
                        ns_set(p, sv);
                        return NODE_NIL;
                    }
                }
                bump(p);
                ns_set(p, sv);
                let mut n = nd(N_ASSOC_CALL, lo, prev_end(mem, p));
                n.a = ty_tok;
                n.e = member;
                n.b = first;
                n.c = count;
                return mem.push_node(n);
            }
            let mut n = nd(N_PATHCONST, lo, prev_end(mem, p));
            n.a = ty_tok;
            n.b = member;
            return mem.push_node(n);
        }
        bump(p);
        let mut n = nd(N_NAME, lo, hi);
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    if k == T_LPAREN {
        bump(p);
        if at(mem, p, T_RPAREN) {
            bump(p);
            return mem.push_node(nd(N_LIT_UNIT, lo, prev_end(mem, p)));
        }
        let sv = ns_set(p, 0);
        let e = p_expr(src, mem, p, 1);
        if e == NODE_NIL {
            ns_set(p, sv);
            return NODE_NIL;
        }
        if at(mem, p, T_COMMA) {

            let mut last = e;
            let mut count: u32 = 1;
            while accept(mem, p, T_COMMA) {
                if at(mem, p, T_RPAREN) {
                    break;
                }
                let el = p_expr(src, mem, p, 1);
                if el == NODE_NIL {
                    ns_set(p, sv);
                    return NODE_NIL;
                }
                mem.set_link(last, el);
                last = el;
                count += 1;
            }
            ns_set(p, sv);
            if !expect(mem, p, T_RPAREN) {
                return NODE_NIL;
            }
            let mut n = nd(N_TUPLE, lo, prev_end(mem, p));
            n.b = e;
            n.c = count;
            return mem.push_node(n);
        }
        ns_set(p, sv);
        if !expect(mem, p, T_RPAREN) {
            return NODE_NIL;
        }
        return e;
    }
    if k == T_LBRACK {
        bump(p);
        let sv = ns_set(p, 0);
        if at(mem, p, T_RBRACK) {
            bump(p);
            ns_set(p, sv);
            let mut n = nd(N_ARRAY_LIT, lo, prev_end(mem, p));
            n.c = 0;
            return mem.push_node(n);
        }
        let e0 = p_expr(src, mem, p, 1);
        if e0 == NODE_NIL {
            ns_set(p, sv);
            return NODE_NIL;
        }
        if accept(mem, p, T_SEMI) {
            let len = p_expr(src, mem, p, 1);
            ns_set(p, sv);
            if len == NODE_NIL {
                return NODE_NIL;
            }
            if !expect(mem, p, T_RBRACK) {
                return NODE_NIL;
            }
            let mut n = nd(N_ARRAY_REPEAT, lo, prev_end(mem, p));
            n.d = e0;
            n.e = len;
            return mem.push_node(n);
        }
        let mut first = NODE_NIL;
        let mut last = NODE_NIL;
        let mut count: u32 = 0;
        chain_push(mem, &mut first, &mut last, &mut count, e0);
        while accept(mem, p, T_COMMA) {
            if at(mem, p, T_RBRACK) {
                break;
            }
            let el = p_expr(src, mem, p, 1);
            if el == NODE_NIL {
                ns_set(p, sv);
                return NODE_NIL;
            }
            chain_push(mem, &mut first, &mut last, &mut count, el);
        }
        ns_set(p, sv);
        if !expect(mem, p, T_RBRACK) {
            return NODE_NIL;
        }
        let mut n = nd(N_ARRAY_LIT, lo, prev_end(mem, p));
        n.b = first;
        n.c = count;
        return mem.push_node(n);
    }
    if k == T_LBRACE {
        return p_block(src, mem, p);
    }
    if k == T_KW_IF {
        return p_if(src, mem, p);
    }
    if k == T_KW_MATCH {
        return p_match(src, mem, p);
    }
    if k == T_KW_RESERVED {
        mem.diag(E_RESERVED_KEYWORD, lo, hi, 0, 0);
        return NODE_NIL;
    }
    mem.diag(E_EXPECTED_EXPR, lo, hi, 0, k as u32);
    NODE_NIL
}

/// `Name { field: expr, shorthand, ... }`
fn p_struct_lit(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let name = p.i;
    let lo = cur(mem, p).pos;
    bump(p);
    bump(p);
    let sv = ns_set(p, 0);
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    loop {
        if at(mem, p, T_RBRACE) {
            break;
        }
        if at(mem, p, T_DOTDOT) {
            let t = cur(mem, p);
            mem.diag(E_STRUCT_UPDATE, t.pos, t.pos + t.len as u32, 0, 0);
            ns_set(p, sv);
            return NODE_NIL;
        }
        let fname = expect_ident(mem, p, false);
        if fname == NODE_NIL {
            ns_set(p, sv);
            return NODE_NIL;
        }
        let mut val = NODE_NIL;
        if accept(mem, p, T_COLON) {
            val = p_expr(src, mem, p, 1);
            if val == NODE_NIL {
                ns_set(p, sv);
                return NODE_NIL;
            }
        }
        let ft = mem.tok(fname);
        let mut n = nd(N_FIELD_INIT, ft.pos, prev_end(mem, p));
        n.a = fname;
        n.e = val;
        let f = mem.push_node(n);
        chain_push(mem, &mut first, &mut last, &mut count, f);
        if !accept(mem, p, T_COMMA) && !at(mem, p, T_RBRACE) {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_COMMA as u32,
                t.kind as u32,
            );
            ns_set(p, sv);
            return NODE_NIL;
        }
    }
    bump(p);
    ns_set(p, sv);
    let mut n = nd(N_STRUCT_LIT, lo, prev_end(mem, p));
    n.a = name;
    n.b = first;
    n.c = count;
    mem.push_node(n)
}

fn p_if(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = if_inner(src, mem, p);
    leave(p);
    r
}

fn if_inner(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let sv = ns_set(p, 1);
    let cond = p_expr(src, mem, p, 1);
    ns_set(p, sv);
    if cond == NODE_NIL {
        return NODE_NIL;
    }
    let then = p_block(src, mem, p);
    if then == NODE_NIL {
        return NODE_NIL;
    }
    let mut els = NODE_NIL;
    if accept(mem, p, T_KW_ELSE) {
        if at(mem, p, T_KW_IF) {
            els = p_if(src, mem, p);
        } else if at(mem, p, T_LBRACE) {
            els = p_block(src, mem, p);
        } else {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_LBRACE as u32,
                t.kind as u32,
            );
            return NODE_NIL;
        }
        if els == NODE_NIL {
            return NODE_NIL;
        }
    }
    let mut n = nd(N_IF, lo, prev_end(mem, p));
    n.d = cond;
    n.e = then;
    n.b = els;
    mem.push_node(n)
}

fn p_match(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = match_inner(src, mem, p);
    leave(p);
    r
}

fn match_inner(src: &str, mem: &mut Mem, p: &mut P) -> u32 {
    let lo = cur(mem, p).pos;
    bump(p);
    let sv = ns_set(p, 1);
    let scrut = p_expr(src, mem, p, 1);
    ns_set(p, sv);
    if scrut == NODE_NIL {
        return NODE_NIL;
    }
    if !expect(mem, p, T_LBRACE) {
        return NODE_NIL;
    }
    let sv2 = ns_set(p, 0);
    let mut first = NODE_NIL;
    let mut last = NODE_NIL;
    let mut count: u32 = 0;
    loop {
        if at(mem, p, T_RBRACE) {
            break;
        }
        if at(mem, p, T_EOF) {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_RBRACE as u32,
                T_EOF as u32,
            );
            break;
        }
        let alo = cur(mem, p).pos;
        accept(mem, p, T_PIPE);
        let mut pfirst = NODE_NIL;
        let mut plast = NODE_NIL;
        let mut pcount: u32 = 0;
        let pat = p_pattern(mem, p);
        if pat == NODE_NIL {
            ns_set(p, sv2);
            return NODE_NIL;
        }
        chain_push(mem, &mut pfirst, &mut plast, &mut pcount, pat);
        while accept(mem, p, T_PIPE) {
            let pat = p_pattern(mem, p);
            if pat == NODE_NIL {
                ns_set(p, sv2);
                return NODE_NIL;
            }
            chain_push(mem, &mut pfirst, &mut plast, &mut pcount, pat);
        }
        if !expect(mem, p, T_FATARROW) {
            ns_set(p, sv2);
            return NODE_NIL;
        }

        let bt = tk(mem, p);
        let body_blocklike = bt == T_KW_IF || bt == T_KW_MATCH || bt == T_LBRACE;
        let body = if bt == T_KW_IF {
            p_if(src, mem, p)
        } else if bt == T_KW_MATCH {
            p_match(src, mem, p)
        } else if bt == T_LBRACE {
            p_block(src, mem, p)
        } else {
            p_expr(src, mem, p, 1)
        };
        if body == NODE_NIL {
            ns_set(p, sv2);
            return NODE_NIL;
        }

        if !accept(mem, p, T_COMMA) && !body_blocklike && !at(mem, p, T_RBRACE) {
            let t = cur(mem, p);
            mem.diag(
                E_EXPECTED_TOKEN,
                t.pos,
                t.pos + t.len as u32,
                T_COMMA as u32,
                t.kind as u32,
            );
            ns_set(p, sv2);
            return NODE_NIL;
        }
        let mut n = nd(N_ARM, alo, prev_end(mem, p));
        n.b = pfirst;
        n.c = pcount;
        n.e = body;
        let arm = mem.push_node(n);
        chain_push(mem, &mut first, &mut last, &mut count, arm);
    }
    expect(mem, p, T_RBRACE);
    ns_set(p, sv2);
    let mut n = nd(N_MATCH, lo, prev_end(mem, p));
    n.d = scrut;
    n.b = first;
    n.c = count;
    mem.push_node(n)
}

fn p_pattern(mem: &mut Mem, p: &mut P) -> u32 {
    let t = cur(mem, p);
    let k = t.kind;
    let lo = t.pos;
    let hi = t.pos + t.len as u32;
    if k == T_MINUS {
        bump(p);
        if at(mem, p, T_INT) {
            let ti = p.i;
            bump(p);
            let mut n = nd(N_PAT_INT, lo, prev_end(mem, p));
            n.a = ti;
            n.x = 1;
            return mem.push_node(n);
        }
        let u = cur(mem, p);
        mem.diag(
            E_EXPECTED_PATTERN,
            u.pos,
            u.pos + u.len as u32,
            0,
            u.kind as u32,
        );
        return NODE_NIL;
    }
    if k == T_INT {
        bump(p);
        let mut n = nd(N_PAT_INT, lo, hi);
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    if k == T_STR {
        bump(p);
        let mut n = nd(N_PAT_STR, lo, hi);
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    if k == T_BYTE {
        bump(p);
        let mut n = nd(N_PAT_BYTE, lo, hi);
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    if k == T_KW_TRUE || k == T_KW_FALSE {
        bump(p);
        let mut n = nd(N_PAT_BOOL, lo, hi);
        n.x = if k == T_KW_TRUE { 1 } else { 0 };
        return mem.push_node(n);
    }
    if k == T_UNDERSCORE {
        bump(p);
        return mem.push_node(nd(N_PAT_WILD, lo, hi));
    }
    if k == T_IDENT {

        if tk1(mem, p) == T_COLONCOLON {
            let ty_tok = p.i;
            bump(p);
            bump(p);
            let variant = expect_ident(mem, p, false);
            if variant == NODE_NIL {
                return NODE_NIL;
            }
            let mut n = nd(N_PAT_ENUM, lo, prev_end(mem, p));
            n.a = ty_tok;
            n.b = variant;
            return mem.push_node(n);
        }

        bump(p);
        let mut n = nd(N_PAT_CONST, lo, hi);
        n.a = p.i - 1;
        return mem.push_node(n);
    }
    mem.diag(E_EXPECTED_PATTERN, lo, hi, 0, k as u32);
    NODE_NIL
}
