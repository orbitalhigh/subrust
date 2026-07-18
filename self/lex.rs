
const E_UNEXPECTED_CHAR: u16 = 0x0101;
const E_UNTERMINATED_STRING: u16 = 0x0102;
const E_BAD_ESCAPE: u16 = 0x0103;
const E_UNTERMINATED_COMMENT: u16 = 0x0104;
const E_DOC_COMMENT: u16 = 0x0105;
const E_BAD_NUMBER: u16 = 0x0106;
const E_BAD_SUFFIX: u16 = 0x0107;
const E_CHAR_LITERAL: u16 = 0x0108;
const E_NON_ASCII: u16 = 0x0109;
const E_STR_TOO_LONG: u16 = 0x010A;
const E_SRC_TOO_BIG: u16 = 0x0001;
const E_TOKEN_TOO_LONG: u16 = 0x0004;
const SRC_MAX: usize = 16777216;
const CAP_TOKS: usize = 256;
const CAP_DIAGS: usize = 16;
#[derive(Clone, Copy)]
struct Diag { code: u16, lo: u32, hi: u32, a: u32, b: u32 }

pub const T_EOF: u16 = 0;
pub const T_IDENT: u16 = 1;
pub const T_INT: u16 = 2;
pub const T_FLOAT: u16 = 3;
pub const T_STR: u16 = 4;
pub const T_UNDERSCORE: u16 = 5;
pub const T_BSTR: u16 = 6;
pub const T_BYTE: u16 = 7;

pub const T_KW_FN: u16 = 10;
pub const T_KW_LET: u16 = 11;
pub const T_KW_MUT: u16 = 12;
pub const T_KW_IF: u16 = 13;
pub const T_KW_ELSE: u16 = 14;
pub const T_KW_WHILE: u16 = 15;
pub const T_KW_LOOP: u16 = 16;
pub const T_KW_FOR: u16 = 17;
pub const T_KW_IN: u16 = 18;
pub const T_KW_BREAK: u16 = 19;
pub const T_KW_CONTINUE: u16 = 20;
pub const T_KW_MATCH: u16 = 21;
pub const T_KW_STRUCT: u16 = 22;
pub const T_KW_CONST: u16 = 23;
pub const T_KW_USE: u16 = 24;
pub const T_KW_AS: u16 = 25;
pub const T_KW_TRUE: u16 = 26;
pub const T_KW_FALSE: u16 = 27;
pub const T_KW_IMPL: u16 = 29;
pub const T_KW_SELF: u16 = 30;
pub const T_KW_RETURN: u16 = 31;
pub const T_KW_PUB: u16 = 32;
pub const T_KW_ENUM: u16 = 33;

pub const T_KW_RESERVED: u16 = 28;

pub const T_LPAREN: u16 = 40;
pub const T_RPAREN: u16 = 41;
pub const T_LBRACE: u16 = 42;
pub const T_RBRACE: u16 = 43;
pub const T_LBRACK: u16 = 44;
pub const T_RBRACK: u16 = 45;
pub const T_COMMA: u16 = 46;
pub const T_SEMI: u16 = 47;
pub const T_COLON: u16 = 48;
pub const T_COLONCOLON: u16 = 49;
pub const T_POUND: u16 = 50;
pub const T_DOT: u16 = 51;
pub const T_DOTDOT: u16 = 52;
pub const T_DOTDOTEQ: u16 = 53;
pub const T_EQ: u16 = 54;
pub const T_EQEQ: u16 = 55;
pub const T_NE: u16 = 56;
pub const T_FATARROW: u16 = 57;
pub const T_ARROW: u16 = 58;
pub const T_LT: u16 = 59;
pub const T_LE: u16 = 60;
pub const T_SHL: u16 = 61;
pub const T_GT: u16 = 62;
pub const T_GE: u16 = 63;
pub const T_SHR: u16 = 64;
pub const T_PLUS: u16 = 65;
pub const T_MINUS: u16 = 66;
pub const T_STAR: u16 = 67;
pub const T_SLASH: u16 = 68;
pub const T_PERCENT: u16 = 69;
pub const T_AMP: u16 = 70;
pub const T_AMPAMP: u16 = 71;
pub const T_PIPE: u16 = 72;
pub const T_PIPEPIPE: u16 = 73;
pub const T_CARET: u16 = 74;
pub const T_BANG: u16 = 75;

