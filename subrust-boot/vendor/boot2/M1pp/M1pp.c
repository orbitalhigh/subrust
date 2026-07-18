/*
 * Tiny single-pass M1pp macro expander. Output is consumed directly by
 * hex2pp -- there is no intermediate M0/hex2 stage. All emission is in
 * the byte/label/directive vocabulary hex2pp accepts.
 *
 * Syntax:
 *   %macro NAME(a, b)
 *   ... body ...
 *   %endm
 *
 *   %struct NAME { f1 f2 ... }   fixed-layout 8-byte-field aggregate
 *   %enum   NAME { l1 l2 ... }   incrementing integer constants
 *
 *   %NAME(x, y)      function-like macro call
 *   ##               token pasting inside macro bodies
 *   !(expr)          evaluate an integer S-expression, emit LE 8-bit hex
 *   @(expr)          evaluate an integer S-expression, emit LE 16-bit hex
 *   %(expr)          evaluate an integer S-expression, emit LE 32-bit hex
 *   $(expr)          evaluate an integer S-expression, emit LE 64-bit hex
 *   %select(c,t,e)   evaluate condition S-expression; expand t if nonzero else e
 *   %str(IDENT)      stringify a single WORD token into a "..."-quoted literal
 *   %bytes(STR)      emit the raw bytes of STR as contiguous hex digits
 *
 *   %frame NAME / %endframe   set/clear a single-slot "current frame"
 *   %local(NAME)              expand to the body of <frame>_FRAME.<NAME>
 *
 * Lexical scoping for control-flow labels is delegated to hex2pp's
 * `.scope` / `.endscope` (which nest). M1pp itself only handles
 * per-expansion macro hygiene labels (`:@name` / `&@name`).
 *
 * Expression syntax is intentionally Lisp-shaped:
 *   atoms: decimal or 0x-prefixed integer literals
 *   calls: (+ a b), (- a b), (* a b), (/ a b), (% a b), (<< a b), (>> a b)
 *          (& a b), (| a b), (^ a b), (~ a), (= a b), (!= a b),
 *          (< a b), (<= a b), (> a b), (>= a b)
 *
 * Flow:
 *   1. lex_source(): scan input_buf into source_tokens[]. Tokens are words,
 *      strings, newlines, parens, commas, and ## paste markers. Whitespace
 *      (excluding newlines) is dropped; # and ; comments are dropped.
 *
 *   2. process_tokens(): main loop driven by a stream stack (streams[]).
 *      The source token array is pushed as the initial stream. Each iteration
 *      pops a token from the top stream:
 *
 *        %macro NAME(p,...) / %endm
 *          -> define_macro(): consume header + body tokens into macros[] and
 *             macro_body_tokens[]; register name and param list. Header is
 *             whitespace-insensitive (newlines inside (...) are skipped);
 *             %endm is recognized anywhere and must be followed by NEWLINE.
 *             A directive that started at line_start consumes its trailing
 *             newline; mid-line directives leave it for the main loop.
 *
 *        !(e) / @(e) / %(e) / $(e) / %select(c,t,e)
 *          -> expand_builtin_call(): parse arg spans, eval S-expression(s) via
 *             eval_expr_range(), emit LE hex or push the chosen token span.
 *             Only fuses when ( is tight against the name (no whitespace).
 *
 *        %NAME(...) matching a defined macro
 *          -> expand_call() -> expand_macro_tokens(): substitute arguments,
 *             apply ## paste via paste_pool_range(), write result into
 *             expand_pool[], then push that slice as a new stream (rescan).
 *             Tight ( required for paren-form; otherwise treated as 0-arg.
 *
 *        Anything else
 *          -> emit_token() / emit_newline() directly into output_buf.
 *
 *      When a stream is exhausted it is popped; pool_used is rewound to the
 *      stream's pool_mark, reclaiming the expand_pool space it used.
 *
 *   3. Write output_buf to the output file.
 *
 * Notes:
 *   - Macros are define-before-use. There is no prescan.
 *   - Expansion rescans by pushing expanded tokens back through the same loop.
 *   - There is no cycle detection. Recursive macros will loop until a limit.
 *   - Only recognized %NAME(...) calls expand. Other text passes through.
 *   - Output formatting is normalized to tokens plus '\n', not preserved.
 */

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Caps chosen to mirror the M1pp.P1 BSS layout, sized so the cc.scm
 * emission of tcc.flat.c (~6.5 MB of macro-rich .P1pp) lexes cleanly.
 * The native binary is host-side, so static globals at these sizes
 * just live in .bss / anonymous mmap without any of the ELF-segment
 * sizing dance the bootstrap m1pp has to do. */
#define MAX_INPUT             16777216    /* 16 MiB */
#define MAX_OUTPUT            134217728   /* 128 MiB */
#define MAX_TEXT              67108864    /* 64 MiB:
                                           * paste tokens, hex literals from
                                           * %(EXPR) evaluation, and per-call
                                           * @local label rewrites all live
                                           * here for the run's lifetime. cc.scm
                                           * triggers hundreds of thousands of
                                           * each across the tcc.c expansion. */
#define MAX_TOKENS            8388608     /*  8 M slots × 32 B = 256 MiB */
#define MAX_MACROS            1024
#define MAX_PARAMS            16
#define MAX_MACRO_BODY_TOKENS MAX_TOKENS
#define MAX_EXPAND            524288      /* 512 K × 32 B = 16 MiB:
                                            * cc.scm wraps each C function in
                                            * %fn(... { body }), and m1pp's
                                            * expand_macro_tokens copies the
                                            * argument tokens into the pool —
                                            * so the entire body of a long
                                            * function is resident in the pool
                                            * while its outer %fn is active.
                                            * tcc.c's next_nomacro1 (~5900
                                            * lines × ~13 m1pp tokens/line ≈
                                            * 77 K tokens, ~2.5 MiB) plus
                                            * inner expansions sit comfortably
                                            * under 16 MiB. */
#define MAX_STACK             64
#define MAX_EXPR_FRAMES       256

enum {
    TOK_WORD,
    TOK_STRING,
    TOK_NEWLINE,
    TOK_LPAREN,
    TOK_RPAREN,
    TOK_COMMA,
    TOK_PASTE,
    TOK_LBRACE,
    TOK_RBRACE
};

enum ExprOp {
    EXPR_ADD,
    EXPR_SUB,
    EXPR_MUL,
    EXPR_DIV,
    EXPR_MOD,
    EXPR_SHL,
    EXPR_SHR,
    EXPR_AND,
    EXPR_OR,
    EXPR_XOR,
    EXPR_NOT,
    EXPR_EQ,
    EXPR_NE,
    EXPR_LT,
    EXPR_LE,
    EXPR_GT,
    EXPR_GE,
    EXPR_STRLEN,
    EXPR_INVALID
};

struct TextSpan {
    const char *ptr;
    int len;
};

struct Token {
    int kind;
    int tight;
    int line;
    struct TextSpan text;
};

struct TokenSpan {
    struct Token *start;
    struct Token *end;
};

struct Macro {
    struct TextSpan name;
    int param_count;
    int has_paste;
    struct TextSpan params[MAX_PARAMS];
    struct Token *body_start;
    struct Token *body_end;
};

struct Stream {
    struct Token *start;
    struct Token *end;
    struct Token *pos;
    int line_start;
    int pool_mark;
};

struct ExprFrame {
    enum ExprOp op;
    long long args[MAX_PARAMS];
    int argc;
};

static char input_buf[MAX_INPUT + 1];
static char output_buf[MAX_OUTPUT + 1];
static char text_buf[MAX_TEXT];

