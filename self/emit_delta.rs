
const IMG_MAGIC: u64 = 0x0000_5352_3169_0001;
const KNIL: u64 = 18446744073709551615;

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
const KRET: u64 = 42;
const KF64: u64 = 44;
const KFNEG: u64 = 45;

const H_TY_U64: HostTy = HostTy { kind: TY_U64, sname: 0, elem: 0, len: 0 };
const H_TY_UNIT: HostTy = HostTy { kind: TY_UNIT, sname: 0, elem: 0, len: 0 };
const BOOT: HostDef = HostDef {
    structs: [H_STRUCT0; HCAP_HSTRUCTS],
    struct_n: 0,
    fns: [
        HostFnDef { name: 0, params: [H_TY_U64], param_n: 1, ret: H_TY_U64 },
        HostFnDef { name: 1, params: [H_TY_U64], param_n: 2, ret: H_TY_UNIT },
        HostFnDef { name: 2, params: [H_TY0], param_n: 0, ret: H_TY_U64 },
        HostFnDef { name: 3, params: [H_TY_U64], param_n: 1, ret: H_TY_UNIT },
    ],
    fn_n: 4,
};

const ECAP_NODES: usize = 96;
const ECAP_VALS: usize = 48;
#[derive(Clone, Copy)]
struct Emit {
    en: [u64; ECAP_NODES * 7],
    en_n: usize,
    ev: [u64; ECAP_VALS],
    ev_n: usize,
    err: bool,
}

fn e_push(e: &mut Emit, kind: u64, x: u64, a: u64, b: u64, c: u64, d: u64) -> u64 {
    let base = e.en_n * 7;
    e.en[base] = kind;
    e.en[base + 1] = x;
    e.en[base + 2] = a;
    e.en[base + 3] = b;
    e.en[base + 4] = c;
    e.en[base + 5] = d;
    e.en[base + 6] = KNIL;
    e.en_n += 1;
    (e.en_n - 1) as u64
}

fn e_val(e: &mut Emit, v: u64) -> u64 {
    let mut i = 0;
    while i < e.ev_n {
        if e.ev[i] == v {
            return i as u64;
        }
        i += 1;
    }
    e.ev[e.ev_n] = v;
    e.ev_n += 1;
    (e.ev_n - 1) as u64
}

fn e_size_at(chk: &Chk, idx: u32) -> u64 {
    chk.size_of(chk.ty[idx as usize]) as u64
}

fn e_node(m: &Mem, chk: &Chk, e: &mut Emit, idx: u32) -> u64 {
    if e.err {
        return KNIL;
    }
    let n = m.node(idx);
    let k = n.kind;
    if k == N_LIT_INT {
        let v = chk.vals[chk.res[idx as usize] as usize];
        let vi = e_val(e, v);
        return e_push(e, KL, 0, vi, 0, 0, 0);
    }
    if k == N_LIT_BOOL {
        let vi = e_val(e, n.x as u64);
        return e_push(e, KL, 0, vi, 0, 0, 0);
    }
    if k == N_NAME {
        if (chk.res[idx as usize] & RES_CONST) != 0 {
            let ci = (chk.res[idx as usize] & RES_MASK) as usize;
            let t = chk.ty[idx as usize];
            if ty_is_struct(t) || ty_is_arr(t) {

                e.err = true;
                return KNIL;
            }
            let v = chk.consts[ci].bits;
            let vi = e_val(e, v);
            return e_push(e, KL, 0, vi, 0, 0, 0);
        }
        let sz = e_size_at(chk, idx);
        return e_push(e, KN, 0, chk.res[idx as usize] as u64, sz, 0, 0);
    }
    if k == N_UNARY {
        if chk.res[idx as usize] == 1 {

            return e_node(m, chk, e, n.e);
        }
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        if n.x == OP_NEG {
            if chk.ty[idx as usize] == TY_F64 {
                return e_push(e, KFNEG, 0, 0, 0, c, 0);
            }

            e.err = true;
            return KNIL;
        }
        return e_push(e, KU, n.x as u64, 0, 0, c, 0);
    }
    if k == N_BINARY {
        let ot = chk.ty[n.d as usize];
        let c = e_node(m, chk, e, n.d);
        let d = e_node(m, chk, e, n.e);
        if c == KNIL || d == KNIL {
            return KNIL;
        }
        if ot == TY_F64 {
            return e_push(e, KF64, n.x as u64, 0, 0, c, d);
        }
        return e_push(e, KB, n.x as u64, 0, 0, c, d);
    }
    if k == N_IF {
        let c = e_node(m, chk, e, n.d);
        let d = e_node(m, chk, e, n.e);
        let a = if n.b == NODE_NIL { KNIL } else { e_node(m, chk, e, n.b) };
        if c == KNIL || d == KNIL {
            return KNIL;
        }
        return e_push(e, KI, 0, a, 0, c, d);
    }
    if k == N_BLOCK {
        let a = e_chain(m, chk, e, n.b);
        let b = if n.e == NODE_NIL { KNIL } else { e_node(m, chk, e, n.e) };
        return e_push(e, KK, 0, a, b, 0, 0);
    }
    if k == N_EXPR_STMT {
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        return e_push(e, KE, 0, 0, 0, c, 0);
    }
    if k == N_LET {
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        let sz = e_size_at(chk, idx);
        return e_push(e, KT, 0, chk.res[idx as usize] as u64, sz, c, 0);
    }
    if k == N_ASSIGN {
        let place = m.node(n.d);
        if place.kind == N_NAME {
            let slot = chk.res[n.d as usize] as u64;
            let sz = e_size_at(chk, n.d);
            let c = e_node(m, chk, e, n.e);
            if c == KNIL {
                return KNIL;
            }
            return e_push(e, KA, n.x as u64, slot, sz, c, 0);
        }

        e.err = true;
        return KNIL;
    }
    if k == N_WHILE {
        let c = e_node(m, chk, e, n.d);
        let d = e_node(m, chk, e, n.e);
        if c == KNIL || d == KNIL {
            return KNIL;
        }
        return e_push(e, KW, 0, 0, 0, c, d);
    }
    if k == N_LOOP {
        let d = e_node(m, chk, e, n.e);
        if d == KNIL {
            return KNIL;
        }
        return e_push(e, KO, 0, 0, 0, 0, d);
    }
    if k == N_BREAK {
        return e_push(e, KR, 0, 0, 0, 0, 0);
    }
    if k == N_CONTINUE {
        return e_push(e, KC, 0, 0, 0, 0, 0);
    }
    if k == N_RETURN {
        if n.e == NODE_NIL {
            return e_push(e, KRET, 0, 0, 0, KNIL, 0);
        }
        let sz = e_size_at(chk, n.e);
        let c = e_node(m, chk, e, n.e);
        if c == KNIL {
            return KNIL;
        }
        return e_push(e, KRET, 0, 0, sz, c, 0);
    }
    if k == N_CALL {
        let args = e_chain(m, chk, e, n.b);
        let count = n.c as u64;
        if (chk.res[idx as usize] & RES_HOST) != 0 {
            let host = (chk.res[idx as usize] & RES_MASK) as u64;

            if host == 3 || host == 2 {
                return e_push(e, KH, host, 0, args, count, 0);
            }
            e.err = true;
            return KNIL;
        }
        let f = (chk.res[idx as usize] & RES_MASK) as u64;
        return e_push(e, KF, 0, f, args, count, 0);
    }

    e.err = true;
    KNIL
}

