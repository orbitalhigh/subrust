// sr1i.rs — a meta-circular interpreter for SR-seed, WRITTEN IN SR-seed
// (the two-level interpreter stack). It reads a checked
// program image (produced by `subrust emit`) from stdin, then evaluates it —
// so the stack is seed -> sr0i -> sr1i -> program.
//
// sr1i is itself SR-seed, hence a valid Rust program (verifiable under rustc
// with the BOOT shims) AND runnable on sr0i. It has no `return` (not in the
// subset): every function is a tail expression / if-else chain. Guest u64
// traps are PREDICTED (checked before computing, so sr0i never traps sr1i on
// the guest's behalf) and signalled by inducing a real trap (1/0), so the
// process exits nonzero with the correct output prefix.
//
// Values live on an operand stack (VSTACK): `eval` PUSHES a value's slots and
// callers pop — the machine.rs model, so multi-slot values (structs/arrays, on
// later rungs) work. The compute core is all one-slot; a block discards each
// statement's value by resetting the stack top, so no per-node size is needed
// until multi-slot lands.
//
// Scope (this rung): the compute/control/recursion core + putb. getb/ld/st/
// f_* and the dialect (structs/arrays/slices/match/...) are the B4 growth work.
//
// Memory map (word addresses in the ld/st space):
//   [0 .. 16)     registers (R_*)
//   [NODES ..)    node pool, 7 words/node
//   [VALS  ..)    literal values, 1 word each
//   [FNS   ..)    fn table, 3 words/fn
//   [STACK ..)    call frames (locals), 1 word/slot
//   [VSTACK ..)   operand stack (values in flight), 1 word/slot
//   [POOL ..)     string pool, bytes packed 8/word (byte-string b"..." data)

const R_FRAME: u64 = 0; // current frame base (slot index into STACK)
const R_SIG: u64 = 1; // 0 normal, 1 break, 2 continue
const R_VN: u64 = 2; // value-stack top (operand stack, slot index into VSTACK)
const R_ENTRY: u64 = 5;
const R_FSIZE: u64 = 6; // current frame size (slots)
const R_RETN: u64 = 7; // return-value slot count (paired with the RETVAL buffer)

const NODES: u64 = 16;
const VALS: u64 = 200016;
const FNS: u64 = 260016;
const STACK: u64 = 300016;   // call frames (locals), 1 word/slot
const VSTACK: u64 = 800016;  // operand stack, 1 word/slot — values in flight
const POOL: u64 = 1000016;   // string pool, bytes packed 8/word (< MEM_WORDS 2^20)
const STRS: u64 = 1010016;   // &str table: (pool off, byte len) per id, 2 words/entry
const RETVAL: u64 = 1013016; // early-return value buffer (survives block/loop stack resets)

const NIL: u64 = 18446744073709551615; // u64::MAX
const POOL_TAG: u64 = 9223372036854775808; // 1<<63: slice addr points into POOL

// node kinds (must match subrust-cli/src/emit.rs)
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
const KS: u64 = 16; // struct literal: a=first field-init(KFI), b=struct size
const KD: u64 = 17; // field access: a=offset, b=field size, c=base, d=base size
// KFI (18) field-init nodes are walked only by KS, never eval'd directly
const KAR: u64 = 19; // array literal: a=first element (link chain)
const KRP: u64 = 20; // array repeat [x;N]: a=elem size, b=count, c=elem
const KIX: u64 = 21; // index arr[i]: a=elem size, b=len, c=base, d=index
const KRF: u64 = 22; // &local: a=slot -> absolute frame address STACK+frame+slot
const KDR: u64 = 23; // *p read: a=pointee size, c=ref expr
const KAD: u64 = 24; // *p = v: b=pointee size, c=ref expr, d=value
const KSL: u64 = 25; // &arr[lo..hi]: x=hi expr|NIL, a=elem size, b=static len, c=base addr (KRF), d=lo expr
const KSX: u64 = 26; // slice index s[i]: a=elem size, c=base slice [addr,len], d=index expr
const KSN: u64 = 27; // slice .len(): c=base slice [addr,len]
const KBS: u64 = 28; // byte-string b"...": a=first val idx -> pushes [POOL_TAG|off, len]
const KSTRL: u64 = 29; // &str .len(): c=receiver (pushes string id)
const KSTRB: u64 = 30; // &str .as_bytes(): c=receiver -> pushes [POOL_TAG|off, len]
const KM: u64 = 31; // match: a=first arm (KMA chain), c=scrutinee (scalar)
const KMA: u64 = 32; // match arm: a=first pattern (KMP chain), c=body expr
const KMP: u64 = 33; // match pattern: x=1 wildcard, else a=val index of the literal
const KCAST: u64 = 34; // `as` int cast: x=signed-target, a=target bit width, c=operand
const KWRAP: u64 = 35; // wrapping_*/rotate_* prim method: x=prim op, a=width, b=signed, c=recv, d=arg
// prim op ids (must match check.rs PRIM_*)
const PRIM_WADD: u64 = 1;
const PRIM_WSUB: u64 = 2;
const PRIM_WMUL: u64 = 3;
const PRIM_WNEG: u64 = 4;
const PRIM_WSHL: u64 = 5;
const PRIM_SADD: u64 = 6; // saturating_add (unsigned only on the chain)
const PRIM_SMUL: u64 = 7; // saturating_mul (unsigned only on the chain)
const PRIM_ROTL: u64 = 10;
const PRIM_ROTR: u64 = 11;
const KCONST: u64 = 36; // aggregate const: a=first val index, b=size -> pushes vals[a..a+size]
const KFADDR: u64 = 37; // field address: a=off, c=base addr -> addr+off
const KIXA: u64 = 38; // index address (static len): a=es, b=len, c=base addr, d=idx
const KIXA_SL: u64 = 39; // index address (slice): a=es, c=fat ptr [addr,len], d=idx
const KDFLD: u64 = 40; // field read through ref: a=off, b=size, c=base ref -> load off..+size
const KIXR: u64 = 41; // index read through ref (static len): a=es, b=len, c=base ref, d=idx
const KRET: u64 = 42; // return: b=value size (0=bare), c=value expr|NIL -> RETVAL buffer + sig 3
const KSL_SL: u64 = 43; // sub-slice a slice: x=hi|NIL, a=es, c=base fat ptr [addr,len], d=lo
const KF64: u64 = 44; // f64 binary op: x=op, c=lhs bits, d=rhs bits (via f_* host fns)
const KFNEG: u64 = 45; // f64 negation: c=operand bits -> flip sign bit
const KFNAN: u64 = 46; // f64 is_nan: c=operand bits -> 1 if NaN else 0
const KTRAP: u64 = 47; // unrepresentable 128-bit value/op: trap if ever reached (dead on the chain)

