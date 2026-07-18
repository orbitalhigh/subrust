// Reference for parse.P1pp: a faithful Rust port of sr0i.c's lexer + parser.
// Emits the node arena — one 7-word record [kind,a,b,c,d,e,link] (u64 LE) per
// node, in creation order — so the P1pp parser can be diffed byte-for-byte.
// NIL = u64::MAX. Build mode (env SR_PARSE): "expr" (default) parses one
// expression via parse_expr(1); "program" parses a whole program (fn/const)
// and also dumps the fn table. Used incrementally: 2a=expr, 2c=program.
use std::io::{Read, Write};

const NIL: u64 = u64::MAX;
// token kinds
const T_EOF: u64 = 0; const T_IDENT: u64 = 1; const T_INT: u64 = 2;
const T_KW_IF: u64 = 6; const T_KW_ELSE: u64 = 7; const T_KW_WHILE: u64 = 8;
const T_KW_LOOP: u64 = 9; const T_KW_BREAK: u64 = 10; const T_KW_CONTINUE: u64 = 11;
const T_KW_TRUE: u64 = 12; const T_KW_FALSE: u64 = 13;
const T_LP: u64 = 14; const T_RP: u64 = 15; const T_LB: u64 = 16; const T_RB: u64 = 17;
const T_COMMA: u64 = 18; const T_SEMI: u64 = 19; const T_COLON: u64 = 20;
const T_ARROW: u64 = 21; const T_EQ: u64 = 22; const T_PLUS: u64 = 23;
const T_MINUS: u64 = 24; const T_STAR: u64 = 25; const T_SLASH: u64 = 26;
const T_PCT: u64 = 27; const T_AMP: u64 = 28; const T_AMPAMP: u64 = 29;
const T_PIPE: u64 = 30; const T_PIPEPIPE: u64 = 31; const T_CARET: u64 = 32;
const T_SHL: u64 = 33; const T_SHR: u64 = 34; const T_BANG: u64 = 35;
const T_EE: u64 = 36; const T_NE: u64 = 37; const T_LT: u64 = 38; const T_LE: u64 = 39;
const T_GT: u64 = 40; const T_GE: u64 = 41; const T_PLUSEQ: u64 = 42;
const T_MINUSEQ: u64 = 43; const T_STAREQ: u64 = 44; const T_SLASHEQ: u64 = 45;
const T_PCTEQ: u64 = 46; const T_AMPEQ: u64 = 47; const T_PIPEEQ: u64 = 48;
const T_CARETEQ: u64 = 49; const T_SHLEQ: u64 = 50; const T_SHREQ: u64 = 51;
const T_KW_CONST: u64 = 52; const T_KW_FN: u64 = 3; const T_KW_LET: u64 = 4;
const T_KW_MUT: u64 = 5;
// node kinds
const N_INT: u64 = 1; const N_BOOL: u64 = 2; const N_NAME: u64 = 3; const N_CALL: u64 = 4;
const N_UNARY: u64 = 5; const N_BIN: u64 = 6; const N_IF: u64 = 7; const N_BLOCK: u64 = 8;
const N_LET: u64 = 9; const N_ASSIGN: u64 = 10; const N_WHILE: u64 = 11; const N_LOOP: u64 = 12;
const N_BREAK: u64 = 13; const N_CONTINUE: u64 = 14; const N_EXPR: u64 = 15;
// op ids
const OP_ADD: u64 = 1; const OP_SUB: u64 = 2; const OP_MUL: u64 = 3; const OP_DIV: u64 = 4;
const OP_REM: u64 = 5; const OP_AND: u64 = 6; const OP_OR: u64 = 7; const OP_BAND: u64 = 8;
const OP_BOR: u64 = 9; const OP_BXOR: u64 = 10; const OP_SHL: u64 = 11; const OP_SHR: u64 = 12;
const OP_EQ: u64 = 13; const OP_NE: u64 = 14; const OP_LT: u64 = 15; const OP_LE: u64 = 16;
const OP_GT: u64 = 17; const OP_GE: u64 = 18;

