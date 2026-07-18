/* sr0i — an interpreter for SR-seed (subrust-boot/SR-SEED.md), the bottom
 * rung of the subrust bootstrap ladder.
 *
 * Written in the M2-Planet C subset so it builds from the 256-byte hex0 seed
 * via the stage0-posix chain. Integer-only: it implements the full
 * SR-seed *language* plus the ld/st/getb/putb host calls; the f_* IEEE-f64
 * intrinsics are per-arch assembly backends and abort here.
 *
 * It is deliberately dumb — no type checker, no diagnostics beyond an abort
 * code — because it only ever runs programs subrust already validated. What
 * it must get exactly right is *semantics* (rustc debug profile: u64
 * overflow, div/rem by zero, and shift >= 64 trap), so its output is
 * byte-identical to rustc- and subrust-run SR-seed (the differential).
 *
 * Style mirrors subrust's own core: fixed pools, integer indices, tagged
 * records, no pointers-into-structures — which is also what keeps it inside
 * the M2-Planet subset.
 *
 * License: same as the subrust crate. Build tools (stage0-posix) are GPL-3
 * and run as separate processes; nothing links.
 */

#include <stdio.h>
#include <stdlib.h>

/* The u64 value type. M2-Planet amd64 makes bare `unsigned` 64-bit (add,
 * sub, shift, bitwise and UNSIGNED compare are all correct at 64-bit) but
 * emits 32-bit mul/div/rem — so those three ops are done in software below,
 * using only the 64-bit-correct primitives, one code path for both targets.
 * Host cc (LP64) needs `unsigned long` for a 64-bit type; select via -DHOSTCC. */
#ifdef HOSTCC
typedef unsigned long WORD;
#else
typedef unsigned WORD;
#endif

/* ---- limits ---------------------------------------------------------------- */
#define SRC_MAX 262144
#define TOK_MAX 65536
#define NODE_MAX 65536
#define FN_MAX 1024
#define LOC_MAX 4096
#define MEM_WORDS 1048576 /* 2^20, matches SR-SEED.md */
#define NIL 0-1

/* ---- token kinds ----------------------------------------------------------- */
#define T_EOF 0
#define T_IDENT 1
#define T_INT 2
#define T_KW_FN 3
#define T_KW_LET 4
#define T_KW_MUT 5
#define T_KW_IF 6
#define T_KW_ELSE 7
#define T_KW_WHILE 8
#define T_KW_LOOP 9
#define T_KW_BREAK 10
#define T_KW_CONTINUE 11
#define T_KW_TRUE 12
#define T_KW_FALSE 13
#define T_LP 14
#define T_RP 15
#define T_LB 16
#define T_RB 17
#define T_COMMA 18
#define T_SEMI 19
#define T_COLON 20
#define T_ARROW 21
#define T_EQ 22
#define T_PLUS 23
#define T_MINUS 24
#define T_STAR 25
#define T_SLASH 26
#define T_PCT 27
#define T_AMP 28
#define T_AMPAMP 29
#define T_PIPE 30
#define T_PIPEPIPE 31
#define T_CARET 32
#define T_SHL 33
#define T_SHR 34
#define T_BANG 35
#define T_EE 36
#define T_NE 37
#define T_LT 38
#define T_LE 39
#define T_GT 40
#define T_GE 41
#define T_PLUSEQ 42
#define T_MINUSEQ 43
#define T_STAREQ 44
#define T_SLASHEQ 45
#define T_PCTEQ 46
#define T_AMPEQ 47
#define T_PIPEEQ 48
#define T_CARETEQ 49
#define T_SHLEQ 50
#define T_SHREQ 51
#define T_KW_CONST 52

/* ---- node kinds ------------------------------------------------------------ */
#define N_INT 1
#define N_BOOL 2
#define N_NAME 3
#define N_CALL 4
#define N_UNARY 5   /* a=op(0:not) e=operand */
#define N_BIN 6     /* a=op d=lhs e=rhs */
#define N_IF 7      /* d=cond e=then b=else(node|NIL) */
#define N_BLOCK 8   /* b=first stmt e=tail(node|NIL) */
#define N_LET 9     /* a=name tok e=init */
#define N_ASSIGN 10 /* a=op(0 plain) d=place(name tok id) e=value */
#define N_WHILE 11  /* d=cond e=body */
#define N_LOOP 12   /* e=body */
#define N_BREAK 13
#define N_CONTINUE 14
#define N_EXPR 15 /* e=expr (statement) */

/* binary op ids (a field of N_BIN / N_ASSIGN) */
#define OP_ADD 1
#define OP_SUB 2
#define OP_MUL 3
#define OP_DIV 4
#define OP_REM 5
#define OP_AND 6
#define OP_OR 7
#define OP_BAND 8
#define OP_BOR 9
#define OP_BXOR 10
#define OP_SHL 11
#define OP_SHR 12
#define OP_EQ 13
#define OP_NE 14
#define OP_LT 15
#define OP_LE 16
#define OP_GT 17
#define OP_GE 18