pub const T_PLUSEQ: u16 = 80;
pub const T_MINUSEQ: u16 = 81;
pub const T_STAREQ: u16 = 82;
pub const T_SLASHEQ: u16 = 83;
pub const T_PERCENTEQ: u16 = 84;
pub const T_AMPEQ: u16 = 85;
pub const T_PIPEEQ: u16 = 86;
pub const T_CARETEQ: u16 = 87;
pub const T_SHLEQ: u16 = 88;
pub const T_SHREQ: u16 = 89;

pub const TOK_LEN_MAX: usize = 65535;

#[derive(Clone, Copy)]
pub struct Tok {
    pub kind: u16,
    pub len: u16,
    pub pos: u32,
}

pub const TOK_NONE: Tok = Tok {
    kind: T_EOF,
    len: 0,
    pos: 0,
};

fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn is_hex(c: u8) -> bool {
    is_digit(c) || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')
}

fn is_alpha(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z')
}

fn is_ident_start(c: u8) -> bool {
    c == b'_' || is_alpha(c)
}

fn is_ident_char(c: u8) -> bool {
    c == b'_' || is_alpha(c) || is_digit(c)
}

fn scan_str_body(b: &[u8], n: usize, i: &mut usize, mem: &mut Mem, start: usize) {
    loop {
        if *i >= n {
            mem.diag(E_UNTERMINATED_STRING, start as u32, n as u32, 0, 0);
            break;
        }
        let s = b[*i];
        if s == b'"' {
            *i += 1;
            break;
        }
        if s == b'\\' {
            if *i + 1 >= n {
                mem.diag(E_UNTERMINATED_STRING, start as u32, n as u32, 0, 0);
                *i = n;
                break;
            }
            let e = b[*i + 1];
            if e == b'n' || e == b'r' || e == b't' || e == b'0' || e == b'\\'
                || e == b'"' || e == b'\''
            {
                *i += 2;
            } else {
                mem.diag(E_BAD_ESCAPE, *i as u32, (*i + 2) as u32, 0, 0);
                *i += 2;
            }
        } else {

            *i += 1;
        }
    }
    if *i - start > TOK_LEN_MAX {
        mem.diag(E_STR_TOO_LONG, start as u32, *i as u32, 0, 0);
    }
}

fn scan_byte_lit(b: &[u8], n: usize, i: &mut usize, mem: &mut Mem, start: usize) {
    if *i < n && b[*i] == b'\\' {
        if *i + 1 < n {
            let e = b[*i + 1];
            if !(e == b'n' || e == b'r' || e == b't' || e == b'0' || e == b'\\'
                || e == b'\'' || e == b'"')
            {
                mem.diag(E_BAD_ESCAPE, *i as u32, (*i + 2) as u32, 0, 0);
            }
            *i += 2;
        } else {
            mem.diag(E_CHAR_LITERAL, start as u32, n as u32, 0, 0);
            *i = n;
            return;
        }
    } else if *i < n && b[*i] != b'\'' && b[*i] < 128 {
        *i += 1;
    } else {

        mem.diag(E_CHAR_LITERAL, start as u32, *i as u32, 0, 0);
        if *i < n && b[*i] != b'\'' {
            *i += 1;
        }
    }
    if *i < n && b[*i] == b'\'' {
        *i += 1;
    } else {
        mem.diag(E_CHAR_LITERAL, start as u32, *i as u32, 0, 0);
    }
}

fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
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

