
use crate::ast::*;
use crate::check::*;
use crate::diag::*;
use crate::{CAP_CSTACK, CAP_FRAME_MEM, CAP_VSTACK, CALL_DEPTH_MAX};

pub const RAN_DONE: u16 = 0;
pub const RAN_HOST: u16 = 1;
pub const RAN_TRAP: u16 = 2;
pub const RAN_RUN: u16 = 3;

const K_EVAL: u16 = 0;
const K_RET: u16 = 1;
const K_PLACE: u16 = 2;

#[derive(Clone, Copy)]
pub struct Ctl {
    pub kind: u16,
    pub step: u16,
    pub node: u32,
    pub a: u32,
    pub b: u32,
}

pub const CTL_NONE: Ctl = Ctl {
    kind: K_EVAL,
    step: 0,
    node: NODE_NIL,
    a: 0,
    b: 0,
};

/// Per-run machine state. Caller-owned (like Mem/Chk); ~0.2 MiB under
/// profile-host. One Instance serves many sequential calls.
pub struct Instance {
    pub frames: [u64; CAP_FRAME_MEM],
    pub frame_base: u32,
    pub frame_top: u32,

    pub vstack: [u64; CAP_VSTACK],
    pub v_n: usize,

    pub cstack: [Ctl; CAP_CSTACK],
    pub c_n: usize,

    pub call_depth: u16,
    pub status: u16,
    pub fuel: u64,

    pub trap_code: u16,
    pub trap_lo: u32,
    pub trap_hi: u32,
    pub trap_msg: u32,

    pub host_id: u16,
    pub host_args: u32,
    pub host_ret: u32,

    pub ret_slots: u32,
}

pub const INSTANCE_INIT: Instance = Instance {
    frames: [0; CAP_FRAME_MEM],
    frame_base: 0,
    frame_top: 0,
    vstack: [0; CAP_VSTACK],
    v_n: 0,
    cstack: [CTL_NONE; CAP_CSTACK],
    c_n: 0,
    call_depth: 0,
    status: RAN_DONE,
    fuel: 0,
    trap_code: 0,
    trap_lo: 0,
    trap_hi: 0,
    trap_msg: NODE_NIL,
    host_id: 0,
    host_args: 0,
    host_ret: 0,
    ret_slots: 0,
};

impl Instance {
    /// Final value slots after RAN_DONE.
    pub fn result(&self) -> &[u64] {
        let n = self.ret_slots as usize;
        if n <= self.v_n {
            &self.vstack[self.v_n - n..self.v_n]
        } else {
            &[]
        }
    }

    /// Marshaled argument slots of the pending host call (RAN_HOST).
    pub fn host_arg_slots(&self) -> &[u64] {
        let n = self.host_args as usize;
        if n <= self.v_n {
            &self.vstack[self.v_n - n..self.v_n]
        } else {
            &[]
        }
    }
}

fn trap(inst: &mut Instance, code: u16, n: Node) {
    inst.status = RAN_TRAP;
    inst.trap_code = code;
    inst.trap_lo = n.lo;
    inst.trap_hi = n.hi;
}

fn vpush(inst: &mut Instance, v: u64, at: Node) -> bool {
    if inst.v_n >= CAP_VSTACK {
        trap(inst, E_T_STACK, at);
        return false;
    }
    inst.vstack[inst.v_n] = v;
    inst.v_n += 1;
    true
}

fn vpop(inst: &mut Instance) -> u64 {
    if inst.v_n == 0 {
        return 0;
    }
    inst.v_n -= 1;
    inst.vstack[inst.v_n]
}

/// Pop a 128-bit value (2 slots: lo pushed first, hi on top).
fn vpop128(inst: &mut Instance) -> u128 {
    let hi = vpop(inst);
    let lo = vpop(inst);
    ((hi as u128) << 64) | (lo as u128)
}

/// Push a 128-bit value as 2 slots (lo then hi).
fn vpush128(inst: &mut Instance, v: u128, at: Node) -> bool {
    if !vpush(inst, v as u64, at) {
        return false;
    }
    vpush(inst, (v >> 64) as u64, at)
}

fn cpush(inst: &mut Instance, c: Ctl, at: Node) -> bool {
    if inst.c_n >= CAP_CSTACK {
        trap(inst, E_T_STACK, at);
        return false;
    }
    inst.cstack[inst.c_n] = c;
    inst.c_n += 1;
    true
}

/// Schedule `node` for evaluation. Leaves (literals, names) are evaluated
/// immediately — half of all evaluations skip the control-stack round trip;
/// everything else gets a control entry. Order is preserved: children were
/// already evaluated strictly in push order.
fn push_eval(mem: &Mem, chk: &Chk, inst: &mut Instance, node: u32, at: Node) -> bool {
    let n = mem.node(node);
    let i = node as usize;
    match n.kind {
        N_LIT_INT | N_LIT_FLOAT | N_PATHCONST | N_LIT_BYTE if chk.size_of(chk.ty[i]) == 1 => {
            vpush(inst, chk.vals[chk.res[i] as usize], n)
        }
        N_LIT_BOOL => vpush(inst, n.x as u64, n),
        N_LIT_STR => vpush(inst, chk.res[i] as u64, n),
        N_LIT_UNIT => true,
        N_NAME => {
            if chk.res[i] & RES_CONST != 0 {
                let k = (chk.res[i] & RES_MASK) as usize;
                push_const(inst, chk, k, chk.ty[i], n)
            } else {
                let size = chk.size_of(chk.ty[i]);
                load_frame(inst, inst.frame_base + chk.res[i], size, n)
            }
        }
        _ => {
            let mut c = CTL_NONE;
            c.kind = K_EVAL;
            c.node = node;
            cpush(inst, c, at)
        }
    }
}