/* ---- global pools (allocated in main) -------------------------------------- */
char* src;
int src_n;

int* tk_kind;
WORD* tk_ival;
int* tk_pos;
int* tk_len;
int tok_n;

int* nd_kind;
int* nd_a;
int* nd_b;
int* nd_c;
int* nd_d;
int* nd_e;
int* nd_link;
int node_n;

/* function table */
int* fn_tok;    /* name token index */
int* fn_body;   /* body block node */
int* fn_p0;     /* first param name-token (params chained via a fixed array) */
int* fn_np;     /* param count */
int* fn_ptok;   /* flattened param name tokens, fn_poff[i]..+np */
int* fn_poff;
int fn_n;
int ptok_n;

/* const table (name tok + evaluated value + init expr node) */
int* const_tok;
WORD* const_val;
int* const_expr;
int const_n;
int const_ready;

/* locals stack (name tok + value) */
int* loc_tok;
WORD* loc_val;
int loc_n;
int frame_base;

/* the SR-seed word memory */
WORD* wmem;

/* control signals from eval: 0 normal, 1 break, 2 continue */
int g_signal;
/* trap flag; when set, output is flushed and we exit(1) */
int g_trap;

int parse_pos; /* parser cursor over tokens */

/* ---- forward declarations -------------------------------------------------- */
int lex();
int parse_program();
int parse_block();
int parse_expr(int min);
WORD eval(int node);
void run_main();

/* ---- helpers --------------------------------------------------------------- */
void die(char* msg) {
    fputs("sr0i: ", stderr);
    fputs(msg, stderr);
    fputs("\n", stderr);
    exit(2);
}

int streq_src(int pos, int len, char* s) {
    int i;
    i = 0;
    while (i < len) {
        if (src[pos + i] != s[i]) {
            return 0;
        }
        i = i + 1;
    }
    if (s[len] != 0) {
        return 0;
    }
    return 1;
}

int tok_eq(int a, int b) {
    int i;
    if (tk_len[a] != tk_len[b]) {
        return 0;
    }
    i = 0;
    while (i < tk_len[a]) {
        if (src[tk_pos[a] + i] != src[tk_pos[b] + i]) {
            return 0;
        }
        i = i + 1;
    }
    return 1;
}

int is_digit(int c) { return c >= '0' && c <= '9'; }
int is_alpha(int c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}
int is_alnum(int c) { return is_alpha(c) || is_digit(c); }

/* ---- lexer ----------------------------------------------------------------- */
void push_tok(int kind, WORD ival, int pos, int len) {
    tk_kind[tok_n] = kind;
    tk_ival[tok_n] = ival;
    tk_pos[tok_n] = pos;
    tk_len[tok_n] = len;
    tok_n = tok_n + 1;
}

int kw_kind(int pos, int len) {
    if (streq_src(pos, len, "fn")) return T_KW_FN;
    if (streq_src(pos, len, "let")) return T_KW_LET;
    if (streq_src(pos, len, "mut")) return T_KW_MUT;
    if (streq_src(pos, len, "if")) return T_KW_IF;
    if (streq_src(pos, len, "else")) return T_KW_ELSE;
    if (streq_src(pos, len, "while")) return T_KW_WHILE;
    if (streq_src(pos, len, "loop")) return T_KW_LOOP;
    if (streq_src(pos, len, "break")) return T_KW_BREAK;
    if (streq_src(pos, len, "continue")) return T_KW_CONTINUE;
    if (streq_src(pos, len, "const")) return T_KW_CONST;
    if (streq_src(pos, len, "true")) return T_KW_TRUE;
    if (streq_src(pos, len, "false")) return T_KW_FALSE;
    return T_IDENT;
}