static struct Token source_tokens[MAX_TOKENS];
static struct Token macro_body_tokens[MAX_MACRO_BODY_TOKENS];
/* Per-body-token classification cached at %macro definition time, so
 * expand_macro_tokens never re-runs find_param / is_local_label_token in
 * its hot loop. param_idx: 0 = not a param, k = params[k-1]. */
static unsigned char macro_body_param_idx[MAX_MACRO_BODY_TOKENS];
static unsigned char macro_body_is_local_label[MAX_MACRO_BODY_TOKENS];
static struct Token expand_pool[MAX_EXPAND];
static struct Macro macros[MAX_MACROS];
static struct Stream streams[MAX_STACK];
static struct TextSpan current_frame;
static int frame_active;

static int text_used;
static int source_count;
static int macro_count;
static int macro_body_used;
static int pool_used;
static int output_used;
static int output_need_space;
static int stream_top;
static int next_expansion_id;
static int current_line;
static int error_line;
static const char *input_path;

static struct Token *arg_starts[MAX_PARAMS];
static struct Token *arg_ends[MAX_PARAMS];
static int arg_count;
static struct Token *call_end_pos;
static int args_have_paste;

static const char *error_msg;

static int fail(const char *msg)
{
    error_msg = msg;
    error_line = current_line;
    return 0;
}

static int is_space_no_nl(int c)
{
    return c == ' ' || c == '\t' || c == '\r' || c == '\f' || c == '\v';
}

static char *append_text_len(const char *s, int len)
{
    int start;

    if (text_used + len + 1 > MAX_TEXT) {
        fail("text overflow");
        return NULL;
    }
    start = text_used;
    memcpy(text_buf + text_used, s, (size_t)len);
    text_used += len;
    text_buf[text_used++] = '\0';
    return text_buf + start;
}

static int push_token(struct Token *buf, int *count, int max_count,
                      int kind, int tight, int line, struct TextSpan text)
{
    if (*count >= max_count) {
        return fail("token overflow");
    }
    buf[*count].kind = kind;
    buf[*count].tight = tight;
    buf[*count].line = line;
    buf[*count].text = text;
    *count += 1;
    return 1;
}

static int push_pool_token(struct Token tok)
{
    if (pool_used >= MAX_EXPAND) {
        return fail("expansion overflow");
    }
    expand_pool[pool_used++] = tok;
    return 1;
}

static int token_text_eq(const struct Token *tok, const char *s)
{
    int len = (int)strlen(s);

    return tok->text.len == len &&
           memcmp(tok->text.ptr, s, (size_t)len) == 0;
}

static int span_eq_token(struct TextSpan span, const struct Token *tok)
{
    return span.len == tok->text.len &&
           memcmp(span.ptr, tok->text.ptr, (size_t)span.len) == 0;
}

static int lex_source(const char *src)
{
    /* Track whether whitespace (space, tab, comment, OR newline) precedes
     * the next token. tight=1 means "no whitespace before me"; only
     * LPAREN's tight bit is consulted, to decide whether %FOO(...) /
     * !(...) etc. are paren-call forms. */
    int i = 0;
    int line = 1;
    int saw_separator = 1;

    while (src[i] != '\0') {
        int start;
        int len;
        int tight;

        current_line = line;

        if (is_space_no_nl((unsigned char)src[i])) {
            saw_separator = 1;
            i++;
            continue;
        }
        if (src[i] == '\n') {
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_NEWLINE, 0, line, (struct TextSpan){src + i, 1})) {
                return 0;
            }
            line++;
            saw_separator = 1;
            i++;
            continue;
        }
        if (src[i] == '"' || src[i] == '\'') {
            int quote = src[i];

            tight = !saw_separator;
            start = i;
            i++;
            while (src[i] != '\0' && src[i] != quote) {
                if (src[i] == '\\' && src[i + 1] != '\0') {
                    /* Skip backslash + next char as a unit so the
                     * close-quote test doesn't fire on `\"`, and so
                     * `\\` doesn't leave the trailing `\` to start a
                     * spurious escape. The escape's *meaning* is
                     * decoded later (e.g. by %bytes); the lexer only
                     * cares about token boundaries. */
                    if (src[i + 1] == '\n') {
                        line++;
                    }
                    i += 2;
                    continue;
                }
                if (src[i] == '\n') {
                    line++;
                }
                i++;
            }
            if (src[i] == quote) {
                i++;
            }
            len = i - start;
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_STRING, tight, current_line, (struct TextSpan){src + start, len})) {
                return 0;
            }
            saw_separator = 0;
            continue;
        }
        if (src[i] == '#' && src[i + 1] == '#') {
            tight = !saw_separator;
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_PASTE, tight, line, (struct TextSpan){src + i, 2})) {
                return 0;
            }
            i += 2;
            saw_separator = 0;
            continue;
        }
        if (src[i] == '#' || src[i] == ';') {
            saw_separator = 1;
            while (src[i] != '\0' && src[i] != '\n') {
                i++;
            }
            continue;
        }
        if (src[i] == '(') {
            tight = !saw_separator;
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_LPAREN, tight, line, (struct TextSpan){src + i, 1})) {
                return 0;
            }
            i++;
            saw_separator = 0;
            continue;
        }
        if (src[i] == ')') {
            tight = !saw_separator;
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_RPAREN, tight, line, (struct TextSpan){src + i, 1})) {
                return 0;
            }
            i++;
            saw_separator = 0;
            continue;
        }
        if (src[i] == ',') {
            tight = !saw_separator;
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_COMMA, tight, line, (struct TextSpan){src + i, 1})) {
                return 0;
            }
            i++;
            saw_separator = 0;
            continue;
        }
        if (src[i] == '{') {
            tight = !saw_separator;
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_LBRACE, tight, line, (struct TextSpan){src + i, 1})) {
                return 0;
            }
            i++;
            saw_separator = 0;
            continue;
        }
        if (src[i] == '}') {
            tight = !saw_separator;
            if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                            TOK_RBRACE, tight, line, (struct TextSpan){src + i, 1})) {
                return 0;
            }
            i++;
            saw_separator = 0;
            continue;
        }

        tight = !saw_separator;
        start = i;
        while (src[i] != '\0' &&
               !is_space_no_nl((unsigned char)src[i]) &&
               src[i] != '\n' &&
               src[i] != '#' &&
               src[i] != ';' &&
               src[i] != '(' &&
               src[i] != ')' &&
               src[i] != ',' &&
               src[i] != '{' &&
               src[i] != '}' &&
               !(src[i] == '#' && src[i + 1] == '#')) {
            i++;
        }
        len = i - start;
        if (!push_token(source_tokens, &source_count, MAX_TOKENS,
                        TOK_WORD, tight, line, (struct TextSpan){src + start, len})) {
            return 0;
        }
        saw_separator = 0;
    }

    return 1;
}

static const struct Macro *find_macro(const struct Token *tok)
{
    int i;

    if (tok->kind != TOK_WORD || tok->text.len < 2) {
        return NULL;
    }
    if (tok->text.ptr[0] != '%') {
        return NULL;
    }
    for (i = 0; i < macro_count; i++) {
        if (macros[i].name.len == tok->text.len - 1 &&
            memcmp(tok->text.ptr + 1,
                   macros[i].name.ptr,
                   (size_t)macros[i].name.len) == 0) {
            return &macros[i];
        }
    }
    return NULL;
}

static int find_param(const struct Macro *m, const struct Token *tok)
{
    int i;

    if (tok->kind != TOK_WORD) {
        return 0;
    }
    for (i = 0; i < m->param_count; i++) {
        if (span_eq_token(m->params[i], tok)) {
            return i + 1;
        }
    }
    return 0;
}

