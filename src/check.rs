
use crate::ast::*;
use crate::diag::*;
use crate::lex::{Tok, T_UNDERSCORE};
use crate::platform::*;
use crate::{CAP_ARRS, CAP_CONSTS, CAP_ENUMS, CAP_FNS, CAP_LOCALS, CAP_NODES, CAP_REFS, CAP_SLICES,
            CAP_STRS, CAP_STR_POOL, CAP_STRUCTS, CAP_TUPLES, CAP_VALS, FRAME_MAX};

pub const TY_ERR: u16 = 0;
pub const TY_UNIT: u16 = 1;
pub const TY_BOOL: u16 = 2;
pub const TY_I8: u16 = 3;
pub const TY_U8: u16 = 4;
pub const TY_I16: u16 = 5;
pub const TY_U16: u16 = 6;
pub const TY_I32: u16 = 7;
pub const TY_U32: u16 = 8;
pub const TY_I64: u16 = 9;
pub const TY_U64: u16 = 10;
pub const TY_I128: u16 = 11;
pub const TY_U128: u16 = 12;
pub const TY_ISIZE: u16 = 13;
pub const TY_USIZE: u16 = 14;
pub const TY_F64: u16 = 15;
pub const TY_STR: u16 = 16;
pub const TY_INTLIT: u16 = 17;
pub const TY_NEVER: u16 = 18;
pub const TY_ANY: u16 = 0xFFFF;

pub const TY_HINT: u16 = 0x8000;

fn is_hint(e: u16) -> bool {
    e != TY_ANY && e & TY_HINT != 0
}

fn exp_ty(e: u16) -> u16 {
    if e == TY_ANY {
        TY_ANY
    } else {
        e & !TY_HINT
    }
}

pub const TY_STRUCT0: u16 = 0x1000;
pub const TY_ARR0: u16 = 0x2000;
pub const TY_REF0: u16 = 0x3000;
pub const TY_SLICE0: u16 = 0x4000;
pub const TY_TUPLE0: u16 = 0x5000;
pub const TY_ENUM0: u16 = 0x6000;
pub const TUP_MAX: usize = 6;

pub const POOL_TAG: u64 = 1 << 63;

pub fn ty_is_struct(t: u16) -> bool {
    t >= TY_STRUCT0 && t < TY_STRUCT0 + CAP_STRUCTS as u16
}

pub fn ty_is_arr(t: u16) -> bool {
    t >= TY_ARR0 && t < TY_ARR0 + CAP_ARRS as u16
}

pub fn ty_is_ref(t: u16) -> bool {
    t >= TY_REF0 && t < TY_REF0 + CAP_REFS as u16
}

pub fn ty_is_tuple(t: u16) -> bool {
    t >= TY_TUPLE0 && t < TY_TUPLE0 + CAP_TUPLES as u16
}
pub fn ty_is_slice(t: u16) -> bool {
    t >= TY_SLICE0 && t < TY_SLICE0 + CAP_SLICES as u16
}
pub fn ty_is_enum(t: u16) -> bool {
    t >= TY_ENUM0 && t < TY_ENUM0 + CAP_ENUMS as u16
}

pub fn ty_is_128(t: u16) -> bool {
    t == TY_I128 || t == TY_U128
}

pub fn ty_is_int(t: u16) -> bool {
    (t >= TY_I8 && t <= TY_USIZE) || t == TY_INTLIT
}

pub fn ty_is_signed(t: u16) -> bool {
    t == TY_I8 || t == TY_I16 || t == TY_I32 || t == TY_I64 || t == TY_I128 || t == TY_ISIZE || t == TY_INTLIT
}

fn int_bits(t: u16) -> u32 {
    match t {
        TY_I8 | TY_U8 => 8,
        TY_I16 | TY_U16 => 16,
        TY_I32 | TY_U32 => 32,
        TY_I128 | TY_U128 => 128,
        _ => 64,
    }
}

fn ty_is_scalar(t: u16) -> bool {
    t == TY_BOOL || (t >= TY_I8 && t <= TY_F64) || t == TY_STR
}

pub const RES_CONST: u32 = 0x8000_0000;
pub const RES_HOST: u32 = 0x8000_0000;
pub const RES_MASK: u32 = 0x7FFF_FFFF;

pub const RES_MPLACE: u32 = 0x4000_0000;
pub const RES_MFN_MASK: u32 = 0x3FFF_FFFF;

pub const RES_SLICE_LEN: u32 = 0x2000_0000;
pub const RES_ARRAY_LEN: u32 = 0x1000_0000;
pub const RES_ALEN_MASK: u32 = 0x0FFF_FFFF;

pub const RES_PRIM: u32 = 0x0800_0000;
pub const RES_PRIM_MASK: u32 = 0x00FF_FFFF;
pub const PRIM_WRAP_ADD: u32 = 1;
pub const PRIM_WRAP_SUB: u32 = 2;
pub const PRIM_WRAP_MUL: u32 = 3;
pub const PRIM_WRAP_NEG: u32 = 4;
pub const PRIM_WRAP_SHL: u32 = 5;
pub const PRIM_SAT_ADD: u32 = 6;
pub const PRIM_SAT_MUL: u32 = 7;
pub const PRIM_TO_BITS: u32 = 8;
pub const PRIM_IS_NAN: u32 = 9;
pub const PRIM_ROTL: u32 = 10;
pub const PRIM_ROTR: u32 = 11;

/// Is a primitive opcode saturating (vs wrapping)?
pub fn prim_is_sat(op: u32) -> bool {
    op == PRIM_SAT_ADD || op == PRIM_SAT_MUL
}

pub const RES_STR_LEN: u32 = 0x0400_0000;
pub const RES_STR_BYTES: u32 = 0x0200_0000;

const LFLAG_RETSAFE: u16 = 0x8000;

pub const RES_DEREF: u32 = 0x8000_0000;
pub const RES_OFF_MASK: u32 = 0x7FFF_FFFF;

#[derive(Clone, Copy)]
pub struct SInfo {
    pub name_tok: u32,
    pub host: u32,
    pub first_field: u32,
    pub field_n: u32,
    pub size: u32,
    pub state: u16,
    pub derives: u16,
}
pub const SINFO_NONE: SInfo = SInfo {
    name_tok: NODE_NIL,
    host: 0,
    first_field: NODE_NIL,
    field_n: 0,
    size: 0,
    state: 0,
    derives: 0,
};

#[derive(Clone, Copy)]
pub struct EInfo {
    pub name_tok: u32,
    pub first_variant: u32,
    pub variant_n: u32,
    pub size: u32,
    pub state: u16,
}
pub const EINFO_NONE: EInfo = EInfo {
    name_tok: NODE_NIL,
    first_variant: NODE_NIL,
    variant_n: 0,
    size: 0,
    state: 0,
};

#[derive(Clone, Copy)]
pub struct AInfo {
    pub elem: u16,
    pub len: u32,
    pub size: u32,
}
pub const AINFO_NONE: AInfo = AInfo {
    elem: TY_ERR,
    len: 0,
    size: 0,
};

#[derive(Clone, Copy)]
pub struct TInfo {
    pub elems: [u16; TUP_MAX],
    pub offs: [u16; TUP_MAX],
    pub count: u16,
    pub size: u32,
}
pub const TINFO_NONE: TInfo = TInfo {
    elems: [TY_ERR; TUP_MAX],
    offs: [0; TUP_MAX],
    count: 0,
    size: 0,
};

#[derive(Clone, Copy)]
pub struct RInfo {
    pub pointee: u16,
    pub mutable: u16,
}
pub const RINFO_NONE: RInfo = RInfo { pointee: TY_ERR, mutable: 0 };

#[derive(Clone, Copy)]
pub struct CInfo {
    pub name_tok: u32,
    pub node: u32,
    pub ty: u16,
    pub state: u16,
    pub bits: u64,
}
pub const CINFO_NONE: CInfo = CInfo {
    name_tok: NODE_NIL,
    node: NODE_NIL,
    ty: TY_ERR,
    state: 0,
    bits: 0,
};

#[derive(Clone, Copy)]
pub struct FInfo {
    pub name_tok: u32,
    pub node: u32,
    pub first_param: u32,
    pub param_n: u32,
    pub ret: u16,
    pub frame: u32,
    pub self_tok: u32,
    pub self_ty: u16,
}
pub const FINFO_NONE: FInfo = FInfo {
    name_tok: NODE_NIL,
    node: NODE_NIL,
    first_param: NODE_NIL,
    param_n: 0,
    ret: TY_UNIT,
    frame: 0,
    self_tok: NODE_NIL,
    self_ty: TY_ERR,
};

#[derive(Clone, Copy)]
pub struct LInfo {
    pub name_tok: u32,
    pub ty: u16,
    pub flags: u16,
    pub slot: u32,
    pub depth: u16,
    pub init: u32,
}
pub const LINFO_NONE: LInfo = LInfo {
    name_tok: NODE_NIL,
    ty: TY_ERR,
    flags: 0,
    slot: 0,
    depth: 0,
    init: NODE_NIL,
};

#[derive(Clone, Copy)]
pub struct StrEntry {
    pub off: u32,
    pub len: u32,
}
pub const STR_NONE: StrEntry = StrEntry { off: 0, len: 0 };

/// Checker memory: the typed-program tables. Caller-owned, like Mem.
pub struct Chk {
    pub ty: [u16; CAP_NODES],
    pub res: [u32; CAP_NODES],

    pub structs: [SInfo; CAP_STRUCTS],
    pub struct_n: usize,
    pub enums: [EInfo; CAP_ENUMS],
    pub enum_n: usize,
    pub arrs: [AInfo; CAP_ARRS],
    pub arr_n: usize,
    pub refs: [RInfo; CAP_REFS],
    pub ref_n: usize,
    pub slices: [RInfo; CAP_SLICES],
    pub slice_n: usize,
    pub tuples: [TInfo; CAP_TUPLES],
    pub tuple_n: usize,
    pub consts: [CInfo; CAP_CONSTS],
    pub const_n: usize,
    pub fns: [FInfo; CAP_FNS],
    pub fn_n: usize,

    pub vals: [u64; CAP_VALS],
    pub val_n: usize,
    pub strs: [StrEntry; CAP_STRS],
    pub str_n: usize,
    pub str_pool: [u8; CAP_STR_POOL],
    pub pool_n: usize,

    locals: [LInfo; CAP_LOCALS],
    local_n: usize,
    depth: u16,
    loop_depth: u16,
    loop_broke: bool,
    next_slot: u32,
    ret_ty: u16,
    ret_borrow_body: u32,
    ce_depth: u16,
    in_const: bool,
    sizing_done: bool,
}

pub const CHK_INIT: Chk = Chk {
    ty: [TY_ERR; CAP_NODES],
    res: [0; CAP_NODES],
    structs: [SINFO_NONE; CAP_STRUCTS],
    struct_n: 0,
    enums: [EINFO_NONE; CAP_ENUMS],
    enum_n: 0,
    arrs: [AINFO_NONE; CAP_ARRS],
    arr_n: 0,
    refs: [RINFO_NONE; CAP_REFS],
    ref_n: 0,
    slices: [RINFO_NONE; CAP_SLICES],
    slice_n: 0,
    tuples: [TINFO_NONE; CAP_TUPLES],
    tuple_n: 0,
    consts: [CINFO_NONE; CAP_CONSTS],
    const_n: 0,
    fns: [FINFO_NONE; CAP_FNS],
    fn_n: 0,
    vals: [0; CAP_VALS],
    val_n: 0,
    strs: [STR_NONE; CAP_STRS],
    str_n: 0,
    str_pool: [0; CAP_STR_POOL],
    pool_n: 0,
    locals: [LINFO_NONE; CAP_LOCALS],
    local_n: 0,
    depth: 0,
    loop_depth: 0,
    loop_broke: false,
    next_slot: 0,
    ret_ty: TY_UNIT,
    ret_borrow_body: NODE_NIL,
    ce_depth: 0,
    in_const: false,
    sizing_done: false,
};

impl Chk {
    pub fn reset(&mut self) {

        self.struct_n = 0;
        self.enum_n = 0;
        self.arr_n = 0;
        self.ref_n = 0;
        self.slice_n = 0;
        self.tuple_n = 0;
        self.const_n = 0;
        self.fn_n = 0;
        self.val_n = 0;
        self.str_n = 0;
        self.pool_n = 0;
        self.local_n = 0;
        self.depth = 0;
        self.loop_depth = 0;
        self.next_slot = 0;
        self.ret_ty = TY_UNIT;
        self.ce_depth = 0;
        self.in_const = false;
        self.sizing_done = false;
        let mut i = 0;
        while i < CAP_NODES {
            self.ty[i] = TY_ERR;
            self.res[i] = 0;
            i += 1;
        }
    }

    pub fn sinfo(&self, t: u16) -> SInfo {
        let k = (t - TY_STRUCT0) as usize;
        if k < self.struct_n {
            self.structs[k]
        } else {
            SINFO_NONE
        }
    }

    pub fn ainfo(&self, t: u16) -> AInfo {
        let k = (t - TY_ARR0) as usize;
        if k < self.arr_n {
            self.arrs[k]
        } else {
            AINFO_NONE
        }
    }

    pub fn einfo(&self, t: u16) -> EInfo {
        let k = (t - TY_ENUM0) as usize;
        if k < self.enum_n {
            self.enums[k]
        } else {
            EINFO_NONE
        }
    }

    pub fn tinfo(&self, t: u16) -> TInfo {
        let k = (t - TY_TUPLE0) as usize;
        if k < self.tuple_n {
            self.tuples[k]
        } else {
            TINFO_NONE
        }
    }

    pub fn rinfo(&self, t: u16) -> RInfo {
        let k = (t - TY_REF0) as usize;
        if k < self.ref_n {
            self.refs[k]
        } else {
            RINFO_NONE
        }
    }

    /// Slice info (element type in `.pointee`, mutability in `.mutable`).
    pub fn slinfo(&self, t: u16) -> RInfo {
        let k = (t - TY_SLICE0) as usize;
        if k < self.slice_n {
            self.slices[k]
        } else {
            RINFO_NONE
        }
    }

    /// Size of a value of type `t`, in u64 slots.
    pub fn size_of(&self, t: u16) -> u32 {
        if t == TY_UNIT || t == TY_ERR {
            return 0;
        }
        if ty_is_struct(t) {
            return self.sinfo(t).size;
        }
        if ty_is_arr(t) {
            return self.ainfo(t).size;
        }
        if ty_is_ref(t) {
            return 1;
        }
        if ty_is_slice(t) {
            return 2;
        }
        if ty_is_tuple(t) {
            return self.tinfo(t).size;
        }
        if ty_is_enum(t) {
            return self.einfo(t).size;
        }
        if ty_is_128(t) {
            return 2;
        }
        1
    }

    /// Bytes of an unescaped string literal entry.
    pub fn str_bytes(&self, idx: u32) -> &[u8] {
        let idx = idx as usize;
        if idx >= self.str_n {
            return &[];
        }
        let e = self.strs[idx];
        let lo = e.off as usize;
        let hi = lo + e.len as usize;
        if hi <= self.pool_n {
            &self.str_pool[lo..hi]
        } else {
            &[]
        }
    }

    /// `&str` runtime span: (pool offset, byte length) for a string id, both 0
    /// if out of range. Backs `.as_bytes()`/`.len()` on `&str` in the machine.
    pub fn str_span(&self, idx: u32) -> (u64, u64) {
        let idx = idx as usize;
        if idx >= self.str_n {
            return (0, 0);
        }
        let e = self.strs[idx];
        (e.off as u64, e.len as u64)
    }

    /// One byte of the string pool by absolute offset (0 out of range). Used
    /// by the machine to read pool-backed byte-string slices.
    pub fn str_pool_at(&self, off: usize) -> u8 {
        if off < self.pool_n {
            self.str_pool[off]
        } else {
            0
        }
    }
}

fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

fn tok_bytes(src: &str, t: Tok) -> &[u8] {
    let lo = t.pos as usize;
    let hi = lo + t.len as usize;
    let b = src.as_bytes();
    if lo <= hi && hi <= b.len() {
        &b[lo..hi]
    } else {
        &[]
    }
}

fn tok_eq(src: &str, mem: &Mem, a: u32, b: u32) -> bool {
    bytes_eq(tok_bytes(src, mem.tok(a)), tok_bytes(src, mem.tok(b)))
}

fn tok_is(src: &str, mem: &Mem, t: u32, s: &[u8]) -> bool {
    bytes_eq(tok_bytes(src, mem.tok(t)), s)
}