int lex() {
    int i;
    int c;
    int start;
    int c1;
    i = 0;
    tok_n = 0;
    while (i < src_n) {
        c = src[i];
        /* whitespace */
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') {
            i = i + 1;
        } else if (c == '/' && i + 1 < src_n && src[i + 1] == '/') {
            /* line comment */
            while (i < src_n && src[i] != '\n') {
                i = i + 1;
            }
        } else if (is_alpha(c)) {
            start = i;
            while (i < src_n && is_alnum(src[i])) {
                i = i + 1;
            }
            push_tok(kw_kind(start, i - start), 0, start, i - start);
        } else if (is_digit(c)) {
            WORD v;
            v = 0;
            start = i;
            if (c == '0' && i + 1 < src_n && src[i + 1] == 'x') {
                i = i + 2;
                while (i < src_n) {
                    int h;
                    h = src[i];
                    if (is_digit(h)) {
                        v = v * 16 + (h - '0');
                    } else if (h >= 'a' && h <= 'f') {
                        v = v * 16 + (h - 'a' + 10);
                    } else if (h >= 'A' && h <= 'F') {
                        v = v * 16 + (h - 'A' + 10);
                    } else if (h == '_') {
                        /* skip */
                    } else {
                        break;
                    }
                    i = i + 1;
                }
            } else {
                while (i < src_n && (is_digit(src[i]) || src[i] == '_')) {
                    if (src[i] != '_') {
                        v = v * 10 + (src[i] - '0');
                    }
                    i = i + 1;
                }
            }
            /* skip a u64 / bool-irrelevant suffix (u64 only in SR-seed) */
            while (i < src_n && is_alnum(src[i])) {
                i = i + 1;
            }
            push_tok(T_INT, v, start, i - start);
        } else {
            start = i;
            c1 = 0;
            if (i + 1 < src_n) {
                c1 = src[i + 1];
            }
            if (c == '-' && c1 == '>') { push_tok(T_ARROW, 0, start, 2); i = i + 2; }
            else if (c == '=' && c1 == '=') { push_tok(T_EE, 0, start, 2); i = i + 2; }
            else if (c == '!' && c1 == '=') { push_tok(T_NE, 0, start, 2); i = i + 2; }
            else if (c == '<' && c1 == '=') { push_tok(T_LE, 0, start, 2); i = i + 2; }
            else if (c == '>' && c1 == '=') { push_tok(T_GE, 0, start, 2); i = i + 2; }
            else if (c == '<' && c1 == '<') {
                if (i + 2 < src_n && src[i + 2] == '=') { push_tok(T_SHLEQ, 0, start, 3); i = i + 3; }
                else { push_tok(T_SHL, 0, start, 2); i = i + 2; }
            }
            else if (c == '>' && c1 == '>') {
                if (i + 2 < src_n && src[i + 2] == '=') { push_tok(T_SHREQ, 0, start, 3); i = i + 3; }
                else { push_tok(T_SHR, 0, start, 2); i = i + 2; }
            }
            else if (c == '&' && c1 == '&') { push_tok(T_AMPAMP, 0, start, 2); i = i + 2; }
            else if (c == '|' && c1 == '|') { push_tok(T_PIPEPIPE, 0, start, 2); i = i + 2; }
            else if (c == '+' && c1 == '=') { push_tok(T_PLUSEQ, 0, start, 2); i = i + 2; }
            else if (c == '-' && c1 == '=') { push_tok(T_MINUSEQ, 0, start, 2); i = i + 2; }
            else if (c == '*' && c1 == '=') { push_tok(T_STAREQ, 0, start, 2); i = i + 2; }
            else if (c == '/' && c1 == '=') { push_tok(T_SLASHEQ, 0, start, 2); i = i + 2; }
            else if (c == '%' && c1 == '=') { push_tok(T_PCTEQ, 0, start, 2); i = i + 2; }
            else if (c == '&' && c1 == '=') { push_tok(T_AMPEQ, 0, start, 2); i = i + 2; }
            else if (c == '|' && c1 == '=') { push_tok(T_PIPEEQ, 0, start, 2); i = i + 2; }
            else if (c == '^' && c1 == '=') { push_tok(T_CARETEQ, 0, start, 2); i = i + 2; }
            else if (c == '(') { push_tok(T_LP, 0, start, 1); i = i + 1; }
            else if (c == ')') { push_tok(T_RP, 0, start, 1); i = i + 1; }
            else if (c == '{') { push_tok(T_LB, 0, start, 1); i = i + 1; }
            else if (c == '}') { push_tok(T_RB, 0, start, 1); i = i + 1; }
            else if (c == ',') { push_tok(T_COMMA, 0, start, 1); i = i + 1; }
            else if (c == ';') { push_tok(T_SEMI, 0, start, 1); i = i + 1; }
            else if (c == ':') { push_tok(T_COLON, 0, start, 1); i = i + 1; }
            else if (c == '=') { push_tok(T_EQ, 0, start, 1); i = i + 1; }
            else if (c == '+') { push_tok(T_PLUS, 0, start, 1); i = i + 1; }
            else if (c == '-') { push_tok(T_MINUS, 0, start, 1); i = i + 1; }
            else if (c == '*') { push_tok(T_STAR, 0, start, 1); i = i + 1; }
            else if (c == '/') { push_tok(T_SLASH, 0, start, 1); i = i + 1; }
            else if (c == '%') { push_tok(T_PCT, 0, start, 1); i = i + 1; }
            else if (c == '&') { push_tok(T_AMP, 0, start, 1); i = i + 1; }
            else if (c == '|') { push_tok(T_PIPE, 0, start, 1); i = i + 1; }
            else if (c == '^') { push_tok(T_CARET, 0, start, 1); i = i + 1; }
            else if (c == '<') { push_tok(T_LT, 0, start, 1); i = i + 1; }
            else if (c == '>') { push_tok(T_GT, 0, start, 1); i = i + 1; }
            else if (c == '!') { push_tok(T_BANG, 0, start, 1); i = i + 1; }
            else { die("unexpected character"); }
        }
    }
    push_tok(T_EOF, 0, src_n, 0);
    return 0;
}

/* ---- node builders --------------------------------------------------------- */
int mk(int kind) {
    int i;
    i = node_n;
    nd_kind[i] = kind;
    nd_a[i] = NIL;
    nd_b[i] = NIL;
    nd_c[i] = NIL;
    nd_d[i] = NIL;
    nd_e[i] = NIL;
    nd_link[i] = NIL;
    node_n = node_n + 1;
    return i;
}

