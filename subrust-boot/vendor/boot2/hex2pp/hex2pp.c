/*
 * hex2pp.c -- reference C implementation of hex2++.
 *
 * See docs/HEX2pp.md for the spec. Brief summary:
 *
 *   Input is scanned once. Label definitions are recorded into a table
 *   on the fly; label references emit zero placeholders and append a
 *   fixup record. After the scan, fixups are resolved against the
 *   completed label table and patched into the output buffer.
 *
 *   Active syntax:
 *     digits in current byte mode    -> raw bytes (HEX or BINARY)
 *     :NAME                          -> label definition
 *     SIGIL NAME [- OTHER]           -> label reference (! @ $ ~ % &)
 *     .align N [PATTERN]             -> pad to N-byte boundary
 *     .fill N B                      -> N copies of byte B
 *     .scope / .endscope             -> local-label scope (nestable)
 *     # ... / ; ...                  -> line comment
 *
 *   Multi-byte reference values are emitted little-endian by default.
 */

#include <ctype.h>
#include <errno.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#define MAX_INPUT_BYTES   (16 * 1024 * 1024)
#define MAX_OUTPUT_BYTES  (128 * 1024 * 1024)
#define MAX_LABELS        (1 << 20)
#define MAX_FIXUPS        (1 << 20)
#define MAX_TEXT          (8 * 1024 * 1024)
#define MAX_TOKEN         4096
#define MAX_SCOPE_DEPTH   32
#define MAX_SCOPE_HISTORY (1 << 22)

enum { HEX_MODE, BINARY_MODE };

struct InFile {
    const char *path;
    char       *buf;
    int         len;
};

struct Label {
    int       name_off;
    int       name_len;
    long long target_ip;
    int       scope_id;     /* 0 = global */
};

struct Fixup {
    long long   out_off;        /* offset in output_buf to patch */
    long long   ip_at_ref;      /* ip immediately after the reference's bytes */
    const char *name;           /* points into input buffer */
    const char *other;          /* NULL if no -OTHER */
    const char *src_path;
    int         name_len;
    int         other_len;
    int         scope_hist_off; /* index into scope_history (depth>0 only) */
    int         scope_depth;
    int         src_line;
    int         sigil;
};

static struct InFile input_file;

static char          text_buf[MAX_TEXT];
static int           text_used;

static struct Label  labels[MAX_LABELS];
static int           label_count;

static struct Fixup  fixups[MAX_FIXUPS];
static int           fixup_count;

static int           scope_history[MAX_SCOPE_HISTORY];
static int           scope_history_used;

static unsigned char output_buf[MAX_OUTPUT_BYTES];
static long long     output_used;

static long long     ip;
static long long     base_address;
static int           byte_mode = HEX_MODE;
static int           big_endian;
static int           non_executable;
static const char   *output_path;
static int           ptrsize = 4;     /* width of '&' and '%'; settable via .ptrsize */
static int           ptrsize_used;    /* a '&'/'%' reference has fixed the width */

static int           scope_stack[MAX_SCOPE_DEPTH];
static int           scope_depth;
static int           scope_seq;

static const char   *cur_path;
static int           cur_line;

/* --- error reporting ---------------------------------------------------- */

static void die(const char *fmt, ...)
{
    va_list ap;
    if (cur_path != NULL) {
        fprintf(stderr, "%s:%d: hex2pp: ", cur_path, cur_line);
    } else {
        fprintf(stderr, "hex2pp: ");
    }
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
    fputc('\n', stderr);
    exit(1);
}

/* --- text / label table ------------------------------------------------- */

static int intern(const char *s, int len)
{
    int off;
    if (text_used + len + 1 > MAX_TEXT) {
        die("text pool overflow");
    }
    off = text_used;
    memcpy(text_buf + off, s, (size_t)len);
    text_buf[off + len] = '\0';
    text_used += len + 1;
    return off;
}

static int name_eq(const struct Label *L, const char *s, int len)
{
    return L->name_len == len && memcmp(text_buf + L->name_off, s, (size_t)len) == 0;
}

static void define_label(const char *s, int len, int scope_id)
{
    if (label_count >= MAX_LABELS) {
        die("too many labels");
    }
    labels[label_count].name_off  = intern(s, len);
    labels[label_count].name_len  = len;
    labels[label_count].target_ip = ip;
    labels[label_count].scope_id  = scope_id;
    label_count++;
}