// operators (subrust OP_*)
const OP_NOT: u64 = 2;
const OP_ADD: u64 = 3;
const OP_SUB: u64 = 4;
const OP_MUL: u64 = 5;
const OP_DIV: u64 = 6;
const OP_REM: u64 = 7;
const OP_AND: u64 = 8;
const OP_OR: u64 = 9;
const OP_BAND: u64 = 10;
const OP_BOR: u64 = 11;
const OP_BXOR: u64 = 12;
const OP_SHL: u64 = 13;
const OP_SHR: u64 = 14;
const OP_EQ: u64 = 15;
const OP_NE: u64 = 16;
const OP_LT: u64 = 17;
const OP_LE: u64 = 18;
const OP_GT: u64 = 19;
const OP_GE: u64 = 20;

// read one little-endian u64 from stdin
fn getw() -> u64 {
    let mut w: u64 = 0;
    let mut i: u64 = 0;
    while i < 8 {
        w = w | (getb() << (i * 8));
        i = i + 1;
    }
    w
}

// node field: (kind,x,a,b,c,d,link) at NODES + idx*7 + f
fn nf(idx: u64, f: u64) -> u64 {
    ld(NODES + idx * 7 + f)
}

fn sig() -> u64 {
    ld(R_SIG)
}

fn load_image() {
    let _magic = getw();
    let nfns = getw();
    let entry = getw();
    let nnodes = getw();
    let nvals = getw();
    let npool = getw(); // string-pool byte count
    let nstrs = getw(); // string-table entry count
    st(R_ENTRY, entry);
    let mut i: u64 = 0;
    while i < nfns * 3 {
        st(FNS + i, getw());
        i = i + 1;
    }
    let mut j: u64 = 0;
    while j < nvals {
        st(VALS + j, getw());
        j = j + 1;
    }
    let mut k: u64 = 0;
    while k < nnodes * 7 {
        st(NODES + k, getw());
        k = k + 1;
    }
    // string pool: ceil(npool/8) packed words
    let nwords = (npool + 7) / 8;
    let mut p: u64 = 0;
    while p < nwords {
        st(POOL + p, getw());
        p = p + 1;
    }
    // string table: nstrs entries of (off, len)
    let mut q: u64 = 0;
    while q < nstrs * 2 {
        st(STRS + q, getw());
        q = q + 1;
    }
}

// one byte of the packed string pool by absolute byte offset
fn poolb(off: u64) -> u64 {
    let word = ld(POOL + off / 8);
    (word >> ((off & 7) * 8)) & 255
}

// induce a real trap (guest overflow/div0/etc.) -> process exits nonzero
fn guest_trap() -> u64 {
    let z: u64 = 0;
    1 / z
}

// truncate/sign-extend v to b bits (== check.rs mask_to); shared by KCAST/KWRAP
fn pmask(v: u64, b: u64, signed: u64) -> u64 {
    if b >= 64 {
        v
    } else {
        let m = (1 << b) - 1;
        let x = v & m;
        if signed == 1 {
            if (x >> (b - 1)) & 1 == 1 {
                x | (NIL ^ m)
            } else {
                x
            }
        } else {
            x
        }
    }
}

