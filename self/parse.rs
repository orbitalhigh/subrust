
const SRC_MAX: usize = 16777216;
const CAP_TOKS: usize = 48;
const CAP_NODES: usize = 40;
const CAP_DIAGS: usize = 8;
pub const E_SRC_TOO_BIG: u16 = 0x0001;
pub const E_TOO_MANY_TOKENS: u16 = 0x0002;
pub const E_TOO_MANY_NODES: u16 = 0x0003;
pub const E_TOKEN_TOO_LONG: u16 = 0x0004;
pub const E_UNEXPECTED_CHAR: u16 = 0x0101;
pub const E_UNTERMINATED_STRING: u16 = 0x0102;
pub const E_BAD_ESCAPE: u16 = 0x0103;
pub const E_UNTERMINATED_COMMENT: u16 = 0x0104;
pub const E_DOC_COMMENT: u16 = 0x0105;
pub const E_BAD_NUMBER: u16 = 0x0106;
pub const E_BAD_SUFFIX: u16 = 0x0107;
pub const E_CHAR_LITERAL: u16 = 0x0108;
pub const E_NON_ASCII: u16 = 0x0109;
pub const E_STR_TOO_LONG: u16 = 0x010A;
pub const E_EXPECTED_TOKEN: u16 = 0x0201;
pub const E_EXPECTED_ITEM: u16 = 0x0202;
pub const E_EXPECTED_EXPR: u16 = 0x0203;
pub const E_EXPECTED_TYPE: u16 = 0x0204;
pub const E_EXPECTED_PATTERN: u16 = 0x0205;
pub const E_RESERVED_KEYWORD: u16 = 0x0206;
pub const E_TOO_DEEP: u16 = 0x0207;
pub const E_CHAINED_COMPARISON: u16 = 0x0208;
pub const E_STRUCT_LIT_HERE: u16 = 0x0209;
pub const E_RANGE_HERE: u16 = 0x020A;
pub const E_METHOD_CALL: u16 = 0x020B;
pub const E_CALL_NOT_NAME: u16 = 0x020C;
pub const E_BAD_ATTR: u16 = 0x020D;
pub const E_BAD_DERIVE: u16 = 0x020E;
pub const E_TUPLE: u16 = 0x020F;
pub const E_STRUCT_UPDATE: u16 = 0x0210;
pub const E_DUP_NAME: u16 = 0x0301;
pub const E_UNDEFINED: u16 = 0x0302;
pub const E_FN_AS_VALUE: u16 = 0x0303;
pub const E_UNKNOWN_FN: u16 = 0x0304;
pub const E_ARG_COUNT: u16 = 0x0305;
pub const E_TYPE_MISMATCH: u16 = 0x0306;
pub const E_LIT_OUT_OF_RANGE: u16 = 0x0307;
pub const E_BAD_CAST: u16 = 0x0308;
pub const E_UNKNOWN_TYPE: u16 = 0x0309;
pub const E_NOT_A_STRUCT: u16 = 0x030A;
pub const E_UNKNOWN_FIELD: u16 = 0x030B;
pub const E_MISSING_FIELD: u16 = 0x030C;
pub const E_DUP_FIELD: u16 = 0x030D;
pub const E_NOT_AN_ARRAY: u16 = 0x030E;
pub const E_ANNOTATION_NEEDED: u16 = 0x030F;
pub const E_NOT_EXHAUSTIVE: u16 = 0x0310;
pub const E_PATTERN_TYPE: u16 = 0x0311;
pub const E_ASSIGN_NOT_PLACE: u16 = 0x0312;
pub const E_ASSIGN_IMMUTABLE: u16 = 0x0313;
pub const E_BREAK_OUTSIDE_LOOP: u16 = 0x0314;
pub const E_CONST_CYCLE: u16 = 0x0315;
pub const E_NOT_CONST: u16 = 0x0316;
pub const E_CONST_TYPE: u16 = 0x0317;
pub const E_MISSING_DERIVE: u16 = 0x0318;
pub const E_FRAME_TOO_BIG: u16 = 0x0319;
pub const E_TOO_MANY_ITEMS: u16 = 0x031A;
pub const E_NO_ELSE: u16 = 0x031B;
pub const E_NEG_UNSIGNED: u16 = 0x031C;
pub const E_USE_UNSUPPORTED: u16 = 0x031D;
pub const E_RECURSIVE_STRUCT: u16 = 0x031E;
pub const E_STR_FIELD: u16 = 0x031F;
pub const E_BAD_OPERAND: u16 = 0x0320;
pub const E_CONST_OVERFLOW: u16 = 0x0321;
pub const E_MISSING_TAIL: u16 = 0x0322;
pub const E_BAD_PATH: u16 = 0x0323;
pub const E_NOT_PLACE_REF: u16 = 0x0324;
pub const E_DEREF_NON_REF: u16 = 0x0325;
pub const E_REF_ESCAPES: u16 = 0x0326;
pub const E_REF_MUT_NEEDED: u16 = 0x0327;
pub const E_UNKNOWN_METHOD: u16 = 0x0328;
pub const E_BAD_RECEIVER: u16 = 0x0329;
pub const E_SUBSLICE_REF: u16 = 0x032A;
pub const E_UNKNOWN_VARIANT: u16 = 0x032B;
pub const E_ENUM_PAYLOAD: u16 = 0x032C;
pub const E_BAD_MACRO: u16 = 0x032D;
pub const E_ASSERT_MSG: u16 = 0x032E;
pub const E_T_ARITH: u16 = 0x0401;
pub const E_T_OOB: u16 = 0x0402;
pub const E_T_FUEL: u16 = 0x0403;
pub const E_T_STACK: u16 = 0x0404;
pub const E_T_HOST: u16 = 0x0405;
pub const E_T_NO_ENTRY: u16 = 0x0406;
pub const E_T_ASSERT: u16 = 0x0407;
pub const E_T_INTERNAL: u16 = 0x040F;

