
use subrust::ast::*;
use subrust::check::*;
use subrust::{Chk, Mem, NODE_NIL};

pub const IMG_MAGIC: u64 = 0x0000_5352_3169_0001;
const NIL: u64 = u64::MAX;

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
const KS: u64 = 16;
const KD: u64 = 17;
const KFI: u64 = 18;
const KAR: u64 = 19;
const KRP: u64 = 20;
const KIX: u64 = 21;
const KRF: u64 = 22;
const KDR: u64 = 23;
const KAD: u64 = 24;
const KSL: u64 = 25;
const KSX: u64 = 26;
const KSN: u64 = 27;
const KBS: u64 = 28;
const KSTRL: u64 = 29;
const KSTRB: u64 = 30;
const KM: u64 = 31;
const KMA: u64 = 32;
const KMP: u64 = 33;
const KCAST: u64 = 34;
const KWRAP: u64 = 35;
const KCONST: u64 = 36;

const KFADDR: u64 = 37;
const KIXA: u64 = 38;
const KIXA_SL: u64 = 39;
const KDFLD: u64 = 40;
const KIXR: u64 = 41;
const KRET: u64 = 42;
const KSL_SL: u64 = 43;

const KF64: u64 = 44;
const KFNEG: u64 = 45;
const KFNAN: u64 = 46;
const KTRAP: u64 = 47;

struct Emit<'a> {
    src: &'a str,
    mem: &'a Mem,
    chk: &'a Chk,
    nodes: Vec<[u64; 7]>,
    vals: Vec<u64>,
    err: Option<String>,
}

impl<'a> Emit<'a> {
    fn push(&mut self, kind: u64, x: u64, a: u64, b: u64, c: u64, d: u64) -> u64 {
        self.nodes.push([kind, x, a, b, c, d, NIL]);
        (self.nodes.len() - 1) as u64
    }

    /// Value size in slots for node `idx` (its checked type). One for scalars;
    /// >1 for aggregates once those land on the chain.
    fn size_at(&self, idx: u32) -> u64 {
        self.chk.size_of(self.chk.ty[idx as usize]) as u64
    }

    fn val(&mut self, v: u64) -> u64 {

        if let Some(i) = self.vals.iter().position(|&x| x == v) {
            return i as u64;
        }
        self.vals.push(v);
        (self.vals.len() - 1) as u64
    }

    fn fail(&mut self, msg: &str, n: Node) {
        if self.err.is_none() {
            let (line, col) = line_col(self.src, n.lo);
            self.err = Some(format!("emit: {msg} at {line}:{col}"));
        }
    }

    /// Emit a chain of statement nodes (subrust `.link` list) linked in the
    /// sr1i image; returns the first sr1i index or NIL.
    fn chain(&mut self, first: u32) -> u64 {
        let mut it = first;
        let mut head = NIL;
        let mut prev = NIL;
        while it != NODE_NIL {
            let e = self.node(it);
            if e == NIL {
                return NIL;
            }
            if head == NIL {
                head = e;
            } else {
                self.nodes[prev as usize][6] = e;
            }
            prev = e;
            it = self.mem.node(it).link;
        }
        head
    }

