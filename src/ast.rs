
use crate::diag::*;
use crate::lex::{Tok, TOK_NONE};
use crate::{CAP_DIAGS, CAP_NODES, CAP_TOKS};

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

/// Fresh node with defaults; caller sets the fields it uses.
pub fn nd(kind: u16, lo: u32, hi: u32) -> Node {
    let mut n = NODE_NONE;
    n.kind = kind;
    n.lo = lo;
    n.hi = hi;
    n
}

/// All load-phase memory: token pool, node pool, diagnostics. The caller
/// owns it (a local, a static, or host-side heap) — the core never
/// allocates. ~0.4 MiB under profile-host.
pub struct Mem {
    pub toks: [Tok; CAP_TOKS],
    pub tok_n: usize,

    pub nodes: [Node; CAP_NODES],
    pub node_n: usize,

    pub diags: [Diag; CAP_DIAGS],
    pub diag_n: usize,
    pub diag_lost: u32,

    pub overflow: bool,

    pub root_first: u32,
    pub root_n: u32,
}

pub const MEM_INIT: Mem = Mem {
    toks: [TOK_NONE; CAP_TOKS],
    tok_n: 0,
    nodes: [NODE_NONE; CAP_NODES],
    node_n: 0,
    diags: [DIAG_NONE; CAP_DIAGS],
    diag_n: 0,
    diag_lost: 0,
    overflow: false,
    root_first: NODE_NIL,
    root_n: 0,
};

impl Mem {
    pub fn reset(&mut self) {
        self.tok_n = 0;
        self.node_n = 0;
        self.diag_n = 0;
        self.diag_lost = 0;
        self.overflow = false;
        self.root_first = NODE_NIL;
        self.root_n = 0;
    }

    pub fn diag(&mut self, code: u16, lo: u32, hi: u32, a: u32, b: u32) {
        if self.diag_n >= CAP_DIAGS {
            self.diag_lost += 1;
            return;
        }
        self.diags[self.diag_n] = Diag {
            code,
            span: Span { lo, hi },
            a,
            b,
        };
        self.diag_n += 1;
    }

    /// Push a token; reports overflow once and drops further tokens.
    pub fn push_tok(&mut self, t: Tok) {
        if self.tok_n >= CAP_TOKS {
            if !self.overflow {
                self.overflow = true;
                self.diag(E_TOO_MANY_TOKENS, t.pos, t.pos, 0, 0);
            }
            return;
        }
        self.toks[self.tok_n] = t;
        self.tok_n += 1;
    }

    pub fn tok(&self, i: u32) -> Tok {
        let i = i as usize;
        if i < self.tok_n {
            self.toks[i]
        } else {
            TOK_NONE
        }
    }

    /// Push a node; returns its index, or NODE_NIL on overflow (diagnosed once).
    pub fn push_node(&mut self, n: Node) -> u32 {
        if self.node_n >= CAP_NODES {
            if !self.overflow {
                self.overflow = true;
                self.diag(E_TOO_MANY_NODES, n.lo, n.hi, 0, 0);
            }
            return NODE_NIL;
        }
        self.nodes[self.node_n] = n;
        self.node_n += 1;
        (self.node_n - 1) as u32
    }

    pub fn node(&self, i: u32) -> Node {
        let i = i as usize;
        if i < self.node_n {
            self.nodes[i]
        } else {
            NODE_NONE
        }
    }

    pub fn set_link(&mut self, i: u32, link: u32) {
        let i = i as usize;
        if i < self.node_n {
            self.nodes[i].link = link;
        }
    }
}

/// Source text of a token (empty on any inconsistency — never panics).
pub fn tok_text(src: &str, t: Tok) -> &str {
    let lo = t.pos as usize;
    let hi = lo + t.len as usize;
    if lo <= hi && hi <= src.len() {
        &src[lo..hi]
    } else {
        ""
    }
}

/// Debug name of a node kind (for dumps and tests).
pub fn node_name(kind: u16) -> &'static str {
    match kind {
        N_FN => "Fn",
        N_STRUCT => "Struct",
        N_CONST => "Const",
        N_USE => "Use",
        N_PARAM => "Param",
        N_FIELD => "Field",
        N_USE_SEG => "UseSeg",
        N_TY_NAME => "TyName",
        N_TY_STR => "TyStr",
        N_TY_ARRAY => "TyArray",
        N_TY_UNIT => "TyUnit",
        N_TY_REF => "TyRef",
        N_TY_SLICE => "TySlice",
        N_LIT_INT => "Int",
        N_LIT_FLOAT => "Float",
        N_LIT_STR => "Str",
        N_LIT_BOOL => "Bool",
        N_LIT_UNIT => "Unit",
        N_NAME => "Name",
        N_CALL => "Call",
        N_DOT => "Dot",
        N_INDEX => "Index",
        N_UNARY => "Unary",
        N_BINARY => "Binary",
        N_CAST => "Cast",
        N_STRUCT_LIT => "StructLit",
        N_FIELD_INIT => "FieldInit",
        N_ARRAY_LIT => "ArrayLit",
        N_ARRAY_REPEAT => "ArrayRepeat",
        N_IF => "If",
        N_MATCH => "Match",
        N_ARM => "Arm",
        N_BLOCK => "Block",
        N_PAT_INT => "PatInt",
        N_PAT_STR => "PatStr",
        N_PAT_BOOL => "PatBool",
        N_PAT_WILD => "PatWild",
        N_PAT_CONST => "PatConst",
        N_LET => "Let",
        N_ASSIGN => "Assign",
        N_EXPR_STMT => "ExprStmt",
        N_WHILE => "While",
        N_LOOP => "Loop",
        N_FOR => "For",
        N_BREAK => "Break",
        N_CONTINUE => "Continue",
        N_PATHCONST => "PathConst",
        N_REFOF => "RefOf",
        N_DEREF => "Deref",
        N_METHOD => "Method",
        N_IMPL => "Impl",
        N_RETURN => "Return",
        N_ASSOC_CALL => "AssocCall",
        N_LIT_BSTR => "ByteStr",
        N_LIT_BYTE => "Byte",
        N_SLICE => "Slice",
        N_PAT_BYTE => "PatByte",
        N_TUPLE => "Tuple",
        N_ENUM => "Enum",
        N_VARIANT => "Variant",
        N_PAT_ENUM => "PatEnum",
        N_ASSERT => "Assert",
        _ => "?",
    }
}

/// Debug name of an operator (Node.x of N_UNARY / N_BINARY / N_ASSIGN).
pub fn op_name(op: u16) -> &'static str {
    match op {
        OP_NEG => "-",
        OP_NOT => "!",
        OP_ADD => "+",
        OP_SUB => "-",
        OP_MUL => "*",
        OP_DIV => "/",
        OP_REM => "%",
        OP_AND => "&&",
        OP_OR => "||",
        OP_BAND => "&",
        OP_BOR => "|",
        OP_BXOR => "^",
        OP_SHL => "<<",
        OP_SHR => ">>",
        OP_EQ => "==",
        OP_NE => "!=",
        OP_LT => "<",
        OP_LE => "<=",
        OP_GT => ">",
        OP_GE => ">=",
        _ => "?",
    }
}