static int emit_newline(void)
{
    if (output_used + 1 >= MAX_OUTPUT) {
        return fail("output overflow");
    }
    output_buf[output_used++] = '\n';
    output_need_space = 0;
    return 1;
}

static int emit_string_as_bytes(const struct Token *tok);
static int emit_hex_value(unsigned long long value, int bytes);
static int is_local_label_token(const struct Token *tok);

static int emit_token(const struct Token *tok)
{
    if (tok->kind == TOK_LBRACE || tok->kind == TOK_RBRACE) {
        return 1;
    }
    if (tok->kind == TOK_STRING) {
        return emit_string_as_bytes(tok);
    }
    if (output_need_space) {
        if (output_used + 1 >= MAX_OUTPUT) {
            return fail("output overflow");
        }
        output_buf[output_used++] = ' ';
    }
    if (output_used + tok->text.len >= MAX_OUTPUT) {
        return fail("output overflow");
    }
    memcpy(output_buf + output_used, tok->text.ptr,
           (size_t)tok->text.len);
    output_used += tok->text.len;
    output_need_space = 1;
    return 1;
}

/* Decode a "..." or '...' literal and emit one TOK_WORD per byte
 * (each token's text is the two hex digits for that byte). Recognised
 * escapes inside the literal: \n \t \r \0 \\ \" \xNN. No NUL is
 * appended; user code writes one explicitly if needed. */
static int emit_string_as_bytes(const struct Token *tok)
{
    const char *src;
    int src_len;
    int src_i;

    if (tok->text.len < 2) {
        return fail("bad string");
    }
    src = tok->text.ptr + 1;
    src_len = tok->text.len - 2;
    src_i = 0;
    while (src_i < src_len) {
        unsigned int b;
        char c = src[src_i++];
        if (c == '\\') {
            char e;
            if (src_i >= src_len) {
                return fail("bad escape");
            }
            e = src[src_i++];
            if (e == 'n')       b = 0x0A;
            else if (e == 't')  b = 0x09;
            else if (e == 'r')  b = 0x0D;
            else if (e == '0')  b = 0x00;
            else if (e == '\\') b = 0x5C;
            else if (e == '"')  b = 0x22;
            else if (e == 'x') {
                int hi, lo, hv, lv;
                if (src_i + 2 > src_len) {
                    return fail("bad escape");
                }
                hi = (unsigned char)src[src_i++];
                lo = (unsigned char)src[src_i++];
                hv = (hi >= '0' && hi <= '9') ? hi - '0' :
                     (hi >= 'a' && hi <= 'f') ? hi - 'a' + 10 :
                     (hi >= 'A' && hi <= 'F') ? hi - 'A' + 10 : -1;
                lv = (lo >= '0' && lo <= '9') ? lo - '0' :
                     (lo >= 'a' && lo <= 'f') ? lo - 'a' + 10 :
                     (lo >= 'A' && lo <= 'F') ? lo - 'A' + 10 : -1;
                if (hv < 0 || lv < 0) {
                    return fail("bad escape");
                }
                b = (unsigned int)((hv << 4) | lv);
            } else {
                return fail("bad escape");
            }
        } else {
            b = (unsigned char)c;
        }
        if (!emit_hex_value((unsigned long long)b, 1)) {
            return 0;
        }
    }
    return 1;
}

static int push_stream_span(struct TokenSpan span, int pool_mark)
{
    struct Stream *s;

    if (stream_top >= MAX_STACK) {
        return fail("stream overflow");
    }
    s = &streams[stream_top++];
    s->start = span.start;
    s->end = span.end;
    s->pos = span.start;
    s->line_start = 1;
    s->pool_mark = pool_mark;
    return 1;
}

static struct Stream *current_stream(void)
{
    if (stream_top <= 0) {
        return NULL;
    }
    return &streams[stream_top - 1];
}

static void pop_stream(void)
{
    if (stream_top <= 0) {
        return;
    }
    stream_top--;
    if (streams[stream_top].pool_mark >= 0) {
        pool_used = streams[stream_top].pool_mark;
    }
}

static int copy_span_to_pool(struct TokenSpan span)
{
    struct Token *tok;

    for (tok = span.start; tok < span.end; tok++) {
        if (!push_pool_token(*tok)) {
            return 0;
        }
    }
    return 1;
}

static int push_pool_stream_from_mark(int mark)
{
    if (pool_used == mark) {
        pool_used = mark;
        return 1;
    }
    return push_stream_span((struct TokenSpan){expand_pool + mark, expand_pool + pool_used},
                            mark);
}

static void skip_newlines(struct Token **pos, struct Token *end)
{
    while (*pos < end && (*pos)->kind == TOK_NEWLINE) {
        *pos += 1;
    }
}

static int emit_decimal_text(long long value, struct TextSpan *out)
{
    /* Render a non-negative integer as decimal into text_buf and
     * return the span. No snprintf; plain reverse-fill. */
    char digits[24];
    int digit_count = 0;
    long long v = value;
    int start;
    int i;

    if (v < 0) {
        return fail("bad directive");
    }
    if (v == 0) {
        digits[digit_count++] = '0';
    } else {
        while (v > 0) {
            digits[digit_count++] = (char)('0' + (v % 10));
            v /= 10;
        }
    }

    if (text_used + digit_count + 1 > MAX_TEXT) {
        return fail("text overflow");
    }
    start = text_used;
    for (i = digit_count - 1; i >= 0; i--) {
        text_buf[text_used++] = digits[i];
    }
    text_buf[text_used++] = '\0';
    out->ptr = text_buf + start;
    out->len = digit_count;
    return 1;
}

static int emit_dotted_name(struct TextSpan base, const char *suffix,
                            int suffix_len, struct TextSpan *out)
{
    int total = base.len + 1 + suffix_len;
    int start;

    if (text_used + total + 1 > MAX_TEXT) {
        return fail("text overflow");
    }
    start = text_used;
    memcpy(text_buf + text_used, base.ptr, (size_t)base.len);
    text_used += base.len;
    text_buf[text_used++] = '.';
    memcpy(text_buf + text_used, suffix, (size_t)suffix_len);
    text_used += suffix_len;
    text_buf[text_used++] = '\0';
    out->ptr = text_buf + start;
    out->len = total;
    return 1;
}

static int define_fielded_macro(struct TextSpan base, const char *suffix,
                                int suffix_len, long long value)
{
    struct Macro *m;
    struct Token body_tok;

    if (macro_count >= MAX_MACROS) {
        return fail("too many macros");
    }
    if (macro_body_used >= MAX_MACRO_BODY_TOKENS) {
        return fail("macro body overflow");
    }
    m = &macros[macro_count];
    memset(m, 0, sizeof(*m));
    if (!emit_dotted_name(base, suffix, suffix_len, &m->name)) {
        return 0;
    }
    m->param_count = 0;
    body_tok.kind = TOK_WORD;
    body_tok.tight = 0;
    body_tok.line = current_line;
    if (!emit_decimal_text(value, &body_tok.text)) {
        return 0;
    }
    m->body_start = macro_body_tokens + macro_body_used;
    macro_body_param_idx[macro_body_used] = 0;
    macro_body_is_local_label[macro_body_used] = 0;
    macro_body_tokens[macro_body_used++] = body_tok;
    m->body_end = macro_body_tokens + macro_body_used;
    macro_count++;
    return 1;
}