    fn node(&mut self, idx: u32) -> u64 {
        if self.err.is_some() {
            return NIL;
        }

        if ty_is_128(self.chk.ty[idx as usize]) {
            return self.push(KTRAP, 0, 0, 0, 0, 0);
        }
        let n = self.mem.node(idx);
        match n.kind {
            N_LIT_INT | N_LIT_FLOAT | N_LIT_BYTE | N_PATHCONST => {

                let v = self.chk.vals[self.chk.res[idx as usize] as usize];
                let vi = self.val(v);
                self.push(KL, 0, vi, 0, 0, 0)
            }
            N_LIT_BSTR => {

                let ri = self.chk.res[idx as usize] as usize;
                let a0 = self.val(self.chk.vals[ri]);
                let a1 = self.val(self.chk.vals[ri + 1]);

                if a1 != a0 + 1 {

                    let a0b = self.vals.len() as u64;
                    self.vals.push(self.chk.vals[ri]);
                    self.vals.push(self.chk.vals[ri + 1]);
                    self.push(KBS, 0, a0b, 0, 0, 0)
                } else {
                    self.push(KBS, 0, a0, 0, 0, 0)
                }
            }
            N_LIT_BOOL => {
                let vi = self.val(n.x as u64);
                self.push(KL, 0, vi, 0, 0, 0)
            }
            N_LIT_STR => {

                let vi = self.val(self.chk.res[idx as usize] as u64);
                self.push(KL, 0, vi, 0, 0, 0)
            }
            N_NAME => {
                if self.chk.res[idx as usize] & RES_CONST != 0 {

                    let k = (self.chk.res[idx as usize] & RES_MASK) as usize;
                    let t = self.chk.ty[idx as usize];
                    if ty_is_struct(t) || ty_is_arr(t) {
                        let base = self.chk.consts[k].bits as usize;
                        let size = self.chk.size_of(t) as usize;
                        let my_base = self.vals.len() as u64;
                        for j in 0..size {
                            self.vals.push(self.chk.vals[base + j]);
                        }
                        return self.push(KCONST, 0, my_base, size as u64, 0, 0);
                    }
                    let v = self.chk.consts[k].bits;
                    let vi = self.val(v);
                    return self.push(KL, 0, vi, 0, 0, 0);
                }
                self.push(KN, 0, self.chk.res[idx as usize] as u64, self.size_at(idx), 0, 0)
            }
            N_UNARY => {
                if self.chk.res[idx as usize] == 1 {

                    return self.node(n.e);
                }
                let c = self.node(n.e);
                if c == NIL {
                    return NIL;
                }
                if n.x == OP_NEG {
                    if self.chk.ty[idx as usize] == TY_F64 {

                        return self.push(KFNEG, 0, 0, 0, c, 0);
                    }

                    self.fail("runtime signed negation not on the chain yet", n);
                    return NIL;
                }
                self.push(KU, n.x as u64, 0, 0, c, 0)
            }
            N_BINARY => {
                let ot = self.chk.ty[n.d as usize];
                let c = self.node(n.d);
                let d = self.node(n.e);
                if c == NIL || d == NIL {
                    return NIL;
                }
                if ot == TY_F64 {
                    self.push(KF64, n.x as u64, 0, 0, c, d)
                } else {
                    self.push(KB, n.x as u64, 0, 0, c, d)
                }
            }
            N_CAST => {

                let from = self.chk.ty[n.d as usize];
                let to = self.chk.ty[idx as usize];
                if ty_is_128(from) || ty_is_128(to) {

                    return self.push(KTRAP, 0, 0, 0, 0, 0);
                }
                if from == TY_F64 || to == TY_F64 {

                    return self.push(KTRAP, 0, 0, 0, 0, 0);
                }
                let bits = match to {
                    TY_I8 | TY_U8 => 8u64,
                    TY_I16 | TY_U16 => 16,
                    TY_I32 | TY_U32 => 32,
                    _ => 64,
                };
                let signed = if ty_is_signed(to) { 1u64 } else { 0 };
                let c = self.node(n.d);
                if c == NIL {
                    return NIL;
                }
                self.push(KCAST, signed, bits, 0, c, 0)
            }
            N_IF => {
                let c = self.node(n.d);
                let d = self.node(n.e);
                let a = if n.b == NODE_NIL { NIL } else { self.node(n.b) };
                self.push(KI, 0, a, 0, c, d)
            }
            N_BLOCK => {
                let a = self.chain(n.b);
                let b = if n.e == NODE_NIL { NIL } else { self.node(n.e) };
                self.push(KK, 0, a, b, 0, 0)
            }
            N_EXPR_STMT => {
                let c = self.node(n.e);
                self.push(KE, 0, 0, 0, c, 0)
            }
            N_LET => {
                let c = self.node(n.e);
                self.push(KT, 0, self.chk.res[idx as usize] as u64, self.size_at(idx), c, 0)
            }
            N_ASSIGN => {
                let place = self.mem.node(n.d);
                if place.kind == N_NAME {

                    let slot = self.chk.res[n.d as usize] as u64;
                    let size = self.size_at(n.d);
                    let c = self.node(n.e);
                    self.push(KA, n.x as u64, slot, size, c, 0)
                } else if n.x == 0 {

                    let size = self.size_at(n.d);
                    let addr = self.paddr(n.d);
                    let value = self.node(n.e);
                    if addr == NIL || value == NIL {
                        return NIL;
                    }
                    self.push(KAD, 0, 0, size, addr, value)
                } else {

                    let addr = self.paddr(n.d);
                    let value = self.node(n.e);
                    if addr == NIL || value == NIL {
                        return NIL;
                    }
                    self.push(KAD, n.x as u64, 0, 1, addr, value)
                }
            }
            N_WHILE => {
                let c = self.node(n.d);
                let d = self.node(n.e);
                self.push(KW, 0, 0, 0, c, d)
            }
            N_LOOP => {
                let d = self.node(n.e);
                self.push(KO, 0, 0, 0, 0, d)
            }
            N_BREAK => self.push(KR, 0, 0, 0, 0, 0),
            N_CONTINUE => self.push(KC, 0, 0, 0, 0, 0),
            N_RETURN => {

                if n.e == NODE_NIL {
                    return self.push(KRET, 0, 0, 0, NIL, 0);
                }
                let size = self.size_at(n.e);
                let c = self.node(n.e);
                if c == NIL {
                    return NIL;
                }
                self.push(KRET, 0, 0, size, c, 0)
            }
            N_CALL => {
                let args = self.chain(n.b);
                let count = n.c as u64;
                if self.chk.res[idx as usize] & RES_HOST != 0 {
                    let host = (self.chk.res[idx as usize] & RES_MASK) as u64;

                    if host == 3 || host == 2 {

                        self.push(KH, host, 0, args, count, 0)
                    } else {
                        self.fail("only putb/getb host calls on the chain yet (ld/st/f_* deferred)", n);
                        NIL
                    }
                } else {
                    let f = (self.chk.res[idx as usize] & RES_MASK) as u64;
                    self.push(KF, 0, f, args, count, 0)
                }
            }
            N_REFOF => {
                let place = self.mem.node(n.e);
                if place.kind == N_SLICE {

                    return self.node(n.e);
                }
                if place.kind == N_NAME
                    && ty_is_arr(self.chk.ty[n.e as usize])
                    && ty_is_slice(self.chk.ty[idx as usize])
                {

                    if self.chk.res[n.e as usize] & RES_CONST != 0 {
                        self.fail("&const-array unsizing not on the chain yet", n);
                        return NIL;
                    }
                    let len = self.chk.ainfo(self.chk.ty[n.e as usize]).len as u64;
                    let es =
                        self.chk.size_of(self.chk.slinfo(self.chk.ty[idx as usize]).pointee) as u64;
                    let base_addr = self.push(KRF, 0, self.chk.res[n.e as usize] as u64, 0, 0, 0);
                    let zero = self.val(0);
                    let lo0 = self.push(KL, 0, zero, 0, 0, 0);
                    return self.push(KSL, NIL, es, len, base_addr, lo0);
                }
                if place.kind == N_ARRAY_LIT && place.c == 0 {

                    let base = self.vals.len() as u64;
                    self.vals.push(0);
                    self.vals.push(0);
                    return self.push(KCONST, 0, base, 2, 0, 0);
                }
                if ty_is_slice(self.chk.ty[idx as usize]) {

                    self.fail("this &→slice form not on the chain yet", n);
                    return NIL;
                }

                self.paddr(n.e)
            }
            N_DEREF => {
                let c = self.node(n.e);
                if c == NIL {
                    return NIL;
                }

                self.push(KDR, 0, self.size_at(idx), 0, c, 0)
            }
            N_SLICE => {
                let bt = self.chk.ty[n.d as usize];
                let es = self.chk.size_of(self.chk.slinfo(self.chk.ty[idx as usize]).pointee) as u64;
                if self.chk.res[idx as usize] != 0 {

                    let base = self.node(n.d);
                    let lo = self.node(n.b);
                    let hi = if n.c == NODE_NIL { NIL } else { self.node(n.c) };
                    if base == NIL || lo == NIL || (n.c != NODE_NIL && hi == NIL) {
                        return NIL;
                    }
                    return self.push(KSL_SL, hi, es, 0, base, lo);
                }

                if !ty_is_arr(bt) {
                    self.fail("slicing this base not on the chain yet", n);
                    return NIL;
                }
                let len = self.chk.ainfo(bt).len as u64;
                let base_addr = self.paddr(n.d);
                if base_addr == NIL {
                    return NIL;
                }
                let lo = self.node(n.b);
                let hi = if n.c == NODE_NIL { NIL } else { self.node(n.c) };
                if lo == NIL || (n.c != NODE_NIL && hi == NIL) {
                    return NIL;
                }
                self.push(KSL, hi, es, len, base_addr, lo)
            }
            N_ARRAY_LIT | N_TUPLE => {

                let mut first = NIL;
                let mut prev = NIL;
                let mut el = n.b;
                while el != NODE_NIL {
                    let e = self.node(el);
                    if e == NIL {
                        return NIL;
                    }
                    if first == NIL {
                        first = e;
                    } else {
                        self.nodes[prev as usize][6] = e;
                    }
                    prev = e;
                    el = self.mem.node(el).link;
                }
                self.push(KAR, 0, first, 0, 0, 0)
            }
            N_ARRAY_REPEAT => {
                let a = self.chk.ainfo(self.chk.ty[idx as usize]);
                let esize = self.chk.size_of(a.elem) as u64;
                let c = self.node(n.d);
                if c == NIL {
                    return NIL;
                }
                self.push(KRP, 0, esize, a.len as u64, c, 0)
            }
            N_INDEX => {
                let bt = self.chk.ty[n.d as usize];
                if self.chk.res[idx as usize] & RES_DEREF != 0 && !ty_is_slice(bt) {

                    let a = self.chk.ainfo(self.chk.rinfo(bt).pointee);
                    let es = self.chk.size_of(a.elem) as u64;
                    let base = self.node(n.d);
                    let iexpr = self.node(n.e);
                    if base == NIL || iexpr == NIL {
                        return NIL;
                    }
                    return self.push(KIXR, 0, es, a.len as u64, base, iexpr);
                }
                if ty_is_arr(bt) {
                    let a = self.chk.ainfo(bt);
                    let esize = self.chk.size_of(a.elem) as u64;
                    let base = self.node(n.d);
                    let iexpr = self.node(n.e);
                    if base == NIL || iexpr == NIL {
                        return NIL;
                    }
                    self.push(KIX, 0, esize, a.len as u64, base, iexpr)
                } else if ty_is_slice(bt) {

                    let es = self.chk.size_of(self.chk.slinfo(bt).pointee) as u64;
                    let base = self.node(n.d);
                    let iexpr = self.node(n.e);
                    if base == NIL || iexpr == NIL {
                        return NIL;
                    }
                    self.push(KSX, 0, es, 0, base, iexpr)
                } else {
                    self.fail("only array/slice indexing on the chain yet", n);
                    NIL
                }
            }
            N_METHOD => {
                let res = self.chk.res[idx as usize];
                if res & RES_ARRAY_LEN != 0 {

                    let v = self.chk.vals[(res & RES_ALEN_MASK) as usize];
                    let vi = self.val(v);
                    self.push(KL, 0, vi, 0, 0, 0)
                } else if res & RES_SLICE_LEN != 0 {
                    let base = self.node(n.d);
                    if base == NIL {
                        return NIL;
                    }
                    self.push(KSN, 0, 0, 0, base, 0)
                } else if res & RES_STR_LEN != 0 {
                    let base = self.node(n.d);
                    if base == NIL {
                        return NIL;
                    }
                    self.push(KSTRL, 0, 0, 0, base, 0)
                } else if res & RES_STR_BYTES != 0 {
                    let base = self.node(n.d);
                    if base == NIL {
                        return NIL;
                    }
                    self.push(KSTRB, 0, 0, 0, base, 0)
                } else if res & RES_PRIM != 0 {

                    let op = (res & RES_PRIM_MASK) as u64;
                    if op == PRIM_TO_BITS as u64 {

                        return self.node(n.d);
                    }
                    if op == PRIM_IS_NAN as u64 {
                        let recv = self.node(n.d);
                        if recv == NIL {
                            return NIL;
                        }
                        return self.push(KFNAN, 0, 0, 0, recv, 0);
                    }
                    let to = self.chk.ty[idx as usize];
                    if ty_is_128(to) {
                        self.fail("128-bit primitive methods not in SR-seed", n);
                        return NIL;
                    }
                    let signed = if ty_is_signed(to) { 1u64 } else { 0 };
                    if (op == PRIM_SAT_ADD as u64 || op == PRIM_SAT_MUL as u64) && signed == 1 {
                        self.fail("signed saturating methods not on the chain yet", n);
                        return NIL;
                    }
                    let bits = match to {
                        TY_I8 | TY_U8 => 8u64,
                        TY_I16 | TY_U16 => 16,
                        TY_I32 | TY_U32 => 32,
                        _ => 64,
                    };
                    let recv = self.node(n.d);
                    if recv == NIL {
                        return NIL;
                    }
                    let arg = if op == PRIM_WRAP_NEG as u64 {
                        NIL
                    } else {
                        let a = self.node(n.b);
                        if a == NIL {
                            return NIL;
                        }
                        a
                    };
                    self.push(KWRAP, op, bits, signed, recv, arg)
                } else {

                    let fidx = (res & RES_MFN_MASK) as u64;
                    let recv = if res & RES_MPLACE != 0 {
                        self.paddr(n.d)
                    } else {
                        self.node(n.d)
                    };
                    if recv == NIL {
                        return NIL;
                    }
                    let mut prev = recv;
                    let mut count = 1u64;
                    let mut arg = n.b;
                    while arg != NODE_NIL {
                        let e = self.node(arg);
                        if e == NIL {
                            return NIL;
                        }
                        self.nodes[prev as usize][6] = e;
                        prev = e;
                        count += 1;
                        arg = self.mem.node(arg).link;
                    }
                    self.push(KF, 0, fidx, recv, count, 0)
                }
            }
            N_MATCH => {

                let scrut = self.node(n.d);
                if scrut == NIL {
                    return NIL;
                }
                let mut first_arm = NIL;
                let mut prev_arm = NIL;
                let mut arm = n.b;
                while arm != NODE_NIL {
                    let an = self.mem.node(arm);
                    let mut first_pat = NIL;
                    let mut prev_pat = NIL;
                    let mut pat = an.b;
                    while pat != NODE_NIL {
                        let pn = self.mem.node(pat);
                        let kmp = match pn.kind {
                            N_PAT_WILD => self.push(KMP, 1, 0, 0, 0, 0),
                            N_PAT_INT | N_PAT_BYTE | N_PAT_CONST => {
                                let v = self.chk.vals[self.chk.res[pat as usize] as usize];
                                let vi = self.val(v);
                                self.push(KMP, 0, vi, 0, 0, 0)
                            }
                            N_PAT_BOOL => {
                                let vi = self.val(pn.x as u64);
                                self.push(KMP, 0, vi, 0, 0, 0)
                            }
                            N_PAT_STR => {
                                let id = self.chk.res[pat as usize] as u64;
                                let vi = self.val(id);
                                self.push(KMP, 0, vi, 0, 0, 0)
                            }
                            _ => {
                                self.fail("match pattern kind not on the chain yet", pn);
                                return NIL;
                            }
                        };
                        if first_pat == NIL {
                            first_pat = kmp;
                        } else {
                            self.nodes[prev_pat as usize][6] = kmp;
                        }
                        prev_pat = kmp;
                        pat = pn.link;
                    }
                    let body = self.node(an.e);
                    if body == NIL {
                        return NIL;
                    }
                    let kma = self.push(KMA, 0, first_pat, 0, body, 0);
                    if first_arm == NIL {
                        first_arm = kma;
                    } else {
                        self.nodes[prev_arm as usize][6] = kma;
                    }
                    prev_arm = kma;
                    arm = an.link;
                }
                self.push(KM, 0, first_arm, 0, scrut, 0)
            }
            N_STRUCT_LIT => {

                let mut first = NIL;
                let mut prev = NIL;
                let mut init = n.b;
                while init != NODE_NIL {
                    let fin = self.mem.node(init);
                    let off = (self.chk.res[init as usize] & 0xFFFF) as u64;
                    let fsize = self.size_at(init);
                    let ie = if fin.e == NODE_NIL {

                        let slot = (self.chk.res[init as usize] >> 16) as u64;
                        self.push(KN, 0, slot, fsize, 0, 0)
                    } else {
                        self.node(fin.e)
                    };
                    if ie == NIL {
                        return NIL;
                    }
                    let fi = self.push(KFI, 0, off, fsize, ie, 0);
                    if first == NIL {
                        first = fi;
                    } else {
                        self.nodes[prev as usize][6] = fi;
                    }
                    prev = fi;
                    init = fin.link;
                }
                self.push(KS, 0, first, self.size_at(idx), 0, 0)
            }
            N_DOT => {
                let off = (self.chk.res[idx as usize] & RES_OFF_MASK) as u64;
                if self.chk.res[idx as usize] & RES_DEREF != 0 {

                    let base = self.node(n.d);
                    if base == NIL {
                        return NIL;
                    }
                    self.push(KDFLD, 0, off, self.size_at(idx), base, 0)
                } else {
                    let base = self.node(n.d);
                    if base == NIL {
                        return NIL;
                    }
                    self.push(KD, 0, off, self.size_at(idx), base, self.size_at(n.d))
                }
            }
            N_ASSOC_CALL => {

                self.node(n.b)
            }
            N_LIT_UNIT => {
                self.fail("bare unit not supported by sr1i yet", n);
                NIL
            }
            _ => {
                self.fail("construct not in the sr1i-runnable subset", n);
                NIL
            }
        }
    }

