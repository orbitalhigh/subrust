
use subrust::diag::*;
use subrust::lex::*;
use subrust::{Mem, MEM_INIT};

fn lex_new(src: &str) -> (Box<Mem>, bool) {
    let mut mem = Box::new(MEM_INIT);
    let ok = subrust::lex_source(src, &mut mem);
    (mem, ok)
}

fn kinds(src: &str) -> Vec<u16> {
    let (mem, ok) = lex_new(src);
    assert!(ok, "unexpected lex error for {src:?}: code {:#06x}", first(&mem));
    let mut v = Vec::new();
    for i in 0..mem.tok_n {
        v.push(mem.toks[i].kind);
    }
    assert_eq!(*v.last().unwrap(), T_EOF);
    v.pop();
    v
}

fn first(mem: &Mem) -> u16 {
    if mem.diag_n > 0 {
        mem.diags[0].code
    } else {
        0
    }
}

fn err(src: &str) -> u16 {
    let (mem, ok) = lex_new(src);
    assert!(!ok, "expected a lex error for {src:?}");
    first(&mem)
}

#[test]
fn basic_stream() {
    assert_eq!(
        kinds("fn main() { let x = 1 + 2.5; }"),
        vec![
            T_KW_FN, T_IDENT, T_LPAREN, T_RPAREN, T_LBRACE, T_KW_LET, T_IDENT, T_EQ, T_INT,
            T_PLUS, T_FLOAT, T_SEMI, T_RBRACE
        ]
    );
}

#[test]
fn operators_maximal_munch() {
    assert_eq!(
        kinds(":: . .. ..= -> => = == != < <= << <<= > >= >> >>="),
        vec![
            T_COLONCOLON, T_DOT, T_DOTDOT, T_DOTDOTEQ, T_ARROW, T_FATARROW, T_EQ, T_EQEQ, T_NE,
            T_LT, T_LE, T_SHL, T_SHLEQ, T_GT, T_GE, T_SHR, T_SHREQ
        ]
    );
    assert_eq!(
        kinds("+ += - -= * *= / /= % %= & && &= | || |= ^ ^= !"),
        vec![
            T_PLUS, T_PLUSEQ, T_MINUS, T_MINUSEQ, T_STAR, T_STAREQ, T_SLASH, T_SLASHEQ,
            T_PERCENT, T_PERCENTEQ, T_AMP, T_AMPAMP, T_AMPEQ, T_PIPE, T_PIPEPIPE, T_PIPEEQ,
            T_CARET, T_CARETEQ, T_BANG
        ]
    );
}

#[test]
fn keywords_and_reserved() {
    assert_eq!(
        kinds("fn let mut if else while loop for in break continue match struct const use as true false"),
        vec![
            T_KW_FN, T_KW_LET, T_KW_MUT, T_KW_IF, T_KW_ELSE, T_KW_WHILE, T_KW_LOOP, T_KW_FOR,
            T_KW_IN, T_KW_BREAK, T_KW_CONTINUE, T_KW_MATCH, T_KW_STRUCT, T_KW_CONST, T_KW_USE,
            T_KW_AS, T_KW_TRUE, T_KW_FALSE
        ]
    );

    assert_eq!(kinds("impl"), vec![T_KW_IMPL]);
    assert_eq!(kinds("self"), vec![T_KW_SELF]);
    assert_eq!(kinds("return"), vec![T_KW_RETURN]);
    assert_eq!(kinds("enum"), vec![T_KW_ENUM]);

    assert_eq!(kinds("Self"), vec![T_KW_RESERVED]);

    assert_eq!(kinds("union"), vec![T_IDENT]);

    assert_eq!(kinds("_ _x x_"), vec![T_UNDERSCORE, T_IDENT, T_IDENT]);
}