pub const NODE_NIL: u32 = 0xFFFF_FFFF;

pub const N_FN: u16 = 1;
pub const N_STRUCT: u16 = 2;
pub const N_CONST: u16 = 3;
pub const N_USE: u16 = 4;
pub const N_PARAM: u16 = 5;
pub const N_FIELD: u16 = 6;
pub const N_USE_SEG: u16 = 7;
pub const N_IMPL: u16 = 8;

pub const N_TY_NAME: u16 = 10;
pub const N_TY_STR: u16 = 11;
pub const N_TY_ARRAY: u16 = 12;
pub const N_TY_UNIT: u16 = 13;
pub const N_TY_REF: u16 = 14;
pub const N_TY_SLICE: u16 = 15;

pub const N_LIT_INT: u16 = 20;
pub const N_LIT_FLOAT: u16 = 21;
pub const N_LIT_STR: u16 = 22;
pub const N_LIT_BSTR: u16 = 64;
pub const N_LIT_BYTE: u16 = 65;
pub const N_SLICE: u16 = 66;
pub const N_PAT_BYTE: u16 = 67;
pub const N_LIT_BOOL: u16 = 23;
pub const N_LIT_UNIT: u16 = 24;
pub const N_NAME: u16 = 25;
pub const N_CALL: u16 = 26;
pub const N_DOT: u16 = 27;
pub const N_INDEX: u16 = 28;
pub const N_UNARY: u16 = 29;
pub const N_BINARY: u16 = 30;
pub const N_CAST: u16 = 31;
pub const N_STRUCT_LIT: u16 = 32;
pub const N_FIELD_INIT: u16 = 33;
pub const N_ARRAY_LIT: u16 = 34;
pub const N_ARRAY_REPEAT: u16 = 35;
pub const N_IF: u16 = 36;
pub const N_MATCH: u16 = 37;
pub const N_ARM: u16 = 38;
pub const N_BLOCK: u16 = 39;

pub const N_PAT_INT: u16 = 45;
pub const N_PAT_STR: u16 = 46;
pub const N_PAT_BOOL: u16 = 47;
pub const N_PAT_WILD: u16 = 48;
pub const N_PAT_CONST: u16 = 49;

pub const N_LET: u16 = 50;
pub const N_ASSIGN: u16 = 51;
pub const N_EXPR_STMT: u16 = 52;
pub const N_WHILE: u16 = 53;
pub const N_LOOP: u16 = 54;
pub const N_FOR: u16 = 55;
pub const N_BREAK: u16 = 56;
pub const N_CONTINUE: u16 = 57;
pub const N_PATHCONST: u16 = 58;
pub const N_REFOF: u16 = 59;
pub const N_DEREF: u16 = 60;
pub const N_METHOD: u16 = 61;
pub const N_RETURN: u16 = 62;
pub const N_ASSOC_CALL: u16 = 63;
pub const N_TUPLE: u16 = 68;
pub const N_ENUM: u16 = 69;
pub const N_VARIANT: u16 = 70;
pub const N_PAT_ENUM: u16 = 71;
pub const N_ASSERT: u16 = 72;

