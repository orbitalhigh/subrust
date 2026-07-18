
const SRC_MAX: usize = 16777216;
const FRAME_MAX: u32 = 1024;

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

const HT_STRUCT: u16 = 0xFFFD;
const HT_ARR: u16 = 0xFFFE;
const HCAP_FIELDS: usize = 1;
const HCAP_PARAMS: usize = 1;
const HCAP_HSTRUCTS: usize = 1;
const HCAP_HFNS: usize = 4;
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

fn host_name(id: u32) -> &[u8] {
    if id == 0 { return b"ld"; }
    if id == 1 { return b"st"; }
    if id == 2 { return b"getb"; }
    if id == 3 { return b"putb"; }
    b""
}
const H_TY0: HostTy = HostTy { kind: 0, sname: 0, elem: 0, len: 0 };
const H_FIELD0: HostField = HostField { name: 0, ty: H_TY0 };
const H_STRUCT0: HostStructDef = HostStructDef { name: 0, fields: [H_FIELD0; HCAP_FIELDS], field_n: 0 };
const H_FN0: HostFnDef = HostFnDef { name: 0, params: [H_TY0; HCAP_PARAMS], param_n: 0, ret: H_TY0 };
const EMPTY_HOST: HostDef = HostDef {
    structs: [H_STRUCT0; HCAP_HSTRUCTS], struct_n: 0,
    fns: [H_FN0; HCAP_HFNS], fn_n: 0,
};

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