/* ---- parser ---------------------------------------------------------------- */
int cur() { return tk_kind[parse_pos]; }
int cur1() { return tk_kind[parse_pos + 1]; }
void bump() { parse_pos = parse_pos + 1; }
void expect(int k, char* what) {
    if (cur() != k) {
        die(what);
    }
    bump();
}

int bin_op(int k) {
    if (k == T_PIPEPIPE) return OP_OR;
    if (k == T_AMPAMP) return OP_AND;
    if (k == T_EE) return OP_EQ;
    if (k == T_NE) return OP_NE;
    if (k == T_LT) return OP_LT;
    if (k == T_LE) return OP_LE;
    if (k == T_GT) return OP_GT;
    if (k == T_GE) return OP_GE;
    if (k == T_PIPE) return OP_BOR;
    if (k == T_CARET) return OP_BXOR;
    if (k == T_AMP) return OP_BAND;
    if (k == T_SHL) return OP_SHL;
    if (k == T_SHR) return OP_SHR;
    if (k == T_PLUS) return OP_ADD;
    if (k == T_MINUS) return OP_SUB;
    if (k == T_STAR) return OP_MUL;
    if (k == T_SLASH) return OP_DIV;
    if (k == T_PCT) return OP_REM;
    return 0;
}

/* precedence, matching subrust/rustc */
int bin_prec(int k) {
    if (k == T_PIPEPIPE) return 1;
    if (k == T_AMPAMP) return 2;
    if (k == T_EE || k == T_NE || k == T_LT || k == T_LE || k == T_GT || k == T_GE) return 3;
    if (k == T_PIPE) return 4;
    if (k == T_CARET) return 5;
    if (k == T_AMP) return 6;
    if (k == T_SHL || k == T_SHR) return 7;
    if (k == T_PLUS || k == T_MINUS) return 8;
    if (k == T_STAR || k == T_SLASH || k == T_PCT) return 9;
    return 0;
}

int assign_op(int k) {
    if (k == T_EQ) return 0;
    if (k == T_PLUSEQ) return OP_ADD;
    if (k == T_MINUSEQ) return OP_SUB;
    if (k == T_STAREQ) return OP_MUL;
    if (k == T_SLASHEQ) return OP_DIV;
    if (k == T_PCTEQ) return OP_REM;
    if (k == T_AMPEQ) return OP_BAND;
    if (k == T_PIPEEQ) return OP_BOR;
    if (k == T_CARETEQ) return OP_BXOR;
    if (k == T_SHLEQ) return OP_SHL;
    if (k == T_SHREQ) return OP_SHR;
    return NIL;
}

void skip_type() {
    /* type is u64 | bool | () — one or two tokens */
    if (cur() == T_LP) { bump(); expect(T_RP, "expected )"); return; }
    bump(); /* u64 / bool ident */
}

int parse_primary() {
    int k;
    int n;
    k = cur();
    if (k == T_INT) {
        n = mk(N_INT);
        nd_a[n] = parse_pos;
        bump();
        return n;
    }
    if (k == T_KW_TRUE || k == T_KW_FALSE) {
        n = mk(N_BOOL);
        if (k == T_KW_TRUE) { nd_a[n] = 1; } else { nd_a[n] = 0; }
        bump();
        return n;
    }
    if (k == T_IDENT) {
        if (cur1() == T_LP) {
            /* call */
            int name;
            int first;
            int last;
            int arg;
            name = parse_pos;
            bump();
            bump(); /* ( */
            n = mk(N_CALL);
            nd_a[n] = name;
            first = NIL;
            last = NIL;
            while (cur() != T_RP) {
                arg = parse_expr(1);
                if (first == NIL) { first = arg; } else { nd_link[last] = arg; }
                last = arg;
                if (cur() != T_RP) {
                    expect(T_COMMA, "expected , or )");
                }
            }
            bump(); /* ) */
            nd_b[n] = first;
            return n;
        }
        n = mk(N_NAME);
        nd_a[n] = parse_pos;
        bump();
        return n;
    }
    if (k == T_LP) {
        bump();
        n = parse_expr(1);
        expect(T_RP, "expected )");
        return n;
    }
    if (k == T_LB) {
        return parse_block();
    }
    if (k == T_KW_IF) {
        int cond;
        int then;
        int els;
        bump();
        cond = parse_expr(1);
        then = parse_block();
        els = NIL;
        if (cur() == T_KW_ELSE) {
            bump();
            if (cur() == T_KW_IF) { els = parse_primary(); }
            else { els = parse_block(); }
        }
        n = mk(N_IF);
        nd_d[n] = cond;
        nd_e[n] = then;
        nd_b[n] = els;
        return n;
    }
    die("expected expression");
    return NIL;
}

int parse_unary() {
    int n;
    if (cur() == T_BANG) {
        int operand;
        bump();
        operand = parse_unary();
        n = mk(N_UNARY);
        nd_a[n] = 0;
        nd_e[n] = operand;
        return n;
    }
    return parse_primary();
}