// (a + b) mod 2^64 without tripping sr0i's overflow trap
fn wadd(a: u64, b: u64) -> u64 {
    if a > NIL - b {
        a - (NIL - b) - 1
    } else {
        a + b
    }
}

// (a - b) mod 2^64 without tripping sr0i's underflow trap
fn wsub(a: u64, b: u64) -> u64 {
    if a >= b {
        a - b
    } else {
        a + ((NIL - b) + 1)
    }
}

// low 64 bits of a*b (schoolbook on 32-bit halves; no partial product overflows)
fn wmul(a: u64, b: u64) -> u64 {
    let ah = a >> 32;
    let al = a & 4294967295;
    let bh = b >> 32;
    let bl = b & 4294967295;
    let lo = al * bl;
    let mid = wadd(al * bh, ah * bl); // ah*bh*2^64 vanishes mod 2^64
    let midlo = (mid & 4294967295) << 32;
    wadd(lo, midlo)
}

// wrapping_shl: shift amount is taken mod the type width
fn wshl(a: u64, b: u64, width: u64) -> u64 {
    a << (b % width)
}

// unsigned saturating add: clamp to the type's max (2^width - 1)
fn sat_add_u(a: u64, b: u64, width: u64) -> u64 {
    if width >= 64 {
        if a > NIL - b {
            NIL
        } else {
            a + b
        }
    } else {
        let m = (1 << width) - 1;
        let s = a + b; // width <= 32 here, so a + b never overflows u64
        if s > m {
            m
        } else {
            s
        }
    }
}

// unsigned saturating mul: clamp to the type's max (2^width - 1)
fn sat_mul_u(a: u64, b: u64, width: u64) -> u64 {
    if width >= 64 {
        if b == 0 {
            0
        } else if a > NIL / b {
            NIL
        } else {
            a * b
        }
    } else {
        let m = (1 << width) - 1;
        let p = a * b; // width <= 32 here, so a * b never overflows u64
        if p > m {
            m
        } else {
            p
        }
    }
}

// rotate within `width` bits; right == 1 for rotate_right
fn wrot(a: u64, b: u64, width: u64, right: u64) -> u64 {
    let mut sh = b % width;
    if right == 1 {
        sh = (width - sh) % width;
    }
    let m = if width >= 64 { NIL } else { (1 << width) - 1 };
    let v = a & m;
    (v << sh) | (v >> ((width - sh) % width))
}

// operand stack: eval pushes a value's slots here; callers pop. Multi-slot
// values (structs/arrays, later rungs) push several slots; the compute core is
// all one-slot (scalars) or zero-slot (unit statements).
fn vpush(v: u64) -> u64 {
    st(VSTACK + ld(R_VN), v);
    st(R_VN, ld(R_VN) + 1);
    0
}
fn vpop() -> u64 {
    st(R_VN, ld(R_VN) - 1);
    ld(VSTACK + ld(R_VN))
}

// u64 binary op with PREDICTED overflow (never triggers sr0i's own traps)
fn do_bin(op: u64, a: u64, b: u64) -> u64 {
    if op == OP_ADD {
        if a > NIL - b { guest_trap() } else { a + b }
    } else if op == OP_SUB {
        if a < b { guest_trap() } else { a - b }
    } else if op == OP_MUL {
        if a == 0 {
            0
        } else if b > NIL / a {
            guest_trap()
        } else {
            a * b
        }
    } else if op == OP_DIV {
        if b == 0 { guest_trap() } else { a / b }
    } else if op == OP_REM {
        if b == 0 { guest_trap() } else { a % b }
    } else if op == OP_BAND {
        a & b
    } else if op == OP_BOR {
        a | b
    } else if op == OP_BXOR {
        a ^ b
    } else if op == OP_SHL {
        if b >= 64 { guest_trap() } else { a << b }
    } else if op == OP_SHR {
        if b >= 64 { guest_trap() } else { a >> b }
    } else if op == OP_EQ {
        if a == b { 1 } else { 0 }
    } else if op == OP_NE {
        if a != b { 1 } else { 0 }
    } else if op == OP_LT {
        if a < b { 1 } else { 0 }
    } else if op == OP_LE {
        if a <= b { 1 } else { 0 }
    } else if op == OP_GT {
        if a > b { 1 } else { 0 }
    } else if op == OP_GE {
        if a >= b { 1 } else { 0 }
    } else {
        guest_trap()
    }
}