pub fn lex(src: &[u8], mem: &mut Mem) -> bool {
    mem.reset();
    if src.len() > SRC_MAX {
        mem.diag(E_SRC_TOO_BIG, 0, 0, 0, 0);
        return false;
    }
    let b = src;
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

pub fn parse(src: &[u8], mem: &mut Mem) -> bool {
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

fn p_item(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_attr(src: &[u8], mem: &mut Mem, p: &mut P, derives: &mut u16) -> bool {
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

fn p_fn(src: &[u8], mem: &mut Mem, p: &mut P, self_tok: u32) -> u32 {
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

fn p_impl(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_struct(src: &[u8], mem: &mut Mem, p: &mut P, derives: u16) -> u32 {
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

fn p_enum(src: &[u8], mem: &mut Mem, p: &mut P, derives: u16) -> u32 {
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

fn p_macro(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_const(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_type(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = type_inner(src, mem, p);
    leave(p);
    r
}

fn type_inner(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_block(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = block_inner(src, mem, p);
    leave(p);
    r
}

fn block_inner(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_let(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_for(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_expr(src: &[u8], mem: &mut Mem, p: &mut P, min: u8) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = expr_inner(src, mem, p, min);
    leave(p);
    r
}

fn expr_inner(src: &[u8], mem: &mut Mem, p: &mut P, min: u8) -> u32 {
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

fn p_cast(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_unary(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = unary_inner(src, mem, p);
    leave(p);
    r
}

fn unary_inner(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_postfix(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_primary(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_struct_lit(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_if(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = if_inner(src, mem, p);
    leave(p);
    r
}

fn if_inner(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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

fn p_match(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
    if !enter(mem, p) {
        return NODE_NIL;
    }
    let r = match_inner(src, mem, p);
    leave(p);
    r
}

fn match_inner(src: &[u8], mem: &mut Mem, p: &mut P) -> u32 {
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
pub const TY_ERR: u16 = 0;
pub const TY_UNIT: u16 = 1;
pub const TY_BOOL: u16 = 2;
pub const TY_I8: u16 = 3;
pub const TY_U8: u16 = 4;
pub const TY_I16: u16 = 5;
pub const TY_U16: u16 = 6;
pub const TY_I32: u16 = 7;
pub const TY_U32: u16 = 8;
pub const TY_I64: u16 = 9;
pub const TY_U64: u16 = 10;
pub const TY_I128: u16 = 11;
pub const TY_U128: u16 = 12;
pub const TY_ISIZE: u16 = 13;
pub const TY_USIZE: u16 = 14;
pub const TY_F64: u16 = 15;
pub const TY_STR: u16 = 16;
pub const TY_INTLIT: u16 = 17;
pub const TY_NEVER: u16 = 18;
pub const TY_ANY: u16 = 0xFFFF;

pub const TY_HINT: u16 = 0x8000;

fn is_hint(e: u16) -> bool {
    e != TY_ANY && e & TY_HINT != 0
}

fn exp_ty(e: u16) -> u16 {
    if e == TY_ANY {
        TY_ANY
    } else {
        e & !TY_HINT
    }
}

pub const TY_STRUCT0: u16 = 0x1000;
pub const TY_ARR0: u16 = 0x2000;
pub const TY_REF0: u16 = 0x3000;
pub const TY_SLICE0: u16 = 0x4000;
pub const TY_TUPLE0: u16 = 0x5000;
pub const TY_ENUM0: u16 = 0x6000;
pub const TUP_MAX: usize = 6;

pub const POOL_TAG: u64 = 1 << 63;

pub fn ty_is_struct(t: u16) -> bool {
    t >= TY_STRUCT0 && t < TY_STRUCT0 + CAP_STRUCTS as u16
}

pub fn ty_is_arr(t: u16) -> bool {
    t >= TY_ARR0 && t < TY_ARR0 + CAP_ARRS as u16
}

pub fn ty_is_ref(t: u16) -> bool {
    t >= TY_REF0 && t < TY_REF0 + CAP_REFS as u16
}

pub fn ty_is_tuple(t: u16) -> bool {
    t >= TY_TUPLE0 && t < TY_TUPLE0 + CAP_TUPLES as u16
}
pub fn ty_is_slice(t: u16) -> bool {
    t >= TY_SLICE0 && t < TY_SLICE0 + CAP_SLICES as u16
}
pub fn ty_is_enum(t: u16) -> bool {
    t >= TY_ENUM0 && t < TY_ENUM0 + CAP_ENUMS as u16
}

pub fn ty_is_128(t: u16) -> bool {
    t == TY_I128 || t == TY_U128
}

pub fn ty_is_int(t: u16) -> bool {
    (t >= TY_I8 && t <= TY_USIZE) || t == TY_INTLIT
}

pub fn ty_is_signed(t: u16) -> bool {
    t == TY_I8 || t == TY_I16 || t == TY_I32 || t == TY_I64 || t == TY_I128 || t == TY_ISIZE || t == TY_INTLIT
}

fn int_bits(t: u16) -> u32 {
    match t {
        TY_I8 | TY_U8 => 8,
        TY_I16 | TY_U16 => 16,
        TY_I32 | TY_U32 => 32,
        TY_I128 | TY_U128 => 128,
        _ => 64,
    }
}

fn ty_is_scalar(t: u16) -> bool {
    t == TY_BOOL || (t >= TY_I8 && t <= TY_F64) || t == TY_STR
}

pub const RES_CONST: u32 = 0x8000_0000;
pub const RES_HOST: u32 = 0x8000_0000;
pub const RES_MASK: u32 = 0x7FFF_FFFF;

pub const RES_MPLACE: u32 = 0x4000_0000;
pub const RES_MFN_MASK: u32 = 0x3FFF_FFFF;

pub const RES_SLICE_LEN: u32 = 0x2000_0000;
pub const RES_ARRAY_LEN: u32 = 0x1000_0000;
pub const RES_ALEN_MASK: u32 = 0x0FFF_FFFF;

pub const RES_PRIM: u32 = 0x0800_0000;
pub const RES_PRIM_MASK: u32 = 0x00FF_FFFF;
pub const PRIM_WRAP_ADD: u32 = 1;
pub const PRIM_WRAP_SUB: u32 = 2;
pub const PRIM_WRAP_MUL: u32 = 3;
pub const PRIM_WRAP_NEG: u32 = 4;
pub const PRIM_WRAP_SHL: u32 = 5;
pub const PRIM_SAT_ADD: u32 = 6;
pub const PRIM_SAT_MUL: u32 = 7;
pub const PRIM_TO_BITS: u32 = 8;
pub const PRIM_IS_NAN: u32 = 9;
pub const PRIM_ROTL: u32 = 10;
pub const PRIM_ROTR: u32 = 11;

pub fn prim_is_sat(op: u32) -> bool {
    op == PRIM_SAT_ADD || op == PRIM_SAT_MUL
}

pub const RES_STR_LEN: u32 = 0x0400_0000;
pub const RES_STR_BYTES: u32 = 0x0200_0000;

const LFLAG_RETSAFE: u16 = 0x8000;

pub const RES_DEREF: u32 = 0x8000_0000;
pub const RES_OFF_MASK: u32 = 0x7FFF_FFFF;

#[derive(Clone, Copy)]
pub struct SInfo {
    pub name_tok: u32,
    pub host: u32,
    pub first_field: u32,
    pub field_n: u32,
    pub size: u32,
    pub state: u16,
    pub derives: u16,
}
pub const SINFO_NONE: SInfo = SInfo {
    name_tok: NODE_NIL,
    host: 0,
    first_field: NODE_NIL,
    field_n: 0,
    size: 0,
    state: 0,
    derives: 0,
};

#[derive(Clone, Copy)]
pub struct EInfo {
    pub name_tok: u32,
    pub first_variant: u32,
    pub variant_n: u32,
    pub size: u32,
    pub state: u16,
}
pub const EINFO_NONE: EInfo = EInfo {
    name_tok: NODE_NIL,
    first_variant: NODE_NIL,
    variant_n: 0,
    size: 0,
    state: 0,
};

#[derive(Clone, Copy)]
pub struct AInfo {
    pub elem: u16,
    pub len: u32,
    pub size: u32,
}
pub const AINFO_NONE: AInfo = AInfo {
    elem: TY_ERR,
    len: 0,
    size: 0,
};

#[derive(Clone, Copy)]
pub struct TInfo {
    pub elems: [u16; TUP_MAX],
    pub offs: [u16; TUP_MAX],
    pub count: u16,
    pub size: u32,
}
pub const TINFO_NONE: TInfo = TInfo {
    elems: [TY_ERR; TUP_MAX],
    offs: [0; TUP_MAX],
    count: 0,
    size: 0,
};

#[derive(Clone, Copy)]
pub struct RInfo {
    pub pointee: u16,
    pub mutable: u16,
}
pub const RINFO_NONE: RInfo = RInfo { pointee: TY_ERR, mutable: 0 };

#[derive(Clone, Copy)]
pub struct CInfo {
    pub name_tok: u32,
    pub node: u32,
    pub ty: u16,
    pub state: u16,
    pub bits: u64,
}
pub const CINFO_NONE: CInfo = CInfo {
    name_tok: NODE_NIL,
    node: NODE_NIL,
    ty: TY_ERR,
    state: 0,
    bits: 0,
};

#[derive(Clone, Copy)]
pub struct FInfo {
    pub name_tok: u32,
    pub node: u32,
    pub first_param: u32,
    pub param_n: u32,
    pub ret: u16,
    pub frame: u32,
    pub self_tok: u32,
    pub self_ty: u16,
}
pub const FINFO_NONE: FInfo = FInfo {
    name_tok: NODE_NIL,
    node: NODE_NIL,
    first_param: NODE_NIL,
    param_n: 0,
    ret: TY_UNIT,
    frame: 0,
    self_tok: NODE_NIL,
    self_ty: TY_ERR,
};

#[derive(Clone, Copy)]
pub struct LInfo {
    pub name_tok: u32,
    pub ty: u16,
    pub flags: u16,
    pub slot: u32,
    pub depth: u16,
    pub init: u32,
}
pub const LINFO_NONE: LInfo = LInfo {
    name_tok: NODE_NIL,
    ty: TY_ERR,
    flags: 0,
    slot: 0,
    depth: 0,
    init: NODE_NIL,
};

#[derive(Clone, Copy)]
pub struct StrEntry {
    pub off: u32,
    pub len: u32,
}
pub const STR_NONE: StrEntry = StrEntry { off: 0, len: 0 };

#[derive(Clone, Copy)]
pub struct Chk {
    pub ty: [u16; CAP_NODES],
    pub res: [u32; CAP_NODES],

    pub structs: [SInfo; CAP_STRUCTS],
    pub struct_n: usize,
    pub enums: [EInfo; CAP_ENUMS],
    pub enum_n: usize,
    pub arrs: [AInfo; CAP_ARRS],
    pub arr_n: usize,
    pub refs: [RInfo; CAP_REFS],
    pub ref_n: usize,
    pub slices: [RInfo; CAP_SLICES],
    pub slice_n: usize,
    pub tuples: [TInfo; CAP_TUPLES],
    pub tuple_n: usize,
    pub consts: [CInfo; CAP_CONSTS],
    pub const_n: usize,
    pub fns: [FInfo; CAP_FNS],
    pub fn_n: usize,

    pub vals: [u64; CAP_VALS],
    pub val_n: usize,
    pub strs: [StrEntry; CAP_STRS],
    pub str_n: usize,
    pub str_pool: [u8; CAP_STR_POOL],
    pub pool_n: usize,

    locals: [LInfo; CAP_LOCALS],
    local_n: usize,
    depth: u16,
    loop_depth: u16,
    loop_broke: bool,
    next_slot: u32,
    ret_ty: u16,
    ret_borrow_body: u32,
    ce_depth: u16,
    in_const: bool,
    sizing_done: bool,
}

pub const CHK_INIT: Chk = Chk {
    ty: [TY_ERR; CAP_NODES],
    res: [0; CAP_NODES],
    structs: [SINFO_NONE; CAP_STRUCTS],
    struct_n: 0,
    enums: [EINFO_NONE; CAP_ENUMS],
    enum_n: 0,
    arrs: [AINFO_NONE; CAP_ARRS],
    arr_n: 0,
    refs: [RINFO_NONE; CAP_REFS],
    ref_n: 0,
    slices: [RINFO_NONE; CAP_SLICES],
    slice_n: 0,
    tuples: [TINFO_NONE; CAP_TUPLES],
    tuple_n: 0,
    consts: [CINFO_NONE; CAP_CONSTS],
    const_n: 0,
    fns: [FINFO_NONE; CAP_FNS],
    fn_n: 0,
    vals: [0; CAP_VALS],
    val_n: 0,
    strs: [STR_NONE; CAP_STRS],
    str_n: 0,
    str_pool: [0; CAP_STR_POOL],
    pool_n: 0,
    locals: [LINFO_NONE; CAP_LOCALS],
    local_n: 0,
    depth: 0,
    loop_depth: 0,
    loop_broke: false,
    next_slot: 0,
    ret_ty: TY_UNIT,
    ret_borrow_body: NODE_NIL,
    ce_depth: 0,
    in_const: false,
    sizing_done: false,
};

impl Chk {
    pub fn reset(&mut self) {

        self.struct_n = 0;
        self.enum_n = 0;
        self.arr_n = 0;
        self.ref_n = 0;
        self.slice_n = 0;
        self.tuple_n = 0;
        self.const_n = 0;
        self.fn_n = 0;
        self.val_n = 0;
        self.str_n = 0;
        self.pool_n = 0;
        self.local_n = 0;
        self.depth = 0;
        self.loop_depth = 0;
        self.next_slot = 0;
        self.ret_ty = TY_UNIT;
        self.ce_depth = 0;
        self.in_const = false;
        self.sizing_done = false;
        let mut i = 0;
        while i < CAP_NODES {
            self.ty[i] = TY_ERR;
            self.res[i] = 0;
            i += 1;
        }
    }

    pub fn sinfo(&self, t: u16) -> SInfo {
        let k = (t - TY_STRUCT0) as usize;
        if k < self.struct_n {
            self.structs[k]
        } else {
            SINFO_NONE
        }
    }

    pub fn ainfo(&self, t: u16) -> AInfo {
        let k = (t - TY_ARR0) as usize;
        if k < self.arr_n {
            self.arrs[k]
        } else {
            AINFO_NONE
        }
    }

    pub fn einfo(&self, t: u16) -> EInfo {
        let k = (t - TY_ENUM0) as usize;
        if k < self.enum_n {
            self.enums[k]
        } else {
            EINFO_NONE
        }
    }

    pub fn tinfo(&self, t: u16) -> TInfo {
        let k = (t - TY_TUPLE0) as usize;
        if k < self.tuple_n {
            self.tuples[k]
        } else {
            TINFO_NONE
        }
    }

    pub fn rinfo(&self, t: u16) -> RInfo {
        let k = (t - TY_REF0) as usize;
        if k < self.ref_n {
            self.refs[k]
        } else {
            RINFO_NONE
        }
    }

    pub fn slinfo(&self, t: u16) -> RInfo {
        let k = (t - TY_SLICE0) as usize;
        if k < self.slice_n {
            self.slices[k]
        } else {
            RINFO_NONE
        }
    }

    pub fn size_of(&self, t: u16) -> u32 {
        if t == TY_UNIT || t == TY_ERR {
            return 0;
        }
        if ty_is_struct(t) {
            return self.sinfo(t).size;
        }
        if ty_is_arr(t) {
            return self.ainfo(t).size;
        }
        if ty_is_ref(t) {
            return 1;
        }
        if ty_is_slice(t) {
            return 2;
        }
        if ty_is_tuple(t) {
            return self.tinfo(t).size;
        }
        if ty_is_enum(t) {
            return self.einfo(t).size;
        }
        if ty_is_128(t) {
            return 2;
        }
        1
    }

    pub fn str_bytes(&self, idx: u32) -> &[u8] {
        let idx = idx as usize;
        if idx >= self.str_n {
            return &[];
        }
        let e = self.strs[idx];
        let lo = e.off as usize;
        let hi = lo + e.len as usize;
        if hi <= self.pool_n {
            &self.str_pool[lo..hi]
        } else {
            &[]
        }
    }

    pub fn str_pool_at(&self, off: usize) -> u8 {
        if off < self.pool_n {
            self.str_pool[off]
        } else {
            0
        }
    }
}

fn tok_bytes(src: &[u8], t: Tok) -> &[u8] {
    let lo = t.pos as usize;
    let hi = lo + t.len as usize;
    let b = src;
    if lo <= hi && hi <= b.len() {
        &b[lo..hi]
    } else {
        &[]
    }
}

fn tok_eq(src: &[u8], mem: &Mem, a: u32, b: u32) -> bool {
    bytes_eq(tok_bytes(src, mem.tok(a)), tok_bytes(src, mem.tok(b)))
}

fn tok_is(src: &[u8], mem: &Mem, t: u32, s: &[u8]) -> bool {
    bytes_eq(tok_bytes(src, mem.tok(t)), s)
}

fn int_ty_named(src: &[u8], mem: &Mem, t: u32) -> u16 {
    let w = tok_bytes(src, mem.tok(t));
    if bytes_eq(w, b"i8") { return TY_I8; }
    if bytes_eq(w, b"u8") { return TY_U8; }
    if bytes_eq(w, b"i16") { return TY_I16; }
    if bytes_eq(w, b"u16") { return TY_U16; }
    if bytes_eq(w, b"i32") { return TY_I32; }
    if bytes_eq(w, b"u32") { return TY_U32; }
    if bytes_eq(w, b"i64") { return TY_I64; }
    if bytes_eq(w, b"u64") { return TY_U64; }
    if bytes_eq(w, b"i128") { return TY_I128; }
    if bytes_eq(w, b"u128") { return TY_U128; }
    if bytes_eq(w, b"isize") { return TY_ISIZE; }
    if bytes_eq(w, b"usize") { return TY_USIZE; }
    TY_ERR
}

fn ndiag(mem: &mut Mem, code: u16, n: Node, a: u32, b: u32) {
    mem.diag(code, n.lo, n.hi, a, b);
}

pub fn check(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef) -> bool {
    let before = mem.diag_n;
    chk.reset();

    let mut h = 0;
    while h < host.struct_n {
        if chk.struct_n >= CAP_STRUCTS {
            mem.diag(E_TOO_MANY_ITEMS, 0, 0, 0, 0);
            return false;
        }
        let mut s = SINFO_NONE;
        s.host = h as u32 + 1;
        s.field_n = host.structs[h].field_n as u32;
        s.state = 2;

        let mut size = 0;
        let mut f = 0;
        while f < host.structs[h].field_n {
            size += host_ty_size(&host.structs[h].fields[f].ty);
            f += 1;
        }
        s.size = size;
        chk.structs[chk.struct_n] = s;
        chk.struct_n += 1;
        h += 1;
    }

    let mut it = mem.root_first;
    while it != NODE_NIL {
        let n = mem.node(it);
        if n.kind == N_STRUCT {
            collect_struct(src, mem, chk, host, it);
        } else if n.kind == N_ENUM {
            collect_enum(src, mem, chk, host, it);
        } else if n.kind == N_CONST {
            collect_const(src, mem, chk, host, it);
        } else if n.kind == N_FN {
            collect_fn(src, mem, chk, host, it, NODE_NIL);
        } else if n.kind == N_IMPL {

            let mut m = n.b;
            while m != NODE_NIL {
                collect_fn(src, mem, chk, host, m, n.a);
                m = mem.node(m).link;
            }
        } else if n.kind == N_USE {
            ndiag(mem, E_USE_UNSUPPORTED, n, 0, 0);
        }
        it = n.link;
    }

    let mut i = 0;
    while i < chk.const_n {
        ck_const(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.enum_n {
        size_enum(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.struct_n {
        size_struct(src, mem, chk, host, i);
        i += 1;
    }
    chk.sizing_done = true;

    let mut i = 0;
    while i < chk.const_n {
        ck_const(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.fn_n {
        sig_fn(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.fn_n {
        ck_fn(src, mem, chk, host, i);
        i += 1;
    }

    mem.diag_n == before && !mem.overflow
}

fn host_ty_size(t: &HostTy) -> u32 {
    if t.kind == TY_UNIT {
        return 0;
    }
    if t.kind == HT_ARR {
        return t.len;
    }
    1
}

fn name_taken(src: &[u8], mem: &Mem, chk: &Chk, host: &HostDef, t: u32) -> bool {
    let w = tok_bytes(src, mem.tok(t));
    let mut i = 0;
    while i < chk.struct_n {
        let s = chk.structs[i];
        if s.host > 0 {
            if bytes_eq(w, host_name(host.structs[(s.host - 1) as usize].name)) {
                return true;
            }
        } else if tok_eq(src, mem, s.name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < chk.enum_n {
        if tok_eq(src, mem, chk.enums[i].name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < chk.const_n {
        if tok_eq(src, mem, chk.consts[i].name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < chk.fn_n {
        if tok_eq(src, mem, chk.fns[i].name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < host.fn_n {
        if bytes_eq(w, host_name(host.fns[i].name)) {
            return true;
        }
        i += 1;
    }
    false
}

fn collect_struct(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32) {
    let n = mem.node(it);
    if name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.struct_n >= CAP_STRUCTS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }
    if n.x & DERIVE_CLONE == 0 || n.x & DERIVE_COPY == 0 {
        ndiag(mem, E_MISSING_DERIVE, n, 0, 0);

    }
    let mut s = SINFO_NONE;
    s.name_tok = n.a;
    s.first_field = n.b;
    s.field_n = n.c;
    s.derives = n.x;
    chk.structs[chk.struct_n] = s;
    chk.struct_n += 1;
}

fn collect_enum(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32) {
    let n = mem.node(it);
    if name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.enum_n >= CAP_ENUMS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }

    if n.x & DERIVE_CLONE == 0 || n.x & DERIVE_COPY == 0 {
        ndiag(mem, E_MISSING_DERIVE, n, 0, 0);
    }

    let mut a = n.b;
    while a != NODE_NIL {
        let an = mem.node(a);
        let mut b = an.link;
        while b != NODE_NIL {
            let bn = mem.node(b);
            if tok_eq(src, mem, an.a, bn.a) {
                ndiag(mem, E_DUP_NAME, bn, 0, 0);
            }
            b = bn.link;
        }
        a = an.link;
    }
    let mut e = EINFO_NONE;
    e.name_tok = n.a;
    e.first_variant = n.b;
    e.variant_n = n.c;
    chk.enums[chk.enum_n] = e;
    chk.enum_n += 1;
}

fn size_enum(_src: &[u8], mem: &mut Mem, chk: &mut Chk, _host: &HostDef, k: usize) {
    if chk.enums[k].state == 2 {
        return;
    }
    chk.enums[k].state = 1;
    let mut v = chk.enums[k].first_variant;
    while v != NODE_NIL {
        let vn = mem.node(v);
        if vn.e != NODE_NIL {
            ndiag(mem, E_ENUM_PAYLOAD, vn, 0, 0);
        }
        v = vn.link;
    }
    chk.enums[k].size = 1;
    chk.enums[k].state = 2;
}

fn find_enum(src: &[u8], mem: &Mem, chk: &Chk, name_tok: u32) -> u16 {
    let mut i = 0;
    while i < chk.enum_n {
        if tok_eq(src, mem, chk.enums[i].name_tok, name_tok) {
            return i as u16;
        }
        i += 1;
    }
    u16::MAX
}

fn variant_tag(src: &[u8], mem: &Mem, chk: &Chk, et: u16, name_tok: u32) -> u32 {
    let e = chk.einfo(et);
    let mut v = e.first_variant;
    let mut tag: u32 = 0;
    while v != NODE_NIL {
        let vn = mem.node(v);
        if tok_eq(src, mem, vn.a, name_tok) {
            return tag;
        }
        tag += 1;
        v = vn.link;
    }
    u32::MAX
}

fn enum_all_seen(chk: &Chk, et: u16, seen: u64) -> bool {
    let vn = chk.einfo(et).variant_n;
    let full = if vn >= 64 { u64::MAX } else { (1u64 << vn) - 1 };
    seen & full == full
}

fn collect_const(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32) {
    let n = mem.node(it);
    if name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.const_n >= CAP_CONSTS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }
    let mut c = CINFO_NONE;
    c.name_tok = n.a;
    c.node = it;
    chk.consts[chk.const_n] = c;
    chk.const_n += 1;
}

fn collect_fn(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32, self_tok: u32) {
    let n = mem.node(it);

    if self_tok == NODE_NIL && name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.fn_n >= CAP_FNS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }
    let mut f = FINFO_NONE;
    f.name_tok = n.a;
    f.node = it;
    f.first_param = n.b;
    f.param_n = n.c;
    f.self_tok = self_tok;
    chk.fns[chk.fn_n] = f;
    chk.fn_n += 1;
}

fn ty_of(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, tn: u32) -> u16 {
    let n = mem.node(tn);
    let t = match n.kind {
        N_TY_UNIT => TY_UNIT,
        N_TY_STR => TY_STR,
        N_TY_REF => {
            let pointee = ty_of(src, mem, chk, host, n.d);
            if pointee == TY_ERR {
                return TY_ERR;
            }
            if ty_is_ref(pointee) {
                ndiag(mem, E_REF_ESCAPES, n, 0, 0);
                return TY_ERR;
            }
            intern_ref(mem, chk, pointee, n.x & FLAG_MUT, n)
        }
        N_TY_SLICE => {
            let elem = ty_of(src, mem, chk, host, n.d);
            if elem == TY_ERR {
                return TY_ERR;
            }
            if elem == TY_UNIT || ty_is_ref(elem) || ty_is_slice(elem) {
                ndiag(mem, if elem == TY_UNIT { E_UNKNOWN_TYPE } else { E_REF_ESCAPES }, n, 0, 0);
                return TY_ERR;
            }
            if ty_is_struct(elem) {
                let k = (elem - TY_STRUCT0) as usize;
                size_struct(src, mem, chk, host, k);
                if chk.structs[k].state != 2 {
                    return TY_ERR;
                }
            }
            intern_slice(mem, chk, elem, n.x & FLAG_MUT, n)
        }
        N_TY_NAME => named_ty(src, mem, chk, host, n.a, n),
        N_TY_ARRAY => {
            let elem = ty_of(src, mem, chk, host, n.d);
            if elem == TY_ERR {
                return TY_ERR;
            }
            if elem == TY_UNIT || ty_is_ref(elem) || ty_is_slice(elem) {
                ndiag(mem, if elem == TY_UNIT { E_UNKNOWN_TYPE } else { E_REF_ESCAPES }, n, 0, 0);
                return TY_ERR;
            }

            if ty_is_struct(elem) {
                let k = (elem - TY_STRUCT0) as usize;
                size_struct(src, mem, chk, host, k);
                if chk.structs[k].state != 2 {
                    return TY_ERR;
                }
            }
            let len = ce_len(src, mem, chk, host, n.e);
            if len == u32::MAX {
                return TY_ERR;
            }
            intern_arr(mem, chk, elem, len, n)
        }
        _ => {
            ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
            TY_ERR
        }
    };
    if tn < CAP_NODES as u32 {
        chk.ty[tn as usize] = t;
    }
    t
}

fn named_ty(src: &[u8], mem: &mut Mem, chk: &Chk, host: &HostDef, name_tok: u32, n: Node) -> u16 {
    let w = tok_bytes(src, mem.tok(name_tok));
    if bytes_eq(w, b"bool") {
        return TY_BOOL;
    }
    if bytes_eq(w, b"i8") {
        return TY_I8;
    }
    if bytes_eq(w, b"u8") {
        return TY_U8;
    }
    if bytes_eq(w, b"i16") {
        return TY_I16;
    }
    if bytes_eq(w, b"u16") {
        return TY_U16;
    }
    if bytes_eq(w, b"isize") {
        return TY_ISIZE;
    }
    if bytes_eq(w, b"i32") {
        return TY_I32;
    }
    if bytes_eq(w, b"u32") {
        return TY_U32;
    }
    if bytes_eq(w, b"i64") {
        return TY_I64;
    }
    if bytes_eq(w, b"u64") {
        return TY_U64;
    }
    if bytes_eq(w, b"i128") {
        return TY_I128;
    }
    if bytes_eq(w, b"u128") {
        return TY_U128;
    }
    if bytes_eq(w, b"usize") {
        return TY_USIZE;
    }
    if bytes_eq(w, b"f64") {
        return TY_F64;
    }
    let s = find_struct(src, mem, chk, host, name_tok);
    if s != u16::MAX {
        return TY_STRUCT0 + s;
    }
    let e = find_enum(src, mem, chk, name_tok);
    if e != u16::MAX {
        return TY_ENUM0 + e;
    }
    ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
    TY_ERR
}

fn find_struct(src: &[u8], mem: &Mem, chk: &Chk, host: &HostDef, name_tok: u32) -> u16 {
    let w = tok_bytes(src, mem.tok(name_tok));
    let mut i = 0;
    while i < chk.struct_n {
        let s = chk.structs[i];
        if s.host > 0 {
            if bytes_eq(w, host_name(host.structs[(s.host - 1) as usize].name)) {
                return i as u16;
            }
        } else if tok_eq(src, mem, s.name_tok, name_tok) {
            return i as u16;
        }
        i += 1;
    }
    u16::MAX
}

fn prim_op(src: &[u8], mem: &Mem, name_tok: u32) -> u32 {
    if tok_is(src, mem, name_tok, b"wrapping_add") {
        return PRIM_WRAP_ADD;
    }
    if tok_is(src, mem, name_tok, b"wrapping_sub") {
        return PRIM_WRAP_SUB;
    }
    if tok_is(src, mem, name_tok, b"wrapping_mul") {
        return PRIM_WRAP_MUL;
    }
    if tok_is(src, mem, name_tok, b"wrapping_neg") {
        return PRIM_WRAP_NEG;
    }
    if tok_is(src, mem, name_tok, b"wrapping_shl") {
        return PRIM_WRAP_SHL;
    }
    if tok_is(src, mem, name_tok, b"saturating_add") {
        return PRIM_SAT_ADD;
    }
    if tok_is(src, mem, name_tok, b"saturating_mul") {
        return PRIM_SAT_MUL;
    }
    if tok_is(src, mem, name_tok, b"rotate_left") {
        return PRIM_ROTL;
    }
    if tok_is(src, mem, name_tok, b"rotate_right") {
        return PRIM_ROTR;
    }
    0
}

fn method_find(src: &[u8], mem: &Mem, chk: &Chk, sty: u16, name_tok: u32) -> usize {
    let mut i = 0;
    while i < chk.fn_n {
        if chk.fns[i].self_ty == sty && tok_eq(src, mem, chk.fns[i].name_tok, name_tok) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

fn intern_ref(mem: &mut Mem, chk: &mut Chk, pointee: u16, mutable: u16, at: Node) -> u16 {
    let mut i = 0;
    while i < chk.ref_n {
        if chk.refs[i].pointee == pointee && chk.refs[i].mutable == mutable {
            return TY_REF0 + i as u16;
        }
        i += 1;
    }
    if chk.ref_n >= CAP_REFS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.refs[chk.ref_n] = RInfo { pointee, mutable };
    chk.ref_n += 1;
    TY_REF0 + (chk.ref_n - 1) as u16
}

fn intern_slice(mem: &mut Mem, chk: &mut Chk, elem: u16, mutable: u16, at: Node) -> u16 {
    let mut i = 0;
    while i < chk.slice_n {
        if chk.slices[i].pointee == elem && chk.slices[i].mutable == mutable {
            return TY_SLICE0 + i as u16;
        }
        i += 1;
    }
    if chk.slice_n >= CAP_SLICES {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.slices[chk.slice_n] = RInfo { pointee: elem, mutable };
    chk.slice_n += 1;
    TY_SLICE0 + (chk.slice_n - 1) as u16
}

fn ck_slice(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, slice_node: u32,
            want_mut: u16, at: Node) -> u16 {
    let sn = mem.node(slice_node);
    let bt = ck_expr(src, mem, chk, host, sn.d, TY_ANY);
    if bt == TY_ERR {
        return TY_ERR;
    }
    let (elem, base_mut, base_is_slice) = if ty_is_arr(bt) {
        let mut rm = false;
        let pt = ck_place(src, mem, chk, host, sn.d, &mut rm);
        if pt == TY_ERR {
            return TY_ERR;
        }
        (chk.ainfo(bt).elem, rm, false)
    } else if ty_is_slice(bt) {
        let si = chk.slinfo(bt);
        (si.pointee, si.mutable != 0, true)
    } else {
        ndiag(mem, E_NOT_AN_ARRAY, sn, 0, bt as u32);
        return TY_ERR;
    };
    if want_mut != 0 && !base_mut {
        ndiag(mem, E_REF_MUT_NEEDED, at, 0, 0);
        return TY_ERR;
    }
    let _ = ck_ex(src, mem, chk, host, sn.b, TY_USIZE);
    if sn.c != NODE_NIL {
        let _ = ck_ex(src, mem, chk, host, sn.c, TY_USIZE);
    }
    chk.res[slice_node as usize] = if base_is_slice { 1 } else { 0 };
    let t = intern_slice(mem, chk, elem, if want_mut != 0 { FLAG_MUT } else { 0 }, at);
    if (slice_node as usize) < CAP_NODES {
        chk.ty[slice_node as usize] = t;
    }
    t
}

fn intern_arr(mem: &mut Mem, chk: &mut Chk, elem: u16, len: u32, at: Node) -> u16 {
    let size = chk.size_of(elem).saturating_mul(len);
    let mut i = 0;
    while i < chk.arr_n {
        if chk.arrs[i].elem == elem && chk.arrs[i].len == len {
            return TY_ARR0 + i as u16;
        }
        i += 1;
    }
    if chk.arr_n >= CAP_ARRS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.arrs[chk.arr_n] = AInfo { elem, len, size };
    chk.arr_n += 1;
    TY_ARR0 + (chk.arr_n - 1) as u16
}

fn intern_tuple(mem: &mut Mem, chk: &mut Chk, elems: &[u16], count: usize, at: Node) -> u16 {
    let mut info = TINFO_NONE;
    info.count = count as u16;
    let mut off = 0u32;
    let mut e = 0;
    while e < count {
        info.elems[e] = elems[e];
        info.offs[e] = off as u16;
        off += chk.size_of(elems[e]);
        e += 1;
    }
    info.size = off;

    let mut i = 0;
    while i < chk.tuple_n {
        let t = chk.tuples[i];
        if t.count == info.count {
            let mut same = true;
            let mut k = 0;
            while k < count {
                if t.elems[k] != info.elems[k] {
                    same = false;
                }
                k += 1;
            }
            if same {
                return TY_TUPLE0 + i as u16;
            }
        }
        i += 1;
    }
    if chk.tuple_n >= CAP_TUPLES {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.tuples[chk.tuple_n] = info;
    chk.tuple_n += 1;
    TY_TUPLE0 + (chk.tuple_n - 1) as u16
}

fn host_ty(_src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, t: &HostTy, at: Node) -> u16 {
    if t.kind == HT_ARR {
        return intern_arr(mem, chk, t.elem, t.len, at);
    }
    if t.kind == HT_STRUCT {
        let mut i = 0;
        while i < chk.struct_n {
            let s = chk.structs[i];
            if s.host > 0 && bytes_eq(host_name(host.structs[(s.host - 1) as usize].name),
                                       host_name(t.sname)) {
                return TY_STRUCT0 + i as u16;
            }
            i += 1;
        }
        ndiag(mem, E_UNKNOWN_TYPE, at, 0, 0);
        return TY_ERR;
    }
    t.kind
}

fn size_struct(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) {
    if chk.structs[k].state == 2 {
        return;
    }
    if chk.structs[k].state == 1 {
        let n = mem.node(chk.structs[k].first_field);
        ndiag(mem, E_RECURSIVE_STRUCT, n, 0, 0);
        chk.structs[k].state = 2;
        return;
    }
    chk.structs[k].state = 1;
    let mut size: u32 = 0;
    let mut f = chk.structs[k].first_field;
    while f != NODE_NIL {
        let fnode = mem.node(f);
        let ft = ty_of(src, mem, chk, host, fnode.e);
        let ft = if ft == TY_STR {

            ndiag(mem, E_STR_FIELD, fnode, 0, 0);
            TY_ERR
        } else if ty_is_ref(ft) || ty_is_slice(ft) {
            ndiag(mem, E_REF_ESCAPES, fnode, 0, 0);
            TY_ERR
        } else {
            ft
        };
        if ty_is_struct(ft) {
            size_struct(src, mem, chk, host, (ft - TY_STRUCT0) as usize);
        }
        chk.ty[f as usize] = ft;
        chk.res[f as usize] = size;
        size += chk.size_of(ft);
        f = fnode.link;
    }
    chk.structs[k].size = size;
    chk.structs[k].state = 2;
}

fn sig_fn(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) {
    let f = chk.fns[k];
    let n = mem.node(f.node);

    if f.self_tok != NODE_NIL {
        let s = find_struct(src, mem, chk, host, f.self_tok);
        if s == u16::MAX {
            ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
        } else {
            let sty = TY_STRUCT0 + s;
            chk.fns[k].self_ty = sty;
            let mut j = 0;
            while j < k {
                if chk.fns[j].self_ty == sty
                    && tok_eq(src, mem, chk.fns[j].name_tok, f.name_tok)
                {
                    ndiag(mem, E_DUP_NAME, n, 0, 0);
                    break;
                }
                j += 1;
            }
        }
    }
    let mut p = f.first_param;
    while p != NODE_NIL {
        let pn = mem.node(p);
        let pt = ty_of(src, mem, chk, host, pn.e);
        chk.ty[p as usize] = pt;
        p = pn.link;
    }
    let ret = if n.d == NODE_NIL {
        TY_UNIT
    } else {
        ty_of(src, mem, chk, host, n.d)
    };

    chk.fns[k].ret = ret;
}

fn push_val(mem: &mut Mem, chk: &mut Chk, bits: u64, at: Node) -> u32 {
    if chk.val_n >= CAP_VALS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return 0;
    }
    chk.vals[chk.val_n] = bits;
    chk.val_n += 1;
    (chk.val_n - 1) as u32
}

fn int_mag(src: &[u8], mem: &mut Mem, chk: &Chk, tok_i: u32, at: Node) -> u64 {
    let _ = chk;
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut mag: u64 = 0;
    let mut i: usize = 0;
    let mut hex = false;
    if w.len() >= 2 && w[0] == b'0' && (w[1] == b'x' || w[1] == b'X') {
        hex = true;
        i = 2;
    }
    let mut overflow = false;
    while i < w.len() {
        let c = w[i];
        if c == b'_' {
            i += 1;
            continue;
        }
        let mut d = 0u64;
        if c >= b'0' && c <= b'9' {
            d = (c - b'0') as u64;
        } else if hex && c >= b'a' && c <= b'f' {
            d = (c - b'a' + 10) as u64;
        } else if hex && c >= b'A' && c <= b'F' {
            d = (c - b'A' + 10) as u64;
        } else {
            break;
        }
        let base: u64 = if hex { 16 } else { 10 };
        let m1 = mag.wrapping_mul(base);
        if mag != 0 && m1 / base != mag {
            overflow = true;
        }
        let m2 = m1.wrapping_add(d);
        if m2 < m1 {
            overflow = true;
        }
        mag = m2;
        i += 1;
    }
    if overflow {
        ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
        return 0;
    }
    mag
}

fn int_suffix(src: &[u8], mem: &Mem, tok_i: u32) -> u16 {
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut s = 0;
    let hex = w.len() >= 2 && w[0] == b'0' && (w[1] == b'x' || w[1] == b'X');
    let mut i = if hex { 2 } else { 0 };
    while i < w.len() {
        let c = w[i];
        let is_digit = (c >= b'0' && c <= b'9')
            || (hex && ((c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')));
        if !is_digit && c != b'_' {
            s = i;
            break;
        }
        i += 1;
    }
    if s == 0 {
        return TY_INTLIT;
    }
    let sfx = &w[s..];
    if bytes_eq(sfx, b"i8") {
        return TY_I8;
    }
    if bytes_eq(sfx, b"u8") {
        return TY_U8;
    }
    if bytes_eq(sfx, b"i16") {
        return TY_I16;
    }
    if bytes_eq(sfx, b"u16") {
        return TY_U16;
    }
    if bytes_eq(sfx, b"isize") {
        return TY_ISIZE;
    }
    if bytes_eq(sfx, b"i32") {
        return TY_I32;
    }
    if bytes_eq(sfx, b"u32") {
        return TY_U32;
    }
    if bytes_eq(sfx, b"i64") {
        return TY_I64;
    }
    if bytes_eq(sfx, b"u64") {
        return TY_U64;
    }
    if bytes_eq(sfx, b"i128") {
        return TY_I128;
    }
    if bytes_eq(sfx, b"u128") {
        return TY_U128;
    }
    if bytes_eq(sfx, b"usize") {
        return TY_USIZE;
    }
    TY_INTLIT
}

fn int_range_ok(mag: u64, neg: bool, t: u16) -> bool {
    let b = int_bits(t);
    if ty_is_signed(t) {
        let half: u64 = 1u64 << (b - 1);
        if neg {
            mag <= half
        } else {
            mag <= half - 1
        }
    } else {
        if neg {
            return false;
        }
        if b == 64 {
            true
        } else {
            mag <= (1u64 << b) - 1
        }
    }
}

fn parse_f64(src: &[u8], mem: &mut Mem, tok_i: u32, at: Node) -> u64 {
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut buf = [0u8; 64];
    let mut n: usize = 0;
    let mut i: usize = 0;
    while i < w.len() {
        let c = w[i];
        if c == b'_' {
            i += 1;
            continue;
        }

        if c == b'f' {
            break;
        }
        if n >= 64 {
            ndiag(mem, E_BAD_NUMBER, at, 0, 0);
            return 0;
        }
        buf[n] = c;
        n += 1;
        i += 1;
    }
    let _ = n;
    ndiag(mem, E_BAD_NUMBER, at, 0, 0);
    0
}

fn byte_lit_val(src: &[u8], mem: &Mem, tok_i: u32) -> u64 {
    let w = tok_bytes(src, mem.tok(tok_i));
    if w.len() < 4 {
        return 0;
    }
    let inner = &w[2..w.len() - 1];
    let byte: u8 = if inner.len() >= 2 && inner[0] == b'\\' {
        match inner[1] {
            b'n' => 10,
            b'r' => 13,
            b't' => 9,
            b'0' => 0,
            b'\\' => 92,
            b'\'' => 39,
            b'"' => 34,
            _ => inner[1],
        }
    } else if inner.len() >= 1 {
        inner[0]
    } else {
        0
    };
    byte as u64
}

fn intern_str(src: &[u8], mem: &mut Mem, chk: &mut Chk, tok_i: u32, prefix: usize, at: Node) -> u32 {
    let w = tok_bytes(src, mem.tok(tok_i));

    let inner = if w.len() >= prefix + 1 { &w[prefix..w.len() - 1] } else { &w[0..0] };
    let start = chk.pool_n;
    let mut i = 0;
    while i < inner.len() {
        let c = inner[i];
        let mut out = 0u8;
        if c == b'\\' && i + 1 < inner.len() {
            let e = inner[i + 1];
            out = match e {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'0' => 0,
                _ => e,
            };
            i += 2;
        } else {
            out = c;
            i += 1;
        }
        if chk.pool_n >= CAP_STR_POOL {
            ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
            return 0;
        }
        chk.str_pool[chk.pool_n] = out;
        chk.pool_n += 1;
    }

    let len = (chk.pool_n - start) as u32;
    let mut k = 0;
    while k < chk.str_n {
        let e = chk.strs[k];
        if e.len == len {
            let a_lo = e.off as usize;
            let mut same = true;
            let mut j = 0;
            while j < len as usize {
                if chk.str_pool[a_lo + j] != chk.str_pool[start + j] {
                    same = false;
                    break;
                }
                j += 1;
            }
            if same {
                chk.pool_n = start;
                return k as u32;
            }
        }
        k += 1;
    }
    if chk.str_n >= CAP_STRS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return 0;
    }
    chk.strs[chk.str_n] = StrEntry {
        off: start as u32,
        len,
    };
    chk.str_n += 1;
    (chk.str_n - 1) as u32
}

fn finalize_int(src: &[u8], mem: &mut Mem, chk: &mut Chk, node: u32, t: u16) {
    let n = mem.node(node);
    let i = node as usize;
    match n.kind {
        N_LIT_INT => {
            chk.res[i] = store_int_lit(src, mem, chk, n.a, false, t, n);
            chk.ty[i] = t;
        }
        N_UNARY => {
            if n.x == OP_NEG {
                if !ty_is_signed(t) {
                    ndiag(mem, E_NEG_UNSIGNED, n, 0, 0);
                    chk.ty[i] = TY_ERR;
                    return;
                }
                let opn = mem.node(n.e);
                if opn.kind == N_LIT_INT {

                    chk.res[n.e as usize] = store_int_lit(src, mem, chk, opn.a, true, t, opn);
                    chk.ty[n.e as usize] = t;
                    chk.res[i] = 1;
                    chk.ty[i] = t;
                    return;
                }
            }
            finalize_int(src, mem, chk, n.e, t);
            chk.ty[i] = t;
        }
        N_BINARY => {
            finalize_int(src, mem, chk, n.d, t);
            if n.x != OP_SHL && n.x != OP_SHR {
                finalize_int(src, mem, chk, n.e, t);
            }
            chk.ty[i] = t;
        }
        N_BLOCK => {
            if n.e != NODE_NIL {
                finalize_int(src, mem, chk, n.e, t);
            }
            chk.ty[i] = t;
        }
        N_IF => {

            finalize_int(src, mem, chk, n.e, t);
            if n.b != NODE_NIL {
                finalize_int(src, mem, chk, n.b, t);
            }
            chk.ty[i] = t;
        }
        N_NAME => {

            resolve_int_local(src, mem, chk, n.a, t);
            chk.ty[i] = t;
        }
        _ => {

            chk.ty[i] = t;
        }
    }
}

fn resolve_int_local(src: &[u8], mem: &mut Mem, chk: &mut Chk, name_tok: u32, t: u16) {
    let li = local_find(src, mem, chk, name_tok);
    if li == usize::MAX || chk.locals[li].ty != TY_INTLIT {
        return;
    }
    chk.locals[li].ty = t;
    let init = chk.locals[li].init;
    if init != NODE_NIL {
        finalize_int(src, mem, chk, init, t);
    }
}

fn concrete(src: &[u8], mem: &mut Mem, chk: &mut Chk, node: u32, t: u16) -> u16 {
    if t == TY_INTLIT {
        finalize_int(src, mem, chk, node, TY_I32);
        return TY_I32;
    }
    t
}

fn ck_const(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) -> u16 {
    let st = chk.consts[k].state;
    if st == 2 {
        return chk.consts[k].ty;
    }
    if st == 3 {
        return TY_ERR;
    }
    if st == 1 {
        let n = mem.node(chk.consts[k].node);
        ndiag(mem, E_CONST_CYCLE, n, 0, 0);
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    chk.consts[k].state = 1;
    let cn = mem.node(chk.consts[k].node);
    let t = ty_of(src, mem, chk, host, cn.d);
    if t == TY_ERR {
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    let is_agg = ty_is_struct(t) || ty_is_arr(t);
    if is_agg && !chk.sizing_done {

        chk.consts[k].state = 0;
        return TY_ERR;
    }
    if !ty_is_scalar(t) && !is_agg {
        ndiag(mem, E_CONST_TYPE, cn, 0, 0);
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    chk.in_const = true;
    let et = ck_ex(src, mem, chk, host, cn.e, t);
    chk.in_const = false;
    if et == TY_ERR {
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    let mut ok = true;
    let bits = if is_agg {
        ce_agg(src, mem, chk, host, cn.e, t, &mut ok)
    } else {
        ce(src, mem, chk, host, cn.e, &mut ok)
    };
    if !ok {
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    chk.consts[k].ty = t;
    chk.consts[k].bits = bits;
    chk.consts[k].state = 2;
    t
}

fn ce_agg(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, t: u16,
          ok: &mut bool) -> u64 {
    let size = chk.size_of(t) as usize;
    let base = chk.val_n;
    if base + size > CAP_VALS {
        ndiag(mem, E_TOO_MANY_ITEMS, mem.node(node), 0, 0);
        *ok = false;
        return 0;
    }
    chk.val_n += size;
    ce_slots(src, mem, chk, host, node, base, ok);
    base as u64
}

fn ce_slots(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, base: usize,
            ok: &mut bool) {
    let n = mem.node(node);
    let i = node as usize;
    let t = chk.ty[i];
    match n.kind {
        N_STRUCT_LIT => {
            let mut init = n.b;
            while init != NODE_NIL {
                let fin = mem.node(init);
                if fin.e == NODE_NIL {

                    ndiag(mem, E_NOT_CONST, fin, 0, 0);
                    *ok = false;
                    return;
                }
                let off = (chk.res[init as usize] & 0xFFFF) as usize;
                ce_slots(src, mem, chk, host, fin.e, base + off, ok);
                if !*ok {
                    return;
                }
                init = fin.link;
            }
        }
        N_ARRAY_LIT => {
            let es = chk.size_of(chk.ainfo(t).elem) as usize;
            let mut el = n.b;
            let mut idx = 0usize;
            while el != NODE_NIL {
                ce_slots(src, mem, chk, host, el, base + idx * es, ok);
                if !*ok {
                    return;
                }
                idx += 1;
                el = mem.node(el).link;
            }
        }
        N_ARRAY_REPEAT => {
            let a = chk.ainfo(t);
            let es = chk.size_of(a.elem) as usize;
            ce_slots(src, mem, chk, host, n.d, base, ok);
            if !*ok {
                return;
            }
            let mut r = 1usize;
            while r < a.len as usize {
                let mut k = 0;
                while k < es {
                    chk.vals[base + r * es + k] = chk.vals[base + k];
                    k += 1;
                }
                r += 1;
            }
        }
        _ => {
            if n.kind == N_NAME && chk.res[i] & RES_CONST != 0 {

                let k = (chk.res[i] & RES_MASK) as usize;
                let ct = ck_const(src, mem, chk, host, k);
                if ct == TY_ERR {
                    *ok = false;
                    return;
                }
                let size = chk.size_of(t) as usize;
                if ty_is_struct(t) || ty_is_arr(t) {
                    let sb = chk.consts[k].bits as usize;
                    let mut kk = 0;
                    while kk < size {
                        chk.vals[base + kk] = chk.vals[sb + kk];
                        kk += 1;
                    }
                } else {
                    chk.vals[base] = chk.consts[k].bits;
                }
            } else if chk.size_of(t) as usize == 1 {

                let v = ce(src, mem, chk, host, node, ok);
                if *ok {
                    chk.vals[base] = v;
                }
            } else {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                *ok = false;
            }
        }
    }
}

fn ce(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, ok: &mut bool) -> u64 {
    if chk.ce_depth > 64 {
        *ok = false;
        return 0;
    }
    chk.ce_depth += 1;
    let r = ce_inner(src, mem, chk, host, node, ok);
    chk.ce_depth -= 1;
    r
}

fn ce_inner(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, ok: &mut bool) -> u64 {
    let n = mem.node(node);
    let i = node as usize;
    let t = chk.ty[i];
    if ty_is_128(t) {

        ndiag(mem, E_NOT_CONST, n, 0, 0);
        *ok = false;
        return 0;
    }
    match n.kind {
        N_LIT_INT | N_LIT_FLOAT | N_LIT_BYTE => chk.vals[chk.res[i] as usize],
        N_LIT_BOOL => n.x as u64,
        N_LIT_STR => chk.res[i] as u64,
        N_NAME => {
            if chk.res[i] & RES_CONST != 0 {
                let k = (chk.res[i] & RES_MASK) as usize;
                let ct = ck_const(src, mem, chk, host, k);
                if ct == TY_ERR {
                    *ok = false;
                    return 0;
                }
                chk.consts[k].bits
            } else {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                *ok = false;
                0
            }
        }
        N_UNARY => {
            if chk.res[i] == 1 {

                return ce(src, mem, chk, host, n.e, ok);
            }
            let v = ce(src, mem, chk, host, n.e, ok);
            if !*ok {
                return 0;
            }
            let mut err = CeErr { any: false };
            let r = un_op(n.x, v, t, &mut err);
            if err.any {
                ndiag(mem, E_CONST_OVERFLOW, n, 0, 0);
                *ok = false;
                return 0;
            }
            r
        }
        N_BINARY => {
            let a = ce(src, mem, chk, host, n.d, ok);
            if !*ok {
                return 0;
            }
            let b = ce(src, mem, chk, host, n.e, ok);
            if !*ok {
                return 0;
            }
            let ot = chk.ty[n.d as usize];
            let mut err = CeErr { any: false };
            let r = ce_bin(n.x, a, b, ot, &mut err);
            if err.any {
                ndiag(mem, E_CONST_OVERFLOW, n, 0, 0);
                *ok = false;
                return 0;
            }
            r
        }
        N_CAST => {
            let v = ce(src, mem, chk, host, n.d, ok);
            if !*ok {
                return 0;
            }
            cast_bits(v, chk.ty[n.d as usize], t)
        }
        _ => {
            ndiag(mem, E_NOT_CONST, n, 0, 0);
            *ok = false;
            0
        }
    }
}

#[derive(Clone, Copy)]
pub struct CeErr {
    pub any: bool,
}

pub fn un_op(op: u16, v: u64, t: u16, err: &mut CeErr) -> u64 {
    if op == OP_NOT {
        if t == TY_BOOL {
            return (v == 0) as u64;
        }
        return mask_to(!v, t);
    }

    if t == TY_F64 {
        return (-f64::from_bits(v)).to_bits();
    }
    if v == min_signed(t) {
        err.any = true;
        return 0;
    }
    mask_to(v.wrapping_neg(), t)
}

pub fn fn_find_name(src: &[u8], mem: &Mem, chk: &Chk, name: &str) -> usize {
    let mut i = 0;
    while i < chk.fn_n {
        if bytes_eq(tok_bytes(src, mem.tok(chk.fns[i].name_tok)), name.as_bytes()) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

pub fn min_signed(t: u16) -> u64 {
    match int_bits(t) {
        8 => 0xFFFF_FFFF_FFFF_FF80,
        16 => 0xFFFF_FFFF_FFFF_8000,
        32 => 0xFFFF_FFFF_8000_0000,
        _ => 0x8000_0000_0000_0000,
    }
}

fn int_cmp(a: u64, b: u64, signed: bool) -> i8 {
    if signed {
        let x = a as i64;
        let y = b as i64;
        if x < y {
            return -1;
        }
        if x > y {
            return 1;
        }
        return 0;
    }
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    0
}

pub fn mask_to(v: u64, t: u16) -> u64 {
    let b = int_bits(t);
    if b == 64 {
        return v;
    }
    let m = (1u64 << b) - 1;
    let x = v & m;
    if ty_is_signed(t) && (x >> (b - 1)) & 1 == 1 {
        x | !m
    } else {
        x
    }
}

pub fn wrap_prim(op: u32, a: u64, b: u64, t: u16) -> u64 {
    let r = match op {
        PRIM_WRAP_ADD => a.wrapping_add(b),
        PRIM_WRAP_SUB => a.wrapping_sub(b),
        PRIM_WRAP_MUL => a.wrapping_mul(b),
        PRIM_WRAP_NEG => 0u64.wrapping_sub(a),
        PRIM_WRAP_SHL => {
            let w = int_bits(t) as u64;
            a.wrapping_shl((b % w) as u32)
        }
        PRIM_ROTL | PRIM_ROTR => {
            let w = int_bits(t) as u32;
            let mut sh = (b as u32) % w;
            if op == PRIM_ROTR {
                sh = (w - sh) % w;
            }
            let m = if w >= 64 { u64::MAX } else { (1u64 << w) - 1 };
            let v = a & m;
            (v << sh) | (v >> ((w - sh) % w))
        }
        _ => 0,
    };
    mask_to(r, t)
}

pub fn wrap_prim128(op: u32, a: u128, b: u128) -> u128 {
    match op {
        PRIM_WRAP_ADD => a.wrapping_add(b),
        PRIM_WRAP_SUB => a.wrapping_sub(b),
        PRIM_WRAP_MUL => a.wrapping_mul(b),
        PRIM_WRAP_NEG => 0u128.wrapping_sub(a),
        PRIM_WRAP_SHL => a.wrapping_shl((b % 128) as u32),
        PRIM_ROTL => a.rotate_left((b % 128) as u32),
        PRIM_ROTR => a.rotate_right((b % 128) as u32),
        _ => 0,
    }
}

pub fn sat_prim(op: u32, a: u64, b: u64, t: u16) -> u64 {
    let w = int_bits(t);
    if ty_is_signed(t) {
        let sa = a as i64 as i128;
        let sb = b as i64 as i128;
        let r = if op == PRIM_SAT_ADD { sa + sb } else { sa * sb };
        let max = (1i128 << (w - 1)) - 1;
        let min = -(1i128 << (w - 1));
        let c = if r > max { max } else if r < min { min } else { r };
        mask_to(c as u64, t)
    } else {
        let ua = a as u128;
        let ub = b as u128;
        let r = if op == PRIM_SAT_ADD { ua + ub } else { ua * ub };
        let max: u128 = if w == 64 { u64::MAX as u128 } else { (1u128 << w) - 1 };
        let c = if r > max { max } else { r };
        mask_to(c as u64, t)
    }
}

pub fn sat_prim128(op: u32, a: u128, b: u128, signed: bool) -> u128 {
    if signed {
        let sa = a as i128;
        let sb = b as i128;
        let r = if op == PRIM_SAT_ADD { sa.saturating_add(sb) } else { sa.saturating_mul(sb) };
        r as u128
    } else if op == PRIM_SAT_ADD {
        a.saturating_add(b)
    } else {
        a.saturating_mul(b)
    }
}

pub fn ce_bin(op: u16, a: u64, b: u64, t: u16, err: &mut CeErr) -> u64 {
    if t == TY_F64 {
        let x = f64::from_bits(a);
        let y = f64::from_bits(b);
        return match op {
            OP_ADD => (x + y).to_bits(),
            OP_SUB => (x - y).to_bits(),
            OP_MUL => (x * y).to_bits(),
            OP_DIV => (x / y).to_bits(),
            OP_REM => (x % y).to_bits(),
            OP_EQ => (x == y) as u64,
            OP_NE => (x != y) as u64,
            OP_LT => (x < y) as u64,
            OP_LE => (x <= y) as u64,
            OP_GT => (x > y) as u64,
            OP_GE => (x >= y) as u64,
            _ => {
                err.any = true;
                0
            }
        };
    }
    if t == TY_BOOL {
        let x = a != 0;
        let y = b != 0;
        return match op {
            OP_AND | OP_BAND => (x && y) as u64,
            OP_OR | OP_BOR => (x || y) as u64,
            OP_BXOR => (x ^ y) as u64,
            OP_EQ => (x == y) as u64,
            OP_NE => (x != y) as u64,
            _ => {
                err.any = true;
                0
            }
        };
    }
    if t == TY_STR {
        return match op {
            OP_EQ => (a == b) as u64,
            OP_NE => (a != b) as u64,
            _ => {
                err.any = true;
                0
            }
        };
    }

    let signed = ty_is_signed(t);
    let bits = int_bits(t);
    match op {
        OP_EQ => return (a == b) as u64,
        OP_NE => return (a != b) as u64,
        OP_LT => return (int_cmp(a, b, signed) < 0) as u64,
        OP_LE => return (int_cmp(a, b, signed) <= 0) as u64,
        OP_GT => return (int_cmp(a, b, signed) > 0) as u64,
        OP_GE => return (int_cmp(a, b, signed) >= 0) as u64,
        OP_BAND => return a & b,
        OP_BOR => return a | b,
        OP_BXOR => return mask_to(a ^ b, t),
        _ => {}
    }
    if op == OP_SHL || op == OP_SHR {

        if b >= bits as u64 {
            err.any = true;
            return 0;
        }
        let sh = b as u32;
        return if op == OP_SHL {
            let r = mask_to(a.wrapping_shl(sh), t);

            r
        } else if signed {
            mask_to(((a as i64) >> sh) as u64, t)
        } else {
            let m = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
            (a & m) >> sh
        };
    }
    if op == OP_DIV || op == OP_REM {
        if b == 0 {
            err.any = true;
            return 0;
        }
        if signed {
            let x = a as i64;
            let y = b as i64;
            if x == (min_signed(t) as i64) && y == -1 {
                err.any = true;
                return 0;
            }
            let r = if op == OP_DIV { x / y } else { x % y };
            return mask_to(r as u64, t);
        }
        let m = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
        let x = a & m;
        let y = b & m;
        return if op == OP_DIV { x / y } else { x % y };
    }

    let r = match op {
        OP_ADD => a.wrapping_add(b),
        OP_SUB => a.wrapping_sub(b),
        OP_MUL => a.wrapping_mul(b),
        _ => {
            err.any = true;
            return 0;
        }
    };
    let rm = mask_to(r, t);
    let of = if signed {
        let x = a as i64 as i128;
        let y = b as i64 as i128;
        let w = match op {
            OP_ADD => x + y,
            OP_SUB => x - y,
            _ => x * y,
        };
        w != (rm as i64 as i128)
    } else {
        let m = if bits == 64 {
            (u64::MAX as u128)
        } else {
            (((1u64 << bits) - 1) as u128)
        };
        let x = ((a & (m as u64)) as u128);
        let y = ((b & (m as u64)) as u128);
        let w = match op {
            OP_ADD => x + y,
            OP_SUB => {
                if x < y {
                    err.any = true;
                    return 0;
                }
                x - y
            }
            _ => x * y,
        };
        w != (rm as u128)
    };
    if of {
        err.any = true;
        return 0;
    }
    rm
}

pub fn ce_bin128(op: u16, a: u128, b: u128, signed: bool, err: &mut CeErr) -> u128 {
    if op == OP_EQ { return (a == b) as u128; }
    if op == OP_NE { return (a != b) as u128; }
    if op == OP_LT { return (if signed { (a as i128) < (b as i128) } else { a < b }) as u128; }
    if op == OP_LE { return (if signed { (a as i128) <= (b as i128) } else { a <= b }) as u128; }
    if op == OP_GT { return (if signed { (a as i128) > (b as i128) } else { a > b }) as u128; }
    if op == OP_GE { return (if signed { (a as i128) >= (b as i128) } else { a >= b }) as u128; }
    if op == OP_BAND { return a & b; }
    if op == OP_BOR { return a | b; }
    if op == OP_BXOR { return a ^ b; }
    if op == OP_SHL || op == OP_SHR {
        if b >= 128 { err.any = true; return 0; }
        let sh = b as u32;
        return if op == OP_SHL {
            a << sh
        } else if signed {
            ((a as i128) >> sh) as u128
        } else {
            a >> sh
        };
    }
    if op == OP_DIV || op == OP_REM {
        if b == 0 { err.any = true; return 0; }
        if signed {
            let x = a as i128;
            let y = b as i128;
            if x == i128::MIN && y == -1 { err.any = true; return 0; }
            return (if op == OP_DIV { x / y } else { x % y }) as u128;
        }
        return if op == OP_DIV { a / b } else { a % b };
    }
    if signed {
        let x = a as i128;
        let y = b as i128;
        if op == OP_ADD {
            let r = x.wrapping_add(y);
            if (x < 0) == (y < 0) && (r < 0) != (x < 0) {
                err.any = true;
                return 0;
            }
            return r as u128;
        }
        if op == OP_SUB {
            let r = x.wrapping_sub(y);
            if (x < 0) != (y < 0) && (r < 0) != (x < 0) {
                err.any = true;
                return 0;
            }
            return r as u128;
        }

        let r = x.wrapping_mul(y);
        if x != 0 {
            if x == -1 && y == i128::MIN {
                err.any = true;
                return 0;
            }
            if r / x != y {
                err.any = true;
                return 0;
            }
        }
        return r as u128;
    }
    if op == OP_ADD {
        let r = a.wrapping_add(b);
        if r < a {
            err.any = true;
            return 0;
        }
        return r;
    }
    if op == OP_SUB {
        if a < b {
            err.any = true;
            return 0;
        }
        return a - b;
    }

    let r = a.wrapping_mul(b);
    if a != 0 && r / a != b {
        err.any = true;
        return 0;
    }
    r
}

pub fn un_op128(op: u16, v: u128, err: &mut CeErr) -> u128 {
    if op == OP_NOT {
        return !v;
    }
    let x = v as i128;
    if x == i128::MIN {
        err.any = true;
        return 0;
    }
    x.wrapping_neg() as u128
}

fn int_mag128(src: &[u8], mem: &mut Mem, tok_i: u32, at: Node) -> u128 {
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut mag: u128 = 0;
    let mut i: usize = 0;
    let mut hex = false;
    if w.len() >= 2 && w[0] == b'0' && (w[1] == b'x' || w[1] == b'X') {
        hex = true;
        i = 2;
    }
    let mut overflow = false;
    while i < w.len() {
        let ch = w[i];
        if ch == b'_' {
            i += 1;
            continue;
        }
        let mut d: u128 = 0;
        if ch >= b'0' && ch <= b'9' {
            d = (ch - b'0') as u128;
        } else if hex && ch >= b'a' && ch <= b'f' {
            d = (ch - b'a' + 10) as u128;
        } else if hex && ch >= b'A' && ch <= b'F' {
            d = (ch - b'A' + 10) as u128;
        } else {
            break;
        }
        let base: u128 = if hex { 16 } else { 10 };
        let m1 = mag.wrapping_mul(base);
        if mag != 0 && m1 / base != mag {
            overflow = true;
        }
        let m2 = m1.wrapping_add(d);
        if m2 < m1 {
            overflow = true;
        }
        mag = m2;
        i += 1;
    }
    if overflow {
        ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
        return 0;
    }
    mag
}

fn int_range_ok128(mag: u128, neg: bool, t: u16) -> bool {
    if ty_is_signed(t) {
        let half: u128 = 1u128 << 127;
        if neg { mag <= half } else { mag <= half - 1 }
    } else {
        !neg
    }
}

fn push_val128(mem: &mut Mem, chk: &mut Chk, v: u128, at: Node) -> u32 {
    let lo = push_val(mem, chk, v as u64, at);
    let _hi = push_val(mem, chk, (v >> 64) as u64, at);
    lo
}

fn store_int_lit(src: &[u8], mem: &mut Mem, chk: &mut Chk, tok_i: u32, neg: bool, t: u16, at: Node) -> u32 {
    if ty_is_128(t) {
        let mag = int_mag128(src, mem, tok_i, at);
        if !int_range_ok128(mag, neg, t) {
            ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
        }
        let v = if neg { mag.wrapping_neg() } else { mag };
        return push_val128(mem, chk, v, at);
    }
    let mag = int_mag(src, mem, chk, tok_i, at);
    if !int_range_ok(mag, neg, t) {
        ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
    }
    let v = if neg { mag.wrapping_neg() } else { mag };
    push_val(mem, chk, v, at)
}

pub fn cast_bits(v: u64, from: u16, to: u16) -> u64 {
    if from == to {
        return v;
    }
    if to == TY_F64 {
        if from == TY_BOOL {
            return ((v != 0) as u8 as f64).to_bits();
        }
        return if ty_is_signed(from) {
            ((v as i64) as f64).to_bits()
        } else {
            let b = int_bits(from);
            let m = if b == 64 { u64::MAX } else { (1u64 << b) - 1 };
            ((v & m) as f64).to_bits()
        };
    }
    if from == TY_F64 {

        let x = f64::from_bits(v);
        return sat_f64_to_int(x, to);
    }

    mask_to(v, to)
}

fn sat_f64_to_int(x: f64, t: u16) -> u64 {
    if x.is_nan() {
        return 0;
    }
    let b = int_bits(t);
    if ty_is_signed(t) {
        let min = -((1i128) << (b - 1)) as i128;
        let max = ((1i128 << (b - 1)) - 1) as i128;
        let v = if x < min as f64 {
            min
        } else if x > max as f64 {
            max
        } else {
            x as i64 as i128
        };
        mask_to(v as i64 as u64, t)
    } else {
        let max: u128 = if b == 64 {
            u64::MAX as u128
        } else {
            ((1u128 << b) - 1) as u128
        };
        let v: u128 = if x <= 0.0 {
            0
        } else if x >= max as f64 {
            max
        } else {
            x as u64 as u128
        };
        v as u64
    }
}

fn ce_len(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32) -> u32 {
    let t = ck_ex(src, mem, chk, host, node, TY_USIZE);
    if t == TY_ERR {
        return u32::MAX;
    }
    let mut ok = true;
    let bits = ce(src, mem, chk, host, node, &mut ok);
    if !ok {
        return u32::MAX;
    }
    if bits > 0x00FF_FFFF {
        let n = mem.node(node);
        ndiag(mem, E_LIT_OUT_OF_RANGE, n, 0, 0);
        return u32::MAX;
    }
    bits as u32
}

fn scope_push(chk: &mut Chk) {
    chk.depth += 1;
}

fn scope_pop(chk: &mut Chk) {
    let d = chk.depth;
    while chk.local_n > 0 && chk.locals[chk.local_n - 1].depth == d {
        chk.local_n -= 1;
    }
    chk.depth -= 1;
}

fn default_pending_ints(src: &[u8], mem: &mut Mem, chk: &mut Chk) {
    let d = chk.depth;
    let mut k = chk.local_n;
    while k > 0 && chk.locals[k - 1].depth == d {
        k -= 1;
        if chk.locals[k].ty == TY_INTLIT {
            let init = chk.locals[k].init;
            chk.locals[k].ty = TY_I32;
            if init != NODE_NIL {
                finalize_int(src, mem, chk, init, TY_I32);
            }
        }
    }
}

fn local_add(src: &[u8], mem: &mut Mem, chk: &mut Chk, name_tok: u32, t: u16, flags: u16, at: Node) -> u32 {
    let size = chk.size_of(t);
    let slot = chk.next_slot;
    chk.next_slot = chk.next_slot.saturating_add(size);
    if chk.next_slot > FRAME_MAX {
        ndiag(mem, E_FRAME_TOO_BIG, at, 0, 0);
    }
    let is_wild = mem.tok(name_tok).kind == T_UNDERSCORE;
    if !is_wild {
        if chk.local_n >= CAP_LOCALS {
            ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
            return slot;
        }
        chk.locals[chk.local_n] = LInfo {
            name_tok,
            ty: t,
            flags,
            slot,
            depth: chk.depth,
            init: NODE_NIL,
        };
        chk.local_n += 1;
    }
    let _ = src;
    slot
}

fn local_find(src: &[u8], mem: &Mem, chk: &Chk, name_tok: u32) -> usize {
    let mut i = chk.local_n;
    while i > 0 {
        i -= 1;
        if tok_eq(src, mem, chk.locals[i].name_tok, name_tok) {
            return i;
        }
    }
    usize::MAX
}

fn const_find(src: &[u8], mem: &Mem, chk: &Chk, name_tok: u32) -> usize {
    let mut i = 0;
    while i < chk.const_n {
        if tok_eq(src, mem, chk.consts[i].name_tok, name_tok) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

fn fn_find(src: &[u8], mem: &Mem, chk: &Chk, name_tok: u32) -> usize {
    let mut i = 0;
    while i < chk.fn_n {
        if tok_eq(src, mem, chk.fns[i].name_tok, name_tok) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

fn host_fn_find(src: &[u8], mem: &Mem, host: &HostDef, name_tok: u32) -> usize {
    let w = tok_bytes(src, mem.tok(name_tok));
    let mut i = 0;
    while i < host.fn_n {
        if bytes_eq(w, host_name(host.fns[i].name)) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

fn place_behind_ref(src: &[u8], mem: &Mem, chk: &Chk, place: u32) -> bool {
    let n = mem.node(place);
    match n.kind {
        N_DEREF => true,
        N_DOT | N_INDEX => {
            chk.res[place as usize] & RES_DEREF != 0 || place_behind_ref(src, mem, chk, n.d)
        }
        _ => false,
    }
}

fn escape_safe(src: &[u8], mem: &Mem, chk: &Chk, node: u32) -> bool {
    let n = mem.node(node);
    match n.kind {
        N_LIT_BSTR | N_LIT_STR => true,
        N_CALL | N_METHOD => true,
        N_REFOF => {
            let e = mem.node(n.e);
            if e.kind == N_SLICE {
                escape_safe(src, mem, chk, n.e)
            } else if e.kind == N_ARRAY_LIT && e.c == 0 {
                true
            } else {
                place_behind_ref(src, mem, chk, n.e)
            }
        }
        N_SLICE => {
            if ty_is_slice(chk.ty[n.d as usize]) {
                escape_safe(src, mem, chk, n.d)
            } else {
                place_behind_ref(src, mem, chk, n.d)
            }
        }
        N_NAME => {
            let li = local_find(src, mem, chk, n.a);
            li != usize::MAX && chk.locals[li].flags & LFLAG_RETSAFE != 0
        }
        N_BLOCK => n.e != NODE_NIL && escape_safe(src, mem, chk, n.e),
        N_IF => {
            n.b != NODE_NIL
                && escape_safe(src, mem, chk, n.e)
                && escape_safe(src, mem, chk, n.b)
        }
        _ => false,
    }
}

fn mark_retsafe_local(chk: &mut Chk, safe: bool) {
    if safe && chk.local_n > 0 {
        chk.locals[chk.local_n - 1].flags |= LFLAG_RETSAFE;
    }
}

fn ck_fn(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) {
    let f = chk.fns[k];
    let n = mem.node(f.node);
    chk.local_n = 0;
    chk.depth = 0;
    chk.loop_depth = 0;
    chk.next_slot = 0;
    chk.ret_ty = f.ret;
    scope_push(chk);
    let mut p = f.first_param;
    while p != NODE_NIL {
        let pn = mem.node(p);
        let pt = chk.ty[p as usize];
        let slot = local_add(src, mem, chk, pn.a, pt, pn.x & FLAG_MUT, pn);
        chk.res[p as usize] = slot;

        let named = mem.tok(pn.a).kind != T_UNDERSCORE;
        mark_retsafe_local(chk, named && (ty_is_ref(pt) || ty_is_slice(pt)));
        p = pn.link;
    }

    chk.ret_borrow_body = if ty_is_ref(f.ret) || ty_is_slice(f.ret) {
        n.e
    } else {
        NODE_NIL
    };

    let _ = ck_ex(src, mem, chk, host, n.e, f.ret);
    scope_pop(chk);
    chk.fns[k].frame = chk.next_slot;
}

fn ck_ex(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, expected: u16) -> u16 {
    let t = ck_expr(src, mem, chk, host, node, expected);
    if t == TY_ERR {
        return TY_ERR;
    }
    if expected == TY_ANY {
        return t;
    }

    if t == TY_NEVER {
        return expected;
    }
    if t == TY_INTLIT && ty_is_int(expected) && expected != TY_INTLIT {
        finalize_int(src, mem, chk, node, expected);
        return expected;
    }

    if t != expected && ty_is_ref(expected) && ty_is_ref(t) {
        let e = chk.rinfo(expected);
        let f = chk.rinfo(t);
        if e.pointee == f.pointee && e.mutable == 0 {
            chk.ty[node as usize] = expected;
            return expected;
        }
    }

    if t != expected && ty_is_slice(expected) && ty_is_slice(t) {
        let e = chk.slinfo(expected);
        let f = chk.slinfo(t);
        if e.pointee == f.pointee && e.mutable == 0 {
            chk.ty[node as usize] = expected;
            return expected;
        }
    }

    if t != expected && ty_is_slice(expected) && mem.node(node).kind == N_REFOF && ty_is_ref(t) {
        let sl = chk.slinfo(expected);
        let rp = chk.rinfo(t);
        if ty_is_arr(rp.pointee)
            && chk.ainfo(rp.pointee).elem == sl.pointee
            && (sl.mutable == 0 || rp.mutable != 0)
        {
            chk.ty[node as usize] = expected;
            return expected;
        }
    }
    if t != expected {
        let n = mem.node(node);
        ndiag(mem, E_TYPE_MISMATCH, n, expected as u32, t as u32);
        return TY_ERR;
    }
    t
}

fn ck_operands(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, at: Node,
               lhs: u32, rhs: u32, guide: u16) -> u16 {
    let l = ck_expr(src, mem, chk, host, lhs, guide);
    if l == TY_ERR {
        return TY_ERR;
    }
    let rguide = if l == TY_INTLIT { guide } else { l };
    let r = ck_expr(src, mem, chk, host, rhs, rguide);
    if r == TY_ERR {
        return TY_ERR;
    }
    if l == TY_INTLIT && r == TY_INTLIT {
        if guide != TY_ANY && ty_is_int(guide) && guide != TY_INTLIT {
            finalize_int(src, mem, chk, lhs, guide);
            finalize_int(src, mem, chk, rhs, guide);
            return guide;
        }
        return TY_INTLIT;
    }
    if l == TY_INTLIT && ty_is_int(r) {
        finalize_int(src, mem, chk, lhs, r);
        return r;
    }
    if r == TY_INTLIT && ty_is_int(l) {
        finalize_int(src, mem, chk, rhs, l);
        return l;
    }
    if l != r {
        ndiag(mem, E_TYPE_MISMATCH, at, l as u32, r as u32);
        return TY_ERR;
    }
    l
}

fn unify2(src: &[u8], mem: &mut Mem, chk: &mut Chk, at: Node, an: u32, a: u16, bn: u32, b: u16,
          guide: u16) -> u16 {

    if a == TY_NEVER {
        return b;
    }
    if b == TY_NEVER {
        return a;
    }
    if a == TY_INTLIT && b == TY_INTLIT {
        if guide != TY_ANY && ty_is_int(guide) && guide != TY_INTLIT {
            finalize_int(src, mem, chk, an, guide);
            finalize_int(src, mem, chk, bn, guide);
            return guide;
        }

        return TY_INTLIT;
    }
    if a == TY_INTLIT && ty_is_int(b) {
        finalize_int(src, mem, chk, an, b);
        return b;
    }
    if b == TY_INTLIT && ty_is_int(a) {
        finalize_int(src, mem, chk, bn, a);
        return a;
    }
    if a == b {
        return a;
    }
    ndiag(mem, E_TYPE_MISMATCH, at, a as u32, b as u32);
    TY_ERR
}

fn ck_expr(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, expected: u16) -> u16 {
    let n = mem.node(node);
    let i = node as usize;
    let t = ck_expr_inner(src, mem, chk, host, node, n, expected);
    if i < CAP_NODES {
        chk.ty[i] = t;
    }
    t
}

fn ck_expr_inner(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, n: Node,
                 expected: u16) -> u16 {
    let i = node as usize;
    match n.kind {
        N_LIT_UNIT => TY_UNIT,
        N_LIT_BOOL => TY_BOOL,
        N_LIT_STR => {
            chk.res[i] = intern_str(src, mem, chk, n.a, 1, n);
            TY_STR
        }
        N_LIT_BYTE => {
            let v = byte_lit_val(src, mem, n.a);
            chk.res[i] = push_val(mem, chk, v, n);
            TY_U8
        }
        N_LIT_BSTR => {

            let id = intern_str(src, mem, chk, n.a, 2, n);
            let e = chk.strs[id as usize];
            let ri = push_val(mem, chk, POOL_TAG | e.off as u64, n);
            let _ = push_val(mem, chk, e.len as u64, n);
            chk.res[i] = ri;
            intern_slice(mem, chk, TY_U8, 0, n)
        }
        N_LIT_FLOAT => {
            let bits = parse_f64(src, mem, n.a, n);
            chk.res[i] = push_val(mem, chk, bits, n);
            TY_F64
        }
        N_LIT_INT => {
            let sfx = int_suffix(src, mem, n.a);
            if sfx != TY_INTLIT {
                chk.res[i] = store_int_lit(src, mem, chk, n.a, false, sfx, n);
                return sfx;
            }
            let ex = exp_ty(expected);
            if ex != TY_ANY && ty_is_int(ex) && ex != TY_INTLIT {
                chk.res[i] = store_int_lit(src, mem, chk, n.a, false, ex, n);
                return ex;
            }

            TY_INTLIT
        }
        N_NAME => {
            let li = local_find(src, mem, chk, n.a);
            if li != usize::MAX {
                chk.res[i] = chk.locals[li].slot;
                return chk.locals[li].ty;
            }
            let ci = const_find(src, mem, chk, n.a);
            if ci != usize::MAX {
                chk.res[i] = RES_CONST | ci as u32;
                return ck_const(src, mem, chk, host, ci);
            }
            if fn_find(src, mem, chk, n.a) != usize::MAX
                || host_fn_find(src, mem, host, n.a) != usize::MAX
            {
                ndiag(mem, E_FN_AS_VALUE, n, 0, 0);
                return TY_ERR;
            }
            ndiag(mem, E_UNDEFINED, n, 0, 0);
            TY_ERR
        }
        N_UNARY => {
            if n.x == OP_NOT {
                let t = ck_expr(src, mem, chk, host, n.e, expected);
                if t == TY_ERR {
                    return TY_ERR;
                }
                if t == TY_BOOL || ty_is_int(t) {
                    return t;
                }
                ndiag(mem, E_BAD_OPERAND, n, OP_NOT as u32, t as u32);
                return TY_ERR;
            }

            let opn = mem.node(n.e);
            if opn.kind == N_LIT_INT {
                let sfx = int_suffix(src, mem, opn.a);
                let ex = exp_ty(expected);
                let target = if sfx != TY_INTLIT {
                    sfx
                } else if ex != TY_ANY && ty_is_int(ex) && ex != TY_INTLIT {
                    ex
                } else {
                    return TY_INTLIT;
                };
                if !ty_is_signed(target) {
                    ndiag(mem, E_NEG_UNSIGNED, n, 0, 0);
                    return TY_ERR;
                }
                chk.res[n.e as usize] = store_int_lit(src, mem, chk, opn.a, true, target, opn);
                chk.ty[n.e as usize] = target;
                chk.res[i] = 1;
                return target;
            }
            let t = ck_expr(src, mem, chk, host, n.e, expected);
            if t == TY_ERR {
                return TY_ERR;
            }
            if t == TY_F64 || t == TY_INTLIT {
                return t;
            }
            if ty_is_int(t) {
                if !ty_is_signed(t) {
                    ndiag(mem, E_NEG_UNSIGNED, n, 0, 0);
                    return TY_ERR;
                }
                return t;
            }
            ndiag(mem, E_BAD_OPERAND, n, OP_NEG as u32, t as u32);
            TY_ERR
        }
        N_BINARY => {
            let expected = if is_hint(expected) { TY_ANY } else { expected };
            let op = n.x;
            if op == OP_AND || op == OP_OR {
                let a = ck_ex(src, mem, chk, host, n.d, TY_BOOL);
                let b = ck_ex(src, mem, chk, host, n.e, TY_BOOL);
                if a == TY_ERR || b == TY_ERR {
                    return TY_ERR;
                }
                return TY_BOOL;
            }
            if op == OP_SHL || op == OP_SHR {
                let l = ck_expr(src, mem, chk, host, n.d, expected);
                if l == TY_ERR {
                    return TY_ERR;
                }
                let l = if l == TY_INTLIT && expected != TY_ANY && ty_is_int(expected)
                    && expected != TY_INTLIT
                {
                    finalize_int(src, mem, chk, n.d, expected);
                    expected
                } else {
                    l
                };
                if !ty_is_int(l) {
                    ndiag(mem, E_BAD_OPERAND, n, op as u32, l as u32);
                    return TY_ERR;
                }
                let r = ck_expr(src, mem, chk, host, n.e, TY_ANY);
                if r == TY_ERR {
                    return TY_ERR;
                }
                let r = concrete(src, mem, chk, n.e, r);
                if !ty_is_int(r) {
                    ndiag(mem, E_BAD_OPERAND, n, op as u32, r as u32);
                    return TY_ERR;
                }
                return l;
            }
            let is_cmp = op == OP_EQ || op == OP_NE || op == OP_LT || op == OP_LE
                || op == OP_GT || op == OP_GE;
            let guide = if is_cmp { TY_ANY } else { expected };
            let t = ck_operands(src, mem, chk, host, n, n.d, n.e, guide);
            if t == TY_ERR {
                return TY_ERR;
            }
            if is_cmp {
                let t = if t == TY_INTLIT {
                    finalize_int(src, mem, chk, n.d, TY_I32);
                    finalize_int(src, mem, chk, n.e, TY_I32);
                    TY_I32
                } else {
                    t
                };
                let eq_only = op == OP_EQ || op == OP_NE;
                let ok = if eq_only {
                    ty_is_int(t) || t == TY_F64 || t == TY_BOOL || t == TY_STR
                } else {
                    ty_is_int(t) || t == TY_F64
                };
                if !ok {
                    ndiag(mem, E_BAD_OPERAND, n, op as u32, t as u32);
                    return TY_ERR;
                }
                return TY_BOOL;
            }
            if op == OP_BAND || op == OP_BOR || op == OP_BXOR {
                if t == TY_BOOL || ty_is_int(t) {
                    return t;
                }
                ndiag(mem, E_BAD_OPERAND, n, op as u32, t as u32);
                return TY_ERR;
            }

            if ty_is_int(t) || t == TY_F64 {
                return t;
            }
            ndiag(mem, E_BAD_OPERAND, n, op as u32, t as u32);
            TY_ERR
        }
        N_CAST => {
            let to = ty_of(src, mem, chk, host, n.e);
            if to == TY_ERR {
                return TY_ERR;
            }

            let guide = if ty_is_int(to) { to | TY_HINT } else { TY_ANY };
            let from = ck_expr(src, mem, chk, host, n.d, guide);
            if from == TY_ERR {
                return TY_ERR;
            }

            let from = concrete(src, mem, chk, n.d, from);
            let from_ok = ty_is_int(from) || from == TY_F64 || from == TY_BOOL;
            let to_ok = ty_is_int(to) || to == TY_F64;
            let pair_ok = from_ok && to_ok && !(from == TY_BOOL && to == TY_F64);
            if !pair_ok {
                ndiag(mem, E_BAD_CAST, n, from as u32, to as u32);
                return TY_ERR;
            }
            to
        }
        N_CALL => {
            if chk.in_const {

                ndiag(mem, E_NOT_CONST, n, 0, 0);
                return TY_ERR;
            }
            let fi = fn_find(src, mem, chk, n.a);
            if fi != usize::MAX {
                chk.res[i] = fi as u32;
                let f = chk.fns[fi];
                if n.c != f.param_n {
                    ndiag(mem, E_ARG_COUNT, n, f.param_n, n.c);
                    return TY_ERR;
                }
                let mut arg = n.b;
                let mut p = f.first_param;
                while arg != NODE_NIL && p != NODE_NIL {
                    let pt = chk.ty[p as usize];
                    let at = ck_ex(src, mem, chk, host, arg, pt);
                    let _ = at;
                    arg = mem.node(arg).link;
                    p = mem.node(p).link;
                }
                return f.ret;
            }
            let hi = host_fn_find(src, mem, host, n.a);
            if hi != usize::MAX {
                chk.res[i] = RES_HOST | hi as u32;
                let hf = &host.fns[hi];
                if n.c as usize != hf.params.len() {
                    ndiag(mem, E_ARG_COUNT, n, hf.params.len() as u32, n.c);
                    return TY_ERR;
                }
                let mut arg = n.b;
                let mut pi = 0;
                while arg != NODE_NIL && pi < hf.params.len() {
                    let pt = host_ty(src, mem, chk, host, &hf.params[pi], n);
                    let _ = ck_ex(src, mem, chk, host, arg, pt);
                    arg = mem.node(arg).link;
                    pi += 1;
                }
                return host_ty(src, mem, chk, host, &hf.ret, n);
            }
            ndiag(mem, E_UNKNOWN_FN, n, 0, 0);
            TY_ERR
        }
        N_METHOD => {
            if chk.in_const {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                return TY_ERR;
            }

            let rt = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if rt == TY_ERR {
                return TY_ERR;
            }

            if ty_is_slice(rt) {
                if tok_is(src, mem, n.a, b"len") && n.c == 0 {
                    chk.res[i] = RES_SLICE_LEN;
                    return TY_USIZE;
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            if ty_is_arr(rt) {
                if tok_is(src, mem, n.a, b"len") && n.c == 0 {
                    let len = chk.ainfo(rt).len as u64;
                    let vi = push_val(mem, chk, len, n);
                    chk.res[i] = RES_ARRAY_LEN | vi;
                    return TY_USIZE;
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            if rt == TY_STR {
                if n.c != 0 {
                    ndiag(mem, E_ARG_COUNT, n, 0, n.c);
                    return TY_ERR;
                }
                if tok_is(src, mem, n.a, b"len") {
                    chk.res[i] = RES_STR_LEN;
                    return TY_USIZE;
                }
                if tok_is(src, mem, n.a, b"as_bytes") {
                    chk.res[i] = RES_STR_BYTES;
                    return intern_slice(mem, chk, TY_U8, 0, n);
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            if ty_is_int(rt) {
                let rt = if rt == TY_INTLIT {
                    finalize_int(src, mem, chk, n.d, TY_I32);
                    TY_I32
                } else {
                    rt
                };
                let op = prim_op(src, mem, n.a);
                if op == 0 {
                    ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                    return TY_ERR;
                }
                let want_args = if op == PRIM_WRAP_NEG { 0 } else { 1 };
                if n.c != want_args {
                    ndiag(mem, E_ARG_COUNT, n, want_args, n.c);
                    return TY_ERR;
                }
                if want_args == 1 {
                    let u32_arg = op == PRIM_WRAP_SHL || op == PRIM_ROTL || op == PRIM_ROTR;
                    let at = if u32_arg { TY_U32 } else { rt };
                    let _ = ck_ex(src, mem, chk, host, n.b, at);
                }
                chk.res[i] = RES_PRIM | op;
                return rt;
            }

            if rt == TY_F64 {
                if n.c != 0 {
                    ndiag(mem, E_ARG_COUNT, n, 0, n.c);
                    return TY_ERR;
                }
                if tok_is(src, mem, n.a, b"to_bits") {
                    chk.res[i] = RES_PRIM | PRIM_TO_BITS;
                    return TY_U64;
                }
                if tok_is(src, mem, n.a, b"is_nan") {
                    chk.res[i] = RES_PRIM | PRIM_IS_NAN;
                    return TY_BOOL;
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            let (sty, recv_is_ref, recv_ref_mut) = if ty_is_struct(rt) {
                (rt, false, false)
            } else if ty_is_ref(rt) {
                let ri = chk.rinfo(rt);
                (ri.pointee, true, ri.mutable != 0)
            } else {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, rt as u32);
                return TY_ERR;
            };
            if !ty_is_struct(sty) {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, sty as u32);
                return TY_ERR;
            }
            let fi = method_find(src, mem, chk, sty, n.a);
            if fi == usize::MAX {
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }
            let f = chk.fns[fi];
            if n.c + 1 != f.param_n {
                ndiag(mem, E_ARG_COUNT, n, f.param_n.wrapping_sub(1), n.c);
                return TY_ERR;
            }

            let recv_pty = chk.ty[f.first_param as usize];
            let want_ref = ty_is_ref(recv_pty);
            let want_mut = want_ref && chk.rinfo(recv_pty).mutable != 0;
            let mut mode_place = false;
            if want_ref {
                if recv_is_ref {

                    if want_mut && !recv_ref_mut {
                        ndiag(mem, E_REF_MUT_NEEDED, n, 0, 0);
                        return TY_ERR;
                    }
                    mode_place = false;
                } else {

                    let mut root_mut = false;
                    let pt = ck_place(src, mem, chk, host, n.d, &mut root_mut);
                    if pt == TY_ERR {
                        return TY_ERR;
                    }
                    if want_mut && !root_mut {
                        ndiag(mem, E_REF_MUT_NEEDED, n, 0, 0);
                        return TY_ERR;
                    }
                    mode_place = true;
                }
            } else {

                if recv_is_ref {

                    ndiag(mem, E_BAD_RECEIVER, n, 0, 0);
                    return TY_ERR;
                }
                mode_place = false;
            }
            chk.res[i] = (if mode_place { RES_MPLACE } else { 0 }) | fi as u32;

            let mut arg = n.b;
            let mut p = mem.node(f.first_param).link;
            while arg != NODE_NIL && p != NODE_NIL {
                let pt = chk.ty[p as usize];
                let _ = ck_ex(src, mem, chk, host, arg, pt);
                arg = mem.node(arg).link;
                p = mem.node(p).link;
            }
            f.ret
        }
        N_DOT => {
            let bt0 = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            if !ty_is_struct(bt) {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, bt as u32);
                return TY_ERR;
            }
            let mut off = 0u32;
            let mut fty = TY_ERR;
            if !field_lookup(src, mem, chk, host, bt, n.a, &mut off, &mut fty) {
                ndiag(mem, E_UNKNOWN_FIELD, n, 0, 0);
                return TY_ERR;
            }
            chk.res[i] = off | if deref { RES_DEREF } else { 0 };
            fty
        }
        N_INDEX => {
            let bt0 = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            let elem = if ty_is_arr(bt) {
                chk.ainfo(bt).elem
            } else if ty_is_slice(bt) {
                chk.slinfo(bt).pointee
            } else {
                ndiag(mem, E_NOT_AN_ARRAY, n, 0, bt0 as u32);
                return TY_ERR;
            };
            let _ = ck_ex(src, mem, chk, host, n.e, TY_USIZE);
            chk.res[i] = if deref { RES_DEREF } else { 0 };
            elem
        }
        N_SLICE => {

            ndiag(mem, E_SUBSLICE_REF, n, 0, 0);
            TY_ERR
        }
        N_STRUCT_LIT => {
            let s = find_struct(src, mem, chk, host, n.a);
            if s == u16::MAX {
                ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
                return TY_ERR;
            }
            let st = TY_STRUCT0 + s;
            let info = chk.structs[s as usize];

            let mut init = n.b;
            while init != NODE_NIL {
                let fin = mem.node(init);
                let mut off = 0u32;
                let mut fty = TY_ERR;
                if !field_lookup(src, mem, chk, host, st, fin.a, &mut off, &mut fty) {
                    ndiag(mem, E_UNKNOWN_FIELD, fin, 0, 0);
                    return TY_ERR;
                }

                let mut other = n.b;
                while other != init {
                    if tok_eq(src, mem, mem.node(other).a, fin.a) {
                        ndiag(mem, E_DUP_FIELD, fin, 0, 0);
                        return TY_ERR;
                    }
                    other = mem.node(other).link;
                }
                chk.res[init as usize] = off;
                chk.ty[init as usize] = fty;
                if fin.e == NODE_NIL {

                    let li = local_find(src, mem, chk, fin.a);
                    if li == usize::MAX {
                        ndiag(mem, E_UNDEFINED, fin, 0, 0);
                        return TY_ERR;
                    }
                    if chk.locals[li].ty != fty {
                        ndiag(mem, E_TYPE_MISMATCH, fin, fty as u32, chk.locals[li].ty as u32);
                        return TY_ERR;
                    }

                    chk.res[init as usize] = off | (chk.locals[li].slot << 16);
                } else {
                    let _ = ck_ex(src, mem, chk, host, fin.e, fty);
                }
                init = fin.link;
            }
            if n.c != info.field_n {
                ndiag(mem, E_MISSING_FIELD, n, info.field_n, n.c);
                return TY_ERR;
            }
            st
        }
        N_TUPLE => {
            if n.c as usize > TUP_MAX || n.c < 2 {
                ndiag(mem, E_TUPLE, n, 0, 0);
                return TY_ERR;
            }
            let exp_tup = expected != TY_ANY && ty_is_tuple(expected)
                && chk.tinfo(expected).count == n.c as u16;
            let exp = chk.tinfo(expected);
            let mut elems = [TY_ERR; TUP_MAX];
            let mut el = n.b;
            let mut i = 0usize;
            while el != NODE_NIL {
                let ee = if exp_tup { exp.elems[i] } else { TY_ANY };
                let et = ck_expr(src, mem, chk, host, el, ee);
                if et == TY_ERR {
                    return TY_ERR;
                }
                elems[i] = if exp_tup && et == TY_INTLIT && ty_is_int(exp.elems[i]) {
                    finalize_int(src, mem, chk, el, exp.elems[i]);
                    exp.elems[i]
                } else {
                    concrete(src, mem, chk, el, et)
                };
                i += 1;
                el = mem.node(el).link;
            }
            intern_tuple(mem, chk, &elems, n.c as usize, n)
        }
        N_ARRAY_LIT => {
            let mut elem_exp = TY_ANY;
            if expected != TY_ANY && ty_is_arr(expected) {
                elem_exp = chk.ainfo(expected).elem;
            }
            if n.c == 0 {
                if elem_exp == TY_ANY {
                    ndiag(mem, E_ANNOTATION_NEEDED, n, 0, 0);
                    return TY_ERR;
                }
                return intern_arr(mem, chk, elem_exp, 0, n);
            }
            let mut el = n.b;
            let mut ety = elem_exp;
            let mut first = true;
            while el != NODE_NIL {
                if first && ety == TY_ANY {
                    let t = ck_expr(src, mem, chk, host, el, TY_ANY);
                    if t == TY_ERR {
                        return TY_ERR;
                    }
                    ety = concrete(src, mem, chk, el, t);
                } else {
                    let t = ck_ex(src, mem, chk, host, el, ety);
                    if t == TY_ERR {
                        return TY_ERR;
                    }
                }
                first = false;
                el = mem.node(el).link;
            }
            intern_arr(mem, chk, ety, n.c, n)
        }
        N_ARRAY_REPEAT => {
            let mut elem_exp = TY_ANY;
            if expected != TY_ANY && ty_is_arr(expected) {
                elem_exp = chk.ainfo(expected).elem;
            }
            let t = ck_expr(src, mem, chk, host, n.d, elem_exp);
            if t == TY_ERR {
                return TY_ERR;
            }
            let ety = if elem_exp != TY_ANY && t == TY_INTLIT {
                finalize_int(src, mem, chk, n.d, elem_exp);
                elem_exp
            } else {
                concrete(src, mem, chk, n.d, t)
            };
            if elem_exp != TY_ANY && ety != elem_exp {
                ndiag(mem, E_TYPE_MISMATCH, n, elem_exp as u32, ety as u32);
                return TY_ERR;
            }
            let len = ce_len(src, mem, chk, host, n.e);
            if len == u32::MAX {
                return TY_ERR;
            }
            intern_arr(mem, chk, ety, len, n)
        }
        N_IF => {
            let expected = if is_hint(expected) { TY_ANY } else { expected };
            let _ = ck_ex(src, mem, chk, host, n.d, TY_BOOL);
            if n.b == NODE_NIL {

                if expected != TY_ANY && expected != TY_UNIT {
                    ndiag(mem, E_NO_ELSE, n, 0, 0);
                    return TY_ERR;
                }
                let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);
                return TY_UNIT;
            }

            let a = ck_expr(src, mem, chk, host, n.e, expected);
            if a == TY_ERR {
                return TY_ERR;
            }
            let bguide = if a == TY_INTLIT || a == TY_NEVER { expected } else { a };
            let b = ck_expr(src, mem, chk, host, n.b, bguide);
            if b == TY_ERR {
                return TY_ERR;
            }
            unify2(src, mem, chk, n, n.e, a, n.b, b, expected)
        }
        N_MATCH => {
            let expected = if is_hint(expected) { TY_ANY } else { expected };
            let st = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if st == TY_ERR {
                return TY_ERR;
            }
            let st = concrete(src, mem, chk, n.d, st);
            if !(ty_is_int(st) || st == TY_BOOL || st == TY_STR || ty_is_enum(st)) {
                ndiag(mem, E_PATTERN_TYPE, n, st as u32, 0);
                return TY_ERR;
            }

            let mut result = TY_ANY;
            let mut saw_wild = false;
            let mut saw_true = false;
            let mut saw_false = false;
            let mut seen_variants: u64 = 0;
            let mut arm = n.b;
            while arm != NODE_NIL {
                let an = mem.node(arm);
                let mut pat = an.b;
                while pat != NODE_NIL {
                    ck_pattern(src, mem, chk, pat, st, &mut saw_wild, &mut saw_true,
                               &mut saw_false, &mut seen_variants);
                    pat = mem.node(pat).link;
                }
                let guide = if result != TY_ANY && result != TY_INTLIT && result != TY_NEVER {
                    result
                } else {
                    expected
                };
                let t = ck_expr(src, mem, chk, host, an.e, guide);
                if t == TY_ERR {
                    return TY_ERR;
                }
                if t == TY_NEVER {

                    if result == TY_ANY {
                        result = TY_NEVER;
                    }
                    arm = an.link;
                    continue;
                }
                if t == TY_INTLIT {
                    if result != TY_ANY && result != TY_INTLIT && result != TY_NEVER {
                        finalize_int(src, mem, chk, an.e, result);
                    } else {
                        result = TY_INTLIT;
                    }
                } else if result == TY_ANY || result == TY_INTLIT || result == TY_NEVER {
                    if result == TY_INTLIT {
                        if !ty_is_int(t) {
                            ndiag(mem, E_TYPE_MISMATCH, an, t as u32, TY_INTLIT as u32);
                            return TY_ERR;
                        }

                        let mut back = n.b;
                        while back != arm {
                            let bk = mem.node(back);
                            if chk.ty[bk.e as usize] == TY_INTLIT {
                                finalize_int(src, mem, chk, bk.e, t);
                            }
                            back = bk.link;
                        }
                    }
                    result = t;
                } else if t != result {
                    ndiag(mem, E_TYPE_MISMATCH, an, result as u32, t as u32);
                    return TY_ERR;
                }
                arm = an.link;
            }
            if result == TY_INTLIT {
                let t = if expected != TY_ANY && ty_is_int(expected) && expected != TY_INTLIT {
                    expected
                } else {
                    TY_I32
                };
                let mut back = n.b;
                while back != NODE_NIL {
                    let bk = mem.node(back);
                    if chk.ty[bk.e as usize] == TY_INTLIT {
                        finalize_int(src, mem, chk, bk.e, t);
                    }
                    back = bk.link;
                }
                result = t;
            }
            let mut back = n.b;
            while back != NODE_NIL {
                chk.ty[back as usize] = result;
                back = mem.node(back).link;
            }
            let exhaustive = saw_wild
                || (st == TY_BOOL && saw_true && saw_false)
                || (ty_is_enum(st) && enum_all_seen(chk, st, seen_variants));
            if !exhaustive {
                ndiag(mem, E_NOT_EXHAUSTIVE, n, 0, 0);
                return TY_ERR;
            }
            if result == TY_ANY {

                ndiag(mem, E_NOT_EXHAUSTIVE, n, 0, 0);
                return TY_ERR;
            }
            result
        }
        N_ASSERT => {

            let _ = ck_ex(src, mem, chk, host, n.c, TY_BOOL);
            chk.res[i] = if n.a != NODE_NIL {
                intern_str(src, mem, chk, n.a, 1, n)
            } else {
                NODE_NIL
            };
            TY_UNIT
        }
        N_BLOCK => ck_block(src, mem, chk, host, node, n, expected),
        N_REFOF => {

            if mem.node(n.e).kind == N_SLICE {
                return ck_slice(src, mem, chk, host, n.e, n.x & FLAG_MUT, n);
            }

            let opn = mem.node(n.e);
            if opn.kind == N_ARRAY_LIT && opn.c == 0 {
                let elem = if expected != TY_ANY && ty_is_slice(expected) {
                    chk.slinfo(expected).pointee
                } else {
                    TY_ANY
                };
                if elem == TY_ANY {
                    ndiag(mem, E_ANNOTATION_NEEDED, n, 0, 0);
                    return TY_ERR;
                }
                return intern_slice(mem, chk, elem, n.x & FLAG_MUT, n);
            }
            let mut root_mut = false;
            let pt = ck_place(src, mem, chk, host, n.e, &mut root_mut);
            if pt == TY_ERR {
                return TY_ERR;
            }
            let want_mut = n.x & FLAG_MUT != 0;
            if want_mut && !root_mut {
                ndiag(mem, E_REF_MUT_NEEDED, n, 0, 0);
                return TY_ERR;
            }
            intern_ref(mem, chk, pt, if want_mut { FLAG_MUT } else { 0 }, n)
        }
        N_DEREF => {
            let rt = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            if rt == TY_ERR {
                return TY_ERR;
            }
            if !ty_is_ref(rt) {
                ndiag(mem, E_DEREF_NON_REF, n, 0, 0);
                return TY_ERR;
            }
            chk.rinfo(rt).pointee
        }
        N_RETURN => {

            if n.e != NODE_NIL {
                let _ = ck_ex(src, mem, chk, host, n.e, chk.ret_ty);

                if (ty_is_ref(chk.ret_ty) || ty_is_slice(chk.ret_ty))
                    && !escape_safe(src, mem, chk, n.e)
                {
                    ndiag(mem, E_REF_ESCAPES, n, 0, 0);
                }
            } else if chk.ret_ty != TY_UNIT {
                ndiag(mem, E_TYPE_MISMATCH, n, chk.ret_ty as u32, TY_UNIT as u32);
            }
            TY_NEVER
        }
        N_ASSIGN => ck_assign(src, mem, chk, host, node),
        N_ASSOC_CALL => {

            if chk.in_const {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                return TY_ERR;
            }
            if tok_is(src, mem, n.a, b"f64") && tok_is(src, mem, n.e, b"from_bits") {
                if n.c != 1 {
                    ndiag(mem, E_ARG_COUNT, n, 1, n.c);
                    return TY_ERR;
                }
                let _ = ck_ex(src, mem, chk, host, n.b, TY_U64);
                return TY_F64;
            }
            ndiag(mem, E_BAD_PATH, n, 0, 0);
            TY_ERR
        }
        N_PATHCONST => {

            let ev = find_enum(src, mem, chk, n.a);
            if ev != u16::MAX {
                let et = TY_ENUM0 + ev;
                let tag = variant_tag(src, mem, chk, et, n.b);
                if tag == u32::MAX {
                    ndiag(mem, E_UNKNOWN_VARIANT, n, 0, 0);
                    return TY_ERR;
                }
                chk.res[i] = push_val(mem, chk, tag as u64, n);
                return et;
            }

            let t = int_ty_named(src, mem, n.a);
            let is_max = tok_is(src, mem, n.b, b"MAX");
            let is_min = tok_is(src, mem, n.b, b"MIN");
            if t == TY_ERR || (!is_max && !is_min) {
                ndiag(mem, E_BAD_PATH, n, 0, 0);
                return TY_ERR;
            }
            if ty_is_128(t) {
                let v: u128 = if is_min {
                    if ty_is_signed(t) { 1u128 << 127 } else { 0 }
                } else if ty_is_signed(t) {
                    (1u128 << 127) - 1
                } else {
                    u128::MAX
                };
                chk.res[i] = push_val128(mem, chk, v, n);
                return t;
            }
            let bits = int_bits(t);
            let v = if is_min {
                if ty_is_signed(t) { min_signed(t) } else { 0 }
            } else if ty_is_signed(t) {
                (1u64 << (bits - 1)) - 1
            } else if bits == 64 {
                u64::MAX
            } else {
                (1u64 << bits) - 1
            };
            chk.res[i] = push_val(mem, chk, v, n);
            t
        }
        _ => {

            ndiag(mem, E_TYPE_MISMATCH, n, expected as u32, TY_ERR as u32);
            TY_ERR
        }
    }
}

fn ck_pattern(src: &[u8], mem: &mut Mem, chk: &mut Chk, pat: u32, st: u16, saw_wild: &mut bool,
              saw_true: &mut bool, saw_false: &mut bool, seen_variants: &mut u64) {
    let p = mem.node(pat);
    let i = pat as usize;
    match p.kind {
        N_PAT_WILD => {
            *saw_wild = true;
            chk.ty[i] = st;
        }
        N_PAT_ENUM => {

            if !ty_is_enum(st) {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, 0);
                return;
            }
            let ev = find_enum(src, mem, chk, p.a);
            if ev == u16::MAX || TY_ENUM0 + ev != st {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32,
                      if ev == u16::MAX { 0 } else { (TY_ENUM0 + ev) as u32 });
                return;
            }
            let tag = variant_tag(src, mem, chk, st, p.b);
            if tag == u32::MAX {
                ndiag(mem, E_UNKNOWN_VARIANT, p, 0, 0);
                return;
            }
            if tag < 64 {
                *seen_variants |= 1u64 << tag;
            }
            chk.res[i] = push_val(mem, chk, tag as u64, p);
            chk.ty[i] = st;
        }
        N_PAT_BOOL => {
            if st != TY_BOOL {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_BOOL as u32);
                return;
            }
            if p.x == 1 {
                *saw_true = true;
            } else {
                *saw_false = true;
            }
            chk.ty[i] = st;
        }
        N_PAT_INT => {
            if !ty_is_int(st) || st == TY_INTLIT {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_I32 as u32);
                return;
            }
            let neg = p.x == 1;
            if neg && !ty_is_signed(st) {
                ndiag(mem, E_NEG_UNSIGNED, p, 0, 0);
                return;
            }
            let mag = int_mag(src, mem, chk, p.a, p);
            if !int_range_ok(mag, neg, st) {
                ndiag(mem, E_LIT_OUT_OF_RANGE, p, 0, 0);
                return;
            }
            let bits = if neg { mag.wrapping_neg() } else { mag };
            chk.res[i] = push_val(mem, chk, mask_to(bits, st), p);
            chk.ty[i] = st;
        }
        N_PAT_STR => {
            if st != TY_STR {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_STR as u32);
                return;
            }
            chk.res[i] = intern_str(src, mem, chk, p.a, 1, p);
            chk.ty[i] = st;
        }
        N_PAT_BYTE => {

            if st != TY_U8 {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_U8 as u32);
                return;
            }
            let v = byte_lit_val(src, mem, p.a);
            chk.res[i] = push_val(mem, chk, v, p);
            chk.ty[i] = st;
        }
        N_PAT_CONST => {

            let ci = const_find(src, mem, chk, p.a);
            if ci == usize::MAX {
                ndiag(mem, E_UNDEFINED, p, 0, 0);
                return;
            }
            let ct = chk.consts[ci].ty;
            if chk.consts[ci].state != 2 || ct == TY_ERR {

                return;
            }
            if ct != st {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, ct as u32);
                return;
            }
            if ty_is_struct(ct) || ty_is_arr(ct) {

                ndiag(mem, E_PATTERN_TYPE, p, st as u32, ct as u32);
                return;
            }

            let bits = chk.consts[ci].bits;
            let cmp = if ty_is_int(st) { mask_to(bits, st) } else { bits };
            chk.res[i] = push_val(mem, chk, cmp, p);
            chk.ty[i] = st;
        }
        _ => {}
    }
}

fn ck_block(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, n: Node,
            expected: u16) -> u16 {
    let _ = node;
    scope_push(chk);
    let mut s = n.b;
    let mut last = NODE_NIL;
    while s != NODE_NIL {
        ck_stmt(src, mem, chk, host, s);
        last = s;
        s = mem.node(s).link;
    }
    let mut t = if n.e != NODE_NIL {
        ck_expr(src, mem, chk, host, n.e, expected)
    } else {
        TY_UNIT
    };

    if t == TY_INTLIT {
        if expected != TY_ANY && ty_is_int(expected) && expected != TY_INTLIT {
            finalize_int(src, mem, chk, n.e, expected);
            t = expected;
        } else if mem.node(n.e).kind == N_NAME {
            t = concrete(src, mem, chk, n.e, t);
        }
    }

    if node == chk.ret_borrow_body && n.e != NODE_NIL && !escape_safe(src, mem, chk, n.e) {
        ndiag(mem, E_REF_ESCAPES, mem.node(n.e), 0, 0);
    }
    default_pending_ints(src, mem, chk);
    scope_pop(chk);

    if n.e == NODE_NIL && last != NODE_NIL {
        let ln = mem.node(last);
        let inner = if ln.kind == N_EXPR_STMT { ln.e } else { last };
        if inner != NODE_NIL && (inner as usize) < CAP_NODES && chk.ty[inner as usize] == TY_NEVER {
            return TY_NEVER;
        }
    }
    if n.e == NODE_NIL && expected != TY_ANY && expected != TY_UNIT {
        ndiag(mem, E_TYPE_MISMATCH, n, expected as u32, TY_UNIT as u32);
        return TY_ERR;
    }
    t
}

fn ck_assign(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32) -> u16 {
    let n = mem.node(node);
    let i = node as usize;
    let mut root_mut = false;
    let mut pt = ck_place(src, mem, chk, host, n.d, &mut root_mut);
    if pt == TY_ERR {
        return TY_ERR;
    }
    if !root_mut {
        ndiag(mem, E_ASSIGN_IMMUTABLE, n, 0, 0);
        return TY_ERR;
    }

    if pt == TY_INTLIT && mem.node(n.d).kind == N_NAME {
        let vt = ck_expr(src, mem, chk, host, n.e, TY_ANY);
        if vt == TY_ERR {
            return TY_ERR;
        }
        let target = if ty_is_int(vt) && vt != TY_INTLIT { vt } else { TY_I32 };
        resolve_int_local(src, mem, chk, mem.node(n.d).a, target);
        pt = target;
    }
    if n.x == 0 {
        let _ = ck_ex(src, mem, chk, host, n.e, pt);
    } else if n.x == OP_SHL || n.x == OP_SHR {
        if !ty_is_int(pt) {
            ndiag(mem, E_BAD_OPERAND, n, n.x as u32, pt as u32);
            return TY_ERR;
        }
        let r = ck_expr(src, mem, chk, host, n.e, TY_ANY);
        if r == TY_ERR {
            return TY_ERR;
        }
        let r = concrete(src, mem, chk, n.e, r);
        if !ty_is_int(r) {
            ndiag(mem, E_BAD_OPERAND, n, n.x as u32, r as u32);
        }
    } else {
        let op_ok = if n.x == OP_BAND || n.x == OP_BOR || n.x == OP_BXOR {
            ty_is_int(pt) || pt == TY_BOOL
        } else {
            ty_is_int(pt) || pt == TY_F64
        };
        if !op_ok {
            ndiag(mem, E_BAD_OPERAND, n, n.x as u32, pt as u32);
            return TY_ERR;
        }
        let _ = ck_ex(src, mem, chk, host, n.e, pt);
    }
    if i < CAP_NODES {
        chk.ty[i] = TY_UNIT;
    }
    TY_UNIT
}

fn ck_stmt(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, s: u32) {
    let n = mem.node(s);
    let i = s as usize;
    match n.kind {
        N_LET => {
          if n.x & FLAG_TUPLE != 0 {

            let it = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            if it == TY_ERR {
                return;
            }
            if !ty_is_tuple(it) || chk.tinfo(it).count != n.c as u16 {
                ndiag(mem, E_TUPLE, n, it as u32, 0);
                return;
            }
            let info = chk.tinfo(it);
            let base = chk.next_slot;
            chk.next_slot = chk.next_slot.saturating_add(info.size);
            if chk.next_slot > FRAME_MAX {
                ndiag(mem, E_FRAME_TOO_BIG, n, 0, 0);
            }

            let mut el = n.b;
            let mut ei = 0usize;
            while el != NODE_NIL {
                let pn = mem.node(el);
                if mem.tok(pn.a).kind != T_UNDERSCORE && chk.local_n < CAP_LOCALS {
                    chk.locals[chk.local_n] = LInfo {
                        name_tok: pn.a,
                        ty: info.elems[ei],
                        flags: pn.x & FLAG_MUT,
                        slot: base + info.offs[ei] as u32,
                        depth: chk.depth,
                        init: NODE_NIL,
                    };
                    chk.local_n += 1;
                }
                ei += 1;
                el = mem.node(el).link;
            }
            chk.res[i] = base;
            chk.ty[i] = it;
            return;
          }
            let t = if n.d != NODE_NIL {
                let annot = ty_of(src, mem, chk, host, n.d);
                if annot == TY_ERR {
                    return;
                }

                let _ = ck_ex(src, mem, chk, host, n.e, annot);
                annot
            } else {
                let r = ck_expr(src, mem, chk, host, n.e, TY_ANY);
                if r == TY_ERR {
                    return;
                }

                if r == TY_INTLIT && mem.tok(n.a).kind != T_UNDERSCORE {
                    r
                } else {
                    concrete(src, mem, chk, n.e, r)
                }
            };

            let slot = local_add(src, mem, chk, n.a, t, n.x & FLAG_MUT, n);
            chk.res[i] = slot;
            chk.ty[i] = t;
            if t == TY_INTLIT {

                chk.locals[chk.local_n - 1].init = n.e;
            }

            if (ty_is_ref(t) || ty_is_slice(t)) && mem.tok(n.a).kind != T_UNDERSCORE {
                mark_retsafe_local(chk, escape_safe(src, mem, chk, n.e));
            }
        }
        N_ASSIGN => {
            let _ = ck_assign(src, mem, chk, host, s);
        }
        N_EXPR_STMT => {
            let t = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            let _ = concrete(src, mem, chk, n.e, t);
        }
        N_WHILE => {
            let saved = chk.loop_depth;
            chk.loop_depth = 0;
            let _ = ck_ex(src, mem, chk, host, n.d, TY_BOOL);
            chk.loop_depth = saved;
            chk.loop_depth += 1;
            let sb = chk.loop_broke;
            chk.loop_broke = false;
            let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);
            chk.loop_broke = sb;
            chk.loop_depth -= 1;
        }
        N_LOOP => {
            chk.loop_depth += 1;
            let sb = chk.loop_broke;
            chk.loop_broke = false;
            let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);

            chk.ty[i] = if chk.loop_broke { TY_UNIT } else { TY_NEVER };
            chk.loop_broke = sb;
            chk.loop_depth -= 1;
        }
        N_FOR => {
            let saved = chk.loop_depth;
            chk.loop_depth = 0;
            let lo = ck_expr(src, mem, chk, host, n.b, TY_ANY);
            if lo == TY_ERR {
                chk.loop_depth = saved;
                return;
            }
            let hi = ck_expr(src, mem, chk, host, n.c, if lo == TY_INTLIT { TY_ANY } else { lo });
            chk.loop_depth = saved;
            if hi == TY_ERR {
                return;
            }
            let vt = if lo == TY_INTLIT && hi == TY_INTLIT {
                finalize_int(src, mem, chk, n.b, TY_I32);
                finalize_int(src, mem, chk, n.c, TY_I32);
                TY_I32
            } else if lo == TY_INTLIT && ty_is_int(hi) {
                finalize_int(src, mem, chk, n.b, hi);
                hi
            } else if hi == TY_INTLIT && ty_is_int(lo) {
                finalize_int(src, mem, chk, n.c, lo);
                lo
            } else if lo == hi && ty_is_int(lo) {
                lo
            } else {
                ndiag(mem, E_TYPE_MISMATCH, n, lo as u32, hi as u32);
                return;
            };
            scope_push(chk);
            let slot = local_add(src, mem, chk, n.a, vt, 0, n);
            chk.res[i] = slot;
            chk.ty[i] = vt;
            chk.loop_depth += 1;
            let sb = chk.loop_broke;
            chk.loop_broke = false;
            let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);
            chk.loop_broke = sb;
            chk.loop_depth -= 1;
            scope_pop(chk);
        }
        N_BREAK | N_CONTINUE => {
            if chk.loop_depth == 0 {
                ndiag(mem, E_BREAK_OUTSIDE_LOOP, n, 0, 0);
            } else if n.kind == N_BREAK {
                chk.loop_broke = true;
            }
        }
        _ => {}
    }
}

fn ck_place(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32,
            root_mut: &mut bool) -> u16 {
    let n = mem.node(node);
    let i = node as usize;
    let t = match n.kind {
        N_NAME => {
            let li = local_find(src, mem, chk, n.a);
            if li == usize::MAX {
                if const_find(src, mem, chk, n.a) != usize::MAX {
                    ndiag(mem, E_ASSIGN_NOT_PLACE, n, 0, 0);
                } else {
                    ndiag(mem, E_UNDEFINED, n, 0, 0);
                }
                return TY_ERR;
            }
            *root_mut = chk.locals[li].flags & FLAG_MUT != 0;
            chk.res[i] = chk.locals[li].slot;

            chk.locals[li].ty
        }
        N_DOT => {
            let bt0 = ck_place(src, mem, chk, host, n.d, root_mut);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                *root_mut = chk.rinfo(bt0).mutable != 0;
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            if !ty_is_struct(bt) {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, bt as u32);
                return TY_ERR;
            }
            let mut off = 0u32;
            let mut fty = TY_ERR;
            if !field_lookup(src, mem, chk, host, bt, n.a, &mut off, &mut fty) {
                ndiag(mem, E_UNKNOWN_FIELD, n, 0, 0);
                return TY_ERR;
            }
            chk.res[i] = off | if deref { RES_DEREF } else { 0 };
            fty
        }
        N_INDEX => {
            let bt0 = ck_place(src, mem, chk, host, n.d, root_mut);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                *root_mut = chk.rinfo(bt0).mutable != 0;
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            let elem = if ty_is_arr(bt) {
                chk.ainfo(bt).elem
            } else if ty_is_slice(bt) {
                *root_mut = chk.slinfo(bt).mutable != 0;
                chk.slinfo(bt).pointee
            } else {
                ndiag(mem, E_NOT_AN_ARRAY, n, 0, bt0 as u32);
                return TY_ERR;
            };
            let _ = ck_ex(src, mem, chk, host, n.e, TY_USIZE);
            chk.res[i] = if deref { RES_DEREF } else { 0 };
            elem
        }
        N_DEREF => {

            let rt = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            if rt == TY_ERR {
                return TY_ERR;
            }
            if !ty_is_ref(rt) {
                ndiag(mem, E_DEREF_NON_REF, n, 0, 0);
                return TY_ERR;
            }
            let ri = chk.rinfo(rt);

            *root_mut = ri.mutable != 0;
            ri.pointee
        }
        _ => {
            ndiag(mem, E_ASSIGN_NOT_PLACE, n, 0, 0);
            return TY_ERR;
        }
    };
    if i < CAP_NODES {
        chk.ty[i] = t;
    }
    t
}

fn field_lookup(src: &[u8], mem: &mut Mem, chk: &mut Chk, host: &HostDef, st: u16, name_tok: u32,
                off: &mut u32, fty: &mut u16) -> bool {
    let info = chk.sinfo(st);
    if info.host > 0 {
        let hs = &host.structs[(info.host - 1) as usize];
        let w = tok_bytes(src, mem.tok(name_tok));
        let mut o = 0u32;
        let mut f = 0;
        while f < hs.field_n {
            if bytes_eq(w, host_name(hs.fields[f].name)) {
                *off = o;
                *fty = host_ty(src, mem, chk, host, &hs.fields[f].ty, NODE_NONE);
                return true;
            }
            o += host_ty_size(&hs.fields[f].ty);
            f += 1;
        }
        return false;
    }
    let mut f = info.first_field;
    while f != NODE_NIL {
        let fnode = mem.node(f);
        if tok_eq(src, mem, fnode.a, name_tok) {
            *off = chk.res[f as usize];
            *fty = chk.ty[f as usize];
            return true;
        }
        f = fnode.link;
    }
    false
}

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

const IMG_MAGIC: u64 = 0x0000_5352_3169_0001;
const KNIL: u64 = 18446744073709551615;

const KL: u64 = 1;
const KN: u64 = 2;
const KB: u64 = 3;
const KU: u64 = 4;
const KI: u64 = 5;
const KK: u64 = 6;
const KE: u64 = 7;
const KT: u64 = 8;
const KA: u64 = 9;
const KW: u64 = 10;
const KO: u64 = 11;
const KR: u64 = 12;
const KC: u64 = 13;
const KF: u64 = 14;
const KH: u64 = 15;
const KRET: u64 = 42;
const KF64: u64 = 44;
const KFNEG: u64 = 45;

const H_TY_U64: HostTy = HostTy { kind: TY_U64, sname: 0, elem: 0, len: 0 };
const H_TY_UNIT: HostTy = HostTy { kind: TY_UNIT, sname: 0, elem: 0, len: 0 };
const BOOT: HostDef = HostDef {
    structs: [H_STRUCT0; HCAP_HSTRUCTS],
    struct_n: 0,
    fns: [
        HostFnDef { name: 0, params: [H_TY_U64], param_n: 1, ret: H_TY_U64 },
        HostFnDef { name: 1, params: [H_TY_U64], param_n: 2, ret: H_TY_UNIT },
        HostFnDef { name: 2, params: [H_TY0], param_n: 0, ret: H_TY_U64 },
        HostFnDef { name: 3, params: [H_TY_U64], param_n: 1, ret: H_TY_UNIT },
    ],
    fn_n: 4,
};

const ECAP_NODES: usize = 96;
const ECAP_VALS: usize = 48;
#[derive(Clone, Copy)]
struct Emit {
    en: [u64; ECAP_NODES * 7],
    en_n: usize,
    ev: [u64; ECAP_VALS],
    ev_n: usize,
    err: bool,
}

fn e_push(e: &mut Emit, kind: u64, x: u64, a: u64, b: u64, c: u64, d: u64) -> u64 {
    let base = e.en_n * 7;
    e.en[base] = kind;
    e.en[base + 1] = x;
    e.en[base + 2] = a;
    e.en[base + 3] = b;
    e.en[base + 4] = c;
    e.en[base + 5] = d;
    e.en[base + 6] = KNIL;
    e.en_n += 1;
    (e.en_n - 1) as u64
}

fn e_val(e: &mut Emit, v: u64) -> u64 {
    let mut i = 0;
    while i < e.ev_n {
        if e.ev[i] == v {
            return i as u64;
        }
        i += 1;
    }
    e.ev[e.ev_n] = v;
    e.ev_n += 1;
    (e.ev_n - 1) as u64
}

fn e_size_at(chk: &Chk, idx: u32) -> u64 {
    chk.size_of(chk.ty[idx as usize]) as u64
}

fn e_node(m: &Mem, chk: &Chk, e: &mut Emit, idx: u32) -> u64 {
    if e.err {
        return KNIL;
    }
    let n = m.node(idx);
    let k = n.kind;
    if k == N_LIT_INT {
        let v = chk.vals[chk.res[idx as usize] as usize];
        let vi = e_val(e, v);
        return e_push(e, KL, 0, vi, 0, 0, 0);
    }
    if k == N_LIT_BOOL {
        let vi = e_val(e, n.x as u64);
        return e_push(e, KL, 0, vi, 0, 0, 0);
    }
    if k == N_NAME {
        if (chk.res[idx as usize] & RES_CONST) != 0 {
            let ci = (chk.res[idx as usize] & RES_MASK) as usize;
            let t = chk.ty[idx as usize];
            if ty_is_struct(t) || ty_is_arr(t) {

                e.err = true;
                return KNIL;
            }
            let v = chk.consts[ci].bits;
            let vi = e_val(e, v);
            return e_push(e, KL, 0, vi, 0, 0, 0);
        }
        let sz = e_size_at(chk, idx);
        return e_push(e, KN, 0, chk.res[idx as usize] as u64, sz, 0, 0);
    }
    if k == N_UNARY {
        if chk.res[idx as usize] == 1 {

            return e_node(m, chk, e, n.e);
        }
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        if n.x == OP_NEG {
            if chk.ty[idx as usize] == TY_F64 {
                return e_push(e, KFNEG, 0, 0, 0, c, 0);
            }

            e.err = true;
            return KNIL;
        }
        return e_push(e, KU, n.x as u64, 0, 0, c, 0);
    }
    if k == N_BINARY {
        let ot = chk.ty[n.d as usize];
        let c = e_node(m, chk, e, n.d);
        let d = e_node(m, chk, e, n.e);
        if c == KNIL || d == KNIL {
            return KNIL;
        }
        if ot == TY_F64 {
            return e_push(e, KF64, n.x as u64, 0, 0, c, d);
        }
        return e_push(e, KB, n.x as u64, 0, 0, c, d);
    }
    if k == N_IF {
        let c = e_node(m, chk, e, n.d);
        let d = e_node(m, chk, e, n.e);
        let a = if n.b == NODE_NIL { KNIL } else { e_node(m, chk, e, n.b) };
        if c == KNIL || d == KNIL {
            return KNIL;
        }
        return e_push(e, KI, 0, a, 0, c, d);
    }
    if k == N_BLOCK {
        let a = e_chain(m, chk, e, n.b);
        let b = if n.e == NODE_NIL { KNIL } else { e_node(m, chk, e, n.e) };
        return e_push(e, KK, 0, a, b, 0, 0);
    }
    if k == N_EXPR_STMT {
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        return e_push(e, KE, 0, 0, 0, c, 0);
    }
    if k == N_LET {
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        let sz = e_size_at(chk, idx);
        return e_push(e, KT, 0, chk.res[idx as usize] as u64, sz, c, 0);
    }
    if k == N_ASSIGN {
        let place = m.node(n.d);
        if place.kind == N_NAME {
            let slot = chk.res[n.d as usize] as u64;
            let sz = e_size_at(chk, n.d);
            let c = e_node(m, chk, e, n.e);
            if c == KNIL {
                return KNIL;
            }
            return e_push(e, KA, n.x as u64, slot, sz, c, 0);
        }

        e.err = true;
        return KNIL;
    }
    if k == N_WHILE {
        let c = e_node(m, chk, e, n.d);
        let d = e_node(m, chk, e, n.e);
        if c == KNIL || d == KNIL {
            return KNIL;
        }
        return e_push(e, KW, 0, 0, 0, c, d);
    }
    if k == N_LOOP {
        let d = e_node(m, chk, e, n.e);
        if d == KNIL {
            return KNIL;
        }
        return e_push(e, KO, 0, 0, 0, 0, d);
    }
    if k == N_BREAK {
        return e_push(e, KR, 0, 0, 0, 0, 0);
    }
    if k == N_CONTINUE {
        return e_push(e, KC, 0, 0, 0, 0, 0);
    }
    if k == N_RETURN {
        if n.e == NODE_NIL {
            return e_push(e, KRET, 0, 0, 0, KNIL, 0);
        }
        let sz = e_size_at(chk, n.e);
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        return e_push(e, KRET, 0, 0, sz, c, 0);
    }
    if k == N_CALL {
        let args = e_chain(m, chk, e, n.b);
        let count = n.c as u64;
        if (chk.res[idx as usize] & RES_HOST) != 0 {
            let host = (chk.res[idx as usize] & RES_MASK) as u64;

            if host == 3 || host == 2 {
                return e_push(e, KH, host, 0, args, count, 0);
            }
            e.err = true;
            return KNIL;
        }
        let f = (chk.res[idx as usize] & RES_MASK) as u64;
        return e_push(e, KF, 0, f, args, count, 0);
    }

    e.err = true;
    KNIL
}

fn e_chain(m: &Mem, chk: &Chk, e: &mut Emit, first: u32) -> u64 {
    let mut it = first;
    let mut head = KNIL;
    let mut prev = KNIL;
    while it != NODE_NIL {
        let en = e_node(m, chk, e, it);
        if en == KNIL {
            return KNIL;
        }
        if head == KNIL {
            head = en;
        } else {
            e.en[(prev as usize) * 7 + 6] = en;
        }
        prev = en;
        it = m.node(it).link;
    }
    head
}

fn wu64(v: u64) {
    let mut j = 0;
    while j < 8 {
        putb((v >> (j * 8)) & 255);
        j += 1;
    }
}

fn fn_is_main(src: &[u8], m: &Mem, name_tok: u32) -> bool {
    let t = m.tok(name_tok);
    if t.len != 4 {
        return false;
    }
    let lo = t.pos as usize;
    src[lo] == 109 && src[lo + 1] == 97 && src[lo + 2] == 105 && src[lo + 3] == 110
}

fn emit_image(src: &[u8], m: &Mem, chk: &Chk) {
    let mut e = Emit {
        en: [0; ECAP_NODES * 7], en_n: 0,
        ev: [0; ECAP_VALS], ev_n: 0,
        err: false,
    };
    let mut fbody = [0u64; CAP_FNS];
    let mut entry = KNIL;
    let mut i = 0;
    while i < chk.fn_n {
        let fnode = m.node(chk.fns[i].node);
        let body = e_node(m, chk, &mut e, fnode.e);
        fbody[i] = body;
        if fn_is_main(src, m, chk.fns[i].name_tok) {
            entry = i as u64;
        }
        i += 1;
    }
    if e.err {
        return;
    }
    if entry == KNIL {
        return;
    }
    wu64(IMG_MAGIC);
    wu64(chk.fn_n as u64);
    wu64(entry);
    wu64(e.en_n as u64);
    wu64(e.ev_n as u64);
    wu64(chk.pool_n as u64);
    wu64(chk.str_n as u64);
    let mut i = 0;
    while i < chk.fn_n {
        wu64(fbody[i]);
        wu64(chk.fns[i].frame as u64);
        wu64(chk.fns[i].param_n as u64);
        i += 1;
    }
    let mut i = 0;
    while i < e.ev_n {
        wu64(e.ev[i]);
        i += 1;
    }
    let mut i = 0;
    while i < e.en_n * 7 {
        wu64(e.en[i]);
        i += 1;
    }
    let mut wi: usize = 0;
    while wi < chk.pool_n {
        let mut word = 0u64;
        let mut b: usize = 0;
        while b < 8 && wi + b < chk.pool_n {
            word |= (chk.str_pool[wi + b] as u64) << (b * 8);
            b += 1;
        }
        wu64(word);
        wi += 8;
    }
    let mut i = 0;
    while i < chk.str_n {
        wu64(chk.strs[i].off as u64);
        wu64(chk.strs[i].len as u64);
        i += 1;
    }
}

fn main() {

    let mut buf = [0u8; 128];
    let mut n: usize = 0;
    loop {
        let c = getb();
        if c == 18446744073709551615 {
            break;
        }
        if n < 128 {
            buf[n] = c as u8;
            n = n + 1;
        }
    }
    let src = &buf[0..n];
    let mut m = Mem {
        toks: [TOK_NONE; CAP_TOKS], tok_n: 0,
        nodes: [NODE_NONE; CAP_NODES], node_n: 0,
        diags: [Diag { code: 0, lo: 0, hi: 0, a: 0, b: 0 }; CAP_DIAGS], diag_n: 0,
        diag_lost: 0, overflow: false, root_first: NODE_NIL, root_n: 0,
    };
    lex(src, &mut m);
    parse(src, &mut m);
    let mut chk = CHK_INIT;
    let host = BOOT;
    let ok = check(src, &mut m, &mut chk, &host);
    if ok {
        emit_image(src, &m, &chk);
    }
}