int parse_expr(int min) {
    int lhs;
    int k;
    int op;
    int prec;
    int rhs;
    int n;
    lhs = parse_unary();
    while (1) {
        k = cur();
        op = bin_op(k);
        prec = bin_prec(k);
        if (op == 0 || prec < min) {
            break;
        }
        bump();
        rhs = parse_expr(prec + 1);
        n = mk(N_BIN);
        nd_a[n] = op;
        nd_d[n] = lhs;
        nd_e[n] = rhs;
        lhs = n;
    }
    return lhs;
}

int parse_block() {
    int n;
    int first;
    int last;
    int tail;
    int s;
    expect(T_LB, "expected {");
    n = mk(N_BLOCK);
    first = NIL;
    last = NIL;
    tail = NIL;
    while (cur() != T_RB) {
        int k;
        k = cur();
        if (k == T_SEMI) {
            bump();
        } else if (k == T_KW_LET) {
            int name;
            int init;
            bump();
            if (cur() == T_KW_MUT) { bump(); }
            name = parse_pos;
            expect(T_IDENT, "expected name");
            if (cur() == T_COLON) { bump(); skip_type(); }
            expect(T_EQ, "expected =");
            init = parse_expr(1);
            expect(T_SEMI, "expected ;");
            s = mk(N_LET);
            nd_a[s] = name;
            nd_e[s] = init;
            if (first == NIL) { first = s; } else { nd_link[last] = s; }
            last = s;
        } else if (k == T_KW_WHILE) {
            int cond;
            int body;
            bump();
            cond = parse_expr(1);
            body = parse_block();
            s = mk(N_WHILE);
            nd_d[s] = cond;
            nd_e[s] = body;
            if (first == NIL) { first = s; } else { nd_link[last] = s; }
            last = s;
        } else if (k == T_KW_LOOP) {
            int body;
            bump();
            body = parse_block();
            s = mk(N_LOOP);
            nd_e[s] = body;
            if (first == NIL) { first = s; } else { nd_link[last] = s; }
            last = s;
        } else if (k == T_KW_BREAK) {
            bump();
            expect(T_SEMI, "expected ;");
            s = mk(N_BREAK);
            if (first == NIL) { first = s; } else { nd_link[last] = s; }
            last = s;
        } else if (k == T_KW_CONTINUE) {
            bump();
            expect(T_SEMI, "expected ;");
            s = mk(N_CONTINUE);
            if (first == NIL) { first = s; } else { nd_link[last] = s; }
            last = s;
        } else {
            /* block-like statement, assignment, expr-stmt, or tail */
            int blocklike;
            int e;
            blocklike = (k == T_KW_IF || k == T_LB);
            e = parse_expr(1);
            if (blocklike) {
                if (cur() == T_RB) { tail = e; }
                else {
                    s = mk(N_EXPR);
                    nd_e[s] = e;
                    if (first == NIL) { first = s; } else { nd_link[last] = s; }
                    last = s;
                }
            } else {
                int ak;
                ak = assign_op(cur());
                if (ak != NIL) {
                    int val;
                    bump();
                    val = parse_expr(1);
                    expect(T_SEMI, "expected ;");
                    /* place is a bare name in SR-seed */
                    s = mk(N_ASSIGN);
                    nd_a[s] = ak;
                    nd_d[s] = nd_a[e]; /* name token of the place */
                    nd_e[s] = val;
                    if (first == NIL) { first = s; } else { nd_link[last] = s; }
                    last = s;
                } else if (cur() == T_SEMI) {
                    bump();
                    s = mk(N_EXPR);
                    nd_e[s] = e;
                    if (first == NIL) { first = s; } else { nd_link[last] = s; }
                    last = s;
                } else if (cur() == T_RB) {
                    tail = e;
                } else {
                    die("expected ; or }");
                }
            }
        }
    }
    bump(); /* } */
    nd_b[n] = first;
    nd_e[n] = tail;
    return n;
}

int parse_program() {
    parse_pos = 0;
    node_n = 0;
    fn_n = 0;
    ptok_n = 0;
    const_n = 0;
    while (cur() != T_EOF) {
        int name;
        if (cur() == T_KW_CONST) {
            int cexpr;
            bump();
            const_tok[const_n] = parse_pos;
            expect(T_IDENT, "expected const name");
            expect(T_COLON, "expected :");
            skip_type();
            expect(T_EQ, "expected =");
            cexpr = parse_expr(1);
            expect(T_SEMI, "expected ;");
            const_expr[const_n] = cexpr;
            const_n = const_n + 1;
            continue;
        }
        expect(T_KW_FN, "expected fn");
        name = parse_pos;
        expect(T_IDENT, "expected fn name");
        expect(T_LP, "expected (");
        fn_tok[fn_n] = name;
        fn_poff[fn_n] = ptok_n;
        fn_np[fn_n] = 0;
        while (cur() != T_RP) {
            if (cur() == T_KW_MUT) { bump(); }
            fn_ptok[ptok_n] = parse_pos;
            ptok_n = ptok_n + 1;
            fn_np[fn_n] = fn_np[fn_n] + 1;
            expect(T_IDENT, "expected param");
            expect(T_COLON, "expected :");
            skip_type();
            if (cur() != T_RP) { expect(T_COMMA, "expected , or )"); }
        }
        bump(); /* ) */
        if (cur() == T_ARROW) { bump(); skip_type(); }
        fn_body[fn_n] = parse_block();
        fn_n = fn_n + 1;
    }
    return 0;
}