    /// Emit a node that computes the ABSOLUTE address of a place expression
    /// (mirrors the machine's `step_place`). Every place resolves to one address
    /// slot on the value stack, which `KAD`/read nodes then store to / load from.
    fn paddr(&mut self, idx: u32) -> u64 {
        if self.err.is_some() {
            return NIL;
        }
        let n = self.mem.node(idx);
        match n.kind {
            N_NAME => {
                if self.chk.res[idx as usize] & RES_CONST != 0 {
                    self.fail("cannot take the address of a const", n);
                    return NIL;
                }

                self.push(KRF, 0, self.chk.res[idx as usize] as u64, 0, 0, 0)
            }
            N_DEREF => {

                self.node(n.e)
            }
            N_DOT => {
                let off = (self.chk.res[idx as usize] & RES_OFF_MASK) as u64;
                let base = if self.chk.res[idx as usize] & RES_DEREF != 0 {
                    self.node(n.d)
                } else {
                    self.paddr(n.d)
                };
                if base == NIL {
                    return NIL;
                }
                self.push(KFADDR, 0, off, 0, base, 0)
            }
            N_INDEX => {
                let bt = self.chk.ty[n.d as usize];
                if ty_is_slice(bt) {
                    let es = self.chk.size_of(self.chk.slinfo(bt).pointee) as u64;
                    let base = self.node(n.d);
                    let iexpr = self.node(n.e);
                    if base == NIL || iexpr == NIL {
                        return NIL;
                    }
                    self.push(KIXA_SL, 0, es, 0, base, iexpr)
                } else if self.chk.res[idx as usize] & RES_DEREF != 0 {

                    let a = self.chk.ainfo(self.chk.rinfo(bt).pointee);
                    let es = self.chk.size_of(a.elem) as u64;
                    let base = self.node(n.d);
                    let iexpr = self.node(n.e);
                    if base == NIL || iexpr == NIL {
                        return NIL;
                    }
                    self.push(KIXA, 0, es, a.len as u64, base, iexpr)
                } else if ty_is_arr(bt) {
                    let a = self.chk.ainfo(bt);
                    let es = self.chk.size_of(a.elem) as u64;
                    let base = self.paddr(n.d);
                    let iexpr = self.node(n.e);
                    if base == NIL || iexpr == NIL {
                        return NIL;
                    }
                    self.push(KIXA, 0, es, a.len as u64, base, iexpr)
                } else {
                    self.fail("address of this index base not on the chain yet", n);
                    NIL
                }
            }
            _ => {
                self.fail("address of this place not on the chain yet", n);
                NIL
            }
        }
    }
}