/// Integer type id named by token `t` (i128/u128 excluded — 2-slot, pending
/// machine support). TY_ERR if not a supported int type name.
fn int_ty_named(src: &str, mem: &Mem, t: u32) -> u16 {
    let w = tok_bytes(src, mem.tok(t));
    if bytes_eq(w, b"i8") { return TY_I8; }
    if bytes_eq(w, b"u8") { return TY_U8; }
    if bytes_eq(w, b"i16") { return TY_I16; }
    if bytes_eq(w, b"u16") { return TY_U16; }
    if bytes_eq(w, b"i32") { return TY_I32; }
    if bytes_eq(w, b"u32") { return TY_U32; }
    if bytes_eq(w, b"i64") { return TY_I64; }
    if bytes_eq(w, b"u64") { return TY_U64; }
    if bytes_eq(w, b"i128") { return TY_I128; }
    if bytes_eq(w, b"u128") { return TY_U128; }
    if bytes_eq(w, b"isize") { return TY_ISIZE; }
    if bytes_eq(w, b"usize") { return TY_USIZE; }
    TY_ERR
}

fn ndiag(mem: &mut Mem, code: u16, n: Node, a: u32, b: u32) {
    mem.diag(code, n.lo, n.hi, a, b);
}

/// Check the parsed program in `mem` against the host API. Fills `chk`.
/// Returns true if there were no errors.
pub fn check(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef) -> bool {
    let before = mem.diag_n;
    chk.reset();

    let mut h = 0;
    while h < host.structs.len() {
        if chk.struct_n >= CAP_STRUCTS {
            mem.diag(E_TOO_MANY_ITEMS, 0, 0, 0, 0);
            return false;
        }
        let mut s = SINFO_NONE;
        s.host = h as u32 + 1;
        s.field_n = host.structs[h].fields.len() as u32;
        s.state = 2;

        let mut size = 0;
        let mut f = 0;
        while f < host.structs[h].fields.len() {
            size += host_ty_size(&host.structs[h].fields[f].ty);
            f += 1;
        }
        s.size = size;
        chk.structs[chk.struct_n] = s;
        chk.struct_n += 1;
        h += 1;
    }

    let mut it = mem.root_first;
    while it != NODE_NIL {
        let n = mem.node(it);
        if n.kind == N_STRUCT {
            collect_struct(src, mem, chk, host, it);
        } else if n.kind == N_ENUM {
            collect_enum(src, mem, chk, host, it);
        } else if n.kind == N_CONST {
            collect_const(src, mem, chk, host, it);
        } else if n.kind == N_FN {
            collect_fn(src, mem, chk, host, it, NODE_NIL);
        } else if n.kind == N_IMPL {

            let mut m = n.b;
            while m != NODE_NIL {
                collect_fn(src, mem, chk, host, m, n.a);
                m = mem.node(m).link;
            }
        } else if n.kind == N_USE {
            ndiag(mem, E_USE_UNSUPPORTED, n, 0, 0);
        }
        it = n.link;
    }

    let mut i = 0;
    while i < chk.const_n {
        ck_const(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.enum_n {
        size_enum(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.struct_n {
        size_struct(src, mem, chk, host, i);
        i += 1;
    }
    chk.sizing_done = true;

    let mut i = 0;
    while i < chk.const_n {
        ck_const(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.fn_n {
        sig_fn(src, mem, chk, host, i);
        i += 1;
    }

    let mut i = 0;
    while i < chk.fn_n {
        ck_fn(src, mem, chk, host, i);
        i += 1;
    }

    mem.diag_n == before && !mem.overflow
}

fn host_ty_size(t: &HostTy) -> u32 {
    if t.kind == TY_UNIT {
        return 0;
    }
    if t.kind == HT_ARR {
        return t.len;
    }
    1
}

/// Does ident token `t` name an existing struct/const/fn/host fn?
fn name_taken(src: &str, mem: &Mem, chk: &Chk, host: &HostDef, t: u32) -> bool {
    let w = tok_bytes(src, mem.tok(t));
    let mut i = 0;
    while i < chk.struct_n {
        let s = chk.structs[i];
        if s.host > 0 {
            if bytes_eq(w, host.structs[(s.host - 1) as usize].name.as_bytes()) {
                return true;
            }
        } else if tok_eq(src, mem, s.name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < chk.enum_n {
        if tok_eq(src, mem, chk.enums[i].name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < chk.const_n {
        if tok_eq(src, mem, chk.consts[i].name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < chk.fn_n {
        if tok_eq(src, mem, chk.fns[i].name_tok, t) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < host.fns.len() {
        if bytes_eq(w, host.fns[i].name.as_bytes()) {
            return true;
        }
        i += 1;
    }
    false
}

fn collect_struct(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32) {
    let n = mem.node(it);
    if name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.struct_n >= CAP_STRUCTS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }
    if n.x & DERIVE_CLONE == 0 || n.x & DERIVE_COPY == 0 {
        ndiag(mem, E_MISSING_DERIVE, n, 0, 0);

    }
    let mut s = SINFO_NONE;
    s.name_tok = n.a;
    s.first_field = n.b;
    s.field_n = n.c;
    s.derives = n.x;
    chk.structs[chk.struct_n] = s;
    chk.struct_n += 1;
}

fn collect_enum(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32) {
    let n = mem.node(it);
    if name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.enum_n >= CAP_ENUMS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }

    if n.x & DERIVE_CLONE == 0 || n.x & DERIVE_COPY == 0 {
        ndiag(mem, E_MISSING_DERIVE, n, 0, 0);
    }

    let mut a = n.b;
    while a != NODE_NIL {
        let an = mem.node(a);
        let mut b = an.link;
        while b != NODE_NIL {
            let bn = mem.node(b);
            if tok_eq(src, mem, an.a, bn.a) {
                ndiag(mem, E_DUP_NAME, bn, 0, 0);
            }
            b = bn.link;
        }
        a = an.link;
    }
    let mut e = EINFO_NONE;
    e.name_tok = n.a;
    e.first_variant = n.b;
    e.variant_n = n.c;
    chk.enums[chk.enum_n] = e;
    chk.enum_n += 1;
}

/// Lay out an enum: one tag slot plus the max variant payload. Field-less enums
/// (the current subset) are a single tag slot.
fn size_enum(_src: &str, mem: &mut Mem, chk: &mut Chk, _host: &HostDef, k: usize) {
    if chk.enums[k].state == 2 {
        return;
    }
    chk.enums[k].state = 1;
    let mut v = chk.enums[k].first_variant;
    while v != NODE_NIL {
        let vn = mem.node(v);
        if vn.e != NODE_NIL {
            ndiag(mem, E_ENUM_PAYLOAD, vn, 0, 0);
        }
        v = vn.link;
    }
    chk.enums[k].size = 1;
    chk.enums[k].state = 2;
}

/// Enum index by ident token, u16::MAX if none.
fn find_enum(src: &str, mem: &Mem, chk: &Chk, name_tok: u32) -> u16 {
    let mut i = 0;
    while i < chk.enum_n {
        if tok_eq(src, mem, chk.enums[i].name_tok, name_tok) {
            return i as u16;
        }
        i += 1;
    }
    u16::MAX
}

/// Variant tag (0-based position) by name within enum type `et`, u32::MAX if none.
fn variant_tag(src: &str, mem: &Mem, chk: &Chk, et: u16, name_tok: u32) -> u32 {
    let e = chk.einfo(et);
    let mut v = e.first_variant;
    let mut tag: u32 = 0;
    while v != NODE_NIL {
        let vn = mem.node(v);
        if tok_eq(src, mem, vn.a, name_tok) {
            return tag;
        }
        tag += 1;
        v = vn.link;
    }
    u32::MAX
}

/// Have all variants of enum `et` been matched (bitset `seen` of variant tags)?
fn enum_all_seen(chk: &Chk, et: u16, seen: u64) -> bool {
    let vn = chk.einfo(et).variant_n;
    let full = if vn >= 64 { u64::MAX } else { (1u64 << vn) - 1 };
    seen & full == full
}

fn collect_const(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32) {
    let n = mem.node(it);
    if name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.const_n >= CAP_CONSTS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }
    let mut c = CINFO_NONE;
    c.name_tok = n.a;
    c.node = it;
    chk.consts[chk.const_n] = c;
    chk.const_n += 1;
}

fn collect_fn(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, it: u32, self_tok: u32) {
    let n = mem.node(it);

    if self_tok == NODE_NIL && name_taken(src, mem, chk, host, n.a) {
        ndiag(mem, E_DUP_NAME, n, 0, 0);
        return;
    }
    if chk.fn_n >= CAP_FNS {
        ndiag(mem, E_TOO_MANY_ITEMS, n, 0, 0);
        return;
    }
    let mut f = FINFO_NONE;
    f.name_tok = n.a;
    f.node = it;
    f.first_param = n.b;
    f.param_n = n.c;
    f.self_tok = self_tok;
    chk.fns[chk.fn_n] = f;
    chk.fn_n += 1;
}

/// Resolve a type node to a type id (interning arrays), or TY_ERR (diagnosed).
fn ty_of(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, tn: u32) -> u16 {
    let n = mem.node(tn);
    let t = match n.kind {
        N_TY_UNIT => TY_UNIT,
        N_TY_STR => TY_STR,
        N_TY_REF => {
            let pointee = ty_of(src, mem, chk, host, n.d);
            if pointee == TY_ERR {
                return TY_ERR;
            }
            if ty_is_ref(pointee) {
                ndiag(mem, E_REF_ESCAPES, n, 0, 0);
                return TY_ERR;
            }
            intern_ref(mem, chk, pointee, n.x & FLAG_MUT, n)
        }
        N_TY_SLICE => {
            let elem = ty_of(src, mem, chk, host, n.d);
            if elem == TY_ERR {
                return TY_ERR;
            }
            if elem == TY_UNIT || ty_is_ref(elem) || ty_is_slice(elem) {
                ndiag(mem, if elem == TY_UNIT { E_UNKNOWN_TYPE } else { E_REF_ESCAPES }, n, 0, 0);
                return TY_ERR;
            }
            if ty_is_struct(elem) {
                let k = (elem - TY_STRUCT0) as usize;
                size_struct(src, mem, chk, host, k);
                if chk.structs[k].state != 2 {
                    return TY_ERR;
                }
            }
            intern_slice(mem, chk, elem, n.x & FLAG_MUT, n)
        }
        N_TY_NAME => named_ty(src, mem, chk, host, n.a, n),
        N_TY_ARRAY => {
            let elem = ty_of(src, mem, chk, host, n.d);
            if elem == TY_ERR {
                return TY_ERR;
            }
            if elem == TY_UNIT || ty_is_ref(elem) || ty_is_slice(elem) {
                ndiag(mem, if elem == TY_UNIT { E_UNKNOWN_TYPE } else { E_REF_ESCAPES }, n, 0, 0);
                return TY_ERR;
            }

            if ty_is_struct(elem) {
                let k = (elem - TY_STRUCT0) as usize;
                size_struct(src, mem, chk, host, k);
                if chk.structs[k].state != 2 {
                    return TY_ERR;
                }
            }
            let len = ce_len(src, mem, chk, host, n.e);
            if len == u32::MAX {
                return TY_ERR;
            }
            intern_arr(mem, chk, elem, len, n)
        }
        _ => {
            ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
            TY_ERR
        }
    };
    if tn < CAP_NODES as u32 {
        chk.ty[tn as usize] = t;
    }
    t
}

fn named_ty(src: &str, mem: &mut Mem, chk: &Chk, host: &HostDef, name_tok: u32, n: Node) -> u16 {
    let w = tok_bytes(src, mem.tok(name_tok));
    if bytes_eq(w, b"bool") {
        return TY_BOOL;
    }
    if bytes_eq(w, b"i8") {
        return TY_I8;
    }
    if bytes_eq(w, b"u8") {
        return TY_U8;
    }
    if bytes_eq(w, b"i16") {
        return TY_I16;
    }
    if bytes_eq(w, b"u16") {
        return TY_U16;
    }
    if bytes_eq(w, b"isize") {
        return TY_ISIZE;
    }
    if bytes_eq(w, b"i32") {
        return TY_I32;
    }
    if bytes_eq(w, b"u32") {
        return TY_U32;
    }
    if bytes_eq(w, b"i64") {
        return TY_I64;
    }
    if bytes_eq(w, b"u64") {
        return TY_U64;
    }
    if bytes_eq(w, b"i128") {
        return TY_I128;
    }
    if bytes_eq(w, b"u128") {
        return TY_U128;
    }
    if bytes_eq(w, b"usize") {
        return TY_USIZE;
    }
    if bytes_eq(w, b"f64") {
        return TY_F64;
    }
    let s = find_struct(src, mem, chk, host, name_tok);
    if s != u16::MAX {
        return TY_STRUCT0 + s;
    }
    let e = find_enum(src, mem, chk, name_tok);
    if e != u16::MAX {
        return TY_ENUM0 + e;
    }
    ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
    TY_ERR
}

/// Struct index by ident token, u16::MAX if none.
fn find_struct(src: &str, mem: &Mem, chk: &Chk, host: &HostDef, name_tok: u32) -> u16 {
    let w = tok_bytes(src, mem.tok(name_tok));
    let mut i = 0;
    while i < chk.struct_n {
        let s = chk.structs[i];
        if s.host > 0 {
            if bytes_eq(w, host.structs[(s.host - 1) as usize].name.as_bytes()) {
                return i as u16;
            }
        } else if tok_eq(src, mem, s.name_tok, name_tok) {
            return i as u16;
        }
        i += 1;
    }
    u16::MAX
}

/// Builtin primitive integer method by name, or 0 if not one.
fn prim_op(src: &str, mem: &Mem, name_tok: u32) -> u32 {
    if tok_is(src, mem, name_tok, b"wrapping_add") {
        return PRIM_WRAP_ADD;
    }
    if tok_is(src, mem, name_tok, b"wrapping_sub") {
        return PRIM_WRAP_SUB;
    }
    if tok_is(src, mem, name_tok, b"wrapping_mul") {
        return PRIM_WRAP_MUL;
    }
    if tok_is(src, mem, name_tok, b"wrapping_neg") {
        return PRIM_WRAP_NEG;
    }
    if tok_is(src, mem, name_tok, b"wrapping_shl") {
        return PRIM_WRAP_SHL;
    }
    if tok_is(src, mem, name_tok, b"saturating_add") {
        return PRIM_SAT_ADD;
    }
    if tok_is(src, mem, name_tok, b"saturating_mul") {
        return PRIM_SAT_MUL;
    }
    if tok_is(src, mem, name_tok, b"rotate_left") {
        return PRIM_ROTL;
    }
    if tok_is(src, mem, name_tok, b"rotate_right") {
        return PRIM_ROTR;
    }
    0
}

/// Index of the inherent method named `name_tok` on receiver struct type
/// `sty`, or usize::MAX. Methods are keyed by (receiver type, name).
fn method_find(src: &str, mem: &Mem, chk: &Chk, sty: u16, name_tok: u32) -> usize {
    let mut i = 0;
    while i < chk.fn_n {
        if chk.fns[i].self_ty == sty && tok_eq(src, mem, chk.fns[i].name_tok, name_tok) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

fn intern_ref(mem: &mut Mem, chk: &mut Chk, pointee: u16, mutable: u16, at: Node) -> u16 {
    let mut i = 0;
    while i < chk.ref_n {
        if chk.refs[i].pointee == pointee && chk.refs[i].mutable == mutable {
            return TY_REF0 + i as u16;
        }
        i += 1;
    }
    if chk.ref_n >= CAP_REFS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.refs[chk.ref_n] = RInfo { pointee, mutable };
    chk.ref_n += 1;
    TY_REF0 + (chk.ref_n - 1) as u16
}

fn intern_slice(mem: &mut Mem, chk: &mut Chk, elem: u16, mutable: u16, at: Node) -> u16 {
    let mut i = 0;
    while i < chk.slice_n {
        if chk.slices[i].pointee == elem && chk.slices[i].mutable == mutable {
            return TY_SLICE0 + i as u16;
        }
        i += 1;
    }
    if chk.slice_n >= CAP_SLICES {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.slices[chk.slice_n] = RInfo { pointee: elem, mutable };
    chk.slice_n += 1;
    TY_SLICE0 + (chk.slice_n - 1) as u16
}

/// Check `&[mut] base[lo..hi]` (an N_SLICE under an N_REFOF) and return the
/// resulting `&[T]` type. Records whether the base is itself a slice (in
/// `chk.res[slice_node]`) so the machine evals vs place-addresses it.
fn ck_slice(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, slice_node: u32,
            want_mut: u16, at: Node) -> u16 {
    let sn = mem.node(slice_node);
    let bt = ck_expr(src, mem, chk, host, sn.d, TY_ANY);
    if bt == TY_ERR {
        return TY_ERR;
    }
    let (elem, base_mut, base_is_slice) = if ty_is_arr(bt) {
        let mut rm = false;
        let pt = ck_place(src, mem, chk, host, sn.d, &mut rm);
        if pt == TY_ERR {
            return TY_ERR;
        }
        (chk.ainfo(bt).elem, rm, false)
    } else if ty_is_slice(bt) {
        let si = chk.slinfo(bt);
        (si.pointee, si.mutable != 0, true)
    } else {
        ndiag(mem, E_NOT_AN_ARRAY, sn, 0, bt as u32);
        return TY_ERR;
    };
    if want_mut != 0 && !base_mut {
        ndiag(mem, E_REF_MUT_NEEDED, at, 0, 0);
        return TY_ERR;
    }
    let _ = ck_ex(src, mem, chk, host, sn.b, TY_USIZE);
    if sn.c != NODE_NIL {
        let _ = ck_ex(src, mem, chk, host, sn.c, TY_USIZE);
    }
    chk.res[slice_node as usize] = if base_is_slice { 1 } else { 0 };
    let t = intern_slice(mem, chk, elem, if want_mut != 0 { FLAG_MUT } else { 0 }, at);
    if (slice_node as usize) < CAP_NODES {
        chk.ty[slice_node as usize] = t;
    }
    t
}

fn intern_arr(mem: &mut Mem, chk: &mut Chk, elem: u16, len: u32, at: Node) -> u16 {
    let size = chk.size_of(elem).saturating_mul(len);
    let mut i = 0;
    while i < chk.arr_n {
        if chk.arrs[i].elem == elem && chk.arrs[i].len == len {
            return TY_ARR0 + i as u16;
        }
        i += 1;
    }
    if chk.arr_n >= CAP_ARRS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.arrs[chk.arr_n] = AInfo { elem, len, size };
    chk.arr_n += 1;
    TY_ARR0 + (chk.arr_n - 1) as u16
}

/// Intern a tuple type from its element types (positional). Computes each
/// element's slot offset and the total size; dedups structurally.
fn intern_tuple(mem: &mut Mem, chk: &mut Chk, elems: &[u16], count: usize, at: Node) -> u16 {
    let mut info = TINFO_NONE;
    info.count = count as u16;
    let mut off = 0u32;
    let mut e = 0;
    while e < count {
        info.elems[e] = elems[e];
        info.offs[e] = off as u16;
        off += chk.size_of(elems[e]);
        e += 1;
    }
    info.size = off;

    let mut i = 0;
    while i < chk.tuple_n {
        let t = chk.tuples[i];
        if t.count == info.count {
            let mut same = true;
            let mut k = 0;
            while k < count {
                if t.elems[k] != info.elems[k] {
                    same = false;
                }
                k += 1;
            }
            if same {
                return TY_TUPLE0 + i as u16;
            }
        }
        i += 1;
    }
    if chk.tuple_n >= CAP_TUPLES {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return TY_ERR;
    }
    chk.tuples[chk.tuple_n] = info;
    chk.tuple_n += 1;
    TY_TUPLE0 + (chk.tuple_n - 1) as u16
}

/// Resolve a host signature type at a use site.
fn host_ty(_src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, t: &HostTy, at: Node) -> u16 {
    if t.kind == HT_ARR {
        return intern_arr(mem, chk, t.elem, t.len, at);
    }
    if t.kind == HT_STRUCT {
        let mut i = 0;
        while i < chk.struct_n {
            let s = chk.structs[i];
            if s.host > 0 && bytes_eq(host.structs[(s.host - 1) as usize].name.as_bytes(),
                                       t.sname.as_bytes()) {
                return TY_STRUCT0 + i as u16;
            }
            i += 1;
        }
        ndiag(mem, E_UNKNOWN_TYPE, at, 0, 0);
        return TY_ERR;
    }
    t.kind
}

/// Compute a user struct's field offsets and total size (cycle-safe).
fn size_struct(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) {
    if chk.structs[k].state == 2 {
        return;
    }
    if chk.structs[k].state == 1 {
        let n = mem.node(chk.structs[k].first_field);
        ndiag(mem, E_RECURSIVE_STRUCT, n, 0, 0);
        chk.structs[k].state = 2;
        return;
    }
    chk.structs[k].state = 1;
    let mut size: u32 = 0;
    let mut f = chk.structs[k].first_field;
    while f != NODE_NIL {
        let fnode = mem.node(f);
        let ft = ty_of(src, mem, chk, host, fnode.e);
        let ft = if ft == TY_STR {

            ndiag(mem, E_STR_FIELD, fnode, 0, 0);
            TY_ERR
        } else if ty_is_ref(ft) || ty_is_slice(ft) {
            ndiag(mem, E_REF_ESCAPES, fnode, 0, 0);
            TY_ERR
        } else {
            ft
        };
        if ty_is_struct(ft) {
            size_struct(src, mem, chk, host, (ft - TY_STRUCT0) as usize);
        }
        chk.ty[f as usize] = ft;
        chk.res[f as usize] = size;
        size += chk.size_of(ft);
        f = fnode.link;
    }
    chk.structs[k].size = size;
    chk.structs[k].state = 2;
}

fn sig_fn(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) {
    let f = chk.fns[k];
    let n = mem.node(f.node);

    if f.self_tok != NODE_NIL {
        let s = find_struct(src, mem, chk, host, f.self_tok);
        if s == u16::MAX {
            ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
        } else {
            let sty = TY_STRUCT0 + s;
            chk.fns[k].self_ty = sty;
            let mut j = 0;
            while j < k {
                if chk.fns[j].self_ty == sty
                    && tok_eq(src, mem, chk.fns[j].name_tok, f.name_tok)
                {
                    ndiag(mem, E_DUP_NAME, n, 0, 0);
                    break;
                }
                j += 1;
            }
        }
    }
    let mut p = f.first_param;
    while p != NODE_NIL {
        let pn = mem.node(p);
        let pt = ty_of(src, mem, chk, host, pn.e);
        chk.ty[p as usize] = pt;
        p = pn.link;
    }
    let ret = if n.d == NODE_NIL {
        TY_UNIT
    } else {
        ty_of(src, mem, chk, host, n.d)
    };

    chk.fns[k].ret = ret;
}

fn push_val(mem: &mut Mem, chk: &mut Chk, bits: u64, at: Node) -> u32 {
    if chk.val_n >= CAP_VALS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return 0;
    }
    chk.vals[chk.val_n] = bits;
    chk.val_n += 1;
    (chk.val_n - 1) as u32
}

/// Decode an integer literal's magnitude (dec or hex, underscores, suffix
/// stripped). u64::MAX as a sentinel would collide with a real literal, so
/// failure is reported through the ok flag in the high result via diag.
fn int_mag(src: &str, mem: &mut Mem, chk: &Chk, tok_i: u32, at: Node) -> u64 {
    let _ = chk;
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut mag: u64 = 0;
    let mut i = 0;
    let mut hex = false;
    if w.len() >= 2 && w[0] == b'0' && (w[1] == b'x' || w[1] == b'X') {
        hex = true;
        i = 2;
    }
    let mut overflow = false;
    while i < w.len() {
        let c = w[i];
        if c == b'_' {
            i += 1;
            continue;
        }
        let d;
        if c >= b'0' && c <= b'9' {
            d = (c - b'0') as u64;
        } else if hex && c >= b'a' && c <= b'f' {
            d = (c - b'a' + 10) as u64;
        } else if hex && c >= b'A' && c <= b'F' {
            d = (c - b'A' + 10) as u64;
        } else {
            break;
        }
        let base: u64 = if hex { 16 } else { 10 };
        let m1 = mag.wrapping_mul(base);
        if mag != 0 && m1 / base != mag {
            overflow = true;
        }
        let m2 = m1.wrapping_add(d);
        if m2 < m1 {
            overflow = true;
        }
        mag = m2;
        i += 1;
    }
    if overflow {
        ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
        return 0;
    }
    mag
}

/// Literal suffix -> concrete type, or TY_INTLIT if unsuffixed.
fn int_suffix(src: &str, mem: &Mem, tok_i: u32) -> u16 {
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut s = 0;
    let hex = w.len() >= 2 && w[0] == b'0' && (w[1] == b'x' || w[1] == b'X');
    let mut i = if hex { 2 } else { 0 };
    while i < w.len() {
        let c = w[i];
        let is_digit = (c >= b'0' && c <= b'9')
            || (hex && ((c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')));
        if !is_digit && c != b'_' {
            s = i;
            break;
        }
        i += 1;
    }
    if s == 0 {
        return TY_INTLIT;
    }
    let sfx = &w[s..];
    if bytes_eq(sfx, b"i8") {
        return TY_I8;
    }
    if bytes_eq(sfx, b"u8") {
        return TY_U8;
    }
    if bytes_eq(sfx, b"i16") {
        return TY_I16;
    }
    if bytes_eq(sfx, b"u16") {
        return TY_U16;
    }
    if bytes_eq(sfx, b"isize") {
        return TY_ISIZE;
    }
    if bytes_eq(sfx, b"i32") {
        return TY_I32;
    }
    if bytes_eq(sfx, b"u32") {
        return TY_U32;
    }
    if bytes_eq(sfx, b"i64") {
        return TY_I64;
    }
    if bytes_eq(sfx, b"u64") {
        return TY_U64;
    }
    if bytes_eq(sfx, b"i128") {
        return TY_I128;
    }
    if bytes_eq(sfx, b"u128") {
        return TY_U128;
    }
    if bytes_eq(sfx, b"usize") {
        return TY_USIZE;
    }
    TY_INTLIT
}

fn int_range_ok(mag: u64, neg: bool, t: u16) -> bool {
    let b = int_bits(t);
    if ty_is_signed(t) {
        let half: u64 = 1u64 << (b - 1);
        if neg {
            mag <= half
        } else {
            mag <= half - 1
        }
    } else {
        if neg {
            return false;
        }
        if b == 64 {
            true
        } else {
            mag <= (1u64 << b) - 1
        }
    }
}

/// Parse an f64 literal exactly as rustc does (core's correctly-rounded
/// parser — hand-rolling this would risk L2 divergence).
fn parse_f64(src: &str, mem: &mut Mem, tok_i: u32, at: Node) -> u64 {
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut buf = [0u8; 64];
    let mut n = 0;
    let mut i = 0;
    while i < w.len() {
        let c = w[i];
        if c == b'_' {
            i += 1;
            continue;
        }

        if c == b'f' {
            break;
        }
        if n >= 64 {
            ndiag(mem, E_BAD_NUMBER, at, 0, 0);
            return 0;
        }
        buf[n] = c;
        n += 1;
        i += 1;
    }
    let s = core::str::from_utf8(&buf[..n]).unwrap_or("");
    match s.parse::<f64>() {
        Ok(v) => v.to_bits(),
        Err(_) => {
            ndiag(mem, E_BAD_NUMBER, at, 0, 0);
            0
        }
    }
}

/// Unescape a string literal token into the pool; returns a str entry index.
/// Decode a byte literal `b'x'` (or `b'\n'`) to its u8 value.
fn byte_lit_val(src: &str, mem: &Mem, tok_i: u32) -> u64 {
    let w = tok_bytes(src, mem.tok(tok_i));
    if w.len() < 4 {
        return 0;
    }
    let inner = &w[2..w.len() - 1];
    let byte: u8 = if inner.len() >= 2 && inner[0] == b'\\' {
        match inner[1] {
            b'n' => 10,
            b'r' => 13,
            b't' => 9,
            b'0' => 0,
            b'\\' => 92,
            b'\'' => 39,
            b'"' => 34,
            other => other,
        }
    } else if inner.len() >= 1 {
        inner[0]
    } else {
        0
    };
    byte as u64
}

/// Intern a `"`-delimited literal's unescaped bytes into the string pool,
/// returning its dedup'd id. `prefix` is the opener length: 1 for `"..."`,
/// 2 for a byte string `b"..."`.
fn intern_str(src: &str, mem: &mut Mem, chk: &mut Chk, tok_i: u32, prefix: usize, at: Node) -> u32 {
    let w = tok_bytes(src, mem.tok(tok_i));

    let inner = if w.len() >= prefix + 1 { &w[prefix..w.len() - 1] } else { &w[0..0] };
    let start = chk.pool_n;
    let mut i = 0;
    while i < inner.len() {
        let c = inner[i];
        let out;
        if c == b'\\' && i + 1 < inner.len() {
            let e = inner[i + 1];
            out = match e {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'0' => 0,
                _ => e,
            };
            i += 2;
        } else {
            out = c;
            i += 1;
        }
        if chk.pool_n >= CAP_STR_POOL {
            ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
            return 0;
        }
        chk.str_pool[chk.pool_n] = out;
        chk.pool_n += 1;
    }

    let len = (chk.pool_n - start) as u32;
    let mut k = 0;
    while k < chk.str_n {
        let e = chk.strs[k];
        if e.len == len {
            let a_lo = e.off as usize;
            let mut same = true;
            let mut j = 0;
            while j < len as usize {
                if chk.str_pool[a_lo + j] != chk.str_pool[start + j] {
                    same = false;
                    break;
                }
                j += 1;
            }
            if same {
                chk.pool_n = start;
                return k as u32;
            }
        }
        k += 1;
    }
    if chk.str_n >= CAP_STRS {
        ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
        return 0;
    }
    chk.strs[chk.str_n] = StrEntry {
        off: start as u32,
        len,
    };
    chk.str_n += 1;
    (chk.str_n - 1) as u32
}

/// Retype an INTLIT expression tree to concrete int type `t`, folding a
/// direct negation into the literal (res[unary] = 1) so `-9223…808i64`
/// never executes a runtime negation — exactly rustc's behavior.
fn finalize_int(src: &str, mem: &mut Mem, chk: &mut Chk, node: u32, t: u16) {
    let n = mem.node(node);
    let i = node as usize;
    match n.kind {
        N_LIT_INT => {
            chk.res[i] = store_int_lit(src, mem, chk, n.a, false, t, n);
            chk.ty[i] = t;
        }
        N_UNARY => {
            if n.x == OP_NEG {
                if !ty_is_signed(t) {
                    ndiag(mem, E_NEG_UNSIGNED, n, 0, 0);
                    chk.ty[i] = TY_ERR;
                    return;
                }
                let opn = mem.node(n.e);
                if opn.kind == N_LIT_INT {

                    chk.res[n.e as usize] = store_int_lit(src, mem, chk, opn.a, true, t, opn);
                    chk.ty[n.e as usize] = t;
                    chk.res[i] = 1;
                    chk.ty[i] = t;
                    return;
                }
            }
            finalize_int(src, mem, chk, n.e, t);
            chk.ty[i] = t;
        }
        N_BINARY => {
            finalize_int(src, mem, chk, n.d, t);
            if n.x != OP_SHL && n.x != OP_SHR {
                finalize_int(src, mem, chk, n.e, t);
            }
            chk.ty[i] = t;
        }
        N_BLOCK => {
            if n.e != NODE_NIL {
                finalize_int(src, mem, chk, n.e, t);
            }
            chk.ty[i] = t;
        }
        N_IF => {

            finalize_int(src, mem, chk, n.e, t);
            if n.b != NODE_NIL {
                finalize_int(src, mem, chk, n.b, t);
            }
            chk.ty[i] = t;
        }
        N_NAME => {

            resolve_int_local(src, mem, chk, n.a, t);
            chk.ty[i] = t;
        }
        _ => {

            chk.ty[i] = t;
        }
    }
}

/// If `name_tok` names a still-polymorphic integer `let` local (bound `TY_INTLIT`
/// pending inference), fix its type to `t` and finalize its recorded
/// initializer (decoding + range-checking it at the real type). No-op otherwise.
fn resolve_int_local(src: &str, mem: &mut Mem, chk: &mut Chk, name_tok: u32, t: u16) {
    let li = local_find(src, mem, chk, name_tok);
    if li == usize::MAX || chk.locals[li].ty != TY_INTLIT {
        return;
    }
    chk.locals[li].ty = t;
    let init = chk.locals[li].init;
    if init != NODE_NIL {
        finalize_int(src, mem, chk, init, t);
    }
}

/// Concretize an INTLIT result at a commit point (rustc's i32 default).
fn concrete(src: &str, mem: &mut Mem, chk: &mut Chk, node: u32, t: u16) -> u16 {
    if t == TY_INTLIT {
        finalize_int(src, mem, chk, node, TY_I32);
        return TY_I32;
    }
    t
}

fn ck_const(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) -> u16 {
    let st = chk.consts[k].state;
    if st == 2 {
        return chk.consts[k].ty;
    }
    if st == 3 {
        return TY_ERR;
    }
    if st == 1 {
        let n = mem.node(chk.consts[k].node);
        ndiag(mem, E_CONST_CYCLE, n, 0, 0);
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    chk.consts[k].state = 1;
    let cn = mem.node(chk.consts[k].node);
    let t = ty_of(src, mem, chk, host, cn.d);
    if t == TY_ERR {
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    let is_agg = ty_is_struct(t) || ty_is_arr(t);
    if is_agg && !chk.sizing_done {

        chk.consts[k].state = 0;
        return TY_ERR;
    }
    if !ty_is_scalar(t) && !is_agg {
        ndiag(mem, E_CONST_TYPE, cn, 0, 0);
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    chk.in_const = true;
    let et = ck_ex(src, mem, chk, host, cn.e, t);
    chk.in_const = false;
    if et == TY_ERR {
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    let mut ok = true;
    let bits = if is_agg {
        ce_agg(src, mem, chk, host, cn.e, t, &mut ok)
    } else {
        ce(src, mem, chk, host, cn.e, &mut ok)
    };
    if !ok {
        chk.consts[k].state = 3;
        return TY_ERR;
    }
    chk.consts[k].ty = t;
    chk.consts[k].bits = bits;
    chk.consts[k].state = 2;
    t
}

/// Const-evaluate an aggregate (struct/array) initializer into a run of slots
/// in `chk.vals`; returns the base offset, stored as the const's `bits`. The
/// machine copies `size_of(t)` slots from there when the const is named.
fn ce_agg(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, t: u16,
          ok: &mut bool) -> u64 {
    let size = chk.size_of(t) as usize;
    let base = chk.val_n;
    if base + size > CAP_VALS {
        ndiag(mem, E_TOO_MANY_ITEMS, mem.node(node), 0, 0);
        *ok = false;
        return 0;
    }
    chk.val_n += size;
    ce_slots(src, mem, chk, host, node, base, ok);
    base as u64
}

/// Write the const value of `node` into `chk.vals[base..base + size_of(ty)]`,
/// matching the machine's slot layout for aggregates exactly (struct fields at
/// their offsets, array elements in order).
fn ce_slots(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, base: usize,
            ok: &mut bool) {
    let n = mem.node(node);
    let i = node as usize;
    let t = chk.ty[i];
    match n.kind {
        N_STRUCT_LIT => {
            let mut init = n.b;
            while init != NODE_NIL {
                let fin = mem.node(init);
                if fin.e == NODE_NIL {

                    ndiag(mem, E_NOT_CONST, fin, 0, 0);
                    *ok = false;
                    return;
                }
                let off = (chk.res[init as usize] & 0xFFFF) as usize;
                ce_slots(src, mem, chk, host, fin.e, base + off, ok);
                if !*ok {
                    return;
                }
                init = fin.link;
            }
        }
        N_ARRAY_LIT => {
            let es = chk.size_of(chk.ainfo(t).elem) as usize;
            let mut el = n.b;
            let mut idx = 0usize;
            while el != NODE_NIL {
                ce_slots(src, mem, chk, host, el, base + idx * es, ok);
                if !*ok {
                    return;
                }
                idx += 1;
                el = mem.node(el).link;
            }
        }
        N_ARRAY_REPEAT => {
            let a = chk.ainfo(t);
            let es = chk.size_of(a.elem) as usize;
            ce_slots(src, mem, chk, host, n.d, base, ok);
            if !*ok {
                return;
            }
            let mut r = 1usize;
            while r < a.len as usize {
                let mut k = 0;
                while k < es {
                    chk.vals[base + r * es + k] = chk.vals[base + k];
                    k += 1;
                }
                r += 1;
            }
        }
        _ => {
            if n.kind == N_NAME && chk.res[i] & RES_CONST != 0 {

                let k = (chk.res[i] & RES_MASK) as usize;
                let ct = ck_const(src, mem, chk, host, k);
                if ct == TY_ERR {
                    *ok = false;
                    return;
                }
                let size = chk.size_of(t) as usize;
                if ty_is_struct(t) || ty_is_arr(t) {
                    let sb = chk.consts[k].bits as usize;
                    let mut kk = 0;
                    while kk < size {
                        chk.vals[base + kk] = chk.vals[sb + kk];
                        kk += 1;
                    }
                } else {
                    chk.vals[base] = chk.consts[k].bits;
                }
            } else if chk.size_of(t) as usize == 1 {

                let v = ce(src, mem, chk, host, node, ok);
                if *ok {
                    chk.vals[base] = v;
                }
            } else {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                *ok = false;
            }
        }
    }
}

/// Const-evaluate a checked expression to bits. Sets *ok=false (diagnosed)
/// on non-const constructs or overflow — rustc errors there too.
fn ce(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, ok: &mut bool) -> u64 {
    if chk.ce_depth > 64 {
        *ok = false;
        return 0;
    }
    chk.ce_depth += 1;
    let r = ce_inner(src, mem, chk, host, node, ok);
    chk.ce_depth -= 1;
    r
}

fn ce_inner(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, ok: &mut bool) -> u64 {
    let n = mem.node(node);
    let i = node as usize;
    let t = chk.ty[i];
    if ty_is_128(t) {

        ndiag(mem, E_NOT_CONST, n, 0, 0);
        *ok = false;
        return 0;
    }
    match n.kind {
        N_LIT_INT | N_LIT_FLOAT | N_LIT_BYTE => chk.vals[chk.res[i] as usize],
        N_LIT_BOOL => n.x as u64,
        N_LIT_STR => chk.res[i] as u64,
        N_NAME => {
            if chk.res[i] & RES_CONST != 0 {
                let k = (chk.res[i] & RES_MASK) as usize;
                let ct = ck_const(src, mem, chk, host, k);
                if ct == TY_ERR {
                    *ok = false;
                    return 0;
                }
                chk.consts[k].bits
            } else {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                *ok = false;
                0
            }
        }
        N_UNARY => {
            if chk.res[i] == 1 {

                return ce(src, mem, chk, host, n.e, ok);
            }
            let v = ce(src, mem, chk, host, n.e, ok);
            if !*ok {
                return 0;
            }
            let mut err = CeErr { any: false };
            let r = un_op(n.x, v, t, &mut err);
            if err.any {
                ndiag(mem, E_CONST_OVERFLOW, n, 0, 0);
                *ok = false;
                return 0;
            }
            r
        }
        N_BINARY => {
            let a = ce(src, mem, chk, host, n.d, ok);
            if !*ok {
                return 0;
            }
            let b = ce(src, mem, chk, host, n.e, ok);
            if !*ok {
                return 0;
            }
            let ot = chk.ty[n.d as usize];
            let mut err = CeErr { any: false };
            let r = ce_bin(n.x, a, b, ot, &mut err);
            if err.any {
                ndiag(mem, E_CONST_OVERFLOW, n, 0, 0);
                *ok = false;
                return 0;
            }
            r
        }
        N_CAST => {
            let v = ce(src, mem, chk, host, n.d, ok);
            if !*ok {
                return 0;
            }
            cast_bits(v, chk.ty[n.d as usize], t)
        }
        _ => {
            ndiag(mem, E_NOT_CONST, n, 0, 0);
            *ok = false;
            0
        }
    }
}

pub struct CeErr {
    pub any: bool,
}

/// Unary op on sign-extended bits; debug-profile semantics (neg overflow =
/// error). Shared by const eval and the machine.
pub fn un_op(op: u16, v: u64, t: u16, err: &mut CeErr) -> u64 {
    if op == OP_NOT {
        if t == TY_BOOL {
            return (v == 0) as u64;
        }
        return mask_to(!v, t);
    }

    if t == TY_F64 {
        return (-f64::from_bits(v)).to_bits();
    }
    if v == min_signed(t) {
        err.any = true;
        return 0;
    }
    mask_to(v.wrapping_neg(), t)
}

/// Function index by source name (entry-point lookup), or usize::MAX.
pub fn fn_find_name(src: &str, mem: &Mem, chk: &Chk, name: &str) -> usize {
    let mut i = 0;
    while i < chk.fn_n {
        if bytes_eq(tok_bytes(src, mem.tok(chk.fns[i].name_tok)), name.as_bytes()) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

pub fn min_signed(t: u16) -> u64 {
    match int_bits(t) {
        8 => 0xFFFF_FFFF_FFFF_FF80,
        16 => 0xFFFF_FFFF_FFFF_8000,
        32 => 0xFFFF_FFFF_8000_0000,
        _ => 0x8000_0000_0000_0000,
    }
}

fn int_cmp(a: u64, b: u64, signed: bool) -> i8 {
    if signed {
        let x = a as i64;
        let y = b as i64;
        if x < y {
            return -1;
        }
        if x > y {
            return 1;
        }
        return 0;
    }
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    0
}

/// Truncate/sign-extend `v` to type t's width, stored sign-extended in u64.
pub fn mask_to(v: u64, t: u16) -> u64 {
    let b = int_bits(t);
    if b == 64 {
        return v;
    }
    let m = (1u64 << b) - 1;
    let x = v & m;
    if ty_is_signed(t) && (x >> (b - 1)) & 1 == 1 {
        x | !m
    } else {
        x
    }
}

/// A wrapping primitive op on 1-slot integers (sign-extended bits in/out).
/// Never traps — `.wrapping_*()` is defined for every input.
pub fn wrap_prim(op: u32, a: u64, b: u64, t: u16) -> u64 {
    let r = match op {
        PRIM_WRAP_ADD => a.wrapping_add(b),
        PRIM_WRAP_SUB => a.wrapping_sub(b),
        PRIM_WRAP_MUL => a.wrapping_mul(b),
        PRIM_WRAP_NEG => 0u64.wrapping_sub(a),
        PRIM_WRAP_SHL => {
            let w = int_bits(t) as u64;
            a.wrapping_shl((b % w) as u32)
        }
        PRIM_ROTL | PRIM_ROTR => {
            let w = int_bits(t) as u32;
            let mut sh = (b as u32) % w;
            if op == PRIM_ROTR {
                sh = (w - sh) % w;
            }
            let m = if w >= 64 { u64::MAX } else { (1u64 << w) - 1 };
            let v = a & m;
            (v << sh) | (v >> ((w - sh) % w))
        }
        _ => 0,
    };
    mask_to(r, t)
}

/// A wrapping primitive op on 128-bit integers (2-slot).
pub fn wrap_prim128(op: u32, a: u128, b: u128) -> u128 {
    match op {
        PRIM_WRAP_ADD => a.wrapping_add(b),
        PRIM_WRAP_SUB => a.wrapping_sub(b),
        PRIM_WRAP_MUL => a.wrapping_mul(b),
        PRIM_WRAP_NEG => 0u128.wrapping_sub(a),
        PRIM_WRAP_SHL => a.wrapping_shl((b % 128) as u32),
        PRIM_ROTL => a.rotate_left((b % 128) as u32),
        PRIM_ROTR => a.rotate_right((b % 128) as u32),
        _ => 0,
    }
}

/// A saturating primitive op on 1-slot integers: clamp to the type's range.
pub fn sat_prim(op: u32, a: u64, b: u64, t: u16) -> u64 {
    let w = int_bits(t);
    if ty_is_signed(t) {
        let sa = a as i64 as i128;
        let sb = b as i64 as i128;
        let r = if op == PRIM_SAT_ADD { sa + sb } else { sa * sb };
        let max = (1i128 << (w - 1)) - 1;
        let min = -(1i128 << (w - 1));
        let c = if r > max { max } else if r < min { min } else { r };
        mask_to(c as u64, t)
    } else {
        let ua = a as u128;
        let ub = b as u128;
        let r = if op == PRIM_SAT_ADD { ua + ub } else { ua * ub };
        let max: u128 = if w == 64 { u64::MAX as u128 } else { (1u128 << w) - 1 };
        let c = if r > max { max } else { r };
        mask_to(c as u64, t)
    }
}

/// A saturating primitive op on 128-bit integers (2-slot).
pub fn sat_prim128(op: u32, a: u128, b: u128, signed: bool) -> u128 {
    if signed {
        let sa = a as i128;
        let sb = b as i128;
        let r = if op == PRIM_SAT_ADD { sa.saturating_add(sb) } else { sa.saturating_mul(sb) };
        r as u128
    } else if op == PRIM_SAT_ADD {
        a.saturating_add(b)
    } else {
        a.saturating_mul(b)
    }
}

/// Constant binary op on sign-extended bits; debug-profile semantics
/// (overflow/div0 = error). Also used by the machine in M3.
pub fn ce_bin(op: u16, a: u64, b: u64, t: u16, err: &mut CeErr) -> u64 {
    if t == TY_F64 {
        let x = f64::from_bits(a);
        let y = f64::from_bits(b);
        return match op {
            OP_ADD => (x + y).to_bits(),
            OP_SUB => (x - y).to_bits(),
            OP_MUL => (x * y).to_bits(),
            OP_DIV => (x / y).to_bits(),
            OP_REM => (x % y).to_bits(),
            OP_EQ => (x == y) as u64,
            OP_NE => (x != y) as u64,
            OP_LT => (x < y) as u64,
            OP_LE => (x <= y) as u64,
            OP_GT => (x > y) as u64,
            OP_GE => (x >= y) as u64,
            _ => {
                err.any = true;
                0
            }
        };
    }
    if t == TY_BOOL {
        let x = a != 0;
        let y = b != 0;
        return match op {
            OP_AND | OP_BAND => (x && y) as u64,
            OP_OR | OP_BOR => (x || y) as u64,
            OP_BXOR => (x ^ y) as u64,
            OP_EQ => (x == y) as u64,
            OP_NE => (x != y) as u64,
            _ => {
                err.any = true;
                0
            }
        };
    }
    if t == TY_STR {
        return match op {
            OP_EQ => (a == b) as u64,
            OP_NE => (a != b) as u64,
            _ => {
                err.any = true;
                0
            }
        };
    }

    let signed = ty_is_signed(t);
    let bits = int_bits(t);
    match op {
        OP_EQ => return (a == b) as u64,
        OP_NE => return (a != b) as u64,
        OP_LT => return (int_cmp(a, b, signed) < 0) as u64,
        OP_LE => return (int_cmp(a, b, signed) <= 0) as u64,
        OP_GT => return (int_cmp(a, b, signed) > 0) as u64,
        OP_GE => return (int_cmp(a, b, signed) >= 0) as u64,
        OP_BAND => return a & b,
        OP_BOR => return a | b,
        OP_BXOR => return mask_to(a ^ b, t),
        _ => {}
    }
    if op == OP_SHL || op == OP_SHR {

        if b >= bits as u64 {
            err.any = true;
            return 0;
        }
        let sh = b as u32;
        return if op == OP_SHL {
            let r = mask_to(a.wrapping_shl(sh), t);

            r
        } else if signed {
            mask_to(((a as i64) >> sh) as u64, t)
        } else {
            let m = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
            (a & m) >> sh
        };
    }
    if op == OP_DIV || op == OP_REM {
        if b == 0 {
            err.any = true;
            return 0;
        }
        if signed {
            let x = a as i64;
            let y = b as i64;
            if x == (min_signed(t) as i64) && y == -1 {
                err.any = true;
                return 0;
            }
            let r = if op == OP_DIV { x / y } else { x % y };
            return mask_to(r as u64, t);
        }
        let m = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
        let x = a & m;
        let y = b & m;
        return if op == OP_DIV { x / y } else { x % y };
    }

    let r = match op {
        OP_ADD => a.wrapping_add(b),
        OP_SUB => a.wrapping_sub(b),
        OP_MUL => a.wrapping_mul(b),
        _ => {
            err.any = true;
            return 0;
        }
    };
    let rm = mask_to(r, t);
    let of = if signed {
        let x = a as i64 as i128;
        let y = b as i64 as i128;
        let w = match op {
            OP_ADD => x + y,
            OP_SUB => x - y,
            _ => x * y,
        };
        w != (rm as i64 as i128)
    } else {
        let m = if bits == 64 {
            u128::from(u64::MAX)
        } else {
            u128::from((1u64 << bits) - 1)
        };
        let x = u128::from(a & (m as u64));
        let y = u128::from(b & (m as u64));
        let w = match op {
            OP_ADD => x + y,
            OP_SUB => {
                if x < y {
                    err.any = true;
                    return 0;
                }
                x - y
            }
            _ => x * y,
        };
        w != u128::from(rm)
    };
    if of {
        err.any = true;
        return 0;
    }
    rm
}

/// 128-bit binary op (native i128/u128); comparisons return 0/1. Shared by
/// the machine for TY_I128/TY_U128 operands.
pub fn ce_bin128(op: u16, a: u128, b: u128, signed: bool, err: &mut CeErr) -> u128 {
    if op == OP_EQ { return (a == b) as u128; }
    if op == OP_NE { return (a != b) as u128; }
    if op == OP_LT { return (if signed { (a as i128) < (b as i128) } else { a < b }) as u128; }
    if op == OP_LE { return (if signed { (a as i128) <= (b as i128) } else { a <= b }) as u128; }
    if op == OP_GT { return (if signed { (a as i128) > (b as i128) } else { a > b }) as u128; }
    if op == OP_GE { return (if signed { (a as i128) >= (b as i128) } else { a >= b }) as u128; }
    if op == OP_BAND { return a & b; }
    if op == OP_BOR { return a | b; }
    if op == OP_BXOR { return a ^ b; }
    if op == OP_SHL || op == OP_SHR {
        if b >= 128 { err.any = true; return 0; }
        let sh = b as u32;
        return if op == OP_SHL {
            a << sh
        } else if signed {
            ((a as i128) >> sh) as u128
        } else {
            a >> sh
        };
    }
    if op == OP_DIV || op == OP_REM {
        if b == 0 { err.any = true; return 0; }
        if signed {
            let x = a as i128;
            let y = b as i128;
            if x == i128::MIN && y == -1 { err.any = true; return 0; }
            return (if op == OP_DIV { x / y } else { x % y }) as u128;
        }
        return if op == OP_DIV { a / b } else { a % b };
    }
    if signed {
        let x = a as i128;
        let y = b as i128;
        if op == OP_ADD {
            let r = x.wrapping_add(y);
            if (x < 0) == (y < 0) && (r < 0) != (x < 0) {
                err.any = true;
                return 0;
            }
            return r as u128;
        }
        if op == OP_SUB {
            let r = x.wrapping_sub(y);
            if (x < 0) != (y < 0) && (r < 0) != (x < 0) {
                err.any = true;
                return 0;
            }
            return r as u128;
        }

        let r = x.wrapping_mul(y);
        if x != 0 {
            if x == -1 && y == i128::MIN {
                err.any = true;
                return 0;
            }
            if r / x != y {
                err.any = true;
                return 0;
            }
        }
        return r as u128;
    }
    if op == OP_ADD {
        let r = a.wrapping_add(b);
        if r < a {
            err.any = true;
            return 0;
        }
        return r;
    }
    if op == OP_SUB {
        if a < b {
            err.any = true;
            return 0;
        }
        return a - b;
    }

    let r = a.wrapping_mul(b);
    if a != 0 && r / a != b {
        err.any = true;
        return 0;
    }
    r
}

/// 128-bit unary op. OP_NOT = bitwise complement; OP_NEG = signed negate.
pub fn un_op128(op: u16, v: u128, err: &mut CeErr) -> u128 {
    if op == OP_NOT {
        return !v;
    }
    let x = v as i128;
    if x == i128::MIN {
        err.any = true;
        return 0;
    }
    x.wrapping_neg() as u128
}

/// Decode a 128-bit integer literal magnitude (dec/hex, underscores, suffix
/// stripped); sets overflow via the diag if it exceeds u128.
fn int_mag128(src: &str, mem: &mut Mem, tok_i: u32, at: Node) -> u128 {
    let w = tok_bytes(src, mem.tok(tok_i));
    let mut mag: u128 = 0;
    let mut i = 0;
    let mut hex = false;
    if w.len() >= 2 && w[0] == b'0' && (w[1] == b'x' || w[1] == b'X') {
        hex = true;
        i = 2;
    }
    let mut overflow = false;
    while i < w.len() {
        let ch = w[i];
        if ch == b'_' {
            i += 1;
            continue;
        }
        let d: u128;
        if ch >= b'0' && ch <= b'9' {
            d = (ch - b'0') as u128;
        } else if hex && ch >= b'a' && ch <= b'f' {
            d = (ch - b'a' + 10) as u128;
        } else if hex && ch >= b'A' && ch <= b'F' {
            d = (ch - b'A' + 10) as u128;
        } else {
            break;
        }
        let base: u128 = if hex { 16 } else { 10 };
        let m1 = mag.wrapping_mul(base);
        if mag != 0 && m1 / base != mag {
            overflow = true;
        }
        let m2 = m1.wrapping_add(d);
        if m2 < m1 {
            overflow = true;
        }
        mag = m2;
        i += 1;
    }
    if overflow {
        ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
        return 0;
    }
    mag
}

fn int_range_ok128(mag: u128, neg: bool, t: u16) -> bool {
    if ty_is_signed(t) {
        let half: u128 = 1u128 << 127;
        if neg { mag <= half } else { mag <= half - 1 }
    } else {
        !neg
    }
}

/// Store a decoded 128-bit value as two u64 slots (lo, hi); returns the lo
/// index (hi is at +1).
fn push_val128(mem: &mut Mem, chk: &mut Chk, v: u128, at: Node) -> u32 {
    let lo = push_val(mem, chk, v as u64, at);
    let _hi = push_val(mem, chk, (v >> 64) as u64, at);
    lo
}

/// Store an integer literal (magnitude decoded from `tok_i`, optionally
/// negated) as the concrete type `t`, handling both 1-slot and 2-slot
/// (128-bit) widths. Returns the value-table index; for folded-neg unary the
/// caller sets res=1 separately.
fn store_int_lit(src: &str, mem: &mut Mem, chk: &mut Chk, tok_i: u32, neg: bool, t: u16, at: Node) -> u32 {
    if ty_is_128(t) {
        let mag = int_mag128(src, mem, tok_i, at);
        if !int_range_ok128(mag, neg, t) {
            ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
        }
        let v = if neg { mag.wrapping_neg() } else { mag };
        return push_val128(mem, chk, v, at);
    }
    let mag = int_mag(src, mem, chk, tok_i, at);
    if !int_range_ok(mag, neg, t) {
        ndiag(mem, E_LIT_OUT_OF_RANGE, at, 0, 0);
    }
    let v = if neg { mag.wrapping_neg() } else { mag };
    push_val(mem, chk, v, at)
}

pub fn cast_bits(v: u64, from: u16, to: u16) -> u64 {
    if from == to {
        return v;
    }
    if to == TY_F64 {
        if from == TY_BOOL {
            return ((v != 0) as u8 as f64).to_bits();
        }
        return if ty_is_signed(from) {
            ((v as i64) as f64).to_bits()
        } else {
            let b = int_bits(from);
            let m = if b == 64 { u64::MAX } else { (1u64 << b) - 1 };
            ((v & m) as f64).to_bits()
        };
    }
    if from == TY_F64 {

        let x = f64::from_bits(v);
        return sat_f64_to_int(x, to);
    }

    mask_to(v, to)
}

fn sat_f64_to_int(x: f64, t: u16) -> u64 {
    if x.is_nan() {
        return 0;
    }
    let b = int_bits(t);
    if ty_is_signed(t) {
        let min = -((1i128) << (b - 1)) as i128;
        let max = ((1i128 << (b - 1)) - 1) as i128;
        let v = if x < min as f64 {
            min
        } else if x > max as f64 {
            max
        } else {
            x as i64 as i128
        };
        mask_to(v as i64 as u64, t)
    } else {
        let max: u128 = if b == 64 {
            u64::MAX as u128
        } else {
            ((1u128 << b) - 1) as u128
        };
        let v: u128 = if x <= 0.0 {
            0
        } else if x >= max as f64 {
            max
        } else {
            x as u64 as u128
        };
        v as u64
    }
}

/// Const-evaluate an array length expression (after ck against usize).
fn ce_len(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32) -> u32 {
    let t = ck_ex(src, mem, chk, host, node, TY_USIZE);
    if t == TY_ERR {
        return u32::MAX;
    }
    let mut ok = true;
    let bits = ce(src, mem, chk, host, node, &mut ok);
    if !ok {
        return u32::MAX;
    }
    if bits > 0x00FF_FFFF {
        let n = mem.node(node);
        ndiag(mem, E_LIT_OUT_OF_RANGE, n, 0, 0);
        return u32::MAX;
    }
    bits as u32
}

fn scope_push(chk: &mut Chk) {
    chk.depth += 1;
}

fn scope_pop(chk: &mut Chk) {
    let d = chk.depth;
    while chk.local_n > 0 && chk.locals[chk.local_n - 1].depth == d {
        chk.local_n -= 1;
    }
    chk.depth -= 1;
}

/// Give any `let` binding in the current scope still awaiting integer inference
/// rustc's default type (i32), finalizing its initializer. Called before
/// leaving a scope so no polymorphic literal is left undecoded for the machine.
fn default_pending_ints(src: &str, mem: &mut Mem, chk: &mut Chk) {
    let d = chk.depth;
    let mut k = chk.local_n;
    while k > 0 && chk.locals[k - 1].depth == d {
        k -= 1;
        if chk.locals[k].ty == TY_INTLIT {
            let init = chk.locals[k].init;
            chk.locals[k].ty = TY_I32;
            if init != NODE_NIL {
                finalize_int(src, mem, chk, init, TY_I32);
            }
        }
    }
}

/// Add a local; returns its slot. `_` gets a slot but no binding.
fn local_add(src: &str, mem: &mut Mem, chk: &mut Chk, name_tok: u32, t: u16, flags: u16, at: Node) -> u32 {
    let size = chk.size_of(t);
    let slot = chk.next_slot;
    chk.next_slot = chk.next_slot.saturating_add(size);
    if chk.next_slot > FRAME_MAX {
        ndiag(mem, E_FRAME_TOO_BIG, at, 0, 0);
    }
    let is_wild = mem.tok(name_tok).kind == T_UNDERSCORE;
    if !is_wild {
        if chk.local_n >= CAP_LOCALS {
            ndiag(mem, E_TOO_MANY_ITEMS, at, 0, 0);
            return slot;
        }
        chk.locals[chk.local_n] = LInfo {
            name_tok,
            ty: t,
            flags,
            slot,
            depth: chk.depth,
            init: NODE_NIL,
        };
        chk.local_n += 1;
    }
    let _ = src;
    slot
}

/// Innermost local with this name, or usize::MAX.
fn local_find(src: &str, mem: &Mem, chk: &Chk, name_tok: u32) -> usize {
    let mut i = chk.local_n;
    while i > 0 {
        i -= 1;
        if tok_eq(src, mem, chk.locals[i].name_tok, name_tok) {
            return i;
        }
    }
    usize::MAX
}

fn const_find(src: &str, mem: &Mem, chk: &Chk, name_tok: u32) -> usize {
    let mut i = 0;
    while i < chk.const_n {
        if tok_eq(src, mem, chk.consts[i].name_tok, name_tok) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

fn fn_find(src: &str, mem: &Mem, chk: &Chk, name_tok: u32) -> usize {
    let mut i = 0;
    while i < chk.fn_n {
        if tok_eq(src, mem, chk.fns[i].name_tok, name_tok) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

fn host_fn_find(src: &str, mem: &Mem, host: &HostDef, name_tok: u32) -> usize {
    let w = tok_bytes(src, mem.tok(name_tok));
    let mut i = 0;
    while i < host.fns.len() {
        if bytes_eq(w, host.fns[i].name.as_bytes()) {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

/// Is the place's storage reached through a reference (a `*p`, or a field/index
/// auto-deref of one), rather than being a callee-local's own slot?
fn place_behind_ref(src: &str, mem: &Mem, chk: &Chk, place: u32) -> bool {
    let n = mem.node(place);
    match n.kind {
        N_DEREF => true,
        N_DOT | N_INDEX => {
            chk.res[place as usize] & RES_DEREF != 0 || place_behind_ref(src, mem, chk, n.d)
        }
        _ => false,
    }
}

/// Is the ref/slice value of `node` safe to return (borrows an input or the pool)?
fn escape_safe(src: &str, mem: &Mem, chk: &Chk, node: u32) -> bool {
    let n = mem.node(node);
    match n.kind {
        N_LIT_BSTR | N_LIT_STR => true,
        N_CALL | N_METHOD => true,
        N_REFOF => {
            let e = mem.node(n.e);
            if e.kind == N_SLICE {
                escape_safe(src, mem, chk, n.e)
            } else if e.kind == N_ARRAY_LIT && e.c == 0 {
                true
            } else {
                place_behind_ref(src, mem, chk, n.e)
            }
        }
        N_SLICE => {
            if ty_is_slice(chk.ty[n.d as usize]) {
                escape_safe(src, mem, chk, n.d)
            } else {
                place_behind_ref(src, mem, chk, n.d)
            }
        }
        N_NAME => {
            let li = local_find(src, mem, chk, n.a);
            li != usize::MAX && chk.locals[li].flags & LFLAG_RETSAFE != 0
        }
        N_BLOCK => n.e != NODE_NIL && escape_safe(src, mem, chk, n.e),
        N_IF => {
            n.b != NODE_NIL
                && escape_safe(src, mem, chk, n.e)
                && escape_safe(src, mem, chk, n.b)
        }
        _ => false,
    }
}

/// Mark the most-recently-added local RETSAFE if it is a ref/slice bound from an
/// escape-safe initializer (or a parameter, which always borrows the caller).
fn mark_retsafe_local(chk: &mut Chk, safe: bool) {
    if safe && chk.local_n > 0 {
        chk.locals[chk.local_n - 1].flags |= LFLAG_RETSAFE;
    }
}

fn ck_fn(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, k: usize) {
    let f = chk.fns[k];
    let n = mem.node(f.node);
    chk.local_n = 0;
    chk.depth = 0;
    chk.loop_depth = 0;
    chk.next_slot = 0;
    chk.ret_ty = f.ret;
    scope_push(chk);
    let mut p = f.first_param;
    while p != NODE_NIL {
        let pn = mem.node(p);
        let pt = chk.ty[p as usize];
        let slot = local_add(src, mem, chk, pn.a, pt, pn.x & FLAG_MUT, pn);
        chk.res[p as usize] = slot;

        let named = mem.tok(pn.a).kind != T_UNDERSCORE;
        mark_retsafe_local(chk, named && (ty_is_ref(pt) || ty_is_slice(pt)));
        p = pn.link;
    }

    chk.ret_borrow_body = if ty_is_ref(f.ret) || ty_is_slice(f.ret) {
        n.e
    } else {
        NODE_NIL
    };

    let _ = ck_ex(src, mem, chk, host, n.e, f.ret);
    scope_pop(chk);
    chk.fns[k].frame = chk.next_slot;
}

/// Check with an expectation: adapts integer literals, then requires equality.
fn ck_ex(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, expected: u16) -> u16 {
    let t = ck_expr(src, mem, chk, host, node, expected);
    if t == TY_ERR {
        return TY_ERR;
    }
    if expected == TY_ANY {
        return t;
    }

    if t == TY_NEVER {
        return expected;
    }
    if t == TY_INTLIT && ty_is_int(expected) && expected != TY_INTLIT {
        finalize_int(src, mem, chk, node, expected);
        return expected;
    }

    if t != expected && ty_is_ref(expected) && ty_is_ref(t) {
        let e = chk.rinfo(expected);
        let f = chk.rinfo(t);
        if e.pointee == f.pointee && e.mutable == 0 {
            chk.ty[node as usize] = expected;
            return expected;
        }
    }

    if t != expected && ty_is_slice(expected) && ty_is_slice(t) {
        let e = chk.slinfo(expected);
        let f = chk.slinfo(t);
        if e.pointee == f.pointee && e.mutable == 0 {
            chk.ty[node as usize] = expected;
            return expected;
        }
    }

    if t != expected && ty_is_slice(expected) && mem.node(node).kind == N_REFOF && ty_is_ref(t) {
        let sl = chk.slinfo(expected);
        let rp = chk.rinfo(t);
        if ty_is_arr(rp.pointee)
            && chk.ainfo(rp.pointee).elem == sl.pointee
            && (sl.mutable == 0 || rp.mutable != 0)
        {
            chk.ty[node as usize] = expected;
            return expected;
        }
    }
    if t != expected {
        let n = mem.node(node);
        ndiag(mem, E_TYPE_MISMATCH, n, expected as u32, t as u32);
        return TY_ERR;
    }
    t
}

/// Unify binary operands with literal adaptation. Returns operand type.
fn ck_operands(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, at: Node,
               lhs: u32, rhs: u32, guide: u16) -> u16 {
    let l = ck_expr(src, mem, chk, host, lhs, guide);
    if l == TY_ERR {
        return TY_ERR;
    }
    let rguide = if l == TY_INTLIT { guide } else { l };
    let r = ck_expr(src, mem, chk, host, rhs, rguide);
    if r == TY_ERR {
        return TY_ERR;
    }
    if l == TY_INTLIT && r == TY_INTLIT {
        if guide != TY_ANY && ty_is_int(guide) && guide != TY_INTLIT {
            finalize_int(src, mem, chk, lhs, guide);
            finalize_int(src, mem, chk, rhs, guide);
            return guide;
        }
        return TY_INTLIT;
    }
    if l == TY_INTLIT && ty_is_int(r) {
        finalize_int(src, mem, chk, lhs, r);
        return r;
    }
    if r == TY_INTLIT && ty_is_int(l) {
        finalize_int(src, mem, chk, rhs, l);
        return l;
    }
    if l != r {
        ndiag(mem, E_TYPE_MISMATCH, at, l as u32, r as u32);
        return TY_ERR;
    }
    l
}

/// Unify two branch types under a HINT (concrete side wins; two polymorphic
/// sides adopt an integer hint or default to i32 — rustc's fallback). Hard
/// contexts re-check the unified result at their own ck_ex boundary.
fn unify2(src: &str, mem: &mut Mem, chk: &mut Chk, at: Node, an: u32, a: u16, bn: u32, b: u16,
          guide: u16) -> u16 {

    if a == TY_NEVER {
        return b;
    }
    if b == TY_NEVER {
        return a;
    }
    if a == TY_INTLIT && b == TY_INTLIT {
        if guide != TY_ANY && ty_is_int(guide) && guide != TY_INTLIT {
            finalize_int(src, mem, chk, an, guide);
            finalize_int(src, mem, chk, bn, guide);
            return guide;
        }

        return TY_INTLIT;
    }
    if a == TY_INTLIT && ty_is_int(b) {
        finalize_int(src, mem, chk, an, b);
        return b;
    }
    if b == TY_INTLIT && ty_is_int(a) {
        finalize_int(src, mem, chk, bn, a);
        return a;
    }
    if a == b {
        return a;
    }
    ndiag(mem, E_TYPE_MISMATCH, at, a as u32, b as u32);
    TY_ERR
}

/// The raw recursive checker. `expected` is guidance for literals and
/// block-like tails; ck_ex enforces it, this function only threads it.
fn ck_expr(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, expected: u16) -> u16 {
    let n = mem.node(node);
    let i = node as usize;
    let t = ck_expr_inner(src, mem, chk, host, node, n, expected);
    if i < CAP_NODES {
        chk.ty[i] = t;
    }
    t
}

fn ck_expr_inner(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, n: Node,
                 expected: u16) -> u16 {
    let i = node as usize;
    match n.kind {
        N_LIT_UNIT => TY_UNIT,
        N_LIT_BOOL => TY_BOOL,
        N_LIT_STR => {
            chk.res[i] = intern_str(src, mem, chk, n.a, 1, n);
            TY_STR
        }
        N_LIT_BYTE => {
            let v = byte_lit_val(src, mem, n.a);
            chk.res[i] = push_val(mem, chk, v, n);
            TY_U8
        }
        N_LIT_BSTR => {

            let id = intern_str(src, mem, chk, n.a, 2, n);
            let e = chk.strs[id as usize];
            let ri = push_val(mem, chk, POOL_TAG | e.off as u64, n);
            let _ = push_val(mem, chk, e.len as u64, n);
            chk.res[i] = ri;
            intern_slice(mem, chk, TY_U8, 0, n)
        }
        N_LIT_FLOAT => {
            let bits = parse_f64(src, mem, n.a, n);
            chk.res[i] = push_val(mem, chk, bits, n);
            TY_F64
        }
        N_LIT_INT => {
            let sfx = int_suffix(src, mem, n.a);
            if sfx != TY_INTLIT {
                chk.res[i] = store_int_lit(src, mem, chk, n.a, false, sfx, n);
                return sfx;
            }
            let ex = exp_ty(expected);
            if ex != TY_ANY && ty_is_int(ex) && ex != TY_INTLIT {
                chk.res[i] = store_int_lit(src, mem, chk, n.a, false, ex, n);
                return ex;
            }

            TY_INTLIT
        }
        N_NAME => {
            let li = local_find(src, mem, chk, n.a);
            if li != usize::MAX {
                chk.res[i] = chk.locals[li].slot;
                return chk.locals[li].ty;
            }
            let ci = const_find(src, mem, chk, n.a);
            if ci != usize::MAX {
                chk.res[i] = RES_CONST | ci as u32;
                return ck_const(src, mem, chk, host, ci);
            }
            if fn_find(src, mem, chk, n.a) != usize::MAX
                || host_fn_find(src, mem, host, n.a) != usize::MAX
            {
                ndiag(mem, E_FN_AS_VALUE, n, 0, 0);
                return TY_ERR;
            }
            ndiag(mem, E_UNDEFINED, n, 0, 0);
            TY_ERR
        }
        N_UNARY => {
            if n.x == OP_NOT {
                let t = ck_expr(src, mem, chk, host, n.e, expected);
                if t == TY_ERR {
                    return TY_ERR;
                }
                if t == TY_BOOL || ty_is_int(t) {
                    return t;
                }
                ndiag(mem, E_BAD_OPERAND, n, OP_NOT as u32, t as u32);
                return TY_ERR;
            }

            let opn = mem.node(n.e);
            if opn.kind == N_LIT_INT {
                let sfx = int_suffix(src, mem, opn.a);
                let ex = exp_ty(expected);
                let target = if sfx != TY_INTLIT {
                    sfx
                } else if ex != TY_ANY && ty_is_int(ex) && ex != TY_INTLIT {
                    ex
                } else {
                    return TY_INTLIT;
                };
                if !ty_is_signed(target) {
                    ndiag(mem, E_NEG_UNSIGNED, n, 0, 0);
                    return TY_ERR;
                }
                chk.res[n.e as usize] = store_int_lit(src, mem, chk, opn.a, true, target, opn);
                chk.ty[n.e as usize] = target;
                chk.res[i] = 1;
                return target;
            }
            let t = ck_expr(src, mem, chk, host, n.e, expected);
            if t == TY_ERR {
                return TY_ERR;
            }
            if t == TY_F64 || t == TY_INTLIT {
                return t;
            }
            if ty_is_int(t) {
                if !ty_is_signed(t) {
                    ndiag(mem, E_NEG_UNSIGNED, n, 0, 0);
                    return TY_ERR;
                }
                return t;
            }
            ndiag(mem, E_BAD_OPERAND, n, OP_NEG as u32, t as u32);
            TY_ERR
        }
        N_BINARY => {
            let expected = if is_hint(expected) { TY_ANY } else { expected };
            let op = n.x;
            if op == OP_AND || op == OP_OR {
                let a = ck_ex(src, mem, chk, host, n.d, TY_BOOL);
                let b = ck_ex(src, mem, chk, host, n.e, TY_BOOL);
                if a == TY_ERR || b == TY_ERR {
                    return TY_ERR;
                }
                return TY_BOOL;
            }
            if op == OP_SHL || op == OP_SHR {
                let l = ck_expr(src, mem, chk, host, n.d, expected);
                if l == TY_ERR {
                    return TY_ERR;
                }
                let l = if l == TY_INTLIT && expected != TY_ANY && ty_is_int(expected)
                    && expected != TY_INTLIT
                {
                    finalize_int(src, mem, chk, n.d, expected);
                    expected
                } else {
                    l
                };
                if !ty_is_int(l) {
                    ndiag(mem, E_BAD_OPERAND, n, op as u32, l as u32);
                    return TY_ERR;
                }
                let r = ck_expr(src, mem, chk, host, n.e, TY_ANY);
                if r == TY_ERR {
                    return TY_ERR;
                }
                let r = concrete(src, mem, chk, n.e, r);
                if !ty_is_int(r) {
                    ndiag(mem, E_BAD_OPERAND, n, op as u32, r as u32);
                    return TY_ERR;
                }
                return l;
            }
            let is_cmp = op == OP_EQ || op == OP_NE || op == OP_LT || op == OP_LE
                || op == OP_GT || op == OP_GE;
            let guide = if is_cmp { TY_ANY } else { expected };
            let t = ck_operands(src, mem, chk, host, n, n.d, n.e, guide);
            if t == TY_ERR {
                return TY_ERR;
            }
            if is_cmp {
                let t = if t == TY_INTLIT {
                    finalize_int(src, mem, chk, n.d, TY_I32);
                    finalize_int(src, mem, chk, n.e, TY_I32);
                    TY_I32
                } else {
                    t
                };
                let eq_only = op == OP_EQ || op == OP_NE;
                let ok = if eq_only {
                    ty_is_int(t) || t == TY_F64 || t == TY_BOOL || t == TY_STR
                } else {
                    ty_is_int(t) || t == TY_F64
                };
                if !ok {
                    ndiag(mem, E_BAD_OPERAND, n, op as u32, t as u32);
                    return TY_ERR;
                }
                return TY_BOOL;
            }
            if op == OP_BAND || op == OP_BOR || op == OP_BXOR {
                if t == TY_BOOL || ty_is_int(t) {
                    return t;
                }
                ndiag(mem, E_BAD_OPERAND, n, op as u32, t as u32);
                return TY_ERR;
            }

            if ty_is_int(t) || t == TY_F64 {
                return t;
            }
            ndiag(mem, E_BAD_OPERAND, n, op as u32, t as u32);
            TY_ERR
        }
        N_CAST => {
            let to = ty_of(src, mem, chk, host, n.e);
            if to == TY_ERR {
                return TY_ERR;
            }

            let guide = if ty_is_int(to) { to | TY_HINT } else { TY_ANY };
            let from = ck_expr(src, mem, chk, host, n.d, guide);
            if from == TY_ERR {
                return TY_ERR;
            }

            let from = concrete(src, mem, chk, n.d, from);
            let from_ok = ty_is_int(from) || from == TY_F64 || from == TY_BOOL;
            let to_ok = ty_is_int(to) || to == TY_F64;
            let pair_ok = from_ok && to_ok && !(from == TY_BOOL && to == TY_F64);
            if !pair_ok {
                ndiag(mem, E_BAD_CAST, n, from as u32, to as u32);
                return TY_ERR;
            }
            to
        }
        N_CALL => {
            if chk.in_const {

                ndiag(mem, E_NOT_CONST, n, 0, 0);
                return TY_ERR;
            }
            let fi = fn_find(src, mem, chk, n.a);
            if fi != usize::MAX {
                chk.res[i] = fi as u32;
                let f = chk.fns[fi];
                if n.c != f.param_n {
                    ndiag(mem, E_ARG_COUNT, n, f.param_n, n.c);
                    return TY_ERR;
                }
                let mut arg = n.b;
                let mut p = f.first_param;
                while arg != NODE_NIL && p != NODE_NIL {
                    let pt = chk.ty[p as usize];
                    let at = ck_ex(src, mem, chk, host, arg, pt);
                    let _ = at;
                    arg = mem.node(arg).link;
                    p = mem.node(p).link;
                }
                return f.ret;
            }
            let hi = host_fn_find(src, mem, host, n.a);
            if hi != usize::MAX {
                chk.res[i] = RES_HOST | hi as u32;
                let hf = &host.fns[hi];
                if n.c as usize != hf.params.len() {
                    ndiag(mem, E_ARG_COUNT, n, hf.params.len() as u32, n.c);
                    return TY_ERR;
                }
                let mut arg = n.b;
                let mut pi = 0;
                while arg != NODE_NIL && pi < hf.params.len() {
                    let pt = host_ty(src, mem, chk, host, &hf.params[pi], n);
                    let _ = ck_ex(src, mem, chk, host, arg, pt);
                    arg = mem.node(arg).link;
                    pi += 1;
                }
                return host_ty(src, mem, chk, host, &hf.ret, n);
            }
            ndiag(mem, E_UNKNOWN_FN, n, 0, 0);
            TY_ERR
        }
        N_METHOD => {
            if chk.in_const {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                return TY_ERR;
            }

            let rt = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if rt == TY_ERR {
                return TY_ERR;
            }

            if ty_is_slice(rt) {
                if tok_is(src, mem, n.a, b"len") && n.c == 0 {
                    chk.res[i] = RES_SLICE_LEN;
                    return TY_USIZE;
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            if ty_is_arr(rt) {
                if tok_is(src, mem, n.a, b"len") && n.c == 0 {
                    let len = chk.ainfo(rt).len as u64;
                    let vi = push_val(mem, chk, len, n);
                    chk.res[i] = RES_ARRAY_LEN | vi;
                    return TY_USIZE;
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            if rt == TY_STR {
                if n.c != 0 {
                    ndiag(mem, E_ARG_COUNT, n, 0, n.c);
                    return TY_ERR;
                }
                if tok_is(src, mem, n.a, b"len") {
                    chk.res[i] = RES_STR_LEN;
                    return TY_USIZE;
                }
                if tok_is(src, mem, n.a, b"as_bytes") {
                    chk.res[i] = RES_STR_BYTES;
                    return intern_slice(mem, chk, TY_U8, 0, n);
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            if ty_is_int(rt) {
                let rt = if rt == TY_INTLIT {
                    finalize_int(src, mem, chk, n.d, TY_I32);
                    TY_I32
                } else {
                    rt
                };
                let op = prim_op(src, mem, n.a);
                if op == 0 {
                    ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                    return TY_ERR;
                }
                let want_args = if op == PRIM_WRAP_NEG { 0 } else { 1 };
                if n.c != want_args {
                    ndiag(mem, E_ARG_COUNT, n, want_args, n.c);
                    return TY_ERR;
                }
                if want_args == 1 {
                    let u32_arg = op == PRIM_WRAP_SHL || op == PRIM_ROTL || op == PRIM_ROTR;
                    let at = if u32_arg { TY_U32 } else { rt };
                    let _ = ck_ex(src, mem, chk, host, n.b, at);
                }
                chk.res[i] = RES_PRIM | op;
                return rt;
            }

            if rt == TY_F64 {
                if n.c != 0 {
                    ndiag(mem, E_ARG_COUNT, n, 0, n.c);
                    return TY_ERR;
                }
                if tok_is(src, mem, n.a, b"to_bits") {
                    chk.res[i] = RES_PRIM | PRIM_TO_BITS;
                    return TY_U64;
                }
                if tok_is(src, mem, n.a, b"is_nan") {
                    chk.res[i] = RES_PRIM | PRIM_IS_NAN;
                    return TY_BOOL;
                }
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }

            let (sty, recv_is_ref, recv_ref_mut) = if ty_is_struct(rt) {
                (rt, false, false)
            } else if ty_is_ref(rt) {
                let ri = chk.rinfo(rt);
                (ri.pointee, true, ri.mutable != 0)
            } else {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, rt as u32);
                return TY_ERR;
            };
            if !ty_is_struct(sty) {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, sty as u32);
                return TY_ERR;
            }
            let fi = method_find(src, mem, chk, sty, n.a);
            if fi == usize::MAX {
                ndiag(mem, E_UNKNOWN_METHOD, n, 0, 0);
                return TY_ERR;
            }
            let f = chk.fns[fi];
            if n.c + 1 != f.param_n {
                ndiag(mem, E_ARG_COUNT, n, f.param_n.wrapping_sub(1), n.c);
                return TY_ERR;
            }

            let recv_pty = chk.ty[f.first_param as usize];
            let want_ref = ty_is_ref(recv_pty);
            let want_mut = want_ref && chk.rinfo(recv_pty).mutable != 0;
            let mode_place;
            if want_ref {
                if recv_is_ref {

                    if want_mut && !recv_ref_mut {
                        ndiag(mem, E_REF_MUT_NEEDED, n, 0, 0);
                        return TY_ERR;
                    }
                    mode_place = false;
                } else {

                    let mut root_mut = false;
                    let pt = ck_place(src, mem, chk, host, n.d, &mut root_mut);
                    if pt == TY_ERR {
                        return TY_ERR;
                    }
                    if want_mut && !root_mut {
                        ndiag(mem, E_REF_MUT_NEEDED, n, 0, 0);
                        return TY_ERR;
                    }
                    mode_place = true;
                }
            } else {

                if recv_is_ref {

                    ndiag(mem, E_BAD_RECEIVER, n, 0, 0);
                    return TY_ERR;
                }
                mode_place = false;
            }
            chk.res[i] = (if mode_place { RES_MPLACE } else { 0 }) | fi as u32;

            let mut arg = n.b;
            let mut p = mem.node(f.first_param).link;
            while arg != NODE_NIL && p != NODE_NIL {
                let pt = chk.ty[p as usize];
                let _ = ck_ex(src, mem, chk, host, arg, pt);
                arg = mem.node(arg).link;
                p = mem.node(p).link;
            }
            f.ret
        }
        N_DOT => {
            let bt0 = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            if !ty_is_struct(bt) {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, bt as u32);
                return TY_ERR;
            }
            let mut off = 0u32;
            let mut fty = TY_ERR;
            if !field_lookup(src, mem, chk, host, bt, n.a, &mut off, &mut fty) {
                ndiag(mem, E_UNKNOWN_FIELD, n, 0, 0);
                return TY_ERR;
            }
            chk.res[i] = off | if deref { RES_DEREF } else { 0 };
            fty
        }
        N_INDEX => {
            let bt0 = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            let elem = if ty_is_arr(bt) {
                chk.ainfo(bt).elem
            } else if ty_is_slice(bt) {
                chk.slinfo(bt).pointee
            } else {
                ndiag(mem, E_NOT_AN_ARRAY, n, 0, bt0 as u32);
                return TY_ERR;
            };
            let _ = ck_ex(src, mem, chk, host, n.e, TY_USIZE);
            chk.res[i] = if deref { RES_DEREF } else { 0 };
            elem
        }
        N_SLICE => {

            ndiag(mem, E_SUBSLICE_REF, n, 0, 0);
            TY_ERR
        }
        N_STRUCT_LIT => {
            let s = find_struct(src, mem, chk, host, n.a);
            if s == u16::MAX {
                ndiag(mem, E_UNKNOWN_TYPE, n, 0, 0);
                return TY_ERR;
            }
            let st = TY_STRUCT0 + s;
            let info = chk.structs[s as usize];

            let mut init = n.b;
            while init != NODE_NIL {
                let fin = mem.node(init);
                let mut off = 0u32;
                let mut fty = TY_ERR;
                if !field_lookup(src, mem, chk, host, st, fin.a, &mut off, &mut fty) {
                    ndiag(mem, E_UNKNOWN_FIELD, fin, 0, 0);
                    return TY_ERR;
                }

                let mut other = n.b;
                while other != init {
                    if tok_eq(src, mem, mem.node(other).a, fin.a) {
                        ndiag(mem, E_DUP_FIELD, fin, 0, 0);
                        return TY_ERR;
                    }
                    other = mem.node(other).link;
                }
                chk.res[init as usize] = off;
                chk.ty[init as usize] = fty;
                if fin.e == NODE_NIL {

                    let li = local_find(src, mem, chk, fin.a);
                    if li == usize::MAX {
                        ndiag(mem, E_UNDEFINED, fin, 0, 0);
                        return TY_ERR;
                    }
                    if chk.locals[li].ty != fty {
                        ndiag(mem, E_TYPE_MISMATCH, fin, fty as u32, chk.locals[li].ty as u32);
                        return TY_ERR;
                    }

                    chk.res[init as usize] = off | (chk.locals[li].slot << 16);
                } else {
                    let _ = ck_ex(src, mem, chk, host, fin.e, fty);
                }
                init = fin.link;
            }
            if n.c != info.field_n {
                ndiag(mem, E_MISSING_FIELD, n, info.field_n, n.c);
                return TY_ERR;
            }
            st
        }
        N_TUPLE => {
            if n.c as usize > TUP_MAX || n.c < 2 {
                ndiag(mem, E_TUPLE, n, 0, 0);
                return TY_ERR;
            }
            let exp_tup = expected != TY_ANY && ty_is_tuple(expected)
                && chk.tinfo(expected).count == n.c as u16;
            let exp = chk.tinfo(expected);
            let mut elems = [TY_ERR; TUP_MAX];
            let mut el = n.b;
            let mut i = 0usize;
            while el != NODE_NIL {
                let ee = if exp_tup { exp.elems[i] } else { TY_ANY };
                let et = ck_expr(src, mem, chk, host, el, ee);
                if et == TY_ERR {
                    return TY_ERR;
                }
                elems[i] = if exp_tup && et == TY_INTLIT && ty_is_int(exp.elems[i]) {
                    finalize_int(src, mem, chk, el, exp.elems[i]);
                    exp.elems[i]
                } else {
                    concrete(src, mem, chk, el, et)
                };
                i += 1;
                el = mem.node(el).link;
            }
            intern_tuple(mem, chk, &elems, n.c as usize, n)
        }
        N_ARRAY_LIT => {
            let mut elem_exp = TY_ANY;
            if expected != TY_ANY && ty_is_arr(expected) {
                elem_exp = chk.ainfo(expected).elem;
            }
            if n.c == 0 {
                if elem_exp == TY_ANY {
                    ndiag(mem, E_ANNOTATION_NEEDED, n, 0, 0);
                    return TY_ERR;
                }
                return intern_arr(mem, chk, elem_exp, 0, n);
            }
            let mut el = n.b;
            let mut ety = elem_exp;
            let mut first = true;
            while el != NODE_NIL {
                if first && ety == TY_ANY {
                    let t = ck_expr(src, mem, chk, host, el, TY_ANY);
                    if t == TY_ERR {
                        return TY_ERR;
                    }
                    ety = concrete(src, mem, chk, el, t);
                } else {
                    let t = ck_ex(src, mem, chk, host, el, ety);
                    if t == TY_ERR {
                        return TY_ERR;
                    }
                }
                first = false;
                el = mem.node(el).link;
            }
            intern_arr(mem, chk, ety, n.c, n)
        }
        N_ARRAY_REPEAT => {
            let mut elem_exp = TY_ANY;
            if expected != TY_ANY && ty_is_arr(expected) {
                elem_exp = chk.ainfo(expected).elem;
            }
            let t = ck_expr(src, mem, chk, host, n.d, elem_exp);
            if t == TY_ERR {
                return TY_ERR;
            }
            let ety = if elem_exp != TY_ANY && t == TY_INTLIT {
                finalize_int(src, mem, chk, n.d, elem_exp);
                elem_exp
            } else {
                concrete(src, mem, chk, n.d, t)
            };
            if elem_exp != TY_ANY && ety != elem_exp {
                ndiag(mem, E_TYPE_MISMATCH, n, elem_exp as u32, ety as u32);
                return TY_ERR;
            }
            let len = ce_len(src, mem, chk, host, n.e);
            if len == u32::MAX {
                return TY_ERR;
            }
            intern_arr(mem, chk, ety, len, n)
        }
        N_IF => {
            let expected = if is_hint(expected) { TY_ANY } else { expected };
            let _ = ck_ex(src, mem, chk, host, n.d, TY_BOOL);
            if n.b == NODE_NIL {

                if expected != TY_ANY && expected != TY_UNIT {
                    ndiag(mem, E_NO_ELSE, n, 0, 0);
                    return TY_ERR;
                }
                let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);
                return TY_UNIT;
            }

            let a = ck_expr(src, mem, chk, host, n.e, expected);
            if a == TY_ERR {
                return TY_ERR;
            }
            let bguide = if a == TY_INTLIT || a == TY_NEVER { expected } else { a };
            let b = ck_expr(src, mem, chk, host, n.b, bguide);
            if b == TY_ERR {
                return TY_ERR;
            }
            unify2(src, mem, chk, n, n.e, a, n.b, b, expected)
        }
        N_MATCH => {
            let expected = if is_hint(expected) { TY_ANY } else { expected };
            let st = ck_expr(src, mem, chk, host, n.d, TY_ANY);
            if st == TY_ERR {
                return TY_ERR;
            }
            let st = concrete(src, mem, chk, n.d, st);
            if !(ty_is_int(st) || st == TY_BOOL || st == TY_STR || ty_is_enum(st)) {
                ndiag(mem, E_PATTERN_TYPE, n, st as u32, 0);
                return TY_ERR;
            }

            let mut result = TY_ANY;
            let mut saw_wild = false;
            let mut saw_true = false;
            let mut saw_false = false;
            let mut seen_variants: u64 = 0;
            let mut arm = n.b;
            while arm != NODE_NIL {
                let an = mem.node(arm);
                let mut pat = an.b;
                while pat != NODE_NIL {
                    ck_pattern(src, mem, chk, pat, st, &mut saw_wild, &mut saw_true,
                               &mut saw_false, &mut seen_variants);
                    pat = mem.node(pat).link;
                }
                let guide = if result != TY_ANY && result != TY_INTLIT && result != TY_NEVER {
                    result
                } else {
                    expected
                };
                let t = ck_expr(src, mem, chk, host, an.e, guide);
                if t == TY_ERR {
                    return TY_ERR;
                }
                if t == TY_NEVER {

                    if result == TY_ANY {
                        result = TY_NEVER;
                    }
                    arm = an.link;
                    continue;
                }
                if t == TY_INTLIT {
                    if result != TY_ANY && result != TY_INTLIT && result != TY_NEVER {
                        finalize_int(src, mem, chk, an.e, result);
                    } else {
                        result = TY_INTLIT;
                    }
                } else if result == TY_ANY || result == TY_INTLIT || result == TY_NEVER {
                    if result == TY_INTLIT {
                        if !ty_is_int(t) {
                            ndiag(mem, E_TYPE_MISMATCH, an, t as u32, TY_INTLIT as u32);
                            return TY_ERR;
                        }

                        let mut back = n.b;
                        while back != arm {
                            let bk = mem.node(back);
                            if chk.ty[bk.e as usize] == TY_INTLIT {
                                finalize_int(src, mem, chk, bk.e, t);
                            }
                            back = bk.link;
                        }
                    }
                    result = t;
                } else if t != result {
                    ndiag(mem, E_TYPE_MISMATCH, an, result as u32, t as u32);
                    return TY_ERR;
                }
                arm = an.link;
            }
            if result == TY_INTLIT {
                let t = if expected != TY_ANY && ty_is_int(expected) && expected != TY_INTLIT {
                    expected
                } else {
                    TY_I32
                };
                let mut back = n.b;
                while back != NODE_NIL {
                    let bk = mem.node(back);
                    if chk.ty[bk.e as usize] == TY_INTLIT {
                        finalize_int(src, mem, chk, bk.e, t);
                    }
                    back = bk.link;
                }
                result = t;
            }
            let mut back = n.b;
            while back != NODE_NIL {
                chk.ty[back as usize] = result;
                back = mem.node(back).link;
            }
            let exhaustive = saw_wild
                || (st == TY_BOOL && saw_true && saw_false)
                || (ty_is_enum(st) && enum_all_seen(chk, st, seen_variants));
            if !exhaustive {
                ndiag(mem, E_NOT_EXHAUSTIVE, n, 0, 0);
                return TY_ERR;
            }
            if result == TY_ANY {

                ndiag(mem, E_NOT_EXHAUSTIVE, n, 0, 0);
                return TY_ERR;
            }
            result
        }
        N_ASSERT => {

            let _ = ck_ex(src, mem, chk, host, n.c, TY_BOOL);
            chk.res[i] = if n.a != NODE_NIL {
                intern_str(src, mem, chk, n.a, 1, n)
            } else {
                NODE_NIL
            };
            TY_UNIT
        }
        N_BLOCK => ck_block(src, mem, chk, host, node, n, expected),
        N_REFOF => {

            if mem.node(n.e).kind == N_SLICE {
                return ck_slice(src, mem, chk, host, n.e, n.x & FLAG_MUT, n);
            }

            let opn = mem.node(n.e);
            if opn.kind == N_ARRAY_LIT && opn.c == 0 {
                let elem = if expected != TY_ANY && ty_is_slice(expected) {
                    chk.slinfo(expected).pointee
                } else {
                    TY_ANY
                };
                if elem == TY_ANY {
                    ndiag(mem, E_ANNOTATION_NEEDED, n, 0, 0);
                    return TY_ERR;
                }
                return intern_slice(mem, chk, elem, n.x & FLAG_MUT, n);
            }
            let mut root_mut = false;
            let pt = ck_place(src, mem, chk, host, n.e, &mut root_mut);
            if pt == TY_ERR {
                return TY_ERR;
            }
            let want_mut = n.x & FLAG_MUT != 0;
            if want_mut && !root_mut {
                ndiag(mem, E_REF_MUT_NEEDED, n, 0, 0);
                return TY_ERR;
            }
            intern_ref(mem, chk, pt, if want_mut { FLAG_MUT } else { 0 }, n)
        }
        N_DEREF => {
            let rt = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            if rt == TY_ERR {
                return TY_ERR;
            }
            if !ty_is_ref(rt) {
                ndiag(mem, E_DEREF_NON_REF, n, 0, 0);
                return TY_ERR;
            }
            chk.rinfo(rt).pointee
        }
        N_RETURN => {

            if n.e != NODE_NIL {
                let _ = ck_ex(src, mem, chk, host, n.e, chk.ret_ty);

                if (ty_is_ref(chk.ret_ty) || ty_is_slice(chk.ret_ty))
                    && !escape_safe(src, mem, chk, n.e)
                {
                    ndiag(mem, E_REF_ESCAPES, n, 0, 0);
                }
            } else if chk.ret_ty != TY_UNIT {
                ndiag(mem, E_TYPE_MISMATCH, n, chk.ret_ty as u32, TY_UNIT as u32);
            }
            TY_NEVER
        }
        N_ASSIGN => ck_assign(src, mem, chk, host, node),
        N_ASSOC_CALL => {

            if chk.in_const {
                ndiag(mem, E_NOT_CONST, n, 0, 0);
                return TY_ERR;
            }
            if tok_is(src, mem, n.a, b"f64") && tok_is(src, mem, n.e, b"from_bits") {
                if n.c != 1 {
                    ndiag(mem, E_ARG_COUNT, n, 1, n.c);
                    return TY_ERR;
                }
                let _ = ck_ex(src, mem, chk, host, n.b, TY_U64);
                return TY_F64;
            }
            ndiag(mem, E_BAD_PATH, n, 0, 0);
            TY_ERR
        }
        N_PATHCONST => {

            let ev = find_enum(src, mem, chk, n.a);
            if ev != u16::MAX {
                let et = TY_ENUM0 + ev;
                let tag = variant_tag(src, mem, chk, et, n.b);
                if tag == u32::MAX {
                    ndiag(mem, E_UNKNOWN_VARIANT, n, 0, 0);
                    return TY_ERR;
                }
                chk.res[i] = push_val(mem, chk, tag as u64, n);
                return et;
            }

            let t = int_ty_named(src, mem, n.a);
            let is_max = tok_is(src, mem, n.b, b"MAX");
            let is_min = tok_is(src, mem, n.b, b"MIN");
            if t == TY_ERR || (!is_max && !is_min) {
                ndiag(mem, E_BAD_PATH, n, 0, 0);
                return TY_ERR;
            }
            if ty_is_128(t) {
                let v: u128 = if is_min {
                    if ty_is_signed(t) { 1u128 << 127 } else { 0 }
                } else if ty_is_signed(t) {
                    (1u128 << 127) - 1
                } else {
                    u128::MAX
                };
                chk.res[i] = push_val128(mem, chk, v, n);
                return t;
            }
            let bits = int_bits(t);
            let v = if is_min {
                if ty_is_signed(t) { min_signed(t) } else { 0 }
            } else if ty_is_signed(t) {
                (1u64 << (bits - 1)) - 1
            } else if bits == 64 {
                u64::MAX
            } else {
                (1u64 << bits) - 1
            };
            chk.res[i] = push_val(mem, chk, v, n);
            t
        }
        _ => {

            ndiag(mem, E_TYPE_MISMATCH, n, expected as u32, TY_ERR as u32);
            TY_ERR
        }
    }
}

fn ck_pattern(src: &str, mem: &mut Mem, chk: &mut Chk, pat: u32, st: u16, saw_wild: &mut bool,
              saw_true: &mut bool, saw_false: &mut bool, seen_variants: &mut u64) {
    let p = mem.node(pat);
    let i = pat as usize;
    match p.kind {
        N_PAT_WILD => {
            *saw_wild = true;
            chk.ty[i] = st;
        }
        N_PAT_ENUM => {

            if !ty_is_enum(st) {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, 0);
                return;
            }
            let ev = find_enum(src, mem, chk, p.a);
            if ev == u16::MAX || TY_ENUM0 + ev != st {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32,
                      if ev == u16::MAX { 0 } else { (TY_ENUM0 + ev) as u32 });
                return;
            }
            let tag = variant_tag(src, mem, chk, st, p.b);
            if tag == u32::MAX {
                ndiag(mem, E_UNKNOWN_VARIANT, p, 0, 0);
                return;
            }
            if tag < 64 {
                *seen_variants |= 1u64 << tag;
            }
            chk.res[i] = push_val(mem, chk, tag as u64, p);
            chk.ty[i] = st;
        }
        N_PAT_BOOL => {
            if st != TY_BOOL {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_BOOL as u32);
                return;
            }
            if p.x == 1 {
                *saw_true = true;
            } else {
                *saw_false = true;
            }
            chk.ty[i] = st;
        }
        N_PAT_INT => {
            if !ty_is_int(st) || st == TY_INTLIT {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_I32 as u32);
                return;
            }
            let neg = p.x == 1;
            if neg && !ty_is_signed(st) {
                ndiag(mem, E_NEG_UNSIGNED, p, 0, 0);
                return;
            }
            let mag = int_mag(src, mem, chk, p.a, p);
            if !int_range_ok(mag, neg, st) {
                ndiag(mem, E_LIT_OUT_OF_RANGE, p, 0, 0);
                return;
            }
            let bits = if neg { mag.wrapping_neg() } else { mag };
            chk.res[i] = push_val(mem, chk, mask_to(bits, st), p);
            chk.ty[i] = st;
        }
        N_PAT_STR => {
            if st != TY_STR {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_STR as u32);
                return;
            }
            chk.res[i] = intern_str(src, mem, chk, p.a, 1, p);
            chk.ty[i] = st;
        }
        N_PAT_BYTE => {

            if st != TY_U8 {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, TY_U8 as u32);
                return;
            }
            let v = byte_lit_val(src, mem, p.a);
            chk.res[i] = push_val(mem, chk, v, p);
            chk.ty[i] = st;
        }
        N_PAT_CONST => {

            let ci = const_find(src, mem, chk, p.a);
            if ci == usize::MAX {
                ndiag(mem, E_UNDEFINED, p, 0, 0);
                return;
            }
            let ct = chk.consts[ci].ty;
            if chk.consts[ci].state != 2 || ct == TY_ERR {

                return;
            }
            if ct != st {
                ndiag(mem, E_PATTERN_TYPE, p, st as u32, ct as u32);
                return;
            }
            if ty_is_struct(ct) || ty_is_arr(ct) {

                ndiag(mem, E_PATTERN_TYPE, p, st as u32, ct as u32);
                return;
            }

            let bits = chk.consts[ci].bits;
            let cmp = if ty_is_int(st) { mask_to(bits, st) } else { bits };
            chk.res[i] = push_val(mem, chk, cmp, p);
            chk.ty[i] = st;
        }
        _ => {}
    }
}

fn ck_block(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32, n: Node,
            expected: u16) -> u16 {
    let _ = node;
    scope_push(chk);
    let mut s = n.b;
    let mut last = NODE_NIL;
    while s != NODE_NIL {
        ck_stmt(src, mem, chk, host, s);
        last = s;
        s = mem.node(s).link;
    }
    let mut t = if n.e != NODE_NIL {
        ck_expr(src, mem, chk, host, n.e, expected)
    } else {
        TY_UNIT
    };

    if t == TY_INTLIT {
        if expected != TY_ANY && ty_is_int(expected) && expected != TY_INTLIT {
            finalize_int(src, mem, chk, n.e, expected);
            t = expected;
        } else if mem.node(n.e).kind == N_NAME {
            t = concrete(src, mem, chk, n.e, t);
        }
    }

    if node == chk.ret_borrow_body && n.e != NODE_NIL && !escape_safe(src, mem, chk, n.e) {
        ndiag(mem, E_REF_ESCAPES, mem.node(n.e), 0, 0);
    }
    default_pending_ints(src, mem, chk);
    scope_pop(chk);

    if n.e == NODE_NIL && last != NODE_NIL {
        let ln = mem.node(last);
        let inner = if ln.kind == N_EXPR_STMT { ln.e } else { last };
        if inner != NODE_NIL && (inner as usize) < CAP_NODES && chk.ty[inner as usize] == TY_NEVER {
            return TY_NEVER;
        }
    }
    if n.e == NODE_NIL && expected != TY_ANY && expected != TY_UNIT {
        ndiag(mem, E_TYPE_MISMATCH, n, expected as u32, TY_UNIT as u32);
        return TY_ERR;
    }
    t
}

/// Check an assignment `place = rhs` (or compound). It is an expression of
/// type `()`; returns TY_UNIT (or TY_ERR). Used from both statement and
/// expression position (e.g. a `match` arm `pat => x = v`).
fn ck_assign(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32) -> u16 {
    let n = mem.node(node);
    let i = node as usize;
    let mut root_mut = false;
    let mut pt = ck_place(src, mem, chk, host, n.d, &mut root_mut);
    if pt == TY_ERR {
        return TY_ERR;
    }
    if !root_mut {
        ndiag(mem, E_ASSIGN_IMMUTABLE, n, 0, 0);
        return TY_ERR;
    }

    if pt == TY_INTLIT && mem.node(n.d).kind == N_NAME {
        let vt = ck_expr(src, mem, chk, host, n.e, TY_ANY);
        if vt == TY_ERR {
            return TY_ERR;
        }
        let target = if ty_is_int(vt) && vt != TY_INTLIT { vt } else { TY_I32 };
        resolve_int_local(src, mem, chk, mem.node(n.d).a, target);
        pt = target;
    }
    if n.x == 0 {
        let _ = ck_ex(src, mem, chk, host, n.e, pt);
    } else if n.x == OP_SHL || n.x == OP_SHR {
        if !ty_is_int(pt) {
            ndiag(mem, E_BAD_OPERAND, n, n.x as u32, pt as u32);
            return TY_ERR;
        }
        let r = ck_expr(src, mem, chk, host, n.e, TY_ANY);
        if r == TY_ERR {
            return TY_ERR;
        }
        let r = concrete(src, mem, chk, n.e, r);
        if !ty_is_int(r) {
            ndiag(mem, E_BAD_OPERAND, n, n.x as u32, r as u32);
        }
    } else {
        let op_ok = if n.x == OP_BAND || n.x == OP_BOR || n.x == OP_BXOR {
            ty_is_int(pt) || pt == TY_BOOL
        } else {
            ty_is_int(pt) || pt == TY_F64
        };
        if !op_ok {
            ndiag(mem, E_BAD_OPERAND, n, n.x as u32, pt as u32);
            return TY_ERR;
        }
        let _ = ck_ex(src, mem, chk, host, n.e, pt);
    }
    if i < CAP_NODES {
        chk.ty[i] = TY_UNIT;
    }
    TY_UNIT
}

fn ck_stmt(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, s: u32) {
    let n = mem.node(s);
    let i = s as usize;
    match n.kind {
        N_LET => {
          if n.x & FLAG_TUPLE != 0 {

            let it = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            if it == TY_ERR {
                return;
            }
            if !ty_is_tuple(it) || chk.tinfo(it).count != n.c as u16 {
                ndiag(mem, E_TUPLE, n, it as u32, 0);
                return;
            }
            let info = chk.tinfo(it);
            let base = chk.next_slot;
            chk.next_slot = chk.next_slot.saturating_add(info.size);
            if chk.next_slot > FRAME_MAX {
                ndiag(mem, E_FRAME_TOO_BIG, n, 0, 0);
            }

            let mut el = n.b;
            let mut ei = 0usize;
            while el != NODE_NIL {
                let pn = mem.node(el);
                if mem.tok(pn.a).kind != T_UNDERSCORE && chk.local_n < CAP_LOCALS {
                    chk.locals[chk.local_n] = LInfo {
                        name_tok: pn.a,
                        ty: info.elems[ei],
                        flags: pn.x & FLAG_MUT,
                        slot: base + info.offs[ei] as u32,
                        depth: chk.depth,
                        init: NODE_NIL,
                    };
                    chk.local_n += 1;
                }
                ei += 1;
                el = mem.node(el).link;
            }
            chk.res[i] = base;
            chk.ty[i] = it;
            return;
          }
            let t = if n.d != NODE_NIL {
                let annot = ty_of(src, mem, chk, host, n.d);
                if annot == TY_ERR {
                    return;
                }

                let _ = ck_ex(src, mem, chk, host, n.e, annot);
                annot
            } else {
                let r = ck_expr(src, mem, chk, host, n.e, TY_ANY);
                if r == TY_ERR {
                    return;
                }

                if r == TY_INTLIT && mem.tok(n.a).kind != T_UNDERSCORE {
                    r
                } else {
                    concrete(src, mem, chk, n.e, r)
                }
            };

            let slot = local_add(src, mem, chk, n.a, t, n.x & FLAG_MUT, n);
            chk.res[i] = slot;
            chk.ty[i] = t;
            if t == TY_INTLIT {

                chk.locals[chk.local_n - 1].init = n.e;
            }

            if (ty_is_ref(t) || ty_is_slice(t)) && mem.tok(n.a).kind != T_UNDERSCORE {
                mark_retsafe_local(chk, escape_safe(src, mem, chk, n.e));
            }
        }
        N_ASSIGN => {
            let _ = ck_assign(src, mem, chk, host, s);
        }
        N_EXPR_STMT => {
            let t = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            let _ = concrete(src, mem, chk, n.e, t);
        }
        N_WHILE => {
            let saved = chk.loop_depth;
            chk.loop_depth = 0;
            let _ = ck_ex(src, mem, chk, host, n.d, TY_BOOL);
            chk.loop_depth = saved;
            chk.loop_depth += 1;
            let sb = chk.loop_broke;
            chk.loop_broke = false;
            let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);
            chk.loop_broke = sb;
            chk.loop_depth -= 1;
        }
        N_LOOP => {
            chk.loop_depth += 1;
            let sb = chk.loop_broke;
            chk.loop_broke = false;
            let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);

            chk.ty[i] = if chk.loop_broke { TY_UNIT } else { TY_NEVER };
            chk.loop_broke = sb;
            chk.loop_depth -= 1;
        }
        N_FOR => {
            let saved = chk.loop_depth;
            chk.loop_depth = 0;
            let lo = ck_expr(src, mem, chk, host, n.b, TY_ANY);
            if lo == TY_ERR {
                chk.loop_depth = saved;
                return;
            }
            let hi = ck_expr(src, mem, chk, host, n.c, if lo == TY_INTLIT { TY_ANY } else { lo });
            chk.loop_depth = saved;
            if hi == TY_ERR {
                return;
            }
            let vt = if lo == TY_INTLIT && hi == TY_INTLIT {
                finalize_int(src, mem, chk, n.b, TY_I32);
                finalize_int(src, mem, chk, n.c, TY_I32);
                TY_I32
            } else if lo == TY_INTLIT && ty_is_int(hi) {
                finalize_int(src, mem, chk, n.b, hi);
                hi
            } else if hi == TY_INTLIT && ty_is_int(lo) {
                finalize_int(src, mem, chk, n.c, lo);
                lo
            } else if lo == hi && ty_is_int(lo) {
                lo
            } else {
                ndiag(mem, E_TYPE_MISMATCH, n, lo as u32, hi as u32);
                return;
            };
            scope_push(chk);
            let slot = local_add(src, mem, chk, n.a, vt, 0, n);
            chk.res[i] = slot;
            chk.ty[i] = vt;
            chk.loop_depth += 1;
            let sb = chk.loop_broke;
            chk.loop_broke = false;
            let _ = ck_ex(src, mem, chk, host, n.e, TY_UNIT);
            chk.loop_broke = sb;
            chk.loop_depth -= 1;
            scope_pop(chk);
        }
        N_BREAK | N_CONTINUE => {
            if chk.loop_depth == 0 {
                ndiag(mem, E_BREAK_OUTSIDE_LOOP, n, 0, 0);
            } else if n.kind == N_BREAK {
                chk.loop_broke = true;
            }
        }
        _ => {}
    }
}

/// Check an assignment place; records field offsets/index types like ck_expr,
/// and reports whether the root binding is mutable.
fn ck_place(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, node: u32,
            root_mut: &mut bool) -> u16 {
    let n = mem.node(node);
    let i = node as usize;
    let t = match n.kind {
        N_NAME => {
            let li = local_find(src, mem, chk, n.a);
            if li == usize::MAX {
                if const_find(src, mem, chk, n.a) != usize::MAX {
                    ndiag(mem, E_ASSIGN_NOT_PLACE, n, 0, 0);
                } else {
                    ndiag(mem, E_UNDEFINED, n, 0, 0);
                }
                return TY_ERR;
            }
            *root_mut = chk.locals[li].flags & FLAG_MUT != 0;
            chk.res[i] = chk.locals[li].slot;

            chk.locals[li].ty
        }
        N_DOT => {
            let bt0 = ck_place(src, mem, chk, host, n.d, root_mut);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                *root_mut = chk.rinfo(bt0).mutable != 0;
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            if !ty_is_struct(bt) {
                ndiag(mem, E_NOT_A_STRUCT, n, 0, bt as u32);
                return TY_ERR;
            }
            let mut off = 0u32;
            let mut fty = TY_ERR;
            if !field_lookup(src, mem, chk, host, bt, n.a, &mut off, &mut fty) {
                ndiag(mem, E_UNKNOWN_FIELD, n, 0, 0);
                return TY_ERR;
            }
            chk.res[i] = off | if deref { RES_DEREF } else { 0 };
            fty
        }
        N_INDEX => {
            let bt0 = ck_place(src, mem, chk, host, n.d, root_mut);
            if bt0 == TY_ERR {
                return TY_ERR;
            }

            let (bt, deref) = if ty_is_ref(bt0) {
                *root_mut = chk.rinfo(bt0).mutable != 0;
                (chk.rinfo(bt0).pointee, true)
            } else {
                (bt0, false)
            };
            let elem = if ty_is_arr(bt) {
                chk.ainfo(bt).elem
            } else if ty_is_slice(bt) {
                *root_mut = chk.slinfo(bt).mutable != 0;
                chk.slinfo(bt).pointee
            } else {
                ndiag(mem, E_NOT_AN_ARRAY, n, 0, bt0 as u32);
                return TY_ERR;
            };
            let _ = ck_ex(src, mem, chk, host, n.e, TY_USIZE);
            chk.res[i] = if deref { RES_DEREF } else { 0 };
            elem
        }
        N_DEREF => {

            let rt = ck_expr(src, mem, chk, host, n.e, TY_ANY);
            if rt == TY_ERR {
                return TY_ERR;
            }
            if !ty_is_ref(rt) {
                ndiag(mem, E_DEREF_NON_REF, n, 0, 0);
                return TY_ERR;
            }
            let ri = chk.rinfo(rt);

            *root_mut = ri.mutable != 0;
            ri.pointee
        }
        _ => {
            ndiag(mem, E_ASSIGN_NOT_PLACE, n, 0, 0);
            return TY_ERR;
        }
    };
    if i < CAP_NODES {
        chk.ty[i] = t;
    }
    t
}

/// Field offset+type by name for user and host structs.
fn field_lookup(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef, st: u16, name_tok: u32,
                off: &mut u32, fty: &mut u16) -> bool {
    let info = chk.sinfo(st);
    if info.host > 0 {
        let hs = &host.structs[(info.host - 1) as usize];
        let w = tok_bytes(src, mem.tok(name_tok));
        let mut o = 0u32;
        let mut f = 0;
        while f < hs.fields.len() {
            if bytes_eq(w, hs.fields[f].name.as_bytes()) {
                *off = o;
                *fty = host_ty(src, mem, chk, host, &hs.fields[f].ty, NODE_NONE);
                return true;
            }
            o += host_ty_size(&hs.fields[f].ty);
            f += 1;
        }
        return false;
    }
    let mut f = info.first_field;
    while f != NODE_NIL {
        let fnode = mem.node(f);
        if tok_eq(src, mem, fnode.a, name_tok) {
            *off = chk.res[f as usize];
            *fty = chk.ty[f as usize];
            return true;
        }
        f = fnode.link;
    }
    false
}