fn push_place(inst: &mut Instance, node: u32, at: Node) -> bool {
    let mut c = CTL_NONE;
    c.kind = K_PLACE;
    c.node = node;
    cpush(inst, c, at)
}

/// Copy `size` slots from the top of the value stack into frames[dst..].
fn store_frame(inst: &mut Instance, dst: u32, size: u32, at: Node) -> bool {
    let dst = dst as usize;
    let size = size as usize;
    if dst + size > CAP_FRAME_MEM || size > inst.v_n {
        trap(inst, E_T_INTERNAL, at);
        return false;
    }
    let src = inst.v_n - size;
    let mut i = 0;
    while i < size {
        inst.frames[dst + i] = inst.vstack[src + i];
        i += 1;
    }
    inst.v_n = src;
    true
}

/// Push `size` slots from frames[src..] onto the value stack.
fn load_frame(inst: &mut Instance, src: u32, size: u32, at: Node) -> bool {
    let src = src as usize;
    let size = size as usize;
    if src + size > CAP_FRAME_MEM || inst.v_n + size > CAP_VSTACK {
        trap(inst, E_T_STACK, at);
        return false;
    }
    let mut i = 0;
    while i < size {
        inst.vstack[inst.v_n + i] = inst.frames[src + i];
        i += 1;
    }
    inst.v_n += size;
    true
}

/// Push a named constant's value. Scalars hold their bits inline; aggregate
/// (struct/array) consts store an offset into `chk.vals` and copy `size` slots.
fn push_const(inst: &mut Instance, chk: &Chk, k: usize, t: u16, at: Node) -> bool {
    if ty_is_struct(t) || ty_is_arr(t) {
        let base = chk.consts[k].bits as usize;
        let size = chk.size_of(t) as usize;
        if inst.v_n + size > CAP_VSTACK {
            trap(inst, E_T_STACK, at);
            return false;
        }
        let mut i = 0;
        while i < size {
            inst.vstack[inst.v_n + i] = chk.vals[base + i];
            i += 1;
        }
        inst.v_n += size;
        return true;
    }
    vpush(inst, chk.consts[k].bits, at)
}

/// Total slot count of a call's arguments (walk the arg chain types).
fn args_slots(mem: &Mem, chk: &Chk, first_arg: u32) -> u32 {
    let mut n = 0;
    let mut a = first_arg;
    while a != NODE_NIL {
        n += chk.size_of(chk.ty[a as usize]);
        a = mem.node(a).link;
    }
    n
}

/// Begin a call of `fns[fn_idx]` with `args` (flattened slots, matching the
/// parameter layout). Runs until RAN_DONE / RAN_HOST / RAN_TRAP.
pub fn start(mem: &Mem, chk: &Chk, inst: &mut Instance, fn_idx: usize, args: &[u64],
             fuel: u64) -> u16 {
    inst.v_n = 0;
    inst.c_n = 0;
    inst.frame_base = 0;
    inst.frame_top = 0;
    inst.call_depth = 0;
    inst.fuel = fuel;
    inst.status = RAN_RUN;
    if fn_idx >= chk.fn_n {
        trap(inst, E_T_NO_ENTRY, NODE_NONE);
        return inst.status;
    }
    let f = chk.fns[fn_idx];
    let fnode = mem.node(f.node);
    if f.frame as usize > CAP_FRAME_MEM {
        trap(inst, E_T_STACK, fnode);
        return inst.status;
    }

    let mut i = 0;
    while i < args.len() && i < CAP_FRAME_MEM {
        inst.frames[i] = args[i];
        i += 1;
    }
    inst.frame_top = f.frame;
    inst.ret_slots = chk.size_of(f.ret);
    if !push_eval(mem, chk, inst, fnode.e, fnode) {
        return inst.status;
    }
    run(mem, chk, inst)
}

/// Resume after RAN_HOST with the host's return slots.
pub fn resume(mem: &Mem, chk: &Chk, inst: &mut Instance, ret: &[u64]) -> u16 {
    if inst.status != RAN_HOST || ret.len() != inst.host_ret as usize {
        trap(inst, E_T_HOST, NODE_NONE);
        return inst.status;
    }

    inst.v_n -= inst.host_args as usize;
    let mut i = 0;
    while i < ret.len() {
        if inst.v_n >= CAP_VSTACK {
            trap(inst, E_T_STACK, NODE_NONE);
            return inst.status;
        }
        inst.vstack[inst.v_n] = ret[i];
        inst.v_n += 1;
        i += 1;
    }
    inst.status = RAN_RUN;
    run(mem, chk, inst)
}

fn run(mem: &Mem, chk: &Chk, inst: &mut Instance) -> u16 {
    while inst.status == RAN_RUN {
        if inst.c_n == 0 {
            inst.status = RAN_DONE;
            break;
        }
        if inst.fuel == 0 {
            let c = inst.cstack[inst.c_n - 1];
            trap(inst, E_T_FUEL, mem.node(c.node));
            break;
        }
        inst.fuel -= 1;
        step(mem, chk, inst);
    }
    inst.status
}

