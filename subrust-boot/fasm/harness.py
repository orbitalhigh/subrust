#!/usr/bin/env python3
# Differential harness: feed the same op+a+b corpus to the rustc reference
# (fref, exact BOOT_SHIMS bodies) and to the amd64 assembly backend, compare
# result bytes. Two NaN results count as equal (IEEE leaves NaN payloads
# unspecified, and Rust's f64 % uses fmod while the asm uses x87 fprem — only
# f_rem can differ, and only in the NaN payload). Everything else must be
# byte-identical.
import struct, subprocess, sys, math

OPS = list(range(9))  # 0..8
NAMES = ["f_add","f_sub","f_mul","f_div","f_rem","f_lt","f_eq","f_from_i","f_to_i"]

# a rich set of f64 bit-patterns AND integer patterns (reused for a and b).
def f(x):  # float -> bits
    return struct.unpack('<Q', struct.pack('<d', x))[0]

SPECIAL = [
    f(0.0), f(-0.0), f(1.0), f(-1.0), f(2.0), f(-2.0), f(0.5), f(-0.5),
    f(3.0), f(10.0), f(0.1), f(-0.1), f(100.0), f(1.5), f(-1.5),
    f(math.pi), f(math.e), f(math.inf), f(-math.inf), f(math.nan),
    f(1e308), f(-1e308), f(1e-308), f(5e-324), f(2.2250738585072014e-308),
    f(1.7976931348623157e308), f(-1.7976931348623157e308),
    f(9007199254740992.0), f(-9007199254740992.0),   # 2^53
    f(9223372036854775808.0), f(-9223372036854775808.0),  # 2^63
    f(1.8446744073709552e19),  # 2^64
    f(4.9), f(-4.9), f(2.5), f(-2.5), f(1234567.89), f(-9876.5),
    # integer patterns (matter for f_from_i, harmless elsewhere)
    0, 1, (1<<64)-1, (1<<63), (1<<63)-1, 2, (1<<64)-2,
    (1<<53), (1<<64)-(1<<53), 1000000, (1<<64)-1000000, 42, 0xdeadbeef,
    0x7ff0000000000001,  # signalling NaN
    0xfff8000000000000, 0x7ff8000000000000,  # quiet NaNs
    0x8000000000000001, 0x0000000000000001,  # tiny subnormals
]

def build_corpus():
    recs = bytearray()
    for op in OPS:
        for a in SPECIAL:
            for b in SPECIAL:
                recs += bytes([op]) + struct.pack('<Q', a) + struct.pack('<Q', b)
    # a pseudo-random sweep (deterministic LCG, no external deps)
    s = 0x123456789abcdef
    def nxt():
        nonlocal s
        s = (s * 6364136223846793005 + 1442695040888963407) & ((1<<64)-1)
        return s
    for _ in range(20000):
        op = nxt() % 9
        a = nxt(); b = nxt()
        recs += bytes([op]) + struct.pack('<Q', a) + struct.pack('<Q', b)
    return bytes(recs)

def is_nan_bits(u):
    # IEEE double NaN: exponent all ones, mantissa != 0
    return (u & 0x7ff0000000000000) == 0x7ff0000000000000 and (u & 0x000fffffffffffff) != 0

def main():
    ref_bin, asm_bin = sys.argv[1], sys.argv[2]
    corpus = build_corpus()
    n = len(corpus) // 17
    ref = subprocess.run([ref_bin], input=corpus, capture_output=True).stdout
    asm = subprocess.run([asm_bin], input=corpus, capture_output=True).stdout
    if len(ref) != 8*n or len(asm) != 8*n:
        print(f"LENGTH MISMATCH: n={n} ref={len(ref)} asm={len(asm)} (want {8*n})")
        sys.exit(1)
    mism = 0
    examples = []
    for k in range(n):
        rb = struct.unpack('<Q', ref[8*k:8*k+8])[0]
        ab = struct.unpack('<Q', asm[8*k:8*k+8])[0]
        if rb == ab:
            continue
        op = corpus[17*k]
        # NaN-payload leniency for the arithmetic ops (add/sub/mul/div/rem):
        # IEEE leaves NaN payloads unspecified, and for a commutative op like
        # fadd, LLVM may place either operand as the SSE destination, so Rust's
        # NaN payload can differ from the asm's fixed operand order. Both-NaN is
        # the IEEE-correct notion of equality here. f_lt/f_eq/f_from_i/f_to_i
        # never yield NaN, so they stay strictly bit-exact.
        if op <= 4 and is_nan_bits(rb) and is_nan_bits(ab):
            continue
        mism += 1
        if len(examples) < 25:
            a = struct.unpack('<Q', corpus[17*k+1:17*k+9])[0]
            b = struct.unpack('<Q', corpus[17*k+9:17*k+17])[0]
            examples.append((NAMES[op], a, b, rb, ab))
    print(f"records: {n}   mismatches: {mism}")
    for nm,a,b,rb,ab in examples:
        print(f"  {nm}  a={a:#018x} b={b:#018x}  ref={rb:#018x} asm={ab:#018x}")
    sys.exit(0 if mism == 0 else 1)

main()
