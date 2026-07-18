//! subrust — an interpreter for a small subset of Rust.
//!
//! `no_std`, no alloc, zero dependencies, `forbid(unsafe_code)`. Scripts are
//! real Rust source files; rustc is the reference implementation (PLAN.md §2).
//!
//! This file and `platform.rs` are the QUARANTINE ZONE (PLAN.md §4): the only
//! modules allowed to use Rust constructs outside the bootstrap dialect
//! (traits, cfg wiring, public-API conveniences). Every other module is pure
//! dialect — the long-term self-hosting target.

#![no_std]
#![forbid(unsafe_code)]

pub mod apis;
pub mod ast;
pub mod check;
pub mod diag;
pub mod lex;
pub mod machine;
pub mod parse;
pub mod platform;

pub use ast::{Mem, Node, MEM_INIT, NODE_NIL};
pub use check::{Chk, CHK_INIT};
pub use diag::{Diag, Span};
pub use lex::Tok;
pub use machine::{Instance, INSTANCE_INIT, RAN_DONE, RAN_HOST, RAN_TRAP};
pub use platform::{HostDef, Platform, SrErr, EMPTY_API, SR_OK};

#[cfg(feature = "profile-host")]
pub const CAP_TOKS: usize = 65536;
#[cfg(feature = "profile-host")]
pub const CAP_NODES: usize = 32768;
#[cfg(feature = "profile-host")]
pub const CAP_DIAGS: usize = 16;
#[cfg(feature = "profile-host")]
pub const CAP_STRUCTS: usize = 64;
#[cfg(feature = "profile-host")]
pub const CAP_ENUMS: usize = 64;
#[cfg(feature = "profile-host")]
pub const CAP_ARRS: usize = 128;
#[cfg(feature = "profile-host")]
pub const CAP_REFS: usize = 64;
#[cfg(feature = "profile-host")]
pub const CAP_SLICES: usize = 64;
#[cfg(feature = "profile-host")]
pub const CAP_TUPLES: usize = 128;
#[cfg(feature = "profile-host")]
pub const CAP_CONSTS: usize = 512;
#[cfg(feature = "profile-host")]
pub const CAP_FNS: usize = 512;
#[cfg(feature = "profile-host")]
pub const CAP_LOCALS: usize = 512;
#[cfg(feature = "profile-host")]
pub const CAP_STRS: usize = 1024;
#[cfg(feature = "profile-host")]
pub const CAP_STR_POOL: usize = 49152;
#[cfg(feature = "profile-host")]
pub const CAP_VALS: usize = 16384;
#[cfg(feature = "profile-host")]
pub const CAP_VSTACK: usize = 2048;
#[cfg(feature = "profile-host")]
pub const CAP_CSTACK: usize = 1024;
#[cfg(feature = "profile-host")]
pub const CAP_FRAME_MEM: usize = 16384;
/// Maximum function frame size, in u64 slots.
pub const FRAME_MAX: u32 = 1024;
/// Maximum script call depth.
pub const CALL_DEPTH_MAX: usize = 64;
/// Largest host-function return value, in slots (pump buffer size).
pub const HOST_RET_MAX: usize = 64;

/// Maximum accepted source length in bytes (positions are u32).
pub const SRC_MAX: usize = 16 * 1024 * 1024;

/// Lex `src` into `mem.toks`. Returns true if there were no errors;
/// diagnostics are in `mem.diags` either way.
pub fn lex_source(src: &str, mem: &mut Mem) -> bool {
    lex::lex(src, mem)
}

/// Lex + parse `src` into `mem` (tokens, node pool, root item chain).
/// Returns true if there were no errors; diagnostics are in `mem.diags`.
pub fn parse_source(src: &str, mem: &mut Mem) -> bool {
    if !lex::lex(src, mem) {
        return false;
    }
    parse::parse(src, mem)
}

/// Lex + parse + check `src` against a host API. On success, (mem, chk) is
/// the typed program the machine runs.
pub fn check_source(src: &str, mem: &mut Mem, chk: &mut Chk, host: &HostDef) -> bool {
    if !parse_source(src, mem) {
        return false;
    }
    check::check(src, mem, chk, host)
}

/// Call script function `name` through a Platform — the ~10-line pump around
/// the pure resumable machine (PLAN.md §7.4). `args` are flattened slots
/// matching the parameter layout; on SR_OK the value is in `inst.result()`.
pub fn call(src: &str, mem: &Mem, chk: &Chk, inst: &mut Instance,
            platform: &mut dyn Platform, name: &str, args: &[u64], fuel: u64) -> SrErr {
    let fi = check::fn_find_name(src, mem, chk, name);
    if fi == usize::MAX {
        return platform::SR_E_LINK;
    }
    let mut st = machine::start(mem, chk, inst, fi, args, fuel);

    let mut ret = [0u64; HOST_RET_MAX];
    loop {
        if st == RAN_DONE {
            return SR_OK;
        }
        if st == RAN_HOST {
            let rn = inst.host_ret as usize;
            if rn > HOST_RET_MAX {
                return platform::SR_E_LINK;
            }
            let mut i = 0;
            while i < rn {
                ret[i] = 0;
                i += 1;
            }
            let e = platform.host_call(inst.host_id, inst.host_arg_slots(), &mut ret[..rn]);
            if e != SR_OK {
                return e;
            }
            st = machine::resume(mem, chk, inst, &ret[..rn]);
            continue;
        }
        return platform::SR_E_TRAP;
    }
}