static long long lookup_label_in(const char *s, int len, const int *stack, int depth)
{
    int i;
    int d;
    int dotted = (len > 0 && s[0] == '.' && depth > 0);
    if (dotted) {
        /* Inside a scope, walk the scope stack innermost-out. A dotted
         * name resolves to the nearest enclosing definition, so an inner
         * scope can shadow an outer one with the same local name. */
        for (d = depth - 1; d >= 0; d--) {
            int sid = stack[d];
            for (i = 0; i < label_count; i++) {
                if (labels[i].scope_id == sid && name_eq(&labels[i], s, len)) {
                    return labels[i].target_ip;
                }
            }
        }
        die("undefined local label '%.*s'", len, s);
    } else {
        for (i = 0; i < label_count; i++) {
            if (labels[i].scope_id == 0 && name_eq(&labels[i], s, len)) {
                return labels[i].target_ip;
            }
        }
        die("undefined label '%.*s'", len, s);
    }
    return 0; /* unreachable */
}

/* --- I/O ---------------------------------------------------------------- */

static void emit_byte(unsigned b)
{
    if (output_used >= MAX_OUTPUT_BYTES) {
        die("output overflow");
    }
    output_buf[output_used++] = (unsigned char)b;
    ip++;
}

static long long emit_zeros(long long n)
{
    long long off = output_used;
    if (output_used + n > MAX_OUTPUT_BYTES) {
        die("output overflow");
    }
    memset(output_buf + output_used, 0, (size_t)n);
    output_used += n;
    ip += n;
    return off;
}

static void emit_fill(long long n, unsigned char b)
{
    if (output_used + n > MAX_OUTPUT_BYTES) {
        die("output overflow");
    }
    memset(output_buf + output_used, b, (size_t)n);
    output_used += n;
    ip += n;
}

static void write_value(long long out_off, long long v, int width, long long lo, long long hi, int range_check)
{
    int i;
    unsigned char bytes[8];

    if (range_check && (v < lo || v > hi)) {
        die("reference out of range: value=%lld, allowed=[%lld,%lld]", v, lo, hi);
    }
    if (width < 1 || width > 8) {
        die("internal: bad reference width %d", width);
    }

    for (i = 0; i < width; i++) {
        bytes[i] = (unsigned char)((unsigned long long)v >> (8 * i)) & 0xff;
    }
    if (big_endian) {
        for (i = 0; i < width; i++) {
            output_buf[out_off + i] = bytes[width - 1 - i];
        }
    } else {
        for (i = 0; i < width; i++) {
            output_buf[out_off + i] = bytes[i];
        }
    }
}

/* --- per-file scanner state -------------------------------------------- */

struct Scanner {
    const char *buf;
    int         len;
    int         pos;
};

static int eatc(struct Scanner *s)
{
    int c;
    if (s->pos >= s->len) return -1;
    c = (unsigned char)s->buf[s->pos++];
    if (c == '\n') cur_line++;
    return c;
}