static int define_fielded(struct Stream *s, long long stride,
                          const char *total_name, int total_name_len)
{
    /* Parses `%struct NAME { f1 f2 ... }` or `%enum NAME { ... }` and
     * synthesizes N+1 zero-parameter macros:
     *   NAME.field_k  -> k * stride
     *   NAME.<total>  -> N * stride    (SIZE for struct, COUNT for enum)
     * The closing } must be immediately followed by TOK_NEWLINE. The
     * newline is consumed iff the directive started at line_start. */
    struct TextSpan base;
    long long index = 0;
    int started_at_line_start = s->line_start;

    s->pos++;
    skip_newlines(&s->pos, s->end);
    if (s->pos >= s->end || s->pos->kind != TOK_WORD) {
        return fail("bad directive");
    }
    base = s->pos->text;
    s->pos++;

    skip_newlines(&s->pos, s->end);
    if (s->pos >= s->end || s->pos->kind != TOK_LBRACE) {
        return fail("bad directive");
    }
    s->pos++;

    for (;;) {
        while (s->pos < s->end &&
               (s->pos->kind == TOK_COMMA || s->pos->kind == TOK_NEWLINE)) {
            s->pos++;
        }
        if (s->pos >= s->end) {
            return fail("unterminated directive");
        }
        if (s->pos->kind == TOK_RBRACE) {
            s->pos++;
            break;
        }
        if (s->pos->kind != TOK_WORD) {
            return fail("bad directive");
        }
        if (!define_fielded_macro(base, s->pos->text.ptr, s->pos->text.len,
                                  index * stride)) {
            return 0;
        }
        s->pos++;
        index++;
    }

    if (!define_fielded_macro(base, total_name, total_name_len, index * stride)) {
        return 0;
    }

    if (s->pos >= s->end || s->pos->kind != TOK_NEWLINE) {
        return fail("expected newline after struct/enum");
    }
    if (started_at_line_start) {
        s->pos++;
        s->line_start = 1;
    }
    return 1;
}

static int define_macro(struct Stream *s)
{
    /* Header is whitespace-insensitive: newlines inside (...) and around
     * the keywords are skipped. Body collection skips newlines that fall
     * between `)` and the first body token (so `%macro N()\nbody\n%endm`
     * has body=[WORD body, NEWLINE], same as the old required-newline form).
     * %endm is recognized anywhere in the body; the next token must be
     * TOK_NEWLINE. The newline is consumed only when the directive started
     * at s->line_start — that way mid-line directives leave the user's
     * trailing newline in the stream for the main loop to emit. */
    struct Macro *m;
    int started_at_line_start = s->line_start;

    if (macro_count >= MAX_MACROS) {
        return fail("too many macros");
    }
    if (macro_body_used >= MAX_MACRO_BODY_TOKENS) {
        return fail("macro body overflow");
    }

    m = &macros[macro_count];
    memset(m, 0, sizeof(*m));
    s->pos++;

    skip_newlines(&s->pos, s->end);
    if (s->pos >= s->end || s->pos->kind != TOK_WORD) {
        return fail("bad macro header");
    }
    m->name = s->pos->text;
    s->pos++;

    skip_newlines(&s->pos, s->end);
    if (s->pos >= s->end || s->pos->kind != TOK_LPAREN) {
        return fail("bad macro header");
    }
    s->pos++;

    skip_newlines(&s->pos, s->end);
    if (s->pos < s->end && s->pos->kind != TOK_RPAREN) {
        while (1) {
            if (m->param_count >= MAX_PARAMS) {
                return fail("bad macro header");
            }
            if (s->pos >= s->end || s->pos->kind != TOK_WORD) {
                return fail("bad macro header");
            }
            m->params[m->param_count] = s->pos->text;
            m->param_count++;
            s->pos++;
            skip_newlines(&s->pos, s->end);
            if (s->pos < s->end && s->pos->kind == TOK_COMMA) {
                s->pos++;
                skip_newlines(&s->pos, s->end);
                continue;
            }
            break;
        }
    }

    if (s->pos >= s->end || s->pos->kind != TOK_RPAREN) {
        return fail("bad macro header");
    }
    s->pos++;
    skip_newlines(&s->pos, s->end);

    m->body_start = macro_body_tokens + macro_body_used;
    while (s->pos < s->end) {
        int idx;

        if (s->pos->kind == TOK_WORD && token_text_eq(s->pos, "%endm")) {
            s->pos++;
            if (s->pos >= s->end || s->pos->kind != TOK_NEWLINE) {
                return fail("expected newline after %endm");
            }
            if (started_at_line_start) {
                s->pos++;
                s->line_start = 1;
            }
            m->body_end = macro_body_tokens + macro_body_used;
            macro_count++;
            return 1;
        }
        if (macro_body_used >= MAX_MACRO_BODY_TOKENS) {
            return fail("macro body overflow");
        }
        idx = macro_body_used;
        macro_body_tokens[idx] = *s->pos;
        macro_body_param_idx[idx] = (unsigned char)find_param(m, s->pos);
        macro_body_is_local_label[idx] =
            is_local_label_token(s->pos) ? 1 : 0;
        if (s->pos->kind == TOK_PASTE) {
            m->has_paste = 1;
        }
        macro_body_used++;
        s->pos++;
    }

    return fail("unterminated macro");
}

static int parse_args(struct Token *lparen, struct Token *limit)
{
    struct Token *tok = lparen + 1;
    struct Token *arg_start = tok;
    int depth = 1;
    int brace_depth = 0;
    int arg_index = 0;

    args_have_paste = 0;

    while (tok < limit) {
        if (tok->kind == TOK_PASTE) {
            args_have_paste = 1;
        }
        if (tok->kind == TOK_LPAREN) {
            depth++;
            tok++;
            continue;
        }
        if (tok->kind == TOK_RPAREN) {
            depth--;
            if (depth == 0) {
                if (brace_depth != 0) {
                    return fail("unbalanced braces");
                }
                if (arg_start == tok && arg_index == 0) {
                    arg_count = 0;
                } else {
                    if (arg_index >= MAX_PARAMS) {
                        return fail("too many args");
                    }
                    arg_starts[arg_index] = arg_start;
                    arg_ends[arg_index] = tok;
                    arg_count = arg_index + 1;
                }
                call_end_pos = tok + 1;
                return 1;
            }
            tok++;
            continue;
        }
        if (tok->kind == TOK_LBRACE) {
            brace_depth++;
            tok++;
            continue;
        }
        if (tok->kind == TOK_RBRACE) {
            if (brace_depth <= 0) {
                return fail("unbalanced braces");
            }
            brace_depth--;
            tok++;
            continue;
        }
        if (tok->kind == TOK_COMMA && depth == 1 && brace_depth == 0) {
            if (arg_index >= MAX_PARAMS) {
                return fail("too many args");
            }
            arg_starts[arg_index] = arg_start;
            arg_ends[arg_index] = tok;
            arg_index++;
            arg_start = tok + 1;
            tok++;
            continue;
        }
        tok++;
    }

    return fail("unterminated macro call");
}

static int arg_is_braced(struct TokenSpan span)
{
    struct Token *tok;
    int depth;

    if (span.end - span.start < 2) {
        return 0;
    }
    if (span.start->kind != TOK_LBRACE ||
        (span.end - 1)->kind != TOK_RBRACE) {
        return 0;
    }
    depth = 0;
    for (tok = span.start; tok < span.end; tok++) {
        if (tok->kind == TOK_LBRACE) {
            depth++;
        } else if (tok->kind == TOK_RBRACE) {
            depth--;
            if (depth == 0 && tok != span.end - 1) {
                return 0;
            }
        }
    }
    return depth == 0;
}

static int copy_arg_tokens_to_pool(struct TokenSpan span)
{
    if (span.start == span.end) {
        return fail("bad macro argument");
    }
    if (arg_is_braced(span)) {
        struct TokenSpan inner;
        inner.start = span.start + 1;
        inner.end = span.end - 1;
        if (inner.start == inner.end) {
            return 1;
        }
        return copy_span_to_pool(inner);
    }
    return copy_span_to_pool(span);
}

