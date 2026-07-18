
pub type SrErr = u16;

pub const SR_OK: SrErr = 0;
pub const SR_E_LOAD: SrErr = 1;
pub const SR_E_TRAP: SrErr = 2;
pub const SR_E_HOST: SrErr = 3;
pub const SR_E_LIMIT: SrErr = 4;
pub const SR_E_LINK: SrErr = 5;

/// crate boundary is stable.
pub trait Platform {
    /// Perform host function `id` (its index in HostDef.fns) with marshaled
    /// argument slots, writing return slots. Aggregates are flattened in
    /// field-declaration order; &str arguments arrive as string-table ids
    /// the embedder resolves via `Chk::str_bytes`.
    fn host_call(&mut self, id: u16, args: &[u64], ret: &mut [u64]) -> SrErr;

    /// Interpreter-internal debug logging (never script output).
    fn dev_log(&mut self, _s: &str) {}
}

/// Composite markers for HostTy.kind (primitives use check::TY_*).
pub const HT_STRUCT: u16 = 0xFFFD;
pub const HT_ARR: u16 = 0xFFFE;

#[derive(Clone, Copy)]
pub struct HostTy {
    pub kind: u16,
    pub sname: &'static str,
    pub elem: u16,
    pub len: u32,
}

/// Primitive host type (`ht(TY_I64)`).
pub const fn ht(kind: u16) -> HostTy {
    HostTy {
        kind,
        sname: "",
        elem: 0,
        len: 0,
    }
}

/// Host struct type by name (`ht_struct("Sensor")`).
pub const fn ht_struct(name: &'static str) -> HostTy {
    HostTy {
        kind: HT_STRUCT,
        sname: name,
        elem: 0,
        len: 0,
    }
}

/// Fixed array of a primitive (`ht_arr(TY_BOOL, 4)`).
pub const fn ht_arr(elem: u16, len: u32) -> HostTy {
    HostTy {
        kind: HT_ARR,
        sname: "",
        elem,
        len,
    }
}

pub struct HostField {
    pub name: &'static str,
    pub ty: HostTy,
}

pub struct HostStructDef {
    pub name: &'static str,
    pub fields: &'static [HostField],
}

pub struct HostFnDef {
    pub name: &'static str,
    pub params: &'static [HostTy],
    pub ret: HostTy,
}

pub struct HostDef {
    pub structs: &'static [HostStructDef],
    pub fns: &'static [HostFnDef],
}

/// No host API at all (pure computation scripts).
pub const EMPTY_API: HostDef = HostDef {
    structs: &[],
    fns: &[],
};
