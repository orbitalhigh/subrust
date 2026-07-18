
use crate::check::{TY_BOOL, TY_F64, TY_I64, TY_STR, TY_U64, TY_UNIT, TY_USIZE};
use crate::platform::*;

/// SR-seed word-memory size (words). See subrust-boot/SR-SEED.md.
pub const BOOT_WORDS: usize = 1 << 20;

/// BOOT_API: the SR-seed rung of the bootstrap ladder
/// (subrust-boot/SR-SEED.md). Ids are table order; `sr0i` implements the
/// same table in assembly.
pub const BOOT_API: HostDef = HostDef {
    structs: &[],
    fns: &[
        HostFnDef { name: "ld", params: &[ht(TY_U64)], ret: ht(TY_U64) },
        HostFnDef { name: "st", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_UNIT) },
        HostFnDef { name: "getb", params: &[], ret: ht(TY_U64) },
        HostFnDef { name: "putb", params: &[ht(TY_U64)], ret: ht(TY_UNIT) },
        HostFnDef { name: "f_add", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_U64) },
        HostFnDef { name: "f_sub", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_U64) },
        HostFnDef { name: "f_mul", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_U64) },
        HostFnDef { name: "f_div", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_U64) },
        HostFnDef { name: "f_rem", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_U64) },
        HostFnDef { name: "f_lt", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_BOOL) },
        HostFnDef { name: "f_eq", params: &[ht(TY_U64), ht(TY_U64)], ret: ht(TY_BOOL) },
        HostFnDef { name: "f_from_i", params: &[ht(TY_U64)], ret: ht(TY_U64) },
        HostFnDef { name: "f_to_i", params: &[ht(TY_U64)], ret: ht(TY_U64) },
    ],
};

pub const TEST_API: HostDef = HostDef {
    structs: &[],
    fns: &[
        HostFnDef {
            name: "print_i64",
            params: &[ht(TY_I64)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "print_u64",
            params: &[ht(TY_U64)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "print_usize",
            params: &[ht(TY_USIZE)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "print_f64",
            params: &[ht(TY_F64)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "print_bool",
            params: &[ht(TY_BOOL)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "print_str",
            params: &[ht(TY_STR)],
            ret: ht(TY_UNIT),
        },
    ],
};

pub const HVAC_API: HostDef = HostDef {
    structs: &[
        HostStructDef {
            name: "Env",
            fields: &[
                HostField { name: "now_s", ty: ht(TY_I64) },
                HostField { name: "dow", ty: ht(TY_I64) },
                HostField { name: "year", ty: ht(TY_I64) },
                HostField { name: "month", ty: ht(TY_I64) },
                HostField { name: "day", ty: ht(TY_I64) },
                HostField { name: "hour", ty: ht(TY_I64) },
                HostField { name: "minute", ty: ht(TY_I64) },
                HostField { name: "second", ty: ht(TY_I64) },
            ],
        },
        HostStructDef {
            name: "Sensor",
            fields: &[
                HostField { name: "value", ty: ht(TY_F64) },
                HostField { name: "age_s", ty: ht(TY_I64) },
                HostField { name: "known", ty: ht(TY_BOOL) },
            ],
        },
        HostStructDef {
            name: "Relays",
            fields: &[
                HostField { name: "on", ty: ht_arr(TY_BOOL, 4) },
                HostField { name: "age_s", ty: ht(TY_I64) },
                HostField { name: "known", ty: ht(TY_BOOL) },
            ],
        },
        HostStructDef {
            name: "Health",
            fields: &[
                HostField { name: "age_s", ty: ht(TY_I64) },
                HostField { name: "uptime_s", ty: ht(TY_I64) },
            ],
        },
    ],
    fns: &[
        HostFnDef {
            name: "sensor",
            params: &[ht(TY_STR)],
            ret: ht_struct("Sensor"),
        },
        HostFnDef {
            name: "sensor_set",
            params: &[ht(TY_STR), ht(TY_F64)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "relays",
            params: &[ht(TY_STR)],
            ret: ht_struct("Relays"),
        },
        HostFnDef {
            name: "relays_set",
            params: &[ht(TY_STR), ht_arr(TY_BOOL, 4)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "relay_send",
            params: &[ht(TY_STR), ht(TY_I64), ht(TY_BOOL)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "health",
            params: &[ht(TY_STR)],
            ret: ht_struct("Health"),
        },
        HostFnDef {
            name: "log",
            params: &[ht(TY_STR)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "debug",
            params: &[ht(TY_STR)],
            ret: ht(TY_UNIT),
        },
        HostFnDef {
            name: "debug_f64",
            params: &[ht(TY_STR), ht(TY_F64)],
            ret: ht(TY_UNIT),
        },
    ],
};