static int is_space_any(int c) { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v'; }

static void skip_ws_and_comments(struct Scanner *s)
{
    int c;
    while (s->pos < s->len) {
        c = (unsigned char)s->buf[s->pos];
        if (is_space_any(c)) {
            eatc(s);
        } else if (c == '#' || c == ';') {
            while (s->pos < s->len && s->buf[s->pos] != '\n') s->pos++;
        } else {
            break;
        }
    }
}

/* --- byte-mode digit handling ----------------------------------------- */

static int byte_digit_count(void)
{
    if (byte_mode == HEX_MODE) return 2;
    return 8; /* BINARY */
}

static int is_byte_digit(int c)
{
    if (byte_mode == HEX_MODE) return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
    return c == '0' || c == '1';
}

static int byte_digit_value(int c)
{
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return 10 + (c - 'a');
    if (c >= 'A' && c <= 'F') return 10 + (c - 'A');
    return -1;
}

static int parse_one_byte_literal(struct Scanner *s, unsigned char *out, int allow_multi, unsigned char *buf, int bufmax, int *outlen)
{
    /* Parse a contiguous run of byte-mode digits (no whitespace inside,
     * since this is for directive arguments where digit-run terminates the
     * argument). Returns number of bytes produced. */
    int need = byte_digit_count();
    int acc = 0;
    int have = 0;
    int produced = 0;
    int c;

    while (s->pos < s->len) {
        c = (unsigned char)s->buf[s->pos];
        if (!is_byte_digit(c)) break;
        s->pos++;
        if (byte_mode == HEX_MODE) acc = (acc << 4) | byte_digit_value(c);
        else                       acc = (acc << 1) | (c - '0');
        have++;
        if (have == need) {
            if (allow_multi) {
                if (produced >= bufmax) die("pattern too large");
                buf[produced++] = (unsigned char)(acc & 0xff);
            } else {
                if (produced > 0) die("byte literal: too many digits");
                *out = (unsigned char)(acc & 0xff);
                produced = 1;
            }
            acc = 0;
            have = 0;
            if (!allow_multi) break;
        }
    }
    if (have != 0) die("byte literal: incomplete digits (%d left over)", have);
    if (produced == 0) die("expected byte literal");
    if (outlen) *outlen = produced;
    return produced;
}

/* Parse a free-flowing byte stream: digits separated by arbitrary
 * whitespace and comments. Stops at any non-digit non-whitespace
 * non-comment character. */
static void parse_byte_stream(struct Scanner *s)
{
    int need = byte_digit_count();
    int acc = 0;
    int have = 0;
    int c;

    for (;;) {
        if (s->pos >= s->len) break;
        c = (unsigned char)s->buf[s->pos];
        if (is_space_any(c)) { eatc(s); continue; }
        if (c == '#' || c == ';') {
            while (s->pos < s->len && s->buf[s->pos] != '\n') s->pos++;
            continue;
        }
        if (!is_byte_digit(c)) break;
        s->pos++;
        if (byte_mode == HEX_MODE) acc = (acc << 4) | byte_digit_value(c);
        else                       acc = (acc << 1) | (c - '0');
        have++;
        if (have == need) {
            emit_byte((unsigned)(acc & 0xff));
            acc = 0;
            have = 0;
        }
    }
    if (have != 0) die("byte stream: incomplete digits at end of run (%d left over)", have);
}

/* --- name / token reading --------------------------------------------- */

static int is_name_terminator(int c)
{
    /* Per spec: names terminated by whitespace, '-', or '>' (the two
     * label-arithmetic separators). We also stop at end-of-line comments
     * and EOF for safety. */
    if (c < 0) return 1;
    if (is_space_any(c)) return 1;
    if (c == '-' || c == '>') return 1;
    if (c == '#' || c == ';') return 1;
    return 0;
}

/* Scan a label name; return pointer span into the input buffer (no copy). */
static int scan_name(struct Scanner *s, const char **out_start)
{
    int start = s->pos;
    while (s->pos < s->len) {
        int c = (unsigned char)s->buf[s->pos];
        if (is_name_terminator(c)) break;
        s->pos++;
    }
    if (s->pos == start) die("expected label name");
    *out_start = s->buf + start;
    return s->pos - start;
}

/* Decimal integer (for directive arity arguments). */
static long long read_decimal(struct Scanner *s)
{
    long long v = 0;
    int saw = 0;
    int c;
    while (s->pos < s->len) {
        c = (unsigned char)s->buf[s->pos];
        if (c < '0' || c > '9') break;
        v = v * 10 + (c - '0');
        saw = 1;
        s->pos++;
    }
    if (!saw) die("expected decimal integer");
    return v;
}

/* --- references ------------------------------------------------------- */

struct SigilInfo {
    int  width;
    int  is_rel;
    long long lo;
    long long hi;
    int  range_check;
};

static struct SigilInfo sigil_info(int c)
{
    struct SigilInfo si = {0};
    switch (c) {
    case '!': si.width = 1; si.is_rel = 1; si.lo = -128;        si.hi = 127;          si.range_check = 1; break;
    case '@': si.width = 2; si.is_rel = 1; si.lo = -32768;      si.hi = 32767;        si.range_check = 1; break;
    case '$': si.width = 2; si.is_rel = 0; si.lo = 0;           si.hi = 65535;        si.range_check = 1; break;
    case '~': si.width = 3; si.is_rel = 1; si.lo = -(1LL << 23); si.hi = (1LL << 23) - 1; si.range_check = 1; break;
    case '%': si.width = ptrsize; si.is_rel = 1; si.lo = 0;     si.hi = 0;            si.range_check = 0; break;
    case '&': si.width = ptrsize; si.is_rel = 0; si.lo = 0;     si.hi = 0;            si.range_check = 0; break;
    default:  die("internal: bad sigil 0x%02x", c);
    }
    return si;
}

static void record_fixup(const char *name, int name_len,
                         const char *other, int other_len,
                         int sigil, long long out_off, long long ip_after)
{
    struct Fixup *f;
    if (fixup_count >= MAX_FIXUPS) die("too many references");
    f = &fixups[fixup_count++];
    f->out_off     = out_off;
    f->ip_at_ref   = ip_after;
    f->name        = name;
    f->name_len    = name_len;
    f->other       = other;
    f->other_len   = other_len;
    f->sigil       = sigil;
    f->src_path    = cur_path;
    f->src_line    = cur_line;
    f->scope_depth = scope_depth;
    if (scope_depth > 0) {
        if (scope_history_used + scope_depth > MAX_SCOPE_HISTORY) {
            die("scope history overflow");
        }
        f->scope_hist_off = scope_history_used;
        memcpy(&scope_history[scope_history_used], scope_stack,
               (size_t)scope_depth * sizeof(int));
        scope_history_used += scope_depth;
    } else {
        f->scope_hist_off = 0;
    }
}

static void process_reference(struct Scanner *s, int sigil)
{
    const char *name_start;
    const char *other_start = NULL;
    int  name_len;
    int  other_len = 0;
    struct SigilInfo si = sigil_info(sigil);
    long long out_off;

    if (sigil == '&' || sigil == '%') ptrsize_used = 1;

    /* Sigil already consumed. Read tight LABEL. */
    if (s->pos >= s->len || is_name_terminator((unsigned char)s->buf[s->pos])) {
        die("sigil '%c' not followed by label name", sigil);
    }
    name_len = scan_name(s, &name_start);

    /* Optional '-' OTHER or '>' OTHER (tight, no whitespace).
     * '>' is a synonym for '-', accepted for hex2 compatibility. */
    if (s->pos < s->len && (s->buf[s->pos] == '-' || s->buf[s->pos] == '>')) {
        s->pos++;
        if (s->pos >= s->len || is_name_terminator((unsigned char)s->buf[s->pos])) {
            die("'-' must be followed by label name");
        }
        other_len = scan_name(s, &other_start);
    }

    out_off = emit_zeros(si.width);
    record_fixup(name_start, name_len, other_start, other_len,
                 sigil, out_off, ip);
}

/* --- directives ------------------------------------------------------- */

static int read_directive_name(struct Scanner *s, char *out, int max)
{
    /* '.' already consumed. Read alpha chars. */
    int n = 0;
    while (s->pos < s->len) {
        int c = (unsigned char)s->buf[s->pos];
        if (!isalpha(c)) break;
        if (n >= max) die("directive name too long");
        out[n++] = (char)c;
        s->pos++;
    }
    if (n == 0) die("expected directive name after '.'");
    return n;
}

static void skip_inline_ws(struct Scanner *s)
{
    /* Directive arguments do NOT cross newlines: `.align N PATTERN` ends
     * at end-of-line, otherwise `.align 8\n cc` would slurp `cc` as
     * pattern. Skip space/tab and inline comments only. */
    int c;
    while (s->pos < s->len) {
        c = (unsigned char)s->buf[s->pos];
        if (c == ' ' || c == '\t' || c == '\r' || c == '\f' || c == '\v') {
            s->pos++;
        } else if (c == '#' || c == ';') {
            while (s->pos < s->len && s->buf[s->pos] != '\n') s->pos++;
        } else {
            break;
        }
    }
}

static void do_align(struct Scanner *s)
{
    long long N;
    long long pad;
    long long target;
    long long i;
    unsigned char patbuf[MAX_TOKEN];
    int patlen = 0;
    int has_pattern = 0;
    int c;

    skip_inline_ws(s);
    N = read_decimal(s);
    if (N <= 0 || (N & (N - 1)) != 0) {
        die(".align: N must be a positive power of two (got %lld)", N);
    }

    /* Optional pattern: peek -- if next non-ws is a byte digit, parse it. */
    skip_inline_ws(s);
    if (s->pos < s->len) {
        c = (unsigned char)s->buf[s->pos];
        if (is_byte_digit(c)) {
            parse_one_byte_literal(s, NULL, 1, patbuf, (int)sizeof(patbuf), &patlen);
            has_pattern = 1;
        }
    }

    target = (ip + N - 1) & ~(N - 1);
    pad = target - ip;
    if (pad <= 0) return;
    if (output_used + pad > MAX_OUTPUT_BYTES) die("output overflow");
    if (!has_pattern) {
        memset(output_buf + output_used, 0, (size_t)pad);
    } else {
        for (i = 0; i < pad; i++) {
            output_buf[output_used + i] = patbuf[i % patlen];
        }
    }
    output_used += pad;
    ip += pad;
}

static void do_fill(struct Scanner *s)
{
    long long N;
    unsigned char b;

    skip_inline_ws(s);
    N = read_decimal(s);
    if (N < 0) die(".fill: N must be non-negative (got %lld)", N);
    skip_inline_ws(s);
    parse_one_byte_literal(s, &b, 0, NULL, 0, NULL);
    if (N > 0) emit_fill(N, b);
}

static void do_ptrsize(struct Scanner *s)
{
    long long N;
    skip_inline_ws(s);
    N = read_decimal(s);
    if (N != 4 && N != 8) {
        die(".ptrsize: N must be 4 or 8 (got %lld)", N);
    }
    if (ptrsize_used && (int)N != ptrsize) {
        die(".ptrsize %lld conflicts with already-used width %d", N, ptrsize);
    }
    ptrsize = (int)N;
}

static void do_scope_open(void)
{
    if (scope_depth >= MAX_SCOPE_DEPTH) die(".scope: depth overflow");
    scope_seq++;
    scope_stack[scope_depth++] = scope_seq;
}

static void do_scope_close(void)
{
    if (scope_depth <= 0) die(".endscope: not in a scope");
    scope_depth--;
}

/* --- main scanner loop ------------------------------------------------ */

static void process_file(struct InFile *f)
{
    struct Scanner s = { f->buf, f->len, 0 };
    cur_path = f->path;
    cur_line = 1;

    for (;;) {
        int c;
        skip_ws_and_comments(&s);
        if (s.pos >= s.len) break;
        c = (unsigned char)s.buf[s.pos];

        if (c == ':') {
            const char *name;
            int n;
            int dotted;
            int scope;
            s.pos++;
            n = scan_name(&s, &name);
            /* A dot-prefixed name is scope-local only inside a .scope;
             * outside, it is an ordinary global name. */
            dotted = (n > 0 && name[0] == '.' && scope_depth > 0);
            scope = dotted ? scope_stack[scope_depth - 1] : 0;
            define_label(name, n, scope);
            continue;
        }

        if (c == '.') {
            char dn[MAX_TOKEN];
            int n;
            s.pos++;
            n = read_directive_name(&s, dn, sizeof(dn));
            if (n == 5 && memcmp(dn, "align", 5) == 0)        do_align(&s);
            else if (n == 4 && memcmp(dn, "fill", 4) == 0)    do_fill(&s);
            else if (n == 5 && memcmp(dn, "scope", 5) == 0)   do_scope_open();
            else if (n == 8 && memcmp(dn, "endscope", 8) == 0) do_scope_close();
            else if (n == 7 && memcmp(dn, "ptrsize", 7) == 0)  do_ptrsize(&s);
            else die("unknown directive '.%.*s'", n, dn);
            continue;
        }

        if (c == '!' || c == '@' || c == '$' || c == '~' || c == '%' || c == '&') {
            s.pos++;
            process_reference(&s, c);
            continue;
        }

        if (is_byte_digit(c)) {
            parse_byte_stream(&s);
            continue;
        }

        die("unexpected character 0x%02x ('%c')", c, isprint(c) ? c : '?');
    }
}

/* --- fixup resolution ------------------------------------------------- */

static void patch_fixups(void)
{
    int i;
    for (i = 0; i < fixup_count; i++) {
        struct Fixup *f = &fixups[i];
        struct SigilInfo si = sigil_info(f->sigil);
        const int *stack = (f->scope_depth > 0)
                         ? &scope_history[f->scope_hist_off]
                         : NULL;
        long long t_label;
        long long value;

        cur_path = f->src_path;
        cur_line = f->src_line;

        t_label = lookup_label_in(f->name, f->name_len, stack, f->scope_depth);
        if (f->other != NULL) {
            long long t_other = lookup_label_in(f->other, f->other_len,
                                                stack, f->scope_depth);
            value = t_label - t_other;
        } else if (si.is_rel) {
            value = t_label - f->ip_at_ref;
        } else {
            value = t_label + base_address;
        }
        write_value(f->out_off, value, si.width, si.lo, si.hi, si.range_check);
    }
}

/* --- argument parsing & top-level ------------------------------------- */

static long long parse_long(const char *s, const char *what)
{
    char *end;
    long long v;
    int base = 10;
    if (s[0] == '0' && (s[1] == 'x' || s[1] == 'X')) base = 16;
    errno = 0;
    v = strtoll(s, &end, base);
    if (errno != 0 || *end != '\0') {
        fprintf(stderr, "hex2pp: invalid %s: %s\n", what, s);
        exit(1);
    }
    return v;
}

static void load_input(const char *path)
{
    FILE *fp;
    long sz;
    char *buf;

    fp = fopen(path, "rb");
    if (fp == NULL) { perror(path); exit(1); }
    if (fseek(fp, 0, SEEK_END) != 0) { perror(path); exit(1); }
    sz = ftell(fp);
    if (sz < 0)               { perror(path); exit(1); }
    if (sz > MAX_INPUT_BYTES) { fprintf(stderr, "%s: input too large\n", path); exit(1); }
    rewind(fp);
    buf = (char *)malloc((size_t)sz + 1);
    if (buf == NULL) { fprintf(stderr, "out of memory\n"); exit(1); }
    if (sz > 0 && fread(buf, 1, (size_t)sz, fp) != (size_t)sz) {
        perror(path);
        exit(1);
    }
    buf[sz] = '\0';
    fclose(fp);

    input_file.path = path;
    input_file.buf  = buf;
    input_file.len  = (int)sz;
}

static void usage(const char *prog)
{
    fprintf(stderr,
        "usage: %s [-B ADDR] [-E|-e] [-b] [-N] IN OUT\n",
        prog);
}

int main(int argc, char **argv)
{
    int i;
    const char *in_path = NULL;

    for (i = 1; i < argc; i++) {
        const char *a = argv[i];
        if (strcmp(a, "-B") == 0) {
            if (++i >= argc) { usage(argv[0]); return 1; }
            base_address = parse_long(argv[i], "base address");
        } else if (strcmp(a, "-E") == 0) {
            big_endian = 1;
        } else if (strcmp(a, "-e") == 0) {
            big_endian = 0;
        } else if (strcmp(a, "-b") == 0) {
            byte_mode = BINARY_MODE;
        } else if (strcmp(a, "-N") == 0) {
            non_executable = 1;
        } else if (a[0] == '-' && a[1] != '\0') {
            fprintf(stderr, "hex2pp: unknown argument: %s\n", a);
            usage(argv[0]);
            return 1;
        } else if (in_path == NULL) {
            in_path = a;
        } else if (output_path == NULL) {
            output_path = a;
        } else {
            fprintf(stderr, "hex2pp: extra positional argument: %s\n", a);
            usage(argv[0]);
            return 1;
        }
    }

    if (in_path == NULL || output_path == NULL) {
        usage(argv[0]);
        return 1;
    }
    load_input(in_path);

    ip = 0;
    output_used = 0;
    scope_depth = 0;
    scope_seq = 0;
    ptrsize = 4;
    ptrsize_used = 0;
    process_file(&input_file);
    if (scope_depth != 0) die(".scope not closed at end of input");
    patch_fixups();

    /* Write output. */
    {
        FILE *fp = fopen(output_path, "wb");
        if (fp == NULL) { perror(output_path); return 1; }
        if (output_used > 0 &&
            fwrite(output_buf, 1, (size_t)output_used, fp) != (size_t)output_used) {
            perror(output_path);
            fclose(fp);
            return 1;
        }
        fclose(fp);
    }

    if (!non_executable) {
        struct stat st;
        if (stat(output_path, &st) == 0 && S_ISREG(st.st_mode)) {
            (void)chmod(output_path, 0750);
        }
    }

    return 0;
}