static int copy_paste_arg_to_pool(struct TokenSpan span)
{
    if (arg_is_braced(span)) {
        return fail("bad macro argument");
    }
    if (span.end - span.start != 1) {
        return fail("bad macro argument");
    }
    return copy_span_to_pool(span);
}

static int append_pasted_token(struct Token *dst,
                               const struct Token *left,
                               const struct Token *right)
{
    char tmp[512];
    char *text_ptr;
    int n;

    n = snprintf(tmp, sizeof(tmp), "%.*s%.*s",
                 left->text.len, left->text.ptr,
                 right->text.len, right->text.ptr);
    if (n < 0 || n >= (int)sizeof(tmp)) {
        return fail("bad paste");
    }
    text_ptr = append_text_len(tmp, n);
    if (text_ptr == NULL) {
        return 0;
    }
    dst->kind = TOK_WORD;
    dst->tight = 0;
    dst->text.ptr = text_ptr;
    dst->text.len = n;
    return 1;
}

static int paste_pool_range(int mark)
{
    /* Skip newlines on both sides of TOK_PASTE: a body like `foo ##\n bar`
     * pastes to `foobar`, discarding the intervening newline. The left
     * operand is the rightmost non-newline already copied to `out`; the
     * right operand is the next non-newline past PASTE in `in`. */
    struct Token *start = expand_pool + mark;
    struct Token *in = start;
    struct Token *out = start;
    struct Token *end = expand_pool + pool_used;

    while (in < end) {
        if (in->kind == TOK_PASTE) {
            struct Token *left = out;
            struct Token *right = in + 1;

            while (left > start && (left - 1)->kind == TOK_NEWLINE) {
                left--;
            }
            if (left == start) {
                pool_used = mark;
                return fail("bad paste");
            }
            left--;
            if (left->kind == TOK_PASTE) {
                pool_used = mark;
                return fail("bad paste");
            }
            while (right < end && right->kind == TOK_NEWLINE) {
                right++;
            }
            if (right >= end || right->kind == TOK_PASTE) {
                pool_used = mark;
                return fail("bad paste");
            }
            if (!append_pasted_token(left, left, right)) {
                pool_used = mark;
                return 0;
            }
            out = left + 1;
            in = right + 1;
            continue;
        }
        if (out != in) {
            *out = *in;
        }
        out++;
        in++;
    }

    pool_used = (int)(out - expand_pool);
    return 1;
}

static int is_local_label_token(const struct Token *tok)
{
    if (tok->kind != TOK_WORD || tok->text.len < 3) {
        return 0;
    }
    if (tok->text.ptr[0] != ':' && tok->text.ptr[0] != '&') {
        return 0;
    }
    if (tok->text.ptr[1] != '@') {
        return 0;
    }
    return 1;
}

static int push_local_label_token(const struct Token *tok, int expansion_id)
{
    /* Rewrite ":@name" -> ":name__NN", "&@name" -> "&name__NN".
     * Build the text directly in text_buf so the resulting span is stable. */
    char digits[16];
    int digit_count = 0;
    int unsigned_id;
    int start;
    int total;
    int i;
    struct Token out;

    unsigned_id = expansion_id;
    if (unsigned_id == 0) {
        digits[digit_count++] = '0';
    } else {
        while (unsigned_id > 0) {
            digits[digit_count++] = (char)('0' + (unsigned_id % 10));
            unsigned_id /= 10;
        }
    }

    /* Reserve: sigil(1) + tail(len-2) + "__"(2) + digits + NUL. */
    total = 1 + (tok->text.len - 2) + 2 + digit_count;
    if (text_used + total + 1 > MAX_TEXT) {
        return fail("text overflow");
    }
    start = text_used;
    text_buf[text_used++] = tok->text.ptr[0];
    memcpy(text_buf + text_used, tok->text.ptr + 2, (size_t)(tok->text.len - 2));
    text_used += tok->text.len - 2;
    text_buf[text_used++] = '_';
    text_buf[text_used++] = '_';
    for (i = digit_count - 1; i >= 0; i--) {
        text_buf[text_used++] = digits[i];
    }
    text_buf[text_used++] = '\0';

    out.kind = TOK_WORD;
    out.tight = 0;
    out.line = current_line;
    out.text.ptr = text_buf + start;
    out.text.len = total;
    return push_pool_token(out);
}

static int expand_macro_tokens(struct Token *call_tok, struct Token *limit,
                               const struct Macro *m, struct Token **after_out,
                               int *mark_out)
{
    struct Token *body_tok;
    struct Token *end_pos;
    int mark;
    int expansion_id;
    int saw_arg_paste = 0;

    if (call_tok + 1 < limit && (call_tok + 1)->kind == TOK_LPAREN &&
        (call_tok + 1)->tight) {
        if (!parse_args(call_tok + 1, limit)) {
            return 0;
        }
        if (arg_count != m->param_count) {
            return fail("wrong arg count");
        }
        end_pos = call_end_pos;
        saw_arg_paste = args_have_paste;
    } else if (m->param_count == 0) {
        arg_count = 0;
        end_pos = call_tok + 1;
    } else {
        return fail("bad macro call");
    }

    expansion_id = ++next_expansion_id;
    mark = pool_used;
    for (body_tok = m->body_start; body_tok < m->body_end; body_tok++) {
        int idx = (int)(body_tok - macro_body_tokens);
        int param_idx = macro_body_param_idx[idx];
        int pasted = 0;
        int ok;

        if (param_idx != 0) {
            struct TokenSpan arg = {arg_starts[param_idx - 1], arg_ends[param_idx - 1]};
            pasted = (body_tok > m->body_start && (body_tok - 1)->kind == TOK_PASTE) ||
                     (body_tok + 1 < m->body_end && (body_tok + 1)->kind == TOK_PASTE);
            ok = pasted ? copy_paste_arg_to_pool(arg) : copy_arg_tokens_to_pool(arg);
            if (!ok) {
                pool_used = mark;
                return 0;
            }
            continue;
        }
        if (macro_body_is_local_label[idx]) {
            if (!push_local_label_token(body_tok, expansion_id)) {
                pool_used = mark;
                return 0;
            }
            continue;
        }
        if (!push_pool_token(*body_tok)) {
            pool_used = mark;
            return 0;
        }
    }

    if ((m->has_paste || saw_arg_paste) && !paste_pool_range(mark)) {
        return 0;
    }
    *after_out = end_pos;
    *mark_out = mark;
    return 1;
}

static int parse_int_token(const struct Token *tok, long long *out)
{
    char tmp[128];
    char *end;
    unsigned long long uv;
    long long sv;

    if (tok->kind != TOK_WORD || tok->text.len <= 0 || tok->text.len >= (int)sizeof(tmp)) {
        return fail("bad integer");
    }
    memcpy(tmp, tok->text.ptr, (size_t)tok->text.len);
    tmp[tok->text.len] = '\0';

    errno = 0;
    if (tmp[0] == '-') {
        sv = strtoll(tmp, &end, 0);
        if (errno != 0 || *end != '\0') {
            return fail("bad integer");
        }
        *out = sv;
        return 1;
    }

    uv = strtoull(tmp, &end, 0);
    if (errno != 0 || *end != '\0') {
        return fail("bad integer");
    }
    *out = (long long)uv;
    return 1;
}