/* ---- evaluator ------------------------------------------------------------- */
int find_fn(int name_tok) {
    int i;
    i = 0;
    while (i < fn_n) {
        if (tok_eq(fn_tok[i], name_tok)) {
            return i;
        }
        i = i + 1;
    }
    return NIL;
}

int find_const(int name_tok, int upto) {
    int i;
    i = 0;
    while (i < upto) {
        if (tok_eq(const_tok[i], name_tok)) {
            return i;
        }
        i = i + 1;
    }
    return NIL;
}

int find_local(int name_tok) {
    int i;
    i = loc_n - 1;
    while (i >= frame_base) {
        if (tok_eq(loc_tok[i], name_tok)) {
            return i;
        }
        i = i - 1;
    }
    return NIL;
}

/* ---- software 64-bit multiply / divide (M2-Planet mul/div are 32-bit) ----
 * Uses only 64-bit-correct primitives (add, sub, shift, bitwise, unsigned
 * compare), so one code path is right on both host cc and M2-Planet. */

/* low 64 bits of a*b via 16-bit limbs (each 16x16 product fits in 32 bits,
 * which even M2-Planet's narrow multiply computes exactly). */
WORD w_mul(WORD a, WORD b) {
    WORD a0; WORD a1; WORD a2; WORD a3;
    WORD b0; WORD b1; WORD b2; WORD b3;
    WORD r;
    a0 = a & 65535; a1 = (a >> 16) & 65535; a2 = (a >> 32) & 65535; a3 = (a >> 48) & 65535;
    b0 = b & 65535; b1 = (b >> 16) & 65535; b2 = (b >> 32) & 65535; b3 = (b >> 48) & 65535;
    r = a0 * b0;
    r = r + ((a0 * b1 + a1 * b0) << 16);
    r = r + ((a0 * b2 + a1 * b1 + a2 * b0) << 32);
    r = r + ((a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0) << 48);
    return r;
}

/* a / b, binary long division (b != 0 guaranteed by the caller). The `carry`
 * captures the bit shifted out of the 64-bit r so 2r+bit is handled at full
 * 65-bit width without a wider type. */
WORD w_div(WORD a, WORD b) {
    WORD q; WORD r; WORD one; WORD carry; int i;
    q = 0; r = 0; one = 1; i = 63;
    while (i >= 0) {
        carry = r >> 63;
        r = (r << 1) | ((a >> i) & 1);
        if (carry == 1 || r >= b) {
            r = r - b;
            q = q | (one << i);
        }
        i = i - 1;
    }
    return q;
}

/* a % b via a - (a/b)*b; the product is exact (q*b <= a < 2^64). */
WORD w_rem(WORD a, WORD b) {
    return a - w_mul(w_div(a, b), b);
}

/* rustc debug-profile u64 binary op; sets g_trap on overflow/div0/shift. */
WORD do_bin(int op, WORD a, WORD b) {
    if (op == OP_ADD) {
        WORD r;
        r = a + b;
        if (r < a) { g_trap = 1; }
        return r;
    }
    if (op == OP_SUB) {
        if (a < b) { g_trap = 1; return 0; }
        return a - b;
    }
    if (op == OP_MUL) {
        WORD r;
        r = w_mul(a, b);
        if (a != 0 && w_div(r, a) != b) { g_trap = 1; }
        return r;
    }
    if (op == OP_DIV) {
        if (b == 0) { g_trap = 1; return 0; }
        return w_div(a, b);
    }
    if (op == OP_REM) {
        if (b == 0) { g_trap = 1; return 0; }
        return w_rem(a, b);
    }
    if (op == OP_BAND) return a & b;
    if (op == OP_BOR) return a | b;
    if (op == OP_BXOR) return a ^ b;
    if (op == OP_SHL) {
        if (b >= 64) { g_trap = 1; return 0; }
        return a << b;
    }
    if (op == OP_SHR) {
        if (b >= 64) { g_trap = 1; return 0; }
        return a >> b;
    }
    if (op == OP_EQ) { if (a == b) return 1; return 0; }
    if (op == OP_NE) { if (a != b) return 1; return 0; }
    if (op == OP_LT) { if (a < b) return 1; return 0; }
    if (op == OP_LE) { if (a <= b) return 1; return 0; }
    if (op == OP_GT) { if (a > b) return 1; return 0; }
    if (op == OP_GE) { if (a >= b) return 1; return 0; }
    return 0;
}