fn e_chain(m: &Mem, chk: &Chk, e: &mut Emit, first: u32) -> u64 {
    let mut it = first;
    let mut head = KNIL;
    let mut prev = KNIL;
    while it != NODE_NIL {
        let en = e_node(m, chk, e, it);
        if en == KNIL {
            return KNIL;
        }
        if head == KNIL {
            head = en;
        } else {
            e.en[(prev as usize) * 7 + 6] = en;
        }
        prev = en;
        it = m.node(it).link;
    }
    head
}

fn wu64(v: u64) {
    let mut j = 0;
    while j < 8 {
        putb((v >> (j * 8)) & 255);
        j += 1;
    }
}

fn fn_is_main(src: &[u8], m: &Mem, name_tok: u32) -> bool {
    let t = m.tok(name_tok);
    if t.len != 4 {
        return false;
    }
    let lo = t.pos as usize;
    src[lo] == 109 && src[lo + 1] == 97 && src[lo + 2] == 105 && src[lo + 3] == 110
}

fn emit_image(src: &[u8], m: &Mem, chk: &Chk) {
    let mut e = Emit {
        en: [0; ECAP_NODES * 7], en_n: 0,
        ev: [0; ECAP_VALS], ev_n: 0,
        err: false,
    };
    let mut fbody = [0u64; CAP_FNS];
    let mut entry = KNIL;
    let mut i = 0;
    while i < chk.fn_n {
        let fnode = m.node(chk.fns[i].node);
        let body = e_node(m, chk, &mut e, fnode.e);
        fbody[i] = body;
        if fn_is_main(src, m, chk.fns[i].name_tok) {
            entry = i as u64;
        }
        i += 1;
    }
    if e.err {
        return;
    }
    if entry == KNIL {
        return;
    }
    wu64(IMG_MAGIC);
    wu64(chk.fn_n as u64);
    wu64(entry);
    wu64(e.en_n as u64);
    wu64(e.ev_n as u64);
    wu64(chk.pool_n as u64);
    wu64(chk.str_n as u64);
    let mut i = 0;
    while i < chk.fn_n {
        wu64(fbody[i]);
        wu64(chk.fns[i].frame as u64);
        wu64(chk.fns[i].param_n as u64);
        i += 1;
    }
    let mut i = 0;
    while i < e.ev_n {
        wu64(e.ev[i]);
        i += 1;
    }
    let mut i = 0;
    while i < e.en_n * 7 {
        wu64(e.en[i]);
        i += 1;
    }
    let mut wi: usize = 0;
    while wi < chk.pool_n {
        let mut word = 0u64;
        let mut b: usize = 0;
        while b < 8 && wi + b < chk.pool_n {
            word |= (chk.str_pool[wi + b] as u64) << (b * 8);
            b += 1;
        }
        wu64(word);
        wi += 8;
    }
    let mut i = 0;
    while i < chk.str_n {
        wu64(chk.strs[i].off as u64);
        wu64(chk.strs[i].len as u64);
        i += 1;
    }
}

fn main() {

    let mut buf = [0u8; 128];
    let mut n: usize = 0;
    loop {
        let c = getb();
        if c == 18446744073709551615 {
            break;
        }
        if n < 128 {
            buf[n] = c as u8;
            n = n + 1;
        }
    }
    let src = &buf[0..n];
    let mut m = Mem {
        toks: [TOK_NONE; CAP_TOKS], tok_n: 0,
        nodes: [NODE_NONE; CAP_NODES], node_n: 0,
        diags: [Diag { code: 0, lo: 0, hi: 0, a: 0, b: 0 }; CAP_DIAGS], diag_n: 0,
        diag_lost: 0, overflow: false, root_first: NODE_NIL, root_n: 0,
    };
    lex(src, &mut m);
    parse(src, &mut m);
    let mut chk = CHK_INIT;
    let host = BOOT;
    let ok = check(src, &mut m, &mut chk, &host);
    if ok {
        emit_image(src, &m, &chk);
    }
}