fn step(mem: &Mem, chk: &Chk, inst: &mut Instance) {
    let ci = inst.c_n - 1;
    let c = inst.cstack[ci];

    if c.kind == K_RET {

        inst.frame_base = c.a;
        inst.frame_top = c.b;
        if inst.call_depth > 0 {
            inst.call_depth -= 1;
        }
        inst.c_n -= 1;
        return;
    }

    let n = mem.node(c.node);
    let i = c.node as usize;

    if c.kind == K_PLACE {
        step_place(mem, chk, inst, ci, c, n, i);
        return;
    }

    match n.kind {

        N_LIT_INT | N_LIT_FLOAT | N_PATHCONST | N_LIT_BSTR | N_LIT_BYTE => {
            inst.c_n -= 1;
            let ri = chk.res[i] as usize;
            if chk.size_of(chk.ty[i]) == 2 {
                vpush(inst, chk.vals[ri], n);
                vpush(inst, chk.vals[ri + 1], n);
            } else {
                vpush(inst, chk.vals[ri], n);
            }
        }
        N_LIT_BOOL => {
            inst.c_n -= 1;
            vpush(inst, n.x as u64, n);
        }
        N_LIT_STR => {
            inst.c_n -= 1;
            vpush(inst, chk.res[i] as u64, n);
        }
        N_LIT_UNIT => {
            inst.c_n -= 1;
        }
        N_REFOF => {

            if c.step == 0 {
                inst.cstack[ci].step = 1;
                let en = mem.node(n.e);
                if en.kind == N_ARRAY_LIT && en.c == 0 {

                    inst.c_n -= 1;
                    if vpush(inst, 0, n) {
                        vpush(inst, 0, n);
                    }
                    return;
                }
                if en.kind == N_SLICE {
                    push_eval(mem, chk, inst, n.e, n);
                } else {
                    push_place(inst, n.e, n);
                }
                return;
            }
            inst.c_n -= 1;

            if ty_is_slice(chk.ty[i]) && mem.node(n.e).kind != N_SLICE {
                let arr_ty = chk.ty[n.e as usize];
                let len = chk.ainfo(arr_ty).len as u64;
                vpush(inst, len, n);
            }
        }
        N_SLICE => {

            if c.step == 0 {
                inst.cstack[ci].step = 1;
                inst.cstack[ci].b = inst.v_n as u32;
                if chk.res[i] != 0 {
                    push_eval(mem, chk, inst, n.d, n);
                } else {
                    push_place(inst, n.d, n);
                }
                return;
            }
            if c.step == 1 {
                inst.cstack[ci].step = 2;
                push_eval(mem, chk, inst, n.b, n);
                return;
            }
            if c.step == 2 {
                inst.cstack[ci].step = 3;
                if n.c != NODE_NIL {
                    push_eval(mem, chk, inst, n.c, n);
                    return;
                }

            }
            inst.c_n -= 1;
            let base_at = c.b as usize;
            let len = if chk.res[i] != 0 {
                inst.vstack[base_at + 1]
            } else {
                chk.ainfo(chk.ty[n.d as usize]).len as u64
            };
            let hi = if n.c != NODE_NIL { vpop(inst) } else { len };
            let lo = vpop(inst);
            let addr = inst.vstack[base_at];
            if lo > hi || hi > len {
                trap(inst, E_T_OOB, n);
                return;
            }
            let es = chk.size_of(chk.slinfo(chk.ty[i]).pointee) as u64;
            inst.v_n = base_at;
            vpush(inst, addr + lo * es, n);
            vpush(inst, hi - lo, n);
        }
        N_DEREF => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
            let addr = vpop(inst) as u32;
            let size = chk.size_of(chk.ty[i]);
            load_frame(inst, addr, size, n);
        }
        N_NAME => {
            inst.c_n -= 1;
            if chk.res[i] & RES_CONST != 0 {
                let k = (chk.res[i] & RES_MASK) as usize;
                push_const(inst, chk, k, chk.ty[i], n);
            } else {
                let size = chk.size_of(chk.ty[i]);
                load_frame(inst, inst.frame_base + chk.res[i], size, n);
            }
        }

        N_UNARY => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
            if chk.res[i] == 1 {
                return;
            }
            if ty_is_128(chk.ty[i]) {
                let v = vpop128(inst);
                let mut err = CeErr { any: false };
                let r = un_op128(n.x, v, &mut err);
                if err.any {
                    trap(inst, E_T_ARITH, n);
                    return;
                }
                vpush128(inst, r, n);
                return;
            }
            let v = vpop(inst);
            let mut err = CeErr { any: false };
            let r = un_op(n.x, v, chk.ty[i], &mut err);
            if err.any {
                trap(inst, E_T_ARITH, n);
                return;
            }
            vpush(inst, r, n);
        }
        N_BINARY => {
            let sc = n.x == OP_AND || n.x == OP_OR;
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            if c.step == 1 {
                if sc {

                    let l = vpop(inst);
                    let done = if n.x == OP_AND { l == 0 } else { l != 0 };
                    if done {
                        inst.c_n -= 1;
                        vpush(inst, l, n);
                        return;
                    }
                    inst.cstack[ci].step = 2;
                    push_eval(mem, chk, inst, n.e, n);
                    return;
                }
                inst.cstack[ci].step = 2;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
            if sc {
                return;
            }
            let ot = chk.ty[n.d as usize];
            if ty_is_128(ot) {

                let b = if n.x == OP_SHL || n.x == OP_SHR {
                    let rt = chk.ty[n.e as usize];
                    if chk.size_of(rt) == 2 { vpop128(inst) } else { vpop(inst) as u128 }
                } else {
                    vpop128(inst)
                };
                let a = vpop128(inst);
                let mut err = CeErr { any: false };
                let r = ce_bin128(n.x, a, b, ty_is_signed(ot), &mut err);
                if err.any {
                    trap(inst, E_T_ARITH, n);
                    return;
                }
                if chk.size_of(chk.ty[i]) == 2 {
                    vpush128(inst, r, n);
                } else {
                    vpush(inst, r as u64, n);
                }
                return;
            }
            let b = vpop(inst);
            let a = vpop(inst);
            let mut err = CeErr { any: false };
            let r = ce_bin(n.x, a, b, ot, &mut err);
            if err.any {
                trap(inst, E_T_ARITH, n);
                return;
            }
            vpush(inst, r, n);
        }
        N_CAST => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            inst.c_n -= 1;
            let from = chk.ty[n.d as usize];
            let to = chk.ty[i];
            if !ty_is_128(from) && !ty_is_128(to) {
                let v = vpop(inst);
                vpush(inst, cast_bits(v, from, to), n);
                return;
            }

            if from == crate::check::TY_F64 {
                let x = f64::from_bits(vpop(inst));

                let r: u128 = if ty_is_signed(to) { (x as i128) as u128 } else { x as u128 };
                vpush128(inst, r, n);
                return;
            }
            let sv: u128 = if ty_is_128(from) {
                vpop128(inst)
            } else {
                let raw = vpop(inst);
                if ty_is_signed(from) { (raw as i64) as i128 as u128 } else { raw as u128 }
            };
            if to == crate::check::TY_F64 {
                let x = if ty_is_signed(from) { (sv as i128) as f64 } else { sv as f64 };
                vpush(inst, x.to_bits(), n);
                return;
            }
            if ty_is_128(to) {
                vpush128(inst, sv, n);
            } else {

                vpush(inst, cast_bits(sv as u64, crate::check::TY_U64, to), n);
            }
        }

        N_DOT => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                inst.cstack[ci].b = inst.v_n as u32;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            inst.c_n -= 1;
            let base_at = c.b as usize;
            let size = chk.size_of(chk.ty[i]) as usize;
            if chk.res[i] & RES_DEREF != 0 {

                let off = chk.res[i] & RES_OFF_MASK;
                let addr = inst.vstack[base_at] as u32;
                inst.v_n = base_at;
                load_frame(inst, addr + off, size as u32, n);
            } else {
                let off = chk.res[i] as usize;
                let mut k = 0;
                while k < size {
                    inst.vstack[base_at + k] = inst.vstack[base_at + off + k];
                    k += 1;
                }
                inst.v_n = base_at + size;
            }
        }
        N_INDEX => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                inst.cstack[ci].b = inst.v_n as u32;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            if c.step == 1 {
                inst.cstack[ci].step = 2;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
            let idx = vpop(inst);
            let base_at = c.b as usize;
            let bt = chk.ty[n.d as usize];
            if chk.res[i] & RES_DEREF != 0 {

                let a = chk.ainfo(chk.rinfo(bt).pointee);
                if idx >= a.len as u64 {
                    trap(inst, E_T_OOB, n);
                    return;
                }
                let es = chk.size_of(a.elem) as usize;
                let addr = inst.vstack[base_at] as usize + idx as usize * es;
                if addr + es > CAP_FRAME_MEM {
                    trap(inst, E_T_INTERNAL, n);
                    return;
                }
                let mut k = 0;
                while k < es {
                    inst.vstack[base_at + k] = inst.frames[addr + k];
                    k += 1;
                }
                inst.v_n = base_at + es;
                return;
            }
            if ty_is_slice(bt) {

                let es = chk.size_of(chk.slinfo(bt).pointee) as usize;
                let addr = inst.vstack[base_at];
                let len = inst.vstack[base_at + 1];
                if idx >= len {
                    trap(inst, E_T_OOB, n);
                    return;
                }
                if addr & POOL_TAG != 0 {

                    let off = (addr & !POOL_TAG) as usize + idx as usize;
                    let byte = chk.str_pool_at(off);
                    inst.vstack[base_at] = byte as u64;
                    inst.v_n = base_at + 1;
                    return;
                }
                let src = addr as usize + idx as usize * es;
                if src + es > CAP_FRAME_MEM {
                    trap(inst, E_T_INTERNAL, n);
                    return;
                }
                let mut k = 0;
                while k < es {
                    inst.vstack[base_at + k] = inst.frames[src + k];
                    k += 1;
                }
                inst.v_n = base_at + es;
                return;
            }
            let a = chk.ainfo(bt);
            if idx >= a.len as u64 {
                trap(inst, E_T_OOB, n);
                return;
            }
            let es = chk.size_of(a.elem) as usize;
            let off = idx as usize * es;
            let mut k = 0;
            while k < es {
                inst.vstack[base_at + k] = inst.vstack[base_at + off + k];
                k += 1;
            }
            inst.v_n = base_at + es;
        }
        N_STRUCT_LIT => {

            if c.step == 0 {
                inst.cstack[ci].a = n.b;
                inst.cstack[ci].b = inst.v_n as u32;
                inst.cstack[ci].step = 1;
                return;
            }
            let mut cur = c.a;

            while cur != NODE_NIL && mem.node(cur).e == NODE_NIL {
                cur = mem.node(cur).link;
            }
            if cur != NODE_NIL {
                inst.cstack[ci].a = mem.node(cur).link;
                push_eval(mem, chk, inst, mem.node(cur).e, n);
                return;
            }

            inst.c_n -= 1;
            let total = chk.size_of(chk.ty[i]) as usize;
            let base = c.b as usize;
            let scratch = inst.v_n;
            if scratch + total > CAP_VSTACK {
                trap(inst, E_T_STACK, n);
                return;
            }
            let mut cursor = base;
            let mut init = n.b;
            while init != NODE_NIL {
                let fin = mem.node(init);
                let off = (chk.res[init as usize] & 0xFFFF) as usize;
                let size = chk.size_of(chk.ty[init as usize]) as usize;
                if fin.e == NODE_NIL {

                    let slot = (inst.frame_base + (chk.res[init as usize] >> 16)) as usize;
                    let mut k = 0;
                    while k < size {
                        inst.vstack[scratch + off + k] = inst.frames[slot + k];
                        k += 1;
                    }
                } else {
                    let mut k = 0;
                    while k < size {
                        inst.vstack[scratch + off + k] = inst.vstack[cursor + k];
                        k += 1;
                    }
                    cursor += size;
                }
                init = fin.link;
            }

            let mut k = 0;
            while k < total {
                inst.vstack[base + k] = inst.vstack[scratch + k];
                k += 1;
            }
            inst.v_n = base + total;
        }
        N_ARRAY_LIT | N_TUPLE => {

            if c.step == 0 {
                inst.cstack[ci].a = n.b;
                inst.cstack[ci].step = 1;
                return;
            }
            let cur = c.a;
            if cur != NODE_NIL {
                inst.cstack[ci].a = mem.node(cur).link;
                push_eval(mem, chk, inst, cur, n);
                return;
            }
            inst.c_n -= 1;
        }
        N_ARRAY_REPEAT => {
            if c.step == 0 {
                inst.cstack[ci].b = inst.v_n as u32;
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            inst.c_n -= 1;
            let a = chk.ainfo(chk.ty[i]);
            let es = chk.size_of(a.elem) as usize;
            let base = c.b as usize;
            let total = es * a.len as usize;
            if base + total > CAP_VSTACK {
                trap(inst, E_T_STACK, n);
                return;
            }
            let mut r = 1;
            while r < a.len as usize {
                let mut k = 0;
                while k < es {
                    inst.vstack[base + r * es + k] = inst.vstack[base + k];
                    k += 1;
                }
                r += 1;
            }
            inst.v_n = base + total;
        }

        N_BLOCK => {
            if c.step == 0 {
                inst.cstack[ci].a = n.b;
                inst.cstack[ci].step = 1;
                return;
            }
            if c.step == 1 {
                let cur = c.a;
                if cur != NODE_NIL {
                    inst.cstack[ci].a = mem.node(cur).link;
                    push_eval(mem, chk, inst, cur, n);
                    return;
                }
                if n.e != NODE_NIL {
                    inst.cstack[ci].step = 2;
                    push_eval(mem, chk, inst, n.e, n);
                    return;
                }
                inst.c_n -= 1;
                return;
            }
            inst.c_n -= 1;
        }
        N_IF => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            if c.step == 1 {
                let cond = vpop(inst);
                inst.cstack[ci].step = 2;
                if cond != 0 {
                    push_eval(mem, chk, inst, n.e, n);
                } else if n.b != NODE_NIL {
                    push_eval(mem, chk, inst, n.b, n);
                }
                return;
            }
            inst.c_n -= 1;
        }
        N_ASSERT => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.c, n);
                return;
            }
            if c.step == 1 {
                let cond = vpop(inst);
                inst.cstack[ci].step = 2;
                if cond == 0 {

                    inst.trap_msg = chk.res[i];
                    trap(inst, E_T_ASSERT, n);
                    return;
                }
                return;
            }
            inst.c_n -= 1;
        }
        N_MATCH => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            if c.step == 1 {
                let v = vpop(inst);
                let mut arm = n.b;
                let mut body = NODE_NIL;
                while arm != NODE_NIL && body == NODE_NIL {
                    let an = mem.node(arm);
                    let mut pat = an.b;
                    while pat != NODE_NIL {
                        let pn = mem.node(pat);
                        let hit = match pn.kind {
                            N_PAT_WILD => true,
                            N_PAT_BOOL => (pn.x as u64) == v,
                            N_PAT_INT | N_PAT_CONST | N_PAT_BYTE | N_PAT_ENUM => {
                                chk.vals[chk.res[pat as usize] as usize] == v
                            }
                            N_PAT_STR => chk.res[pat as usize] as u64 == v,
                            _ => false,
                        };
                        if hit {
                            body = an.e;
                            break;
                        }
                        pat = pn.link;
                    }
                    arm = an.link;
                }
                inst.cstack[ci].step = 2;
                if body == NODE_NIL {

                    trap(inst, E_T_INTERNAL, n);
                    return;
                }
                push_eval(mem, chk, inst, body, n);
                return;
            }
            inst.c_n -= 1;
        }
        N_WHILE => {
            if c.step == 0 {
                inst.cstack[ci].a = inst.v_n as u32;
                inst.cstack[ci].b = inst.v_n as u32;
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.d, n);
                return;
            }
            if c.step == 1 {
                let cond = vpop(inst);
                if cond == 0 {
                    inst.c_n -= 1;
                    return;
                }
                inst.cstack[ci].step = 2;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }

            inst.cstack[ci].step = 1;
            push_eval(mem, chk, inst, n.d, n);
        }
        N_LOOP => {
            if c.step == 0 {
                inst.cstack[ci].a = inst.v_n as u32;
                inst.cstack[ci].b = inst.v_n as u32;
                inst.cstack[ci].step = 1;
            }
            push_eval(mem, chk, inst, n.e, n);
        }
        N_FOR => {
            if c.step == 0 {
                inst.cstack[ci].b = inst.v_n as u32;
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.b, n);
                return;
            }
            if c.step == 1 {

                let size = 1;
                if !store_frame(inst, inst.frame_base + chk.res[i], size, n) {
                    return;
                }
                inst.cstack[ci].step = 2;
                push_eval(mem, chk, inst, n.c, n);
                return;
            }
            if c.step == 2 {
                inst.cstack[ci].a = inst.v_n as u32;
                inst.cstack[ci].step = 3;
                return;
            }
            if c.step == 3 {

                let vt = chk.ty[i];
                let var = inst.frames[(inst.frame_base + chk.res[i]) as usize];
                let hi = inst.vstack[inst.v_n - 1];
                let op = if n.x & FLAG_INCLUSIVE != 0 { OP_LE } else { OP_LT };
                let mut err = CeErr { any: false };
                let go = ce_bin(op, var, hi, vt, &mut err);
                if go == 0 {
                    inst.v_n -= 1;
                    inst.c_n -= 1;
                    return;
                }
                inst.cstack[ci].step = 4;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }

            let vt = chk.ty[i];
            let slot = (inst.frame_base + chk.res[i]) as usize;
            let var = inst.frames[slot];
            let hi = inst.vstack[inst.v_n - 1];
            if n.x & FLAG_INCLUSIVE != 0 && var == hi {
                inst.v_n -= 1;
                inst.c_n -= 1;
                return;
            }
            let mut err = CeErr { any: false };
            let inc = ce_bin(OP_ADD, var, 1, vt, &mut err);
            if err.any {
                trap(inst, E_T_ARITH, n);
                return;
            }
            inst.frames[slot] = inc;
            inst.cstack[ci].step = 3;
        }
        N_BREAK | N_CONTINUE => {
            inst.c_n -= 1;
            unwind_loop(mem, chk, inst, n, n.kind == N_BREAK);
        }
        N_RETURN => {
            if c.step == 0 && n.e != NODE_NIL {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            let rs = if n.e != NODE_NIL {
                chk.size_of(chk.ty[n.e as usize]) as usize
            } else {
                0
            };
            unwind_return(inst, rs, n);
        }

        N_LET => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
            let size = chk.size_of(chk.ty[i]);
            store_frame(inst, inst.frame_base + chk.res[i], size, n);
        }
        N_EXPR_STMT => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
            let size = chk.size_of(chk.ty[n.e as usize]) as usize;
            if size <= inst.v_n {
                inst.v_n -= size;
            }
        }
        N_ASSIGN => {
            if c.step == 0 {

                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            if c.step == 1 {
                inst.cstack[ci].step = 2;
                push_place(inst, n.d, n);
                return;
            }
            inst.c_n -= 1;
            let addr = vpop(inst) as u32;
            let pt = chk.ty[n.d as usize];
            if n.x == 0 {
                let size = chk.size_of(pt);
                store_frame(inst, addr, size, n);
                return;
            }

            let rhs = vpop(inst);
            let cur = inst.frames[addr as usize];
            let mut err = CeErr { any: false };
            let r = ce_bin(n.x, cur, rhs, pt, &mut err);
            if err.any {
                trap(inst, E_T_ARITH, n);
                return;
            }
            inst.frames[addr as usize] = r;
        }

        N_ASSOC_CALL => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.b, n);
                return;
            }
            inst.c_n -= 1;
        }

        N_CALL => {
            if c.step == 0 {
                inst.cstack[ci].a = n.b;
                inst.cstack[ci].step = 1;
                return;
            }
            if c.step == 1 {
                let cur = c.a;
                if cur != NODE_NIL {
                    inst.cstack[ci].a = mem.node(cur).link;
                    push_eval(mem, chk, inst, cur, n);
                    return;
                }
                inst.cstack[ci].step = 2;

            }

            inst.c_n -= 1;
            let nargs = args_slots(mem, chk, n.b);
            if chk.res[i] & RES_HOST != 0 {

                inst.host_id = (chk.res[i] & RES_MASK) as u16;
                inst.host_args = nargs;
                inst.host_ret = chk.size_of(chk.ty[i]);
                inst.status = RAN_HOST;
                return;
            }
            let f = chk.fns[(chk.res[i] & RES_MASK) as usize];
            if inst.call_depth as usize >= CALL_DEPTH_MAX {
                trap(inst, E_T_STACK, n);
                return;
            }
            let new_base = inst.frame_top;
            if new_base as usize + f.frame as usize > CAP_FRAME_MEM {
                trap(inst, E_T_STACK, n);
                return;
            }

            if !store_frame(inst, new_base, nargs, n) {
                return;
            }
            let mut rc = CTL_NONE;
            rc.kind = K_RET;
            rc.a = inst.frame_base;
            rc.b = inst.frame_top;
            rc.node = inst.v_n as u32;
            if !cpush(inst, rc, n) {
                return;
            }
            inst.frame_base = new_base;
            inst.frame_top = new_base + f.frame;
            inst.call_depth += 1;
            let body = mem.node(f.node).e;
            push_eval(mem, chk, inst, body, n);
        }

        N_METHOD => {

            if chk.res[i] & RES_SLICE_LEN != 0 {
                if c.step == 0 {
                    inst.cstack[ci].step = 1;
                    push_eval(mem, chk, inst, n.d, n);
                    return;
                }
                inst.c_n -= 1;
                let len = vpop(inst);
                let _addr = vpop(inst);
                vpush(inst, len, n);
                return;
            }

            if chk.res[i] & RES_ARRAY_LEN != 0 {
                if c.step == 0 {
                    inst.cstack[ci].step = 1;
                    inst.cstack[ci].b = inst.v_n as u32;
                    push_eval(mem, chk, inst, n.d, n);
                    return;
                }
                inst.c_n -= 1;
                inst.v_n = c.b as usize;
                let vi = (chk.res[i] & RES_ALEN_MASK) as usize;
                vpush(inst, chk.vals[vi], n);
                return;
            }

            if chk.res[i] & RES_STR_LEN != 0 {
                if c.step == 0 {
                    inst.cstack[ci].step = 1;
                    push_eval(mem, chk, inst, n.d, n);
                    return;
                }
                inst.c_n -= 1;
                let id = vpop(inst) as u32;
                let (_off, len) = chk.str_span(id);
                vpush(inst, len, n);
                return;
            }

            if chk.res[i] & RES_STR_BYTES != 0 {
                if c.step == 0 {
                    inst.cstack[ci].step = 1;
                    push_eval(mem, chk, inst, n.d, n);
                    return;
                }
                inst.c_n -= 1;
                let id = vpop(inst) as u32;
                let (off, len) = chk.str_span(id);
                vpush(inst, POOL_TAG | off, n);
                vpush(inst, len, n);
                return;
            }

            if chk.res[i] & RES_PRIM != 0 {
                if c.step == 0 {
                    inst.cstack[ci].step = 1;
                    inst.cstack[ci].a = n.b;
                    push_eval(mem, chk, inst, n.d, n);
                    return;
                }
                if c.step == 1 {
                    let cur = c.a;
                    if cur != NODE_NIL {
                        inst.cstack[ci].a = mem.node(cur).link;
                        push_eval(mem, chk, inst, cur, n);
                        return;
                    }
                    inst.cstack[ci].step = 2;
                }
                inst.c_n -= 1;
                let op = chk.res[i] & RES_PRIM_MASK;

                if op == PRIM_TO_BITS {
                    return;
                }
                if op == PRIM_IS_NAN {
                    let a = vpop(inst);
                    vpush(inst, f64::from_bits(a).is_nan() as u64, n);
                    return;
                }
                let rt = chk.ty[i];
                if ty_is_128(rt) {
                    if op == PRIM_WRAP_NEG {
                        let a = vpop128(inst);
                        vpush128(inst, wrap_prim128(op, a, 0), n);
                    } else if op == PRIM_WRAP_SHL || op == PRIM_ROTL || op == PRIM_ROTR {
                        let sh = vpop(inst) as u128;
                        let a = vpop128(inst);
                        vpush128(inst, wrap_prim128(op, a, sh), n);
                    } else {
                        let b = vpop128(inst);
                        let a = vpop128(inst);
                        let r = if prim_is_sat(op) {
                            sat_prim128(op, a, b, ty_is_signed(rt))
                        } else {
                            wrap_prim128(op, a, b)
                        };
                        vpush128(inst, r, n);
                    }
                } else if op == PRIM_WRAP_NEG {
                    let a = vpop(inst);
                    vpush(inst, wrap_prim(op, a, 0, rt), n);
                } else {
                    let b = vpop(inst);
                    let a = vpop(inst);
                    let r = if prim_is_sat(op) {
                        sat_prim(op, a, b, rt)
                    } else {
                        wrap_prim(op, a, b, rt)
                    };
                    vpush(inst, r, n);
                }
                return;
            }

            if c.step == 0 {
                inst.cstack[ci].step = 1;
                inst.cstack[ci].a = n.b;
                if chk.res[i] & RES_MPLACE != 0 {
                    push_place(inst, n.d, n);
                } else {
                    push_eval(mem, chk, inst, n.d, n);
                }
                return;
            }
            if c.step == 1 {
                let cur = c.a;
                if cur != NODE_NIL {
                    inst.cstack[ci].a = mem.node(cur).link;
                    push_eval(mem, chk, inst, cur, n);
                    return;
                }
                inst.cstack[ci].step = 2;

            }
            inst.c_n -= 1;
            let f = chk.fns[(chk.res[i] & RES_MFN_MASK) as usize];

            let mut nargs = 0u32;
            let mut p = f.first_param;
            while p != NODE_NIL {
                nargs += chk.size_of(chk.ty[p as usize]);
                p = mem.node(p).link;
            }
            if inst.call_depth as usize >= CALL_DEPTH_MAX {
                trap(inst, E_T_STACK, n);
                return;
            }
            let new_base = inst.frame_top;
            if new_base as usize + f.frame as usize > CAP_FRAME_MEM {
                trap(inst, E_T_STACK, n);
                return;
            }
            if !store_frame(inst, new_base, nargs, n) {
                return;
            }
            let mut rc = CTL_NONE;
            rc.kind = K_RET;
            rc.a = inst.frame_base;
            rc.b = inst.frame_top;
            rc.node = inst.v_n as u32;
            if !cpush(inst, rc, n) {
                return;
            }
            inst.frame_base = new_base;
            inst.frame_top = new_base + f.frame;
            inst.call_depth += 1;
            let body = mem.node(f.node).e;
            push_eval(mem, chk, inst, body, n);
        }

        _ => {
            trap(inst, E_T_INTERNAL, n);
        }
    }
}