fn line_col(src: &str, pos: u32) -> (usize, usize) {
    let pos = (pos as usize).min(src.len());
    let before = &src[..pos];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let ls = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    (line, pos - ls + 1)
}

/// Serialize the checked program to image bytes, or Err(message).
pub fn emit_image(src: &str, mem: &Mem, chk: &Chk) -> Result<Vec<u8>, String> {
    let mut e = Emit {
        src,
        mem,
        chk,
        nodes: Vec::new(),
        vals: Vec::new(),
        err: None,
    };

    let mut fn_bodies = Vec::new();
    let mut entry = NIL;
    for i in 0..chk.fn_n {
        let f = chk.fns[i];
        let body = e.node(mem.node(f.node).e);
        fn_bodies.push((body, f.frame as u64, f.param_n as u64));
        if fn_is_main(src, mem, f.name_tok) {
            entry = i as u64;
        }
    }
    if let Some(msg) = e.err {
        return Err(msg);
    }
    if entry == NIL {
        return Err("emit: no `fn main`".to_string());
    }

    let mut out = Vec::new();
    let mut w = |v: u64| out.extend_from_slice(&v.to_le_bytes());
    w(IMG_MAGIC);
    w(chk.fn_n as u64);
    w(entry);
    w(e.nodes.len() as u64);
    w(e.vals.len() as u64);
    w(chk.pool_n as u64);
    w(chk.str_n as u64);
    for (body, frame, np) in &fn_bodies {
        w(*body);
        w(*frame);
        w(*np);
    }
    for v in &e.vals {
        w(*v);
    }
    for nd in &e.nodes {
        for f in nd {
            w(*f);
        }
    }

    let mut wi = 0;
    while wi < chk.pool_n {
        let mut word = 0u64;
        let mut b = 0;
        while b < 8 && wi + b < chk.pool_n {
            word |= (chk.str_pool[wi + b] as u64) << (b * 8);
            b += 1;
        }
        w(word);
        wi += 8;
    }

    for s in &chk.strs[..chk.str_n] {
        w(s.off as u64);
        w(s.len as u64);
    }
    Ok(out)
}

fn fn_is_main(src: &str, mem: &Mem, name_tok: u32) -> bool {
    let t = mem.tok(name_tok);
    let lo = t.pos as usize;
    let hi = lo + t.len as usize;
    src.as_bytes().get(lo..hi) == Some(b"main")
}