fn is_digit(c: u8) -> bool { c >= b'0' && c <= b'9' }
fn is_alpha(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' }
fn is_alnum(c: u8) -> bool { is_alpha(c) || is_digit(c) }
fn kw(s: &[u8]) -> u64 {
    match s {
        b"fn" => T_KW_FN, b"let" => T_KW_LET, b"mut" => T_KW_MUT, b"if" => T_KW_IF,
        b"else" => T_KW_ELSE, b"while" => T_KW_WHILE, b"loop" => T_KW_LOOP,
        b"break" => T_KW_BREAK, b"continue" => T_KW_CONTINUE, b"const" => T_KW_CONST,
        b"true" => T_KW_TRUE, b"false" => T_KW_FALSE, _ => T_IDENT,
    }
}
// (kind, ival, pos, len)
fn lex(src: &[u8]) -> Vec<(u64, u64, usize, usize)> {
    let n = src.len(); let mut t = Vec::new(); let mut i = 0;
    while i < n {
        let c = src[i];
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' { i += 1; }
        else if c == b'/' && i + 1 < n && src[i + 1] == b'/' { while i < n && src[i] != b'\n' { i += 1; } }
        else if is_alpha(c) { let s = i; while i < n && is_alnum(src[i]) { i += 1; } t.push((kw(&src[s..i]), 0, s, i - s)); }
        else if is_digit(c) {
            let mut v: u64 = 0; let s = i;
            if c == b'0' && i + 1 < n && src[i + 1] == b'x' {
                i += 2;
                while i < n { let h = src[i];
                    if is_digit(h) { v = v.wrapping_mul(16).wrapping_add((h - b'0') as u64); }
                    else if h >= b'a' && h <= b'f' { v = v.wrapping_mul(16).wrapping_add((h - b'a' + 10) as u64); }
                    else if h >= b'A' && h <= b'F' { v = v.wrapping_mul(16).wrapping_add((h - b'A' + 10) as u64); }
                    else if h == b'_' { } else { break; } i += 1; }
            } else { while i < n && (is_digit(src[i]) || src[i] == b'_') { if src[i] != b'_' { v = v.wrapping_mul(10).wrapping_add((src[i] - b'0') as u64); } i += 1; } }
            while i < n && is_alnum(src[i]) { i += 1; }
            t.push((T_INT, v, s, i - s));
        } else {
            let s = i; let c1 = if i + 1 < n { src[i + 1] } else { 0 };
            macro_rules! p2 { ($k:expr) => {{ t.push(($k, 0, s, 2)); i += 2; }} }
            macro_rules! p1 { ($k:expr) => {{ t.push(($k, 0, s, 1)); i += 1; }} }
            if c == b'-' && c1 == b'>' { p2!(T_ARROW) } else if c == b'=' && c1 == b'=' { p2!(T_EE) }
            else if c == b'!' && c1 == b'=' { p2!(T_NE) } else if c == b'<' && c1 == b'=' { p2!(T_LE) }
            else if c == b'>' && c1 == b'=' { p2!(T_GE) }
            else if c == b'<' && c1 == b'<' { if i + 2 < n && src[i + 2] == b'=' { t.push((T_SHLEQ, 0, s, 3)); i += 3; } else { p2!(T_SHL) } }
            else if c == b'>' && c1 == b'>' { if i + 2 < n && src[i + 2] == b'=' { t.push((T_SHREQ, 0, s, 3)); i += 3; } else { p2!(T_SHR) } }
            else if c == b'&' && c1 == b'&' { p2!(T_AMPAMP) } else if c == b'|' && c1 == b'|' { p2!(T_PIPEPIPE) }
            else if c == b'+' && c1 == b'=' { p2!(T_PLUSEQ) } else if c == b'-' && c1 == b'=' { p2!(T_MINUSEQ) }
            else if c == b'*' && c1 == b'=' { p2!(T_STAREQ) } else if c == b'/' && c1 == b'=' { p2!(T_SLASHEQ) }
            else if c == b'%' && c1 == b'=' { p2!(T_PCTEQ) } else if c == b'&' && c1 == b'=' { p2!(T_AMPEQ) }
            else if c == b'|' && c1 == b'=' { p2!(T_PIPEEQ) } else if c == b'^' && c1 == b'=' { p2!(T_CARETEQ) }
            else if c == b'(' { p1!(T_LP) } else if c == b')' { p1!(T_RP) } else if c == b'{' { p1!(T_LB) }
            else if c == b'}' { p1!(T_RB) } else if c == b',' { p1!(T_COMMA) } else if c == b';' { p1!(T_SEMI) }
            else if c == b':' { p1!(T_COLON) } else if c == b'=' { p1!(T_EQ) } else if c == b'+' { p1!(T_PLUS) }
            else if c == b'-' { p1!(T_MINUS) } else if c == b'*' { p1!(T_STAR) } else if c == b'/' { p1!(T_SLASH) }
            else if c == b'%' { p1!(T_PCT) } else if c == b'&' { p1!(T_AMP) } else if c == b'|' { p1!(T_PIPE) }
            else if c == b'^' { p1!(T_CARET) } else if c == b'<' { p1!(T_LT) } else if c == b'>' { p1!(T_GT) }
            else if c == b'!' { p1!(T_BANG) } else { std::process::exit(2); }
        }
    }
    t.push((T_EOF, 0, n, 0));
    t
}

struct P { tk: Vec<(u64, u64, usize, usize)>, pos: usize, nd: Vec<[u64; 7]>,
           fn_tok: Vec<u64>, fn_body: Vec<u64>, fn_poff: Vec<u64>, fn_np: Vec<u64>, fn_ptok: Vec<u64>,
           const_tok: Vec<u64>, const_expr: Vec<u64> }
impl P {
    fn mk(&mut self, kind: u64) -> u64 { let i = self.nd.len() as u64; self.nd.push([kind, NIL, NIL, NIL, NIL, NIL, NIL]); i }
    fn set(&mut self, n: u64, f: usize, v: u64) { self.nd[n as usize][f] = v; }
    fn cur(&self) -> u64 { self.tk[self.pos].0 }
    fn cur1(&self) -> u64 { self.tk[self.pos + 1].0 }
    fn bump(&mut self) { self.pos += 1; }
    fn expect(&mut self, k: u64) { if self.cur() != k { std::process::exit(3); } self.bump(); }
    fn skip_type(&mut self) { if self.cur() == T_LP { self.bump(); self.expect(T_RP); return; } self.bump(); }

    fn primary(&mut self) -> u64 {
        let k = self.cur();
        if k == T_INT { let n = self.mk(N_INT); self.set(n, 1, self.pos as u64); self.bump(); return n; }
        if k == T_KW_TRUE || k == T_KW_FALSE { let n = self.mk(N_BOOL); self.set(n, 1, if k == T_KW_TRUE { 1 } else { 0 }); self.bump(); return n; }
        if k == T_IDENT {
            if self.cur1() == T_LP {
                let name = self.pos as u64; self.bump(); self.bump();
                let n = self.mk(N_CALL); self.set(n, 1, name);
                let (mut first, mut last) = (NIL, NIL);
                while self.cur() != T_RP {
                    let arg = self.expr(1);
                    if first == NIL { first = arg; } else { self.set(last, 6, arg); }
                    last = arg;
                    if self.cur() != T_RP { self.expect(T_COMMA); }
                }
                self.bump(); self.set(n, 2, first); return n;
            }
            let n = self.mk(N_NAME); self.set(n, 1, self.pos as u64); self.bump(); return n;
        }
        if k == T_LP { self.bump(); let n = self.expr(1); self.expect(T_RP); return n; }
        if k == T_LB { return self.block(); }
        if k == T_KW_IF {
            self.bump(); let cond = self.expr(1); let then = self.block();
            let mut els = NIL;
            if self.cur() == T_KW_ELSE { self.bump(); if self.cur() == T_KW_IF { els = self.primary(); } else { els = self.block(); } }
            let n = self.mk(N_IF); self.set(n, 4, cond); self.set(n, 5, then); self.set(n, 2, els); return n;
        }
        std::process::exit(4);
    }
    fn unary(&mut self) -> u64 {
        if self.cur() == T_BANG { self.bump(); let o = self.unary(); let n = self.mk(N_UNARY); self.set(n, 1, 0); self.set(n, 5, o); return n; }
        self.primary()
    }
    fn binop(k: u64) -> u64 {
        match k { T_PIPEPIPE => OP_OR, T_AMPAMP => OP_AND, T_EE => OP_EQ, T_NE => OP_NE, T_LT => OP_LT,
            T_LE => OP_LE, T_GT => OP_GT, T_GE => OP_GE, T_PIPE => OP_BOR, T_CARET => OP_BXOR, T_AMP => OP_BAND,
            T_SHL => OP_SHL, T_SHR => OP_SHR, T_PLUS => OP_ADD, T_MINUS => OP_SUB, T_STAR => OP_MUL,
            T_SLASH => OP_DIV, T_PCT => OP_REM, _ => 0 }
    }
    fn binprec(k: u64) -> u64 {
        match k { T_PIPEPIPE => 1, T_AMPAMP => 2, T_EE | T_NE | T_LT | T_LE | T_GT | T_GE => 3, T_PIPE => 4,
            T_CARET => 5, T_AMP => 6, T_SHL | T_SHR => 7, T_PLUS | T_MINUS => 8, T_STAR | T_SLASH | T_PCT => 9, _ => 0 }
    }
    fn assign_op(k: u64) -> u64 {
        match k { T_EQ => 0, T_PLUSEQ => OP_ADD, T_MINUSEQ => OP_SUB, T_STAREQ => OP_MUL, T_SLASHEQ => OP_DIV,
            T_PCTEQ => OP_REM, T_AMPEQ => OP_BAND, T_PIPEEQ => OP_BOR, T_CARETEQ => OP_BXOR, T_SHLEQ => OP_SHL,
            T_SHREQ => OP_SHR, _ => NIL }
    }
    fn expr(&mut self, min: u64) -> u64 {
        let mut lhs = self.unary();
        loop {
            let k = self.cur(); let op = Self::binop(k); let prec = Self::binprec(k);
            if op == 0 || prec < min { break; }
            self.bump(); let rhs = self.expr(prec + 1);
            let n = self.mk(N_BIN); self.set(n, 1, op); self.set(n, 4, lhs); self.set(n, 5, rhs); lhs = n;
        }
        lhs
    }
    fn block(&mut self) -> u64 {
        self.expect(T_LB);
        let n = self.mk(N_BLOCK);
        let (mut first, mut last, mut tail) = (NIL, NIL, NIL);
        while self.cur() != T_RB {
            let k = self.cur();
            let s;
            if k == T_SEMI { self.bump(); continue; }
            else if k == T_KW_LET {
                self.bump(); if self.cur() == T_KW_MUT { self.bump(); }
                let name = self.pos as u64; self.expect(T_IDENT);
                if self.cur() == T_COLON { self.bump(); self.skip_type(); }
                self.expect(T_EQ); let init = self.expr(1); self.expect(T_SEMI);
                s = self.mk(N_LET); self.set(s, 1, name); self.set(s, 5, init);
            } else if k == T_KW_WHILE {
                self.bump(); let cond = self.expr(1); let body = self.block();
                s = self.mk(N_WHILE); self.set(s, 4, cond); self.set(s, 5, body);
            } else if k == T_KW_LOOP {
                self.bump(); let body = self.block(); s = self.mk(N_LOOP); self.set(s, 5, body);
            } else if k == T_KW_BREAK { self.bump(); self.expect(T_SEMI); s = self.mk(N_BREAK); }
            else if k == T_KW_CONTINUE { self.bump(); self.expect(T_SEMI); s = self.mk(N_CONTINUE); }
            else {
                let blocklike = k == T_KW_IF || k == T_LB;
                let e = self.expr(1);
                if blocklike {
                    if self.cur() == T_RB { tail = e; continue; }
                    else { s = self.mk(N_EXPR); self.set(s, 5, e); }
                } else {
                    let ak = Self::assign_op(self.cur());
                    if ak != NIL {
                        self.bump(); let val = self.expr(1); self.expect(T_SEMI);
                        s = self.mk(N_ASSIGN); self.set(s, 1, ak); let place = self.nd[e as usize][1]; self.set(s, 4, place); self.set(s, 5, val);
                    } else if self.cur() == T_SEMI { self.bump(); s = self.mk(N_EXPR); self.set(s, 5, e); }
                    else if self.cur() == T_RB { tail = e; continue; }
                    else { std::process::exit(5); }
                }
            }
            if first == NIL { first = s; } else { self.set(last, 6, s); }
            last = s;
        }
        self.bump();
        self.set(n, 2, first); self.set(n, 5, tail); n
    }
    fn program(&mut self) {
        while self.cur() != T_EOF {
            if self.cur() == T_KW_CONST {
                self.bump(); self.const_tok.push(self.pos as u64);
                self.expect(T_IDENT); self.expect(T_COLON); self.skip_type(); self.expect(T_EQ);
                let ce = self.expr(1); self.expect(T_SEMI); self.const_expr.push(ce); continue;
            }
            self.expect(T_KW_FN);
            let name = self.pos as u64; self.expect(T_IDENT); self.expect(T_LP);
            self.fn_tok.push(name); self.fn_poff.push(self.fn_ptok.len() as u64); let mut np = 0u64;
            while self.cur() != T_RP {
                if self.cur() == T_KW_MUT { self.bump(); }
                self.fn_ptok.push(self.pos as u64); np += 1;
                self.expect(T_IDENT); self.expect(T_COLON); self.skip_type();
                if self.cur() != T_RP { self.expect(T_COMMA); }
            }
            self.bump();
            if self.cur() == T_ARROW { self.bump(); self.skip_type(); }
            let body = self.block(); self.fn_body.push(body); self.fn_np.push(np);
        }
    }
}

fn main() {
    let mut src = Vec::new();
    std::io::stdin().read_to_end(&mut src).unwrap();
    let tk = lex(&src);
    let mut p = P { tk, pos: 0, nd: Vec::new(), fn_tok: Vec::new(), fn_body: Vec::new(),
                    fn_poff: Vec::new(), fn_np: Vec::new(), fn_ptok: Vec::new(),
                    const_tok: Vec::new(), const_expr: Vec::new() };
    let mode = std::env::var("SR_PARSE").unwrap_or_else(|_| "expr".into());
    let mut out = Vec::new();
    if mode == "program" {
        p.program();
        // header: node_n, fn_n, ptok_n, const_n
        out.extend_from_slice(&(p.nd.len() as u64).to_le_bytes());
        out.extend_from_slice(&(p.fn_tok.len() as u64).to_le_bytes());
        out.extend_from_slice(&(p.fn_ptok.len() as u64).to_le_bytes());
        out.extend_from_slice(&(p.const_tok.len() as u64).to_le_bytes());
        for row in &p.nd { for w in row { out.extend_from_slice(&w.to_le_bytes()); } }
        for k in 0..p.fn_tok.len() {
            out.extend_from_slice(&p.fn_tok[k].to_le_bytes());
            out.extend_from_slice(&p.fn_body[k].to_le_bytes());
            out.extend_from_slice(&p.fn_np[k].to_le_bytes());
            out.extend_from_slice(&p.fn_poff[k].to_le_bytes());
        }
        for t in &p.fn_ptok { out.extend_from_slice(&t.to_le_bytes()); }
        for k in 0..p.const_tok.len() {
            out.extend_from_slice(&p.const_tok[k].to_le_bytes());
            out.extend_from_slice(&p.const_expr[k].to_le_bytes());
        }
    } else {
        if mode == "block" { let _ = p.block(); } else { let _ = p.expr(1); }
        for row in &p.nd { for w in row { out.extend_from_slice(&w.to_le_bytes()); } }
    }
    std::io::stdout().write_all(&out).unwrap();
}
