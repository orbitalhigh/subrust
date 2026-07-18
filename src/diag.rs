
#[derive(Clone, Copy)]
pub struct Span {
    pub lo: u32,
    pub hi: u32,
}

pub const SPAN_NONE: Span = Span { lo: 0, hi: 0 };

#[derive(Clone, Copy)]
pub struct Diag {
    pub code: u16,
    pub span: Span,

    pub a: u32,
    pub b: u32,
}

pub const DIAG_NONE: Diag = Diag { code: 0, span: SPAN_NONE, a: 0, b: 0 };

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

/// Static message for a code. Hosts with a renderer (subrust-cli) compose
/// richer text from the args; this is the fallback.
pub fn diag_text(code: u16) -> &'static str {
    match code {
        E_SRC_TOO_BIG => "source file too large",
        E_TOO_MANY_TOKENS => "program too large: token pool exhausted",
        E_TOO_MANY_NODES => "program too large: node pool exhausted",
        E_TOKEN_TOO_LONG => "token too long",

        E_UNEXPECTED_CHAR => "unexpected character",
        E_UNTERMINATED_STRING => "unterminated string literal",
        E_BAD_ESCAPE => "unsupported escape (subrust allows \\n \\r \\t \\0 \\\\ \\\" \\')",
        E_UNTERMINATED_COMMENT => "unterminated block comment",
        E_DOC_COMMENT => "doc comments are not in the subset; use regular `//` comments",
        E_BAD_NUMBER => "malformed number literal",
        E_BAD_SUFFIX => "unsupported literal suffix (i8 u8 i16 u16 i32 u32 i64 u64 isize usize f64)",
        E_CHAR_LITERAL => "character literals are not in subrust v0.1",
        E_NON_ASCII => "non-ASCII is only allowed inside string literals and comments",
        E_STR_TOO_LONG => "string literal too long",

        E_EXPECTED_TOKEN => "unexpected token",
        E_EXPECTED_ITEM => "expected an item: `fn`, `struct`, `const` or `use`",
        E_EXPECTED_EXPR => "expected an expression",
        E_EXPECTED_TYPE => "expected a type",
        E_EXPECTED_PATTERN => "expected a pattern: integer, string, `true`, `false` or `_`",
        E_RESERVED_KEYWORD => "reserved Rust keyword; not in the subrust subset",
        E_TOO_DEEP => "too deeply nested (static depth limit)",
        E_CHAINED_COMPARISON => "comparison operators cannot be chained; add parentheses",
        E_STRUCT_LIT_HERE => "struct literals are not allowed here; wrap in parentheses",
        E_RANGE_HERE => "range expressions are only allowed in `for` loop heads",
        E_METHOD_CALL => "method calls are not in subrust v0.1; use a free function",
        E_CALL_NOT_NAME => "only named functions can be called",
        E_BAD_ATTR => "the only supported attribute is `#[derive(...)]` on a struct",
        E_BAD_DERIVE => "unsupported derive (subrust v0.1: Clone, Copy)",
        E_TUPLE => "tuples are not in subrust v0.1",
        E_STRUCT_UPDATE => "struct update syntax (`..base`) is not in subrust v0.1",

        E_DUP_NAME => "this name is already defined",
        E_UNDEFINED => "undefined name",
        E_FN_AS_VALUE => "functions are not values in subrust v0.1; call it",
        E_UNKNOWN_FN => "unknown function",
        E_ARG_COUNT => "wrong number of arguments",
        E_TYPE_MISMATCH => "type mismatch",
        E_LIT_OUT_OF_RANGE => "literal out of range for its type",
        E_BAD_CAST => "unsupported cast",
        E_UNKNOWN_TYPE => "unknown type",
        E_NOT_A_STRUCT => "field access on a non-struct value",
        E_UNKNOWN_FIELD => "no such field on this struct",
        E_MISSING_FIELD => "struct literal is missing fields",
        E_DUP_FIELD => "field specified more than once",
        E_NOT_AN_ARRAY => "indexing a non-array value",
        E_ANNOTATION_NEEDED => "cannot infer the type here; add a type annotation",
        E_NOT_EXHAUSTIVE => "match is not exhaustive; add a `_` arm",
        E_PATTERN_TYPE => "pattern type does not match the value being matched",
        E_ASSIGN_NOT_PLACE => "not assignable; assign to a variable, field or element",
        E_ASSIGN_IMMUTABLE => "cannot assign: binding is not declared `mut`",
        E_BREAK_OUTSIDE_LOOP => "`break`/`continue` outside of a loop",
        E_CONST_CYCLE => "constant definition cycle",
        E_NOT_CONST => "expression is not constant-evaluable in subrust v0.1",
        E_CONST_TYPE => "constants must have a scalar type in v0.1 (int, f64, bool, &str)",
        E_MISSING_DERIVE => "structs and enums must carry #[derive(Clone, Copy)] in subrust v0.1",
        E_FRAME_TOO_BIG => "function needs too much stack space",
        E_TOO_MANY_ITEMS => "program too large: a checker table is full",
        E_NO_ELSE => "this `if` is used as a value, so it needs an `else`",
        E_NEG_UNSIGNED => "cannot negate an unsigned integer",
        E_USE_UNSUPPORTED => "`use` is not needed: the host API is already in scope",
        E_RECURSIVE_STRUCT => "recursive struct type",
        E_STR_FIELD => "&str cannot be stored in a struct in v0.1 (references)",
        E_BAD_OPERAND => "operator not supported for this type",
        E_CONST_OVERFLOW => "overflow or division by zero in constant evaluation",
        E_MISSING_TAIL => "function must end with a tail expression of its return type",
        E_BAD_PATH => "unknown associated constant (supported: <inttype>::MAX / ::MIN)",
        E_NOT_PLACE_REF => "can only take a reference to a variable, field or element",
        E_DEREF_NON_REF => "cannot dereference a non-reference value",
        E_REF_ESCAPES => "references may only be function parameters/arguments in this version (not stored, returned, or in aggregates)",
        E_REF_MUT_NEEDED => "cannot take `&mut` of an immutable place",
        E_UNKNOWN_METHOD => "no method with this name on the receiver's type",
        E_BAD_RECEIVER => "unsupported receiver: methods need `&self`/`&mut self` on a place or reference receiver in this version",
        E_SUBSLICE_REF => "a sub-slice must be borrowed: write `&base[lo..hi]` (or `&mut …`)",
        E_UNKNOWN_VARIANT => "no such variant on this enum",
        E_ENUM_PAYLOAD => "enum variant payloads are not yet in the subset (field-less enums only)",
        E_BAD_MACRO => "unknown macro; only `assert!` is recognized in the subset",
        E_ASSERT_MSG => "assert! message must be a plain string literal (no format arguments)",

        E_T_ARITH => "trap: arithmetic overflow or division by zero",
        E_T_ASSERT => "trap: assertion failed",
        E_T_OOB => "trap: index out of bounds",
        E_T_FUEL => "trap: fuel exhausted",
        E_T_STACK => "trap: machine stack limit reached",
        E_T_HOST => "trap: host call failed",
        E_T_NO_ENTRY => "no such entry function",
        E_T_INTERNAL => "trap: machine internal error",

        _ => "unknown error",
    }
}