#[test]
fn numbers() {
    assert_eq!(
        kinds("0x1F 1_000 1e3 2.5 2.5e-2 1f64 5usize 3i64 0xEEi64"),
        vec![
            T_INT, T_INT, T_FLOAT, T_FLOAT, T_FLOAT, T_FLOAT, T_INT, T_INT, T_INT
        ]
    );

    assert_eq!(kinds("1."), vec![T_INT, T_DOT]);
    assert_eq!(kinds("1..4"), vec![T_INT, T_DOTDOT, T_INT]);
    assert_eq!(kinds("x.0"), vec![T_IDENT, T_DOT, T_INT]);

    assert_eq!(kinds("0x1f64"), vec![T_INT]);

    assert_eq!(err("0x"), E_BAD_NUMBER);
    assert_eq!(err("0x1.2"), E_BAD_NUMBER);
    assert_eq!(kinds("1i16"), vec![T_INT]);
    assert_eq!(err("1f32"), E_BAD_SUFFIX);
    assert_eq!(err("1.5f32"), E_BAD_SUFFIX);
    assert_eq!(err("1em"), E_BAD_SUFFIX);
    assert_eq!(err("0b1"), E_BAD_SUFFIX);
}

#[test]
fn strings() {
    assert_eq!(kinds("\"hi\""), vec![T_STR]);

    assert_eq!(kinds("b\"hi\""), vec![T_BSTR]);
    assert_eq!(kinds("b\"a\\nb\""), vec![T_BSTR]);
    assert_eq!(kinds("b b1 byte"), vec![T_IDENT, T_IDENT, T_IDENT]);
    assert_eq!(kinds("b\"\""), vec![T_BSTR]);

    assert_eq!(kinds("b'a'"), vec![T_BYTE]);
    assert_eq!(kinds("b'\\n'"), vec![T_BYTE]);
    assert_eq!(kinds("b'0' b'_'"), vec![T_BYTE, T_BYTE]);
    assert_eq!(err("b'ab'"), E_CHAR_LITERAL);
    assert_eq!(err("'a'"), E_CHAR_LITERAL);
    assert_eq!(kinds("\"a\\n\\t\\r\\0\\\\\\\"\\'\""), vec![T_STR]);
    assert_eq!(kinds("\"multi\nline\""), vec![T_STR]);
    assert_eq!(kinds("\"héllo→\""), vec![T_STR]);
    assert_eq!(err("\"unterminated"), E_UNTERMINATED_STRING);
    assert_eq!(err("\"bad \\q escape\""), E_BAD_ESCAPE);
    assert_eq!(err("\"hex \\x41\""), E_BAD_ESCAPE);
}

#[test]
fn comments() {
    assert_eq!(kinds("// line\n1"), vec![T_INT]);
    assert_eq!(kinds("//// four slashes is a plain comment\n1"), vec![T_INT]);
    assert_eq!(kinds("/* block /* nested */ still */ 1"), vec![T_INT]);
    assert_eq!(kinds("/**/ 1"), vec![T_INT]);
    assert_eq!(kinds("/***/ 1"), vec![T_INT]);
    assert_eq!(err("/// doc\n1"), E_DOC_COMMENT);
    assert_eq!(err("//! inner doc\n1"), E_DOC_COMMENT);
    assert_eq!(err("/** doc */ 1"), E_DOC_COMMENT);
    assert_eq!(err("/*! inner */ 1"), E_DOC_COMMENT);
    assert_eq!(err("/* unterminated"), E_UNTERMINATED_COMMENT);
}

#[test]
fn rejected_lexemes() {
    assert_eq!(err("'a'"), E_CHAR_LITERAL);
    assert_eq!(err("let é = 3;"), E_NON_ASCII);
    assert_eq!(err("a @ b"), E_UNEXPECTED_CHAR);
    assert_eq!(err("a ? b"), E_UNEXPECTED_CHAR);
}

#[test]
fn spans_are_exact() {
    let (mem, ok) = lex_new("let abc = 42;");
    assert!(ok);

    assert_eq!(mem.toks[1].pos, 4);
    assert_eq!(mem.toks[1].len, 3);

    assert_eq!(mem.toks[3].pos, 10);
    assert_eq!(mem.toks[3].len, 2);
}