// evaluate a binary node (handles short-circuit && ||); leaves 1 slot on VSTACK
fn eval_bin(node: u64) -> u64 {
    let op = nf(node, 1);
    if op == OP_AND {
        eval(nf(node, 4));
        if sig() != 0 {
            0
        } else {
            let a = vpop();
            if a == 0 { vpush(0) } else { eval(nf(node, 5)) } // eval(b) leaves b on VSTACK
        }
    } else if op == OP_OR {
        eval(nf(node, 4));
        if sig() != 0 {
            0
        } else {
            let a = vpop();
            if a != 0 { vpush(1) } else { eval(nf(node, 5)) }
        }
    } else {
        eval(nf(node, 4));
        eval(nf(node, 5));
        if sig() != 0 {
            0
        } else {
            let b = vpop();
            let a = vpop();
            vpush(do_bin(op, a, b))
        }
    }
}

// evaluate a block: run the stmt chain (stopping on a signal), then the tail.
// Each statement's value (if any) is discarded by resetting the operand stack
// to the pre-statement level — so no per-statement value size is needed.
fn eval_block(node: u64) -> u64 {
    let mut s = nf(node, 2);
    while s != NIL {
        let lvl = ld(R_VN);
        eval(s);
        st(R_VN, lvl); // discard whatever the statement pushed
        if sig() != 0 {
            s = NIL;
        } else {
            s = nf(s, 6);
        }
    }
    let tail = nf(node, 3);
    if sig() != 0 {
        0
    } else if tail != NIL {
        eval(tail) // the tail's value is the block's value (left on VSTACK)
    } else {
        0
    }
}

// a `while`/`loop` body iteration is shared: run body, fold the break/continue
// signal. returns 1 if the loop should stop (break), 0 to keep going.
fn run_loop_body(body: u64) -> u64 {
    let lvl = ld(R_VN);
    eval(body);
    st(R_VN, lvl); // a loop body is unit; drop anything it left
    let s = sig();
    if s == 1 {
        st(R_SIG, 0);
        1
    } else if s == 2 {
        st(R_SIG, 0);
        0
    } else if s == 3 {
        1 // return: stop the loop but PRESERVE the signal so it propagates
    } else {
        0
    }
}

fn eval_while(node: u64) -> u64 {
    let mut go: u64 = 1;
    while go == 1 {
        eval(nf(node, 4));
        if sig() != 0 {
            go = 0;
        } else if vpop() == 0 {
            go = 0;
        } else if run_loop_body(nf(node, 5)) == 1 {
            go = 0;
        }
    }
    0
}

fn eval_loop(node: u64) -> u64 {
    let mut go: u64 = 1;
    while go == 1 {
        if run_loop_body(nf(node, 5)) == 1 {
            go = 0;
        }
    }
    0
}

// call a user function: new frame above the caller's live slots
fn eval_call(node: u64) -> u64 {
    let f = nf(node, 2);
    let np = ld(FNS + f * 3 + 2);
    let body = ld(FNS + f * 3);
    let frame = ld(FNS + f * 3 + 1);
    let new_base = ld(R_FRAME) + ld(R_FSIZE);
    // Evaluate ALL args onto VSTACK (each pushes its own slot count — a scalar
    // is 1 slot, a slice fat pointer 2, a struct N), then copy the whole
    // contiguous block into the callee frame's param region. Args are evaluated
    // left-to-right in the CALLER's frame, so the VSTACK block already matches
    // the checker's cumulative-size param slot layout.
    let vbase = ld(R_VN);
    let mut a = nf(node, 3);
    let mut i: u64 = 0;
    while i < np {
        eval(a); // push the arg value (its slots)
        if sig() != 0 {
            i = np; // abort on signal (shouldn't happen in a call arg)
        } else {
            a = nf(a, 6);
            i = i + 1;
        }
    }
    if sig() != 0 {
        0
    } else {
        let total = ld(R_VN) - vbase; // total param slots pushed
        let mut k: u64 = 0;
        while k < total {
            st(STACK + new_base + k, ld(VSTACK + vbase + k));
            k = k + 1;
        }
        st(R_VN, vbase); // pop all arg slots off VSTACK
        let saved_base = ld(R_FRAME);
        let saved_size = ld(R_FSIZE);
        st(R_FRAME, new_base);
        st(R_FSIZE, frame);
        eval(body); // leaves the return value on VSTACK (unless an early return)
        if ld(R_SIG) == 3 {
            // early `return`: discard partial vstack, replay the RETVAL buffer at
            // the body's result level, and clear the signal (caught at the boundary)
            st(R_VN, vbase);
            let rn = ld(R_RETN);
            let mut r: u64 = 0;
            while r < rn {
                vpush(ld(RETVAL + r));
                r = r + 1;
            }
            st(R_SIG, 0);
        }
        st(R_FRAME, saved_base);
        st(R_FSIZE, saved_size);
        0
    }
}