fn kw_lookup(w: &[u8]) -> u16 {

    if bytes_eq(w, b"fn") {
        return T_KW_FN;
    }
    if bytes_eq(w, b"let") {
        return T_KW_LET;
    }
    if bytes_eq(w, b"mut") {
        return T_KW_MUT;
    }
    if bytes_eq(w, b"if") {
        return T_KW_IF;
    }
    if bytes_eq(w, b"else") {
        return T_KW_ELSE;
    }
    if bytes_eq(w, b"while") {
        return T_KW_WHILE;
    }
    if bytes_eq(w, b"loop") {
        return T_KW_LOOP;
    }
    if bytes_eq(w, b"for") {
        return T_KW_FOR;
    }
    if bytes_eq(w, b"in") {
        return T_KW_IN;
    }
    if bytes_eq(w, b"break") {
        return T_KW_BREAK;
    }
    if bytes_eq(w, b"continue") {
        return T_KW_CONTINUE;
    }
    if bytes_eq(w, b"match") {
        return T_KW_MATCH;
    }
    if bytes_eq(w, b"struct") {
        return T_KW_STRUCT;
    }
    if bytes_eq(w, b"enum") {
        return T_KW_ENUM;
    }
    if bytes_eq(w, b"const") {
        return T_KW_CONST;
    }
    if bytes_eq(w, b"use") {
        return T_KW_USE;
    }
    if bytes_eq(w, b"as") {
        return T_KW_AS;
    }
    if bytes_eq(w, b"true") {
        return T_KW_TRUE;
    }
    if bytes_eq(w, b"false") {
        return T_KW_FALSE;
    }
    if bytes_eq(w, b"impl") {
        return T_KW_IMPL;
    }
    if bytes_eq(w, b"self") {
        return T_KW_SELF;
    }
    if bytes_eq(w, b"return") {
        return T_KW_RETURN;
    }
    if bytes_eq(w, b"pub") {
        return T_KW_PUB;
    }

    if bytes_eq(w, b"abstract")
        || bytes_eq(w, b"async")
        || bytes_eq(w, b"await")
        || bytes_eq(w, b"become")
        || bytes_eq(w, b"box")
        || bytes_eq(w, b"crate")
        || bytes_eq(w, b"do")
        || bytes_eq(w, b"dyn")
        || bytes_eq(w, b"extern")
        || bytes_eq(w, b"final")
        || bytes_eq(w, b"macro")
        || bytes_eq(w, b"mod")
        || bytes_eq(w, b"move")
        || bytes_eq(w, b"override")
        || bytes_eq(w, b"priv")
        || bytes_eq(w, b"ref")
        || bytes_eq(w, b"Self")
        || bytes_eq(w, b"static")
        || bytes_eq(w, b"super")
        || bytes_eq(w, b"trait")
        || bytes_eq(w, b"try")
        || bytes_eq(w, b"type")
        || bytes_eq(w, b"typeof")
        || bytes_eq(w, b"unsafe")
        || bytes_eq(w, b"unsized")
        || bytes_eq(w, b"virtual")
        || bytes_eq(w, b"where")
        || bytes_eq(w, b"yield")
    {
        return T_KW_RESERVED;
    }
    T_IDENT
}

fn suffix_kind(kind: u16, s: &[u8]) -> u16 {
    if kind == T_INT {
        if bytes_eq(s, b"i8")
            || bytes_eq(s, b"u8")
            || bytes_eq(s, b"i16")
            || bytes_eq(s, b"u16")
            || bytes_eq(s, b"isize")
            || bytes_eq(s, b"i32")
            || bytes_eq(s, b"u32")
            || bytes_eq(s, b"i64")
            || bytes_eq(s, b"u64")
            || bytes_eq(s, b"i128")
            || bytes_eq(s, b"u128")
            || bytes_eq(s, b"usize")
        {
            return T_INT;
        }
        if bytes_eq(s, b"f64") {
            return T_FLOAT;
        }
        return T_EOF;
    }

    if bytes_eq(s, b"f64") {
        return T_FLOAT;
    }
    T_EOF
}

