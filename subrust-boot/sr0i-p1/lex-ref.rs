// Reference for lex.P1pp: a faithful Rust port of sr0i.c's lex(). Same
// tokenization; emits one 4-word record [kind, ival, pos, len] (u64 LE) per
// token, T_EOF included. Diffing this against the P1pp lexer's dump is
// diverse double-execution of the SR-seed tokenizer.
use std::io::{Read, Write};

// token kinds (must match sr0i.c)
const T_EOF: u64 = 0; const T_IDENT: u64 = 1; const T_INT: u64 = 2;
const T_KW_FN: u64 = 3; const T_KW_LET: u64 = 4; const T_KW_MUT: u64 = 5;
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
const T_KW_CONST: u64 = 52;

fn is_digit(c: u8) -> bool { c >= b'0' && c <= b'9' }
fn is_alpha(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' }
fn is_alnum(c: u8) -> bool { is_alpha(c) || is_digit(c) }

fn kw_kind(s: &[u8]) -> u64 {
    match s {
        b"fn" => T_KW_FN, b"let" => T_KW_LET, b"mut" => T_KW_MUT, b"if" => T_KW_IF,
        b"else" => T_KW_ELSE, b"while" => T_KW_WHILE, b"loop" => T_KW_LOOP,
        b"break" => T_KW_BREAK, b"continue" => T_KW_CONTINUE, b"const" => T_KW_CONST,
        b"true" => T_KW_TRUE, b"false" => T_KW_FALSE, _ => T_IDENT,
    }
}

fn main() {
    let mut src = Vec::new();
    std::io::stdin().read_to_end(&mut src).unwrap();
    let n = src.len();
    let mut out: Vec<u8> = Vec::new();
    let mut emit = |k: u64, v: u64, p: usize, l: usize| {
        out.extend_from_slice(&k.to_le_bytes());
        out.extend_from_slice(&v.to_le_bytes());
        out.extend_from_slice(&(p as u64).to_le_bytes());
        out.extend_from_slice(&(l as u64).to_le_bytes());
    };
    let mut i = 0usize;
    while i < n {
        let c = src[i];
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' {
            i += 1;
        } else if c == b'/' && i + 1 < n && src[i + 1] == b'/' {
            while i < n && src[i] != b'\n' { i += 1; }
        } else if is_alpha(c) {
            let start = i;
            while i < n && is_alnum(src[i]) { i += 1; }
            emit(kw_kind(&src[start..i]), 0, start, i - start);
        } else if is_digit(c) {
            let mut v: u64 = 0;
            let start = i;
            if c == b'0' && i + 1 < n && src[i + 1] == b'x' {
                i += 2;
                while i < n {
                    let h = src[i];
                    if is_digit(h) { v = v.wrapping_mul(16).wrapping_add((h - b'0') as u64); }
                    else if h >= b'a' && h <= b'f' { v = v.wrapping_mul(16).wrapping_add((h - b'a' + 10) as u64); }
                    else if h >= b'A' && h <= b'F' { v = v.wrapping_mul(16).wrapping_add((h - b'A' + 10) as u64); }
                    else if h == b'_' { }
                    else { break; }
                    i += 1;
                }
            } else {
                while i < n && (is_digit(src[i]) || src[i] == b'_') {
                    if src[i] != b'_' { v = v.wrapping_mul(10).wrapping_add((src[i] - b'0') as u64); }
                    i += 1;
                }
            }
            while i < n && is_alnum(src[i]) { i += 1; }
            emit(T_INT, v, start, i - start);
        } else {
            let start = i;
            let c1 = if i + 1 < n { src[i + 1] } else { 0 };
            if c == b'-' && c1 == b'>' { emit(T_ARROW, 0, start, 2); i += 2; }
            else if c == b'=' && c1 == b'=' { emit(T_EE, 0, start, 2); i += 2; }
            else if c == b'!' && c1 == b'=' { emit(T_NE, 0, start, 2); i += 2; }
            else if c == b'<' && c1 == b'=' { emit(T_LE, 0, start, 2); i += 2; }
            else if c == b'>' && c1 == b'=' { emit(T_GE, 0, start, 2); i += 2; }
            else if c == b'<' && c1 == b'<' {
                if i + 2 < n && src[i + 2] == b'=' { emit(T_SHLEQ, 0, start, 3); i += 3; }
                else { emit(T_SHL, 0, start, 2); i += 2; }
            }
            else if c == b'>' && c1 == b'>' {
                if i + 2 < n && src[i + 2] == b'=' { emit(T_SHREQ, 0, start, 3); i += 3; }
                else { emit(T_SHR, 0, start, 2); i += 2; }
            }
            else if c == b'&' && c1 == b'&' { emit(T_AMPAMP, 0, start, 2); i += 2; }
            else if c == b'|' && c1 == b'|' { emit(T_PIPEPIPE, 0, start, 2); i += 2; }
            else if c == b'+' && c1 == b'=' { emit(T_PLUSEQ, 0, start, 2); i += 2; }
            else if c == b'-' && c1 == b'=' { emit(T_MINUSEQ, 0, start, 2); i += 2; }
            else if c == b'*' && c1 == b'=' { emit(T_STAREQ, 0, start, 2); i += 2; }
            else if c == b'/' && c1 == b'=' { emit(T_SLASHEQ, 0, start, 2); i += 2; }
            else if c == b'%' && c1 == b'=' { emit(T_PCTEQ, 0, start, 2); i += 2; }
            else if c == b'&' && c1 == b'=' { emit(T_AMPEQ, 0, start, 2); i += 2; }
            else if c == b'|' && c1 == b'=' { emit(T_PIPEEQ, 0, start, 2); i += 2; }
            else if c == b'^' && c1 == b'=' { emit(T_CARETEQ, 0, start, 2); i += 2; }
            else if c == b'(' { emit(T_LP, 0, start, 1); i += 1; }
            else if c == b')' { emit(T_RP, 0, start, 1); i += 1; }
            else if c == b'{' { emit(T_LB, 0, start, 1); i += 1; }
            else if c == b'}' { emit(T_RB, 0, start, 1); i += 1; }
            else if c == b',' { emit(T_COMMA, 0, start, 1); i += 1; }
            else if c == b';' { emit(T_SEMI, 0, start, 1); i += 1; }
            else if c == b':' { emit(T_COLON, 0, start, 1); i += 1; }
            else if c == b'=' { emit(T_EQ, 0, start, 1); i += 1; }
            else if c == b'+' { emit(T_PLUS, 0, start, 1); i += 1; }
            else if c == b'-' { emit(T_MINUS, 0, start, 1); i += 1; }
            else if c == b'*' { emit(T_STAR, 0, start, 1); i += 1; }
            else if c == b'/' { emit(T_SLASH, 0, start, 1); i += 1; }
            else if c == b'%' { emit(T_PCT, 0, start, 1); i += 1; }
            else if c == b'&' { emit(T_AMP, 0, start, 1); i += 1; }
            else if c == b'|' { emit(T_PIPE, 0, start, 1); i += 1; }
            else if c == b'^' { emit(T_CARET, 0, start, 1); i += 1; }
            else if c == b'<' { emit(T_LT, 0, start, 1); i += 1; }
            else if c == b'>' { emit(T_GT, 0, start, 1); i += 1; }
            else if c == b'!' { emit(T_BANG, 0, start, 1); i += 1; }
            else { std::process::exit(2); } // unexpected character -> sentinel exit
        }
    }
    emit(T_EOF, 0, n, 0);
    std::io::stdout().write_all(&out).unwrap();
}