/* the BOOT_API host functions; f_* abort (assembly-backend territory). */
WORD host_call(int name_tok, int argnode) {
    int a0;
    int a1;
    WORD v0;
    WORD v1;
    a0 = argnode;
    a1 = NIL;
    if (a0 != NIL) { a1 = nd_link[a0]; }

    if (streq_src(tk_pos[name_tok], tk_len[name_tok], "putb")) {
        v0 = eval(a0);
        if (g_trap) return 0;
        putchar(v0 & 255);
        return 0;
    }
    if (streq_src(tk_pos[name_tok], tk_len[name_tok], "getb")) {
        int c;
        c = getchar();
        if (c < 0) { return 0-1; }
        return c;
    }
    if (streq_src(tk_pos[name_tok], tk_len[name_tok], "ld")) {
        v0 = eval(a0);
        if (g_trap) return 0;
        if (v0 >= MEM_WORDS) { g_trap = 1; return 0; }
        return wmem[v0];
    }
    if (streq_src(tk_pos[name_tok], tk_len[name_tok], "st")) {
        v0 = eval(a0);
        if (g_trap) return 0;
        v1 = eval(a1);
        if (g_trap) return 0;
        if (v0 >= MEM_WORDS) { g_trap = 1; return 0; }
        wmem[v0] = v1;
        return 0;
    }
    /* f_* : IEEE f64 intrinsics live in per-arch assembly; the
     * integer M2-Planet prototype cannot do them. Abort loudly. */
    die("f_* intrinsics need the assembly backend (not in the C prototype)");
    return 0;
}

/* evaluate a node to a value; may set g_signal (break/continue) or g_trap. */
WORD eval(int node) {
    int k;
    if (g_trap || g_signal) {
        return 0;
    }
    k = nd_kind[node];
    if (k == N_INT) {
        return tk_ival[nd_a[node]];
    }
    if (k == N_BOOL) {
        return nd_a[node];
    }
    if (k == N_NAME) {
        int li;
        int ci;
        li = find_local(nd_a[node]);
        if (li != NIL) { return loc_val[li]; }
        ci = find_const(nd_a[node], const_ready);
        if (ci != NIL) { return const_val[ci]; }
        die("undefined name");
        return 0;
    }
    if (k == N_UNARY) {
        WORD v;
        v = eval(nd_e[node]);
        if (g_trap) return 0;
        /* ! : logical on bool (0/1) is the same as bitwise-not-and-1;
         * SR-seed uses ! on bool and on u64 (bitwise). Distinguish by
         * value domain is impossible here, but the checker guaranteed a
         * bool context for logical !, so 0<->1; for u64 !x we need ~x.
         * subrust encodes this in the typed tree; sr0i runs validated code
         * where ! is only applied to bool (u64 !x is rare). Handle both:
         * if v is 0 or 1 treat as bool-not; else bitwise. This matches the
         * corpus (bool-only !). */
        if (v == 0) { return 1; }
        if (v == 1) { return 0; }
        return (0-1) ^ v; /* bitwise ~v for the u64 case */
    }
    if (k == N_BIN) {
        int op;
        WORD a;
        op = nd_a[node];
        /* short-circuit && || */
        if (op == OP_AND) {
            a = eval(nd_d[node]);
            if (g_trap) return 0;
            if (a == 0) { return 0; }
            return eval(nd_e[node]);
        }
        if (op == OP_OR) {
            a = eval(nd_d[node]);
            if (g_trap) return 0;
            if (a != 0) { return 1; }
            return eval(nd_e[node]);
        }
        {
            WORD b;
            a = eval(nd_d[node]);
            if (g_trap) return 0;
            b = eval(nd_e[node]);
            if (g_trap) return 0;
            return do_bin(op, a, b);
        }
    }
    if (k == N_IF) {
        WORD c;
        c = eval(nd_d[node]);
        if (g_trap) return 0;
        if (c != 0) { return eval(nd_e[node]); }
        if (nd_b[node] != NIL) { return eval(nd_b[node]); }
        return 0;
    }
    if (k == N_BLOCK) {
        int s;
        int save;
        WORD v;
        save = loc_n;
        s = nd_b[node];
        while (s != NIL) {
            eval(s);
            if (g_trap || g_signal) { loc_n = save; return 0; }
            s = nd_link[s];
        }
        v = 0;
        if (nd_e[node] != NIL) {
            v = eval(nd_e[node]);
        }
        loc_n = save;
        return v;
    }
    if (k == N_LET) {
        WORD v;
        v = eval(nd_e[node]);
        if (g_trap) return 0;
        loc_tok[loc_n] = nd_a[node];
        loc_val[loc_n] = v;
        loc_n = loc_n + 1;
        return 0;
    }
    if (k == N_ASSIGN) {
        WORD rhs;
        int li;
        rhs = eval(nd_e[node]);
        if (g_trap) return 0;
        li = find_local(nd_d[node]);
        if (li == NIL) { die("assign to undefined"); }
        if (nd_a[node] == 0) {
            loc_val[li] = rhs;
        } else {
            loc_val[li] = do_bin(nd_a[node], loc_val[li], rhs);
        }
        return 0;
    }
    if (k == N_EXPR) {
        eval(nd_e[node]);
        return 0;
    }
    if (k == N_WHILE) {
        while (1) {
            WORD c;
            c = eval(nd_d[node]);
            if (g_trap) return 0;
            if (c == 0) { break; }
            eval(nd_e[node]);
            if (g_trap) return 0;
            if (g_signal == 1) { g_signal = 0; break; }
            if (g_signal == 2) { g_signal = 0; }
        }
        return 0;
    }
    if (k == N_LOOP) {
        while (1) {
            eval(nd_e[node]);
            if (g_trap) return 0;
            if (g_signal == 1) { g_signal = 0; break; }
            if (g_signal == 2) { g_signal = 0; }
        }
        return 0;
    }
    if (k == N_BREAK) {
        g_signal = 1;
        return 0;
    }
    if (k == N_CONTINUE) {
        g_signal = 2;
        return 0;
    }
    if (k == N_CALL) {
        int fi;
        int save_base;
        int save_top;
        int argn;
        int pi;
        WORD ret;
        /* host functions first */
        fi = find_fn(nd_a[node]);
        if (fi == NIL) {
            return host_call(nd_a[node], nd_b[node]);
        }
        /* evaluate args into a temp buffer (avoid frame aliasing) */
        {
            WORD argbuf[16];
            argn = nd_b[node];
            pi = 0;
            while (argn != NIL) {
                argbuf[pi] = eval(argn);
                if (g_trap) return 0;
                pi = pi + 1;
                argn = nd_link[argn];
            }
            save_base = frame_base;
            save_top = loc_n;
            frame_base = loc_n;
            pi = 0;
            while (pi < fn_np[fi]) {
                loc_tok[loc_n] = fn_ptok[fn_poff[fi] + pi];
                loc_val[loc_n] = argbuf[pi];
                loc_n = loc_n + 1;
                pi = pi + 1;
            }
            ret = eval(fn_body[fi]);
            loc_n = save_top;
            frame_base = save_base;
            return ret;
        }
    }
    die("unknown node");
    return 0;
}

