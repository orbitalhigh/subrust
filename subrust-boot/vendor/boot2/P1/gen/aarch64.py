from common import (
    AddI,
    ArchDef,
    BranchReg,
    CondB,
    CondBZ,
    Enter,
    La,
    LaBr,
    LdArg,
    Li,
    LogI,
    Mem,
    Mov,
    Nullary,
    Rrr,
    ShiftI,
    le32,
    round_up,
)


NAT = {
    'a0': 0,
    'a1': 1,
    'a2': 2,
    'a3': 3,
    'x4': 4,
    'x5': 5,
    't0': 9,
    't1': 10,
    't2': 11,
    's0': 19,
    's1': 20,
    's2': 21,
    's3': 22,
    'sp': 31,
    'xzr': 31,
    'lr': 30,
    'br': 17,
    'scratch': 16,
    'x8': 8,
    'save0': 23,
    'save1': 24,
    'save2': 25,
}


RRR_BASE = {
    'ADD': 0x8B000000,
    'SUB': 0xCB000000,
    'AND': 0x8A000000,
    'OR': 0xAA000000,
    'XOR': 0xCA000000,
    'SHL': 0x9AC02000,
    'SHR': 0x9AC02400,
    'SAR': 0x9AC02800,
    'DIV': 0x9AC00C00,
}


SYSCALL_NUMBERS = {
    'SYS_READ': 63,
    'SYS_WRITE': 64,
    'SYS_CLOSE': 57,
    'SYS_OPENAT': 56,
    'SYS_EXIT': 93,
    'SYS_CLONE': 220,
    'SYS_EXECVE': 221,
    'SYS_WAITID': 95,
}


def aa_rrr(base, rd, ra, rb):
    d = NAT[rd]
    a = NAT[ra]
    b = NAT[rb]
    return le32(base | (b << 16) | (a << 5) | d)


def aa_add_imm(rd, ra, imm12, sub=False):
    d = NAT[rd]
    a = NAT[ra]
    base = 0xD1000000 if sub else 0x91000000
    return le32(base | ((imm12 & 0xFFF) << 10) | (a << 5) | d)


def aa_mov_rr(dst, src):
    if dst == 'sp':
        return aa_add_imm('sp', src, 0, sub=False)
    if src == 'sp':
        return aa_add_imm(dst, 'sp', 0, sub=False)
    d = NAT[dst]
    s = NAT[src]
    return le32(0xAA000000 | (s << 16) | (31 << 5) | d)


def aa_ubfm(rd, ra, immr, imms):
    d = NAT[rd]
    a = NAT[ra]
    return le32(0xD3400000 | (immr << 16) | (imms << 10) | (a << 5) | d)


def aa_sbfm(rd, ra, immr, imms):
    d = NAT[rd]
    a = NAT[ra]
    return le32(0x93400000 | (immr << 16) | (imms << 10) | (a << 5) | d)


def aa_movz(rd, imm16):
    d = NAT[rd]
    return le32(0xD2800000 | ((imm16 & 0xFFFF) << 5) | d)


def aa_movn(rd, imm16):
    d = NAT[rd]
    return le32(0x92800000 | ((imm16 & 0xFFFF) << 5) | d)


def aa_materialize_small_imm(rd, imm):
    if imm >= 0:
        return aa_movz(rd, imm)
    return aa_movn(rd, (~imm) & 0xFFFF)


def aa_ldst_uimm12(base, rt, rn, off_bytes, size_log2):
    imm12 = off_bytes >> size_log2
    t = NAT[rt]
    n = NAT[rn]
    return le32(base | (imm12 << 10) | (n << 5) | t)


def aa_ldst_unscaled(base, rt, rn, off):
    imm9 = off & 0x1FF
    t = NAT[rt]
    n = NAT[rn]
    return le32(base | (imm9 << 12) | (n << 5) | t)


def aa_mem(op, rt, rn, off):
    bases = {
        'LD': (0xF9400000, 3, 0xF8400000),
        'ST': (0xF9000000, 3, 0xF8000000),
        'LB': (0x39400000, 0, 0x38400000),
        'SB': (0x39000000, 0, 0x38000000),
    }
    uimm_base, size_log2, unscaled_base = bases[op]
    scale = 1 << size_log2
    if off >= 0 and off % scale == 0 and off < (4096 << size_log2):
        return aa_ldst_uimm12(uimm_base, rt, rn, off, size_log2)
    if -256 <= off <= 255:
        return aa_ldst_unscaled(unscaled_base, rt, rn, off)
    if -2048 <= off <= 2047:
        if off >= 0:
            addr = aa_add_imm('scratch', rn, off, sub=False)
        else:
            addr = aa_add_imm('scratch', rn, -off, sub=True)
        return addr + aa_ldst_uimm12(uimm_base, rt, 'scratch', 0, size_log2)
    raise ValueError(f'aarch64 offset out of range for {op}: {off}')