/// `return`: unwind the control stack to the enclosing call's K_RET frame,
/// moving the `rs` return-value slots down to the function's vstack base. The
/// K_RET frame is left in place so it restores frames on the next step; if no
/// K_RET is found we are in the entry function and the run completes.
fn unwind_return(inst: &mut Instance, rs: usize, at: Node) {
    if rs > inst.v_n {
        trap(inst, E_T_INTERNAL, at);
        return;
    }
    let src = inst.v_n - rs;
    let vbase;
    loop {
        if inst.c_n == 0 {
            vbase = 0;
            break;
        }
        let c = inst.cstack[inst.c_n - 1];
        if c.kind == K_RET {
            vbase = c.node as usize;
            break;
        }
        inst.c_n -= 1;
    }
    let mut k = 0;
    while k < rs {
        inst.vstack[vbase + k] = inst.vstack[src + k];
        k += 1;
    }
    inst.v_n = vbase + rs;
}

/// break/continue: unwind the control stack to the innermost loop.
fn unwind_loop(mem: &Mem, chk: &Chk, inst: &mut Instance, at: Node, is_break: bool) {
    loop {
        if inst.c_n == 0 {
            trap(inst, E_T_INTERNAL, at);
            return;
        }
        let c = inst.cstack[inst.c_n - 1];
        if c.kind == K_RET {
            trap(inst, E_T_INTERNAL, at);
            return;
        }
        let k = mem.node(c.node).kind;
        if c.kind == K_EVAL && (k == N_WHILE || k == N_LOOP || k == N_FOR) && c.step != 0 {
            if is_break {
                inst.v_n = c.b as usize;
                inst.c_n -= 1;
                return;
            }
            inst.v_n = c.a as usize;
            let ci = inst.c_n - 1;
            if k == N_WHILE {

                inst.cstack[ci].step = 1;
                let n = mem.node(c.node);
                push_eval(mem, chk, inst, n.d, n);
            } else if k == N_LOOP {
                let n = mem.node(c.node);
                push_eval(mem, chk, inst, n.e, n);
            } else {

                inst.cstack[ci].step = 4;

            }
            return;
        }
        inst.c_n -= 1;
    }
}