static enum ExprOp expr_op_code(const struct Token *tok)
{
    if (tok->kind != TOK_WORD) {
        return EXPR_INVALID;
    }
    if (token_text_eq(tok, "+")) {
        return EXPR_ADD;
    }
    if (token_text_eq(tok, "-")) {
        return EXPR_SUB;
    }
    if (token_text_eq(tok, "*")) {
        return EXPR_MUL;
    }
    if (token_text_eq(tok, "/")) {
        return EXPR_DIV;
    }
    if (token_text_eq(tok, "%")) {
        return EXPR_MOD;
    }
    if (token_text_eq(tok, "<<")) {
        return EXPR_SHL;
    }
    if (token_text_eq(tok, ">>")) {
        return EXPR_SHR;
    }
    if (token_text_eq(tok, "&")) {
        return EXPR_AND;
    }
    if (token_text_eq(tok, "|")) {
        return EXPR_OR;
    }
    if (token_text_eq(tok, "^")) {
        return EXPR_XOR;
    }
    if (token_text_eq(tok, "~")) {
        return EXPR_NOT;
    }
    if (token_text_eq(tok, "=")) {
        return EXPR_EQ;
    }
    if (token_text_eq(tok, "!=")) {
        return EXPR_NE;
    }
    if (token_text_eq(tok, "<")) {
        return EXPR_LT;
    }
    if (token_text_eq(tok, "<=")) {
        return EXPR_LE;
    }
    if (token_text_eq(tok, ">")) {
        return EXPR_GT;
    }
    if (token_text_eq(tok, ">=")) {
        return EXPR_GE;
    }
    if (token_text_eq(tok, "strlen")) {
        return EXPR_STRLEN;
    }
    return EXPR_INVALID;
}

static int apply_expr_op(enum ExprOp op, const long long *args, int argc, long long *out)
{
    int i;

    switch (op) {
    case EXPR_ADD:
        if (argc < 1) {
            return fail("bad expression");
        }
        *out = args[0];
        for (i = 1; i < argc; i++) {
            *out += args[i];
        }
        return 1;
    case EXPR_SUB:
        if (argc < 1) {
            return fail("bad expression");
        }
        *out = (argc == 1) ? -args[0] : args[0];
        for (i = 1; i < argc; i++) {
            *out -= args[i];
        }
        return 1;
    case EXPR_MUL:
        if (argc < 1) {
            return fail("bad expression");
        }
        *out = args[0];
        for (i = 1; i < argc; i++) {
            *out *= args[i];
        }
        return 1;
    case EXPR_DIV:
        if (argc != 2 || args[1] == 0) {
            return fail("bad expression");
        }
        *out = args[0] / args[1];
        return 1;
    case EXPR_MOD:
        if (argc != 2 || args[1] == 0) {
            return fail("bad expression");
        }
        *out = args[0] % args[1];
        return 1;
    case EXPR_SHL:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = (long long)((unsigned long long)args[0] << args[1]);
        return 1;
    case EXPR_SHR:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = args[0] >> args[1];
        return 1;
    case EXPR_AND:
        if (argc < 1) {
            return fail("bad expression");
        }
        *out = args[0];
        for (i = 1; i < argc; i++) {
            *out &= args[i];
        }
        return 1;
    case EXPR_OR:
        if (argc < 1) {
            return fail("bad expression");
        }
        *out = args[0];
        for (i = 1; i < argc; i++) {
            *out |= args[i];
        }
        return 1;
    case EXPR_XOR:
        if (argc < 1) {
            return fail("bad expression");
        }
        *out = args[0];
        for (i = 1; i < argc; i++) {
            *out ^= args[i];
        }
        return 1;
    case EXPR_NOT:
        if (argc != 1) {
            return fail("bad expression");
        }
        *out = ~args[0];
        return 1;
    case EXPR_EQ:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = (args[0] == args[1]);
        return 1;
    case EXPR_NE:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = (args[0] != args[1]);
        return 1;
    case EXPR_LT:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = (args[0] < args[1]);
        return 1;
    case EXPR_LE:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = (args[0] <= args[1]);
        return 1;
    case EXPR_GT:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = (args[0] > args[1]);
        return 1;
    case EXPR_GE:
        if (argc != 2) {
            return fail("bad expression");
        }
        *out = (args[0] >= args[1]);
        return 1;
    case EXPR_STRLEN:
    case EXPR_INVALID:
        break;
    }

    return fail("bad expression");
}

static int eval_expr_range(struct TokenSpan span, long long *out);

static int expand_local_into_pool(struct Token *call_tok, struct Token *limit,
                                  struct Token **after_out, int *mark_out)
{
    /* Resolve %local(NAME) against the current frame: build the lookup
     * key "<frame>_FRAME.<NAME>" and copy the matching macro's body
     * into the pool. NAME must be exactly one WORD token. The pool
     * mark and the position past the call's `)` are returned so the
     * caller can either push the body as a stream (process_tokens) or
     * recursively eval it as an expression (eval_expr_atom). */
    char name[256];
    int frame_len;
    int arg_len;
    int name_len;
    int i;
    const struct Macro *m = NULL;
    struct Token *arg_tok;
    int mark = pool_used;

    if (call_tok + 1 >= limit || (call_tok + 1)->kind != TOK_LPAREN ||
        !(call_tok + 1)->tight) {
        return fail("bad builtin");
    }
    if (!parse_args(call_tok + 1, limit)) {
        return 0;
    }
    if (arg_count != 1) {
        return fail("bad builtin");
    }
    if (arg_ends[0] - arg_starts[0] != 1) {
        return fail("bad builtin");
    }
    arg_tok = arg_starts[0];
    if (arg_tok->kind != TOK_WORD) {
        return fail("bad builtin");
    }
    if (!frame_active) {
        return fail("local outside frame");
    }

    frame_len = current_frame.len;
    arg_len = arg_tok->text.len;
    name_len = frame_len + 7 /* _FRAME. */ + arg_len;
    if (name_len >= (int)sizeof(name)) {
        return fail("local name too long");
    }
    memcpy(name, current_frame.ptr, (size_t)frame_len);
    memcpy(name + frame_len, "_FRAME.", 7);
    memcpy(name + frame_len + 7, arg_tok->text.ptr, (size_t)arg_len);

    for (i = 0; i < macro_count; i++) {
        if (macros[i].name.len == name_len &&
            memcmp(macros[i].name.ptr, name, (size_t)name_len) == 0) {
            m = &macros[i];
            break;
        }
    }
    if (m == NULL) {
        return fail("unknown local");
    }

    if (!copy_span_to_pool((struct TokenSpan){m->body_start, m->body_end})) {
        pool_used = mark;
        return 0;
    }
    *after_out = call_end_pos;
    *mark_out = mark;
    return 1;
}

static int eval_expr_atom(struct Token *tok, struct Token *limit,
                          struct Token **after_out, long long *out)
{
    const struct Macro *macro;
    struct Token *after;
    int mark;

    if (tok->kind == TOK_WORD && token_text_eq(tok, "%local")) {
        if (!expand_local_into_pool(tok, limit, &after, &mark)) {
            return 0;
        }
        if (pool_used == mark) {
            pool_used = mark;
            return fail("bad expression");
        }
        if (!eval_expr_range((struct TokenSpan){expand_pool + mark, expand_pool + pool_used}, out)) {
            pool_used = mark;
            return 0;
        }
        pool_used = mark;
        *after_out = after;
        return 1;
    }

    macro = find_macro(tok);
    if (macro != NULL &&
        ((tok + 1 < limit && (tok + 1)->kind == TOK_LPAREN &&
          (tok + 1)->tight) ||
         macro->param_count == 0)) {
        if (!expand_macro_tokens(tok, limit, macro, &after, &mark)) {
            return 0;
        }
        if (pool_used == mark) {
            pool_used = mark;
            return fail("bad expression");
        }
        if (!eval_expr_range((struct TokenSpan){expand_pool + mark, expand_pool + pool_used}, out)) {
            pool_used = mark;
            return 0;
        }
        pool_used = mark;
        *after_out = after;
        return 1;
    }