// assemble a struct literal: reserve `size` slots on VSTACK, then evaluate each
// field init (which pushes above) and copy it down into its offset.
fn eval_struct(node: u64) -> u64 {
    let sbase = ld(R_VN);
    let ssize = nf(node, 3); // b = struct size
    st(R_VN, sbase + ssize); // reserve the struct's slots
    let mut fi = nf(node, 2); // a = first field-init (KFI)
    while fi != NIL {
        eval(nf(fi, 4)); // c = init expr, pushes `fsize` slots above sbase+ssize
        if sig() != 0 {
            fi = NIL;
        } else {
            let off = nf(fi, 2);
            let mut k = nf(fi, 3); // fsize
            while k > 0 {
                k = k - 1;
                st(VSTACK + sbase + off + k, vpop());
            }
            fi = nf(fi, 6); // link to next field-init
        }
    }
    0
}

fn eval(node: u64) -> u64 {
    if node == NIL {
        0
    } else if sig() != 0 {
        0
    } else {
        let kind = nf(node, 0);
        if kind == KL {
            vpush(ld(VALS + nf(node, 2)))
        } else if kind == KBS {
            // byte-string literal: push the precomputed [POOL_TAG|off, len]
            let a = nf(node, 2);
            vpush(ld(VALS + a));
            vpush(ld(VALS + a + 1))
        } else if kind == KCONST {
            // aggregate const: push `size` slots from the val table
            let a = nf(node, 2);
            let size = nf(node, 3);
            let mut k: u64 = 0;
            while k < size {
                vpush(ld(VALS + a + k));
                k = k + 1;
            }
            0
        } else if kind == KN {
            // load `size` slots from the local's frame region onto VSTACK
            let base = STACK + ld(R_FRAME) + nf(node, 2);
            let size = nf(node, 3);
            let mut k: u64 = 0;
            while k < size {
                vpush(ld(base + k));
                k = k + 1;
            }
            0
        } else if kind == KU {
            eval(nf(node, 4));
            if sig() != 0 {
                0
            } else {
                let v = vpop();
                if v == 0 {
                    vpush(1)
                } else if v == 1 {
                    vpush(0)
                } else {
                    vpush(NIL ^ v)
                }
            }
        } else if kind == KB {
            eval_bin(node)
        } else if kind == KI {
            eval(nf(node, 4));
            if sig() != 0 {
                0
            } else if vpop() != 0 {
                eval(nf(node, 5))
            } else {
                eval(nf(node, 2)) // else node or NIL (eval(NIL) pushes nothing)
            }
        } else if kind == KK {
            eval_block(node)
        } else if kind == KE {
            eval(nf(node, 4)); // inner value pushed; the block discards it
            0
        } else if kind == KT {
            eval(nf(node, 4)); // init pushes `size` slots
            if sig() == 0 {
                let base = STACK + ld(R_FRAME) + nf(node, 2);
                let mut k = nf(node, 3); // size
                while k > 0 {
                    k = k - 1;
                    st(base + k, vpop()); // top slot -> highest offset first
                }
            }
            0
        } else if kind == KA {
            eval(nf(node, 4));
            if sig() == 0 {
                let base = STACK + ld(R_FRAME) + nf(node, 2);
                let op = nf(node, 1);
                if op == 0 {
                    let mut k = nf(node, 3); // size
                    while k > 0 {
                        k = k - 1;
                        st(base + k, vpop());
                    }
                } else {
                    // compound assignment is scalar (one slot)
                    let v = vpop();
                    st(base, do_bin(op, ld(base), v));
                }
            }
            0
        } else if kind == KW {
            eval_while(node)
        } else if kind == KO {
            eval_loop(node)
        } else if kind == KR {
            st(R_SIG, 1);
            0
        } else if kind == KC {
            st(R_SIG, 2);
            0
        } else if kind == KH {
            if nf(node, 1) == 2 {
                // getb(): read one byte (0 args); u64::MAX on EOF
                vpush(getb())
            } else {
                // putb(byte): evaluate the single argument, output its low byte
                eval(nf(node, 3));
                if sig() == 0 {
                    putb(vpop());
                }
                0
            }
        } else if kind == KF {
            eval_call(node)
        } else if kind == KRET {
            eval(nf(node, 4)); // c = value expr, or NIL for bare `return`
            if sig() != 0 {
                0
            } else {
                // stash the return value in RETVAL (block/loop resets can't clobber it)
                let size = nf(node, 3);
                let mut k = size;
                while k > 0 {
                    k = k - 1;
                    st(RETVAL + k, vpop());
                }
                st(R_RETN, size);
                st(R_SIG, 3); // return signal, caught at the enclosing eval_call
                0
            }
        } else if kind == KRF {
            // &local: value is the local's absolute word address in STACK
            vpush(STACK + ld(R_FRAME) + nf(node, 2))
        } else if kind == KDR {
            eval(nf(node, 4)); // c = ref expr, pushes the address (1 slot)
            if sig() != 0 {
                0
            } else {
                let addr = vpop();
                let size = nf(node, 2);
                let mut k: u64 = 0;
                while k < size {
                    vpush(ld(addr + k));
                    k = k + 1;
                }
                0
            }
        } else if kind == KAD {
            eval(nf(node, 4)); // c = addr expr, push address
            if sig() != 0 {
                0
            } else {
                let op = nf(node, 1); // 0 = plain store, else compound op
                if op == 0 {
                    eval(nf(node, 5)); // d = value, push `size` slots above the address
                    if sig() != 0 {
                        0
                    } else {
                        let size = nf(node, 3);
                        let addr = ld(VSTACK + ld(R_VN) - size - 1); // address below the value
                        let mut k = size;
                        while k > 0 {
                            k = k - 1;
                            st(addr + k, vpop());
                        }
                        vpop(); // drop the address slot
                        0
                    }
                } else {
                    // compound `*place op= rhs`: scalar read-modify-write
                    let addr = ld(VSTACK + ld(R_VN) - 1); // peek the address
                    let cur = ld(addr);
                    eval(nf(node, 5)); // rhs, push 1 slot
                    if sig() != 0 {
                        0
                    } else {
                        let rhs = vpop();
                        st(addr, do_bin(op, cur, rhs));
                        vpop(); // drop the address slot
                        0
                    }
                }
            }
        } else if kind == KS {
            eval_struct(node)
        } else if kind == KAR {
            // array literal: eval each element in order — they push the layout
            let mut el = nf(node, 2);
            while el != NIL {
                eval(el);
                if sig() != 0 {
                    el = NIL;
                } else {
                    el = nf(el, 6); // link
                }
            }
            0
        } else if kind == KRP {
            eval(nf(node, 4)); // c = elem, push `esize` slots
            if sig() != 0 {
                0
            } else {
                let esize = nf(node, 2);
                let count = nf(node, 3);
                let base = ld(R_VN) - esize; // the first copy on VSTACK
                let mut r: u64 = 1;
                while r < count {
                    let mut k: u64 = 0;
                    while k < esize {
                        vpush(ld(VSTACK + base + k));
                        k = k + 1;
                    }
                    r = r + 1;
                }
                0
            }
        } else if kind == KIX {
            eval(nf(node, 4)); // c = base, pushes len*esize slots (the array)
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // d = index, pushes 1
                if sig() != 0 {
                    0
                } else {
                    let idx = vpop();
                    let esize = nf(node, 2);
                    let len = nf(node, 3);
                    if idx >= len {
                        guest_trap()
                    } else {
                        let astart = ld(R_VN) - len * esize;
                        let estart = astart + idx * esize;
                        let mut k: u64 = 0;
                        while k < esize {
                            st(VSTACK + astart + k, ld(VSTACK + estart + k));
                            k = k + 1;
                        }
                        st(R_VN, astart + esize); // keep just the element
                        0
                    }
                }
            }
        } else if kind == KD {
            eval(nf(node, 4)); // c = base, pushes `bsize` slots
            if sig() != 0 {
                0
            } else {
                let bsize = nf(node, 5);
                let off = nf(node, 2);
                let fsize = nf(node, 3);
                let sstart = ld(R_VN) - bsize; // struct base on VSTACK
                let fstart = sstart + off;
                let mut k: u64 = 0;
                while k < fsize {
                    st(VSTACK + sstart + k, ld(VSTACK + fstart + k));
                    k = k + 1;
                }
                st(R_VN, sstart + fsize); // keep just the field's slots
                0
            }
        } else if kind == KSL {
            eval(nf(node, 4)); // c = base addr (KRF), pushes 1 slot
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // d = lo, pushes 1 slot
                if sig() != 0 {
                    0
                } else {
                    let hinode = nf(node, 1); // x = hi expr, or NIL (open-ended)
                    eval(hinode); // pushes 1 slot, or nothing when NIL
                    if sig() != 0 {
                        0
                    } else {
                        let len = nf(node, 3); // static array length
                        let es = nf(node, 2);
                        let hi = if hinode == NIL { len } else { vpop() };
                        let lo = vpop();
                        let addr = vpop();
                        if lo > hi {
                            guest_trap()
                        } else if hi > len {
                            guest_trap()
                        } else {
                            vpush(addr + lo * es);
                            vpush(hi - lo);
                            0
                        }
                    }
                }
            }
        } else if kind == KSX {
            eval(nf(node, 4)); // c = base slice, pushes [addr, len]
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // d = index, pushes 1 slot
                if sig() != 0 {
                    0
                } else {
                    let idx = vpop();
                    let len = vpop();
                    let addr = vpop();
                    let es = nf(node, 2);
                    if idx >= len {
                        guest_trap()
                    } else if addr >= POOL_TAG {
                        // byte-string slice: one u8 from the packed pool (es==1)
                        vpush(poolb((addr ^ POOL_TAG) + idx))
                    } else {
                        let src = addr + idx * es;
                        let mut k: u64 = 0;
                        while k < es {
                            vpush(ld(src + k));
                            k = k + 1;
                        }
                        0
                    }
                }
            }
        } else if kind == KFADDR {
            eval(nf(node, 4)); // c = base (addr-yielding), pushes 1 addr slot
            if sig() != 0 {
                0
            } else {
                let addr = vpop();
                vpush(addr + nf(node, 2)) // + field offset
            }
        } else if kind == KIXA {
            eval(nf(node, 4)); // c = base addr
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // d = index
                if sig() != 0 {
                    0
                } else {
                    let idx = vpop();
                    let addr = vpop();
                    let len = nf(node, 3);
                    if idx >= len {
                        guest_trap()
                    } else {
                        vpush(addr + idx * nf(node, 2)) // + idx*es
                    }
                }
            }
        } else if kind == KIXA_SL {
            eval(nf(node, 4)); // c = fat ptr, pushes [addr, len]
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // d = index
                if sig() != 0 {
                    0
                } else {
                    let idx = vpop();
                    let len = vpop();
                    let addr = vpop();
                    if idx >= len {
                        guest_trap()
                    } else {
                        vpush(addr + idx * nf(node, 2))
                    }
                }
            }
        } else if kind == KDFLD {
            eval(nf(node, 4)); // c = base ref, pushes the address
            if sig() != 0 {
                0
            } else {
                let src = vpop() + nf(node, 2); // addr + off
                let size = nf(node, 3);
                let mut k: u64 = 0;
                while k < size {
                    vpush(ld(src + k));
                    k = k + 1;
                }
                0
            }
        } else if kind == KIXR {
            eval(nf(node, 4)); // c = base ref -> array address
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // d = index
                if sig() != 0 {
                    0
                } else {
                    let idx = vpop();
                    let addr = vpop();
                    let len = nf(node, 3);
                    let es = nf(node, 2);
                    if idx >= len {
                        guest_trap()
                    } else {
                        let src = addr + idx * es;
                        let mut k: u64 = 0;
                        while k < es {
                            vpush(ld(src + k));
                            k = k + 1;
                        }
                        0
                    }
                }
            }
        } else if kind == KSL_SL {
            eval(nf(node, 4)); // c = base slice -> [addr, len]
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // d = lo
                if sig() != 0 {
                    0
                } else {
                    let hinode = nf(node, 1);
                    eval(hinode); // hi, or nothing when NIL (open-ended)
                    if sig() != 0 {
                        0
                    } else {
                        let es = nf(node, 2);
                        if hinode == NIL {
                            let lo = vpop();
                            let len = vpop();
                            let addr = vpop();
                            if lo > len {
                                guest_trap()
                            } else {
                                vpush(addr + lo * es);
                                vpush(len - lo);
                                0
                            }
                        } else {
                            let hi = vpop();
                            let lo = vpop();
                            let len = vpop();
                            let addr = vpop();
                            if lo > hi {
                                guest_trap()
                            } else if hi > len {
                                guest_trap()
                            } else {
                                vpush(addr + lo * es);
                                vpush(hi - lo);
                                0
                            }
                        }
                    }
                }
            }
        } else if kind == KSN {
            eval(nf(node, 4)); // c = base slice, pushes [addr, len]
            if sig() != 0 {
                0
            } else {
                let len = vpop();
                vpop(); // drop addr
                vpush(len)
            }
        } else if kind == KSTRL {
            eval(nf(node, 4)); // c = receiver, pushes the 1-slot string id
            if sig() != 0 {
                0
            } else {
                let id = vpop();
                vpush(ld(STRS + id * 2 + 1)) // byte length
            }
        } else if kind == KSTRB {
            eval(nf(node, 4)); // c = receiver, pushes the 1-slot string id
            if sig() != 0 {
                0
            } else {
                let id = vpop();
                vpush(POOL_TAG + ld(STRS + id * 2)); // [POOL_TAG|off,
                vpush(ld(STRS + id * 2 + 1)) //          len]
            }
        } else if kind == KCAST {
            eval(nf(node, 4)); // c = operand, pushes 1 slot
            if sig() != 0 {
                0
            } else {
                let v = vpop();
                let b = nf(node, 2); // target bit width
                if b >= 64 {
                    vpush(v)
                } else {
                    let m = (1 << b) - 1; // low b bits (b < 64, no overflow)
                    let x = v & m;
                    if nf(node, 1) == 1 {
                        // signed target: sign-extend when the top bit is set
                        if (x >> (b - 1)) & 1 == 1 {
                            vpush(x | (NIL ^ m))
                        } else {
                            vpush(x)
                        }
                    } else {
                        vpush(x)
                    }
                }
            }
        } else if kind == KWRAP {
            eval(nf(node, 4)); // c = receiver, pushes 1 slot
            if sig() != 0 {
                0
            } else {
                let op = nf(node, 1);
                let width = nf(node, 2);
                let signed = nf(node, 3);
                if op == PRIM_WNEG {
                    let a = vpop();
                    vpush(pmask(wsub(0, a), width, signed))
                } else {
                    eval(nf(node, 5)); // d = arg, pushes 1 slot
                    if sig() != 0 {
                        0
                    } else {
                        let b = vpop();
                        let a = vpop();
                        let r = if op == PRIM_WADD {
                            wadd(a, b)
                        } else if op == PRIM_WSUB {
                            wsub(a, b)
                        } else if op == PRIM_WMUL {
                            wmul(a, b)
                        } else if op == PRIM_WSHL {
                            wshl(a, b, width)
                        } else if op == PRIM_SADD {
                            sat_add_u(a, b, width)
                        } else if op == PRIM_SMUL {
                            sat_mul_u(a, b, width)
                        } else if op == PRIM_ROTL {
                            wrot(a, b, width, 0)
                        } else {
                            wrot(a, b, width, 1)
                        };
                        vpush(pmask(r, width, signed))
                    }
                }
            }
        } else if kind == KF64 {
            eval(nf(node, 4)); // lhs f64 bits
            if sig() != 0 {
                0
            } else {
                eval(nf(node, 5)); // rhs f64 bits
                if sig() != 0 {
                    0
                } else {
                    let b = vpop();
                    let a = vpop();
                    let op = nf(node, 1);
                    // arithmetic returns bits; comparisons return 0/1. sr0i's C
                    // prototype has no f_*, so this only runs under rustc-sr1i.
                    if op == OP_ADD {
                        vpush(f_add(a, b))
                    } else if op == OP_SUB {
                        vpush(f_sub(a, b))
                    } else if op == OP_MUL {
                        vpush(f_mul(a, b))
                    } else if op == OP_DIV {
                        vpush(f_div(a, b))
                    } else if op == OP_REM {
                        vpush(f_rem(a, b))
                    } else if op == OP_EQ {
                        if f_eq(a, b) {
                            vpush(1)
                        } else {
                            vpush(0)
                        }
                    } else if op == OP_NE {
                        if f_eq(a, b) {
                            vpush(0)
                        } else {
                            vpush(1)
                        }
                    } else if op == OP_LT {
                        if f_lt(a, b) {
                            vpush(1)
                        } else {
                            vpush(0)
                        }
                    } else if op == OP_LE {
                        if f_lt(a, b) {
                            vpush(1)
                        } else if f_eq(a, b) {
                            vpush(1)
                        } else {
                            vpush(0)
                        }
                    } else if op == OP_GT {
                        if f_lt(b, a) {
                            vpush(1)
                        } else {
                            vpush(0)
                        }
                    } else if op == OP_GE {
                        if f_lt(b, a) {
                            vpush(1)
                        } else if f_eq(a, b) {
                            vpush(1)
                        } else {
                            vpush(0)
                        }
                    } else {
                        guest_trap()
                    }
                }
            }
        } else if kind == KFNEG {
            eval(nf(node, 4)); // operand f64 bits
            if sig() != 0 {
                0
            } else {
                vpush(vpop() ^ POOL_TAG) // flip the sign bit (1<<63)
            }
        } else if kind == KTRAP {
            guest_trap() // a 128-bit path was reached — must stay dead on this chain
        } else if kind == KFNAN {
            eval(nf(node, 4)); // operand f64 bits
            if sig() != 0 {
                0
            } else {
                let v = vpop();
                let exp = (v >> 52) & 2047; // 11 exponent bits
                let mant = v & 4503599627370495; // 52 mantissa bits (2^52 - 1)
                if exp == 2047 {
                    if mant != 0 {
                        vpush(1)
                    } else {
                        vpush(0)
                    }
                } else {
                    vpush(0)
                }
            }
        } else if kind == KM {
            eval(nf(node, 4)); // c = scrutinee (scalar), pushes 1 slot
            if sig() != 0 {
                0
            } else {
                let v = vpop();
                let mut arm = nf(node, 2); // a = first arm (KMA)
                let mut done: u64 = 0;
                while arm != NIL && done == 0 {
                    let mut pat = nf(arm, 2); // a = first pattern (KMP)
                    let mut hit: u64 = 0;
                    while pat != NIL && hit == 0 {
                        if nf(pat, 1) == 1 {
                            hit = 1; // wildcard
                        } else if ld(VALS + nf(pat, 2)) == v {
                            hit = 1; // literal match
                        } else {
                            pat = nf(pat, 6); // link -> next pattern
                        }
                    }
                    if hit == 1 {
                        eval(nf(arm, 4)); // c = body (leaves the arm's value)
                        done = 1;
                    } else {
                        arm = nf(arm, 6); // link -> next arm
                    }
                }
                if done == 0 {
                    guest_trap() // checker guarantees exhaustiveness
                } else {
                    0
                }
            }
        } else {
            guest_trap()
        }
    }
}

fn main() {
    load_image();
    st(R_SIG, 0);
    st(R_FRAME, 0);
    st(R_VN, 0);
    let entry = ld(R_ENTRY);
    st(R_FSIZE, ld(FNS + entry * 3 + 1));
    eval(ld(FNS + entry * 3));
}