pub const FLAG_MUT: u16 = 1;
pub const FLAG_INCLUSIVE: u16 = 1;
pub const DERIVE_CLONE: u16 = 1;
pub const DERIVE_COPY: u16 = 2;
pub const FLAG_TUPLE: u16 = 2;

pub const OP_NEG: u16 = 1;
pub const OP_NOT: u16 = 2;
pub const OP_ADD: u16 = 3;
pub const OP_SUB: u16 = 4;
pub const OP_MUL: u16 = 5;
pub const OP_DIV: u16 = 6;
pub const OP_REM: u16 = 7;
pub const OP_AND: u16 = 8;
pub const OP_OR: u16 = 9;
pub const OP_BAND: u16 = 10;
pub const OP_BOR: u16 = 11;
pub const OP_BXOR: u16 = 12;
pub const OP_SHL: u16 = 13;
pub const OP_SHR: u16 = 14;
pub const OP_EQ: u16 = 15;
pub const OP_NE: u16 = 16;
pub const OP_LT: u16 = 17;
pub const OP_LE: u16 = 18;
pub const OP_GT: u16 = 19;
pub const OP_GE: u16 = 20;

#[derive(Clone, Copy)]
pub struct Node {
    pub kind: u16,
    pub x: u16,
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
    pub e: u32,
    pub link: u32,
    pub lo: u32,
    pub hi: u32,
}

pub const NODE_NONE: Node = Node {
    kind: 0,
    x: 0,
    a: NODE_NIL,
    b: NODE_NIL,
    c: NODE_NIL,
    d: NODE_NIL,
    e: NODE_NIL,
    link: NODE_NIL,
    lo: 0,
    hi: 0,
};

pub fn nd(kind: u16, lo: u32, hi: u32) -> Node {
    let mut n = NODE_NONE;
    n.kind = kind;
    n.lo = lo;
    n.hi = hi;
    n
}

#[derive(Clone, Copy)]
struct Diag { code: u16, lo: u32, hi: u32, a: u32, b: u32 }
#[derive(Clone, Copy)]
struct Mem {
    toks: [Tok; CAP_TOKS], tok_n: usize,
    nodes: [Node; CAP_NODES], node_n: usize,
    diags: [Diag; CAP_DIAGS], diag_n: usize,
    overflow: bool,
    root_first: u32, root_n: u32,
}
impl Mem {
    fn reset(&mut self) {
        self.tok_n = 0; self.node_n = 0; self.diag_n = 0;
        self.overflow = false; self.root_first = NODE_NIL; self.root_n = 0;
    }
    fn diag(&mut self, code: u16, lo: u32, hi: u32, a: u32, b: u32) {
        if self.diag_n < CAP_DIAGS {
            self.diags[self.diag_n] = Diag { code: code, lo: lo, hi: hi, a: a, b: b };
            self.diag_n += 1;
        }
    }
    fn push_tok(&mut self, t: Tok) {
        if self.tok_n >= CAP_TOKS {
            if !self.overflow { self.overflow = true; self.diag(E_TOO_MANY_TOKENS, t.pos, t.pos, 0, 0); }
            return;
        }
        self.toks[self.tok_n] = t;
        self.tok_n += 1;
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
        self.nodes[self.node_n] = n;
        self.node_n += 1;
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
fn main() {

    let src = "fn f(x: i64) -> i64 { if x > 0 { x } else { 0 - x } }";
    let mut m = Mem {
        toks: [TOK_NONE; CAP_TOKS], tok_n: 0,
        nodes: [NODE_NONE; CAP_NODES], node_n: 0,
        diags: [Diag { code: 0, lo: 0, hi: 0, a: 0, b: 0 }; CAP_DIAGS], diag_n: 0,
        overflow: false,
        root_first: NODE_NIL, root_n: 0,
    };
    let lexed = lex(src, &mut m);
    print_bool(lexed);
    print_usize(m.tok_n);
    let ok = parse(src, &mut m);
    print_bool(ok);
    print_usize(m.diag_n);
    print_usize(m.node_n);
    print_i64(m.root_n as i64);

    let mut it = m.root_first;
    while it != NODE_NIL {
        let n = m.node(it);
        print_i64(n.kind as i64);
        it = n.link;
    }
}

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
pub const PARSE_DEPTH_CAP: u16 = 64;

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
    let b = src.as_bytes();
    let lo = t.pos as usize;
    if t.len as usize != s.len() { return false; }
    let mut i: usize = 0;
    while i < s.len() {
        if lo + i >= b.len() || b[lo + i] != s[i] { return false; }
        i += 1;
    }
    true
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