    if (!parse_int_token(tok, out)) {
        return 0;
    }
    *after_out = tok + 1;
    return 1;
}

static int eval_expr_range(struct TokenSpan span, long long *out)
{
    struct ExprFrame frames[MAX_EXPR_FRAMES];
    int frame_top = 0;
    struct Token *pos = span.start;
    long long value = 0;
    long long result = 0;
    int have_value = 0;
    int have_result = 0;

    for (;;) {
        if (have_value) {
            if (frame_top > 0) {
                struct ExprFrame *frame = &frames[frame_top - 1];

                if (frame->argc >= MAX_PARAMS) {
                    return fail("bad expression");
                }
                frame->args[frame->argc++] = value;
                have_value = 0;
                continue;
            }
            if (have_result) {
                return fail("bad expression");
            }
            result = value;
            have_result = 1;
            have_value = 0;
            continue;
        }

        skip_newlines(&pos, span.end);
        if (pos >= span.end) {
            break;
        }
        if (pos->line > 0) {
            current_line = pos->line;
        }

        if (pos->kind == TOK_LPAREN) {
            enum ExprOp op;

            pos++;
            skip_newlines(&pos, span.end);
            if (pos >= span.end) {
                return fail("bad expression");
            }
            op = expr_op_code(pos);
            if (op == EXPR_INVALID) {
                return fail("bad expression");
            }
            pos++;
            if (op == EXPR_STRLEN) {
                /* strlen is degenerate: argument is a TOK_STRING atom,
                 * not a recursive expression. Handle inline and yield
                 * the string's raw byte count (span.len - 2). */
                skip_newlines(&pos, span.end);
                if (pos >= span.end || pos->kind != TOK_STRING) {
                    return fail("bad expression");
                }
                if (pos->text.len < 2 || pos->text.ptr[0] != '"') {
                    return fail("bad expression");
                }
                value = (long long)(pos->text.len - 2);
                pos++;
                skip_newlines(&pos, span.end);
                if (pos >= span.end || pos->kind != TOK_RPAREN) {
                    return fail("bad expression");
                }
                pos++;
                have_value = 1;
                continue;
            }
            if (frame_top >= MAX_EXPR_FRAMES) {
                return fail("expression overflow");
            }
            frames[frame_top].op = op;
            frames[frame_top].argc = 0;
            frame_top++;
            continue;
        }

        if (pos->kind == TOK_RPAREN) {
            if (frame_top <= 0) {
                return fail("bad expression");
            }
            if (!apply_expr_op(frames[frame_top - 1].op,
                               frames[frame_top - 1].args,
                               frames[frame_top - 1].argc,
                               &value)) {
                return 0;
            }
            frame_top--;
            pos++;
            have_value = 1;
            continue;
        }

        if (!eval_expr_atom(pos, span.end, &pos, &value)) {
            return 0;
        }
        have_value = 1;
    }

    if (frame_top != 0 || !have_result) {
        return fail("bad expression");
    }
    if (pos != span.end) {
        return fail("bad expression");
    }

    *out = result;
    return 1;
}

static int emit_hex_value(unsigned long long value, int bytes)
{
    /* Emit the bytes as bare little-endian hex digits. hex2pp's byte-
     * stream parser groups every two hex digits into one byte; no
     * quoting or separators are needed. */
    char tmp[17];
    static const char hex[] = "0123456789ABCDEF";
    struct Token tok;
    int i;
    char *text_ptr;
    int total_len = 2 * bytes;

    for (i = 0; i < bytes; i++) {
        unsigned int b = (unsigned int)((value >> (8 * i)) & 0xFF);
        tmp[2 * i] = hex[b >> 4];
        tmp[2 * i + 1] = hex[b & 0x0F];
    }
    tmp[total_len] = '\0';

    text_ptr = append_text_len(tmp, total_len);
    if (text_ptr == NULL) {
        return 0;
    }
    tok.kind = TOK_WORD;
    tok.tight = 0;
    tok.line = current_line;
    tok.text.ptr = text_ptr;
    tok.text.len = total_len;
    return emit_token(&tok);
}

static int expand_builtin_call(struct Stream *s, const struct Token *tok)
{
    long long value;

    if (tok + 1 >= s->end || (tok + 1)->kind != TOK_LPAREN) {
        return fail("bad builtin");
    }
    if (!parse_args((struct Token *)tok + 1, s->end)) {
        return 0;
    }

    if (token_text_eq(tok, "!") || token_text_eq(tok, "@") ||
        token_text_eq(tok, "%") || token_text_eq(tok, "$")) {
        struct TokenSpan arg;
        struct Token *end_pos;
        int bytes;

        if (arg_count != 1) {
            return fail("bad builtin");
        }
        arg.start = arg_starts[0];
        arg.end = arg_ends[0];
        end_pos = call_end_pos;
        if (!eval_expr_range(arg, &value)) {
            return 0;
        }
        s->pos = end_pos;
        s->line_start = 0;
        bytes = token_text_eq(tok, "!") ? 1 :
                token_text_eq(tok, "@") ? 2 :
                token_text_eq(tok, "%") ? 4 : 8;
        return emit_hex_value((unsigned long long)value, bytes);
    }

    if (token_text_eq(tok, "%select")) {
        struct TokenSpan cond_arg, then_arg, else_arg, chosen;
        struct Token *end_pos;
        int mark;

        if (arg_count != 3) {
            return fail("bad builtin");
        }
        cond_arg.start = arg_starts[0]; cond_arg.end = arg_ends[0];
        then_arg.start = arg_starts[1]; then_arg.end = arg_ends[1];
        else_arg.start = arg_starts[2]; else_arg.end = arg_ends[2];
        end_pos = call_end_pos;
        if (!eval_expr_range(cond_arg, &value)) {
            return 0;
        }
        chosen = (value != 0) ? then_arg : else_arg;
        s->pos = end_pos;
        s->line_start = 0;
        if (chosen.start == chosen.end) {
            return 1;
        }
        mark = pool_used;
        if (!copy_span_to_pool(chosen)) {
            pool_used = mark;
            return 0;
        }
        return push_pool_stream_from_mark(mark);
    }

    if (token_text_eq(tok, "%local")) {
        struct Token *after;
        int mark;

        if (!expand_local_into_pool((struct Token *)tok, s->end, &after, &mark)) {
            return 0;
        }
        s->pos = after;
        s->line_start = 0;
        return push_pool_stream_from_mark(mark);
    }

    if (token_text_eq(tok, "%str")) {
        struct Token *arg_tok;
        struct Token *end_pos;
        struct Token out_tok;
        char *text_ptr;
        int orig_len;
        int out_len;

        if (arg_count != 1) {
            return fail("bad builtin");
        }
        if (arg_ends[0] - arg_starts[0] != 1) {
            return fail("bad builtin");
        }
        arg_tok = arg_starts[0];
        if (arg_tok->kind != TOK_WORD) {
            return fail("bad builtin");
        }
        end_pos = call_end_pos;

        orig_len = arg_tok->text.len;
        out_len = orig_len + 2;
        if (text_used + out_len + 1 > MAX_TEXT) {
            return fail("text overflow");
        }
        text_ptr = text_buf + text_used;
        text_buf[text_used++] = '"';
        memcpy(text_buf + text_used, arg_tok->text.ptr, (size_t)orig_len);
        text_used += orig_len;
        text_buf[text_used++] = '"';
        text_buf[text_used++] = '\0';

        out_tok.kind = TOK_STRING;
        out_tok.tight = 0;
        out_tok.line = current_line;
        out_tok.text.ptr = text_ptr;
        out_tok.text.len = out_len;
        s->pos = end_pos;
        s->line_start = 0;
        return emit_token(&out_tok);
    }

    return fail("bad builtin");
}