fn push(mem: &mut Mem, kind: u16, pos: usize, end: usize) {
    let mut len = end - pos;
    if len > TOK_LEN_MAX {
        mem.diag(E_TOKEN_TOO_LONG, pos as u32, end as u32, 0, 0);
        len = TOK_LEN_MAX;
    }
    mem.push_tok(Tok {
        kind,
        len: len as u16,
        pos: pos as u32,
    });
}

pub fn lex(src: &str, mem: &mut Mem) -> bool {
    mem.reset();
    if src.len() > SRC_MAX {
        mem.diag(E_SRC_TOO_BIG, 0, 0, 0, 0);
        return false;
    }
    let b = src.as_bytes();
    let n = b.len();
    let mut i: usize = 0;

    loop {

        loop {
            while i < n && (b[i] == b' ' || b[i] == b'\t' || b[i] == b'\r' || b[i] == b'\n') {
                i += 1;
            }
            if i + 1 < n && b[i] == b'/' && b[i + 1] == b'/' {

                let doc = (i + 2 < n && b[i + 2] == b'!')
                    || (i + 2 < n && b[i + 2] == b'/' && !(i + 3 < n && b[i + 3] == b'/'));
                if doc {
                    mem.diag(E_DOC_COMMENT, i as u32, (i + 3) as u32, 0, 0);
                }
                while i < n && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if i + 1 < n && b[i] == b'/' && b[i + 1] == b'*' {

                let doc = (i + 2 < n && b[i + 2] == b'!')
                    || (i + 2 < n
                        && b[i + 2] == b'*'
                        && !(i + 3 < n && (b[i + 3] == b'*' || b[i + 3] == b'/')));
                if doc {
                    mem.diag(E_DOC_COMMENT, i as u32, (i + 3) as u32, 0, 0);
                }
                let open = i;
                i += 2;
                let mut depth = 1;
                while i < n && depth > 0 {
                    if i + 1 < n && b[i] == b'/' && b[i + 1] == b'*' {
                        depth += 1;
                        i += 2;
                    } else if i + 1 < n && b[i] == b'*' && b[i + 1] == b'/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if depth > 0 {
                    mem.diag(E_UNTERMINATED_COMMENT, open as u32, n as u32, 0, 0);
                }
                continue;
            }
            break;
        }
        if i >= n {
            break;
        }

        let start = i;
        let c = b[i];

        if c == b'b' && i + 1 < n && b[i + 1] == b'"' {
            i += 2;
            scan_str_body(b, n, &mut i, mem, start);
            push(mem, T_BSTR, start, i);
            continue;
        }
        if c == b'b' && i + 1 < n && b[i + 1] == b'\'' {
            i += 2;
            scan_byte_lit(b, n, &mut i, mem, start);
            push(mem, T_BYTE, start, i);
            continue;
        }

        if is_ident_start(c) {
            i += 1;
            while i < n && is_ident_char(b[i]) {
                i += 1;
            }
            let w = &b[start..i];
            let mut kind = kw_lookup(w);
            if kind == T_IDENT && w.len() == 1 && w[0] == b'_' {
                kind = T_UNDERSCORE;
            }
            push(mem, kind, start, i);
            continue;
        }

        if is_digit(c) {
            let mut kind = T_INT;
            if c == b'0' && i + 1 < n && (b[i + 1] == b'x' || b[i + 1] == b'X') {
                i += 2;
                let mut digits = 0;
                while i < n && (is_hex(b[i]) || b[i] == b'_') {
                    if b[i] != b'_' {
                        digits += 1;
                    }
                    i += 1;
                }
                if digits == 0 {
                    mem.diag(E_BAD_NUMBER, start as u32, i as u32, 0, 0);
                }
                if i < n && b[i] == b'.' {
                    mem.diag(E_BAD_NUMBER, start as u32, (i + 1) as u32, 0, 0);
                    i += 1;
                }
            } else {
                while i < n && (is_digit(b[i]) || b[i] == b'_') {
                    i += 1;
                }

                if i + 1 < n && b[i] == b'.' && is_digit(b[i + 1]) {
                    kind = T_FLOAT;
                    i += 1;
                    while i < n && (is_digit(b[i]) || b[i] == b'_') {
                        i += 1;
                    }
                }

                if i < n && (b[i] == b'e' || b[i] == b'E') {
                    let mut j = i + 1;
                    if j < n && (b[j] == b'+' || b[j] == b'-') {
                        j += 1;
                    }
                    if j < n && is_digit(b[j]) {
                        kind = T_FLOAT;
                        i = j;
                        while i < n && (is_digit(b[i]) || b[i] == b'_') {
                            i += 1;
                        }
                    }

                }
            }

            let sfx = i;
            while i < n && is_ident_char(b[i]) {
                i += 1;
            }
            if i > sfx {
                let k = suffix_kind(kind, &b[sfx..i]);
                if k == T_EOF {
                    mem.diag(E_BAD_SUFFIX, sfx as u32, i as u32, 0, 0);
                } else {
                    kind = k;
                }
            }
            push(mem, kind, start, i);
            continue;
        }

        if c == b'"' {
            i += 1;
            scan_str_body(b, n, &mut i, mem, start);
            push(mem, T_STR, start, i);
            continue;
        }

        let c1 = if i + 1 < n { b[i + 1] } else { 0 };
        let c2 = if i + 2 < n { b[i + 2] } else { 0 };
        let mut kind = T_EOF;
        let mut adv: usize = 1;
        match c {
            b'(' => kind = T_LPAREN,
            b')' => kind = T_RPAREN,
            b'{' => kind = T_LBRACE,
            b'}' => kind = T_RBRACE,
            b'[' => kind = T_LBRACK,
            b']' => kind = T_RBRACK,
            b',' => kind = T_COMMA,
            b';' => kind = T_SEMI,
            b'#' => kind = T_POUND,
            b':' => {
                if c1 == b':' {
                    kind = T_COLONCOLON;
                    adv = 2;
                } else {
                    kind = T_COLON;
                }
            }
            b'.' => {
                if c1 == b'.' {
                    if c2 == b'=' {
                        kind = T_DOTDOTEQ;
                        adv = 3;
                    } else {
                        kind = T_DOTDOT;
                        adv = 2;
                    }
                } else {
                    kind = T_DOT;
                }
            }
            b'=' => {
                if c1 == b'=' {
                    kind = T_EQEQ;
                    adv = 2;
                } else if c1 == b'>' {
                    kind = T_FATARROW;
                    adv = 2;
                } else {
                    kind = T_EQ;
                }
            }
            b'!' => {
                if c1 == b'=' {
                    kind = T_NE;
                    adv = 2;
                } else {
                    kind = T_BANG;
                }
            }
            b'<' => {
                if c1 == b'=' {
                    kind = T_LE;
                    adv = 2;
                } else if c1 == b'<' {
                    if c2 == b'=' {
                        kind = T_SHLEQ;
                        adv = 3;
                    } else {
                        kind = T_SHL;
                        adv = 2;
                    }
                } else {
                    kind = T_LT;
                }
            }
            b'>' => {
                if c1 == b'=' {
                    kind = T_GE;
                    adv = 2;
                } else if c1 == b'>' {
                    if c2 == b'=' {
                        kind = T_SHREQ;
                        adv = 3;
                    } else {
                        kind = T_SHR;
                        adv = 2;
                    }
                } else {
                    kind = T_GT;
                }
            }
            b'+' => {
                if c1 == b'=' {
                    kind = T_PLUSEQ;
                    adv = 2;
                } else {
                    kind = T_PLUS;
                }
            }
            b'-' => {
                if c1 == b'=' {
                    kind = T_MINUSEQ;
                    adv = 2;
                } else if c1 == b'>' {
                    kind = T_ARROW;
                    adv = 2;
                } else {
                    kind = T_MINUS;
                }
            }
            b'*' => {
                if c1 == b'=' {
                    kind = T_STAREQ;
                    adv = 2;
                } else {
                    kind = T_STAR;
                }
            }
            b'/' => {

                if c1 == b'=' {
                    kind = T_SLASHEQ;
                    adv = 2;
                } else {
                    kind = T_SLASH;
                }
            }
            b'%' => {
                if c1 == b'=' {
                    kind = T_PERCENTEQ;
                    adv = 2;
                } else {
                    kind = T_PERCENT;
                }
            }
            b'&' => {
                if c1 == b'&' {
                    kind = T_AMPAMP;
                    adv = 2;
                } else if c1 == b'=' {
                    kind = T_AMPEQ;
                    adv = 2;
                } else {
                    kind = T_AMP;
                }
            }
            b'|' => {
                if c1 == b'|' {
                    kind = T_PIPEPIPE;
                    adv = 2;
                } else if c1 == b'=' {
                    kind = T_PIPEEQ;
                    adv = 2;
                } else {
                    kind = T_PIPE;
                }
            }
            b'^' => {
                if c1 == b'=' {
                    kind = T_CARETEQ;
                    adv = 2;
                } else {
                    kind = T_CARET;
                }
            }
            b'\'' => {

                let mut j = i + 1;
                if j < n && b[j] == b'\\' {
                    j += 1;
                }
                if j < n {
                    j += 1;
                }
                if j < n && b[j] == b'\'' {
                    j += 1;
                }
                mem.diag(E_CHAR_LITERAL, i as u32, j as u32, 0, 0);
                i = j;
            }
            _ => {
                if c >= 0x80 {

                    let mut j = i + 1;
                    while j < n && (b[j] & 0xC0) == 0x80 {
                        j += 1;
                    }
                    mem.diag(E_NON_ASCII, i as u32, j as u32, 0, 0);
                    i = j;
                } else {
                    mem.diag(E_UNEXPECTED_CHAR, i as u32, (i + 1) as u32, 0, 0);
                    i += 1;
                }
            }
        }
        if kind != T_EOF {
            push(mem, kind, start, start + adv);
            i = start + adv;
        }
    }

    push(mem, T_EOF, n, n);
    mem.diag_n == 0
}

#[derive(Clone, Copy)]
struct Mem { toks: [Tok; CAP_TOKS], tok_n: usize, diags: [Diag; CAP_DIAGS], diag_n: usize }
impl Mem {
    fn reset(&mut self) { self.tok_n = 0; self.diag_n = 0; }
    fn diag(&mut self, code: u16, lo: u32, hi: u32, a: u32, b: u32) {
        if self.diag_n < CAP_DIAGS {
            self.diags[self.diag_n] = Diag { code: code, lo: lo, hi: hi, a: a, b: b };
            self.diag_n += 1;
        }
    }
    fn push_tok(&mut self, t: Tok) {
        if self.tok_n < CAP_TOKS { self.toks[self.tok_n] = t; self.tok_n += 1; }
    }
}
fn main() {

    let src = "fn add(a: i64) -> i64 { let x = 0xFF; return a == x <= 1; } // done";
    let mut m = Mem {
        toks: [Tok { kind: 0, len: 0, pos: 0 }; CAP_TOKS], tok_n: 0,
        diags: [Diag { code: 0, lo: 0, hi: 0, a: 0, b: 0 }; CAP_DIAGS], diag_n: 0,
    };
    let ok = lex(src, &mut m);
    print_bool(ok);
    print_usize(m.tok_n);
    print_usize(m.diag_n);
    let mut i: usize = 0;
    while i < m.tok_n {
        let t = m.toks[i];
        print_i64(t.kind as i64); print_i64(t.pos as i64); print_i64(t.len as i64);
        i += 1;
    }
}