def aa_cmp_skip(op, ra, rb):
    a = NAT[ra]
    b = NAT[rb]
    cmp_hex = le32(0xEB000000 | (b << 16) | (a << 5) | 31)
    skip_cond = {
        'BEQ': 1,
        'BNE': 0,
        'BLT': 10,
        'BLTU': 2,
    }[op]
    return cmp_hex + le32(0x54000040 | skip_cond)


def aa_br(reg):
    return le32(0xD61F0000 | (NAT[reg] << 5))


def aa_blr(reg):
    return le32(0xD63F0000 | (NAT[reg] << 5))


def aa_ret():
    return le32(0xD65F03C0)


def aa_epilogue():
    # Frame teardown, shared by ERET, TAIL, TAILR. Loads lr and the
    # saved caller sp from the hidden header at native_sp+0/+8, then
    # unwinds sp. Does NOT transfer control; the caller appends an
    # aa_ret / aa_br as appropriate.
    return (
        aa_mem('LD', 'lr', 'sp', 0)
        + aa_mem('LD', 'x8', 'sp', 8)
        + aa_mov_rr('sp', 'x8')
    )


def aa_lit64_prefix(rd):
    ## 64-bit literal-pool prefix for LI: ldr xN, [pc,#8]; b PC+12.
    ## The 8 bytes that follow in source become the literal; b skips them.
    d = NAT[rd]
    ldr_lit = 0x58000040 | d
    b_plus12 = 0x14000003
    return le32(ldr_lit) + le32(b_plus12)


def aa_lit32_prefix(rd):
    ## 32-bit literal-pool prefix for LA / LA_BR: ldr wN, [pc,#8]; b PC+8.
    ## ldr w zero-extends into the full 64-bit register, so a 4-byte literal
    ## is enough for any address in the stage0 layout (base 0x00600000,
    ## programs well under 4 GB). This lets source use `&label` directly
    ## without padding to 8 bytes.
    d = NAT[rd]
    ldr_lit = 0x18000040 | d
    b_plus8 = 0x14000002
    return le32(ldr_lit) + le32(b_plus8)


def encode_li(_arch, row):
    return aa_lit64_prefix(row.rd)


def encode_la(_arch, row):
    return aa_lit32_prefix(row.rd)


def encode_labr(_arch, _row):
    return aa_lit32_prefix('br')


def encode_mov(_arch, row):
    # Portable `sp` is the frame-local base, which is 16 bytes above
    # native sp (the backend's 2-word hidden header sits at the low end
    # of each frame allocation). So reading sp into a register yields
    # native_sp + 16, not native_sp itself.
    if row.rs == 'sp':
        return aa_add_imm(row.rd, 'sp', 16, sub=False)
    return aa_mov_rr(row.rd, row.rs)


def encode_rrr(_arch, row):
    if row.op == 'MUL':
        d = NAT[row.rd]
        a = NAT[row.ra]
        b = NAT[row.rb]
        return le32(0x9B000000 | (b << 16) | (31 << 10) | (a << 5) | d)
    if row.op == 'REM':
        d = NAT[row.rd]
        a = NAT[row.ra]
        b = NAT[row.rb]
        sc = NAT['scratch']
        sdiv = 0x9AC00C00 | (b << 16) | (a << 5) | sc
        msub = 0x9B008000 | (b << 16) | (a << 10) | (sc << 5) | d
        return le32(sdiv) + le32(msub)
    return aa_rrr(RRR_BASE[row.op], row.rd, row.ra, row.rb)


def encode_addi(_arch, row):
    if row.imm >= 0:
        return aa_add_imm(row.rd, row.ra, row.imm, sub=False)
    return aa_add_imm(row.rd, row.ra, -row.imm, sub=True)


def encode_logi(_arch, row):
    seq = aa_materialize_small_imm('scratch', row.imm)
    base = {
        'ANDI': 0x8A000000,
        'ORI': 0xAA000000,
    }[row.op]
    return seq + aa_rrr(base, row.rd, row.ra, 'scratch')


def encode_shifti(_arch, row):
    if row.op == 'SHLI':
        return aa_ubfm(row.rd, row.ra, (-row.imm) & 63, 63 - row.imm)
    if row.op == 'SHRI':
        return aa_ubfm(row.rd, row.ra, row.imm, 63)
    return aa_sbfm(row.rd, row.ra, row.imm, 63)