static int expand_call(struct Stream *s, const struct Macro *macro)
{
    struct Token *after;
    int mark;

    if (!expand_macro_tokens(s->pos, s->end, macro, &after, &mark)) {
        return 0;
    }
    s->pos = after;
    s->line_start = 0;
    return push_pool_stream_from_mark(mark);
}

static int push_frame(struct Stream *s)
{
    /* %frame NAME sets the single-slot current frame, used by %local
     * lookup. Frames do not nest: a second %frame before %endframe is
     * an error. The header behaves like %scope (newlines after the
     * name are absorbed when the directive appeared at line_start). */
    int started_at_line_start = s->line_start;

    s->pos++;
    skip_newlines(&s->pos, s->end);
    if (s->pos >= s->end || s->pos->kind != TOK_WORD) {
        return fail("bad frame header");
    }
    if (frame_active) {
        return fail("frame already active");
    }
    current_frame = s->pos->text;
    frame_active = 1;
    s->pos++;
    if (started_at_line_start) {
        skip_newlines(&s->pos, s->end);
        s->line_start = 1;
    }
    return 1;
}

static int pop_frame(struct Stream *s)
{
    /* %endframe must be immediately followed by TOK_NEWLINE; the newline
     * is consumed iff %endframe itself appeared at line_start. */
    int started_at_line_start = s->line_start;

    s->pos++;
    if (!frame_active) {
        return fail("frame underflow");
    }
    frame_active = 0;
    if (s->pos >= s->end || s->pos->kind != TOK_NEWLINE) {
        return fail("expected newline after %endframe");
    }
    if (started_at_line_start) {
        s->pos++;
        s->line_start = 1;
    }
    return 1;
}

static int process_tokens(void)
{
    if (!push_stream_span((struct TokenSpan){source_tokens, source_tokens + source_count}, -1)) {
        return 0;
    }

    /* Per-token dispatch is gated on the first byte of WORD tokens.
     * Plain pass-through tokens (e.g. hex literals, bare identifiers)
     * fail the c0=='%' / c0 in {!,@,$} test in one byte compare and go
     * straight to emit_token. Within the c0=='%' branch we dispatch on
     * the second byte to pick the matching directive/builtin without
     * walking ~9 token_text_eq probes. */
    for (;;) {
        struct Stream *s;
        struct Token *tok;

        s = current_stream();
        if (s == NULL) {
            break;
        }
        if (s->pos >= s->end) {
            pop_stream();
            continue;
        }

        tok = s->pos;
        if (tok->line > 0) {
            current_line = tok->line;
        }

        if (tok->kind == TOK_NEWLINE) {
            s->pos++;
            s->line_start = 1;
            if (!emit_newline()) {
                return 0;
            }
            continue;
        }

        if (tok->kind == TOK_WORD && tok->text.len >= 1) {
            const char *p = tok->text.ptr;
            int len = tok->text.len;
            char c0 = p[0];
            int has_paren = (tok + 1 < s->end &&
                             (tok + 1)->kind == TOK_LPAREN &&
                             (tok + 1)->tight);

            if (c0 == '%' && len >= 2) {
                char c1 = p[1];
                const struct Macro *macro;
                int handled = 0;

                switch (c1) {
                case 'm':
                    if (len == 6 && memcmp(p + 2, "acro", 4) == 0) {
                        if (!define_macro(s)) return 0;
                        handled = 1;
                    }
                    break;
                case 's':
                    if (len == 7 && memcmp(p + 2, "truct", 5) == 0) {
                        if (!define_fielded(s, 8, "SIZE", 4)) return 0;
                        handled = 1;
                    } else if (has_paren && len == 7 &&
                               memcmp(p + 2, "elect", 5) == 0) {
                        if (!expand_builtin_call(s, tok)) return 0;
                        handled = 1;
                    } else if (has_paren && len == 4 &&
                               memcmp(p + 2, "tr", 2) == 0) {
                        if (!expand_builtin_call(s, tok)) return 0;
                        handled = 1;
                    }
                    break;
                case 'e':
                    if (len == 5 && memcmp(p + 2, "num", 3) == 0) {
                        if (!define_fielded(s, 1, "COUNT", 5)) return 0;
                        handled = 1;
                    } else if (len == 9 &&
                               memcmp(p + 2, "ndframe", 7) == 0) {
                        if (!pop_frame(s)) return 0;
                        handled = 1;
                    }
                    break;
                case 'f':
                    if (len == 6 && memcmp(p + 2, "rame", 4) == 0) {
                        if (!push_frame(s)) return 0;
                        handled = 1;
                    }
                    break;
                case 'b':
                    if (has_paren && len == 6 &&
                        memcmp(p + 2, "ytes", 4) == 0) {
                        if (!expand_builtin_call(s, tok)) return 0;
                        handled = 1;
                    }
                    break;
                case 'l':
                    if (has_paren && len == 6 &&
                        memcmp(p + 2, "ocal", 4) == 0) {
                        if (!expand_builtin_call(s, tok)) return 0;
                        handled = 1;
                    }
                    break;
                }

                if (handled) {
                    continue;
                }

                macro = find_macro(tok);
                if (macro != NULL &&
                    (has_paren || macro->param_count == 0)) {
                    if (!expand_call(s, macro)) return 0;
                    continue;
                }
            } else if (len == 1 &&
                       (c0 == '!' || c0 == '@' ||
                        c0 == '$' || c0 == '%')) {
                if (has_paren) {
                    if (!expand_builtin_call(s, tok)) return 0;
                    continue;
                }
            }
        }

        s->pos++;
        s->line_start = 0;
        if (!emit_token(tok)) {
            return 0;
        }
    }

    if (frame_active) {
        return fail("frame not closed");
    }

    if (output_used >= MAX_OUTPUT) {
        return fail("output overflow");
    }
    output_buf[output_used] = '\0';
    return 1;
}

int main(int argc, char **argv)
{
    FILE *in;
    FILE *out;
    size_t nread;

    if (argc != 3) {
        fprintf(stderr, "usage: %s input.M1 output.M1\n", argv[0]);
        return 1;
    }

    input_path = argv[1];
    in = fopen(argv[1], "rb");
    if (in == NULL) {
        perror(argv[1]);
        return 1;
    }
    nread = fread(input_buf, 1, MAX_INPUT, in);
    if (ferror(in)) {
        perror(argv[1]);
        fclose(in);
        return 1;
    }
    fclose(in);
    if (nread >= MAX_INPUT) {
        fprintf(stderr, "input too large\n");
        return 1;
    }
    input_buf[nread] = '\0';

    if (!lex_source(input_buf) || !process_tokens()) {
        fprintf(stderr, "%s:%d: m1macro: %s\n",
                input_path != NULL ? input_path : "?",
                error_line,
                error_msg != NULL ? error_msg : "failed");
        return 1;
    }

    out = fopen(argv[2], "wb");
    if (out == NULL) {
        perror(argv[2]);
        return 1;
    }
    if (fwrite(output_buf, 1, (size_t)output_used, out) != (size_t)output_used) {
        perror(argv[2]);
        fclose(out);
        return 1;
    }
    fclose(out);
    fprintf(stderr, "text_used=%d output_used=%d\n", text_used, output_used);
    return 0;
}