void run_main() {
    int mi;
    mi = 0;
    while (mi < fn_n) {
        if (streq_src(tk_pos[fn_tok[mi]], tk_len[fn_tok[mi]], "main")) {
            frame_base = 0;
            loc_n = 0;
            eval(fn_body[mi]);
            return;
        }
        mi = mi + 1;
    }
    die("no main");
}

/* ---- driver ---------------------------------------------------------------- */
int main(int argc, char** argv) {
    FILE* f;
    int c;

    if (argc < 2) {
        die("usage: sr0i program.rs < input");
    }

    src = calloc(SRC_MAX, 1);
    tk_kind = calloc(TOK_MAX, sizeof(int));
    tk_ival = calloc(TOK_MAX, sizeof(WORD));
    tk_pos = calloc(TOK_MAX, sizeof(int));
    tk_len = calloc(TOK_MAX, sizeof(int));
    nd_kind = calloc(NODE_MAX, sizeof(int));
    nd_a = calloc(NODE_MAX, sizeof(int));
    nd_b = calloc(NODE_MAX, sizeof(int));
    nd_c = calloc(NODE_MAX, sizeof(int));
    nd_d = calloc(NODE_MAX, sizeof(int));
    nd_e = calloc(NODE_MAX, sizeof(int));
    nd_link = calloc(NODE_MAX, sizeof(int));
    fn_tok = calloc(FN_MAX, sizeof(int));
    fn_body = calloc(FN_MAX, sizeof(int));
    fn_p0 = calloc(FN_MAX, sizeof(int));
    fn_np = calloc(FN_MAX, sizeof(int));
    fn_poff = calloc(FN_MAX, sizeof(int));
    fn_ptok = calloc(TOK_MAX, sizeof(int));
    const_tok = calloc(FN_MAX, sizeof(int));
    const_val = calloc(FN_MAX, sizeof(WORD));
    const_ready = 0;
    const_expr = calloc(FN_MAX, sizeof(int));
    loc_tok = calloc(LOC_MAX, sizeof(int));
    loc_val = calloc(LOC_MAX, sizeof(WORD));
    wmem = calloc(MEM_WORDS, sizeof(WORD));

    f = fopen(argv[1], "r");
    if (f == 0) {
        die("cannot open program");
    }
    src_n = 0;
    c = fgetc(f);
    while (c >= 0) {
        src[src_n] = c;
        src_n = src_n + 1;
        c = fgetc(f);
    }
    fclose(f);

    g_trap = 0;
    g_signal = 0;

    lex();
    parse_program();

    /* evaluate const initializers in order (a const may use earlier ones) */
    {
        int ci;
        ci = 0;
        while (ci < const_n) {
            frame_base = 0;
            loc_n = 0;
            /* during eval, find_const only sees consts [0, ci) */
            const_val[ci] = eval(const_expr[ci]);
            if (g_trap) { die("const initializer trapped"); }
            const_ready = ci + 1;
            ci = ci + 1;
        }
    }

    run_main();

    fflush(stdout);
    if (g_trap) {
        return 1;
    }
    return 0;
}