def encode_mem(_arch, row):
    # Portable sp points to the frame-local base; the 2-word hidden
    # header sits at native_sp+0/+8 and is not portable-addressable.
    # Shift sp-relative offsets past the header.
    off = row.off + 16 if row.rn == 'sp' else row.off
    return aa_mem(row.op, row.rt, row.rn, off)


def encode_ldarg(_arch, row):
    return aa_mem('LD', 'scratch', 'sp', 8) + aa_mem('LD', row.rd, 'scratch', 16 + 8 * row.slot)


def encode_branch_reg(_arch, row):
    if row.kind == 'BR':
        return aa_br(row.rs)
    if row.kind == 'CALLR':
        return aa_blr(row.rs)
    if row.kind == 'TAILR':
        return aa_epilogue() + aa_br(row.rs)
    raise ValueError(f'unknown branch-reg kind: {row.kind}')


def encode_condb(_arch, row):
    return aa_cmp_skip(row.op, row.ra, row.rb) + aa_br('br')


def encode_condbz(_arch, row):
    a = NAT[row.ra]
    br_hex = aa_br('br')
    if row.op == 'BEQZ':
        return le32(0xB5000000 | (2 << 5) | a) + br_hex
    if row.op == 'BNEZ':
        return le32(0xB4000000 | (2 << 5) | a) + br_hex
    cmp_zero = le32(0xEB1F001F | (a << 5))
    bge = le32(0x54000040 | 10)
    return cmp_zero + bge + br_hex


def encode_enter(arch, row):
    frame_bytes = round_up(arch.stack_align, 2 * arch.word_bytes + row.size)
    return (
        aa_add_imm('sp', 'sp', frame_bytes, sub=True)
        + aa_mem('ST', 'lr', 'sp', 0)
        + aa_add_imm('x8', 'sp', frame_bytes, sub=False)
        + aa_mem('ST', 'x8', 'sp', 8)
    )


def encode_nullary(_arch, row):
    if row.kind == 'B':
        return aa_br('br')
    if row.kind == 'CALL':
        return aa_blr('br')
    if row.kind == 'RET':
        return aa_ret()
    if row.kind == 'ERET':
        return aa_epilogue() + aa_ret()
    if row.kind == 'TAIL':
        return aa_epilogue() + aa_br('br')
    if row.kind == 'SYSCALL':
        return ''.join([
            aa_mov_rr('x8', 'a0'),
            aa_mov_rr('save0', 'a1'),
            aa_mov_rr('save1', 'a2'),
            aa_mov_rr('save2', 'a3'),
            aa_mov_rr('a0', 'save0'),
            aa_mov_rr('a1', 'save1'),
            aa_mov_rr('a2', 'save2'),
            aa_mov_rr('a3', 't0'),
            aa_mov_rr('x4', 's0'),
            aa_mov_rr('x5', 's1'),
            le32(0xD4000001),
            aa_mov_rr('a1', 'save0'),
            aa_mov_rr('a2', 'save1'),
            aa_mov_rr('a3', 'save2'),
        ])
    raise ValueError(f'unknown nullary kind: {row.kind}')


def aa_start_stub():
    # Backend-owned :_start stub per docs/P1.md §Program Entry. Captures
    # argc from [sp] and argv pointer from sp+8, calls p1_main under the
    # one-word direct-result convention (a0=argc, a1=argv), then issues a
    # native Linux sys_exit with p1_main's return value. Mirrors the
    # m1pp-path stub in p1/P1-aarch64.M1pp (`%p1_entry`).
    #
    # Raw hex outside `DEFINE` bodies must be single-quoted so bootstrap
    # M0 treats it as a literal byte run rather than a token.
    def q(hex_bytes):
        return f"'{hex_bytes}'"
    return [
        ':_start',
        q(aa_mem('LD', 'a0', 'sp', 0)),
        q(aa_add_imm('a1', 'sp', 8, sub=False)),
        q(aa_lit32_prefix('br')),
        '&p1_main',
        q(aa_blr('br')),
        q(aa_movz('x8', 93)),
        q(le32(0xD4000001)),
    ]


ENCODERS = {
    Li: encode_li,
    La: encode_la,
    LaBr: encode_labr,
    Mov: encode_mov,
    Rrr: encode_rrr,
    AddI: encode_addi,
    LogI: encode_logi,
    ShiftI: encode_shifti,
    Mem: encode_mem,
    LdArg: encode_ldarg,
    Nullary: encode_nullary,
    BranchReg: encode_branch_reg,
    CondB: encode_condb,
    CondBZ: encode_condbz,
    Enter: encode_enter,
}


ARCH = ArchDef(
    name='aarch64',
    word_bytes=8,
    stack_align=16,
    syscall_numbers=SYSCALL_NUMBERS,
    encoders=ENCODERS,
    start_stub=aa_start_stub,
)