fn step_place(mem: &Mem, chk: &Chk, inst: &mut Instance, ci: usize, c: Ctl, n: Node, i: usize) {
    match n.kind {
        N_NAME => {
            inst.c_n -= 1;
            let addr = inst.frame_base + chk.res[i];
            vpush(inst, addr as u64, n);
        }
        N_DOT => {
            if c.step == 0 {
                inst.cstack[ci].step = 1;
                if chk.res[i] & RES_DEREF != 0 {

                    push_eval(mem, chk, inst, n.d, n);
                } else {
                    push_place(inst, n.d, n);
                }
                return;
            }
            inst.c_n -= 1;
            let addr = vpop(inst);
            let off = (chk.res[i] & RES_OFF_MASK) as u64;
            vpush(inst, addr + off, n);
        }
        N_INDEX => {
            let bt = chk.ty[n.d as usize];
            let is_slice = ty_is_slice(bt);
            let is_deref = chk.res[i] & RES_DEREF != 0;
            if c.step == 0 {
                inst.cstack[ci].step = 1;

                if is_slice || is_deref {
                    push_eval(mem, chk, inst, n.d, n);
                } else {
                    push_place(inst, n.d, n);
                }
                return;
            }
            if c.step == 1 {
                inst.cstack[ci].step = 2;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
            let idx = vpop(inst);
            if is_slice {
                let len = vpop(inst);
                let addr = vpop(inst);
                if idx >= len {
                    trap(inst, E_T_OOB, n);
                    return;
                }
                let es = chk.size_of(chk.slinfo(bt).pointee) as u64;
                vpush(inst, addr + idx * es, n);
                return;
            }
            let addr = vpop(inst);
            let a = if is_deref { chk.ainfo(chk.rinfo(bt).pointee) } else { chk.ainfo(bt) };
            if idx >= a.len as u64 {
                trap(inst, E_T_OOB, n);
                return;
            }
            let es = chk.size_of(a.elem) as u64;
            vpush(inst, addr + idx * es, n);
        }
        N_DEREF => {

            if c.step == 0 {
                inst.cstack[ci].step = 1;
                push_eval(mem, chk, inst, n.e, n);
                return;
            }
            inst.c_n -= 1;
        }
        _ => {
            trap(inst, E_T_INTERNAL, n);
        }
    }
}
