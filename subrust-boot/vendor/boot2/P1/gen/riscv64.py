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
    'a0': 10,
    'a1': 11,
    'a2': 12,
    'a3': 13,
    'a4': 14,
    'a5': 15,
    'a6': 16,
    'a7': 17,
    't0': 5,
    't1': 6,
    't2': 7,
    's0': 9,
    's1': 18,
    's2': 19,
    's3': 20,
    'sp': 2,
    'zero': 0,
    'ra': 1,
    'fp': 8,
    'br': 31,
    'scratch': 30,
    'save0': 29,
    'save1': 28,
    'save2': 16,
}


RRR_BASE = {
    'ADD': 0x00000033,
    'SUB': 0x40000033,
    'AND': 0x00007033,
    'OR':  0x00006033,
    'XOR': 0x00004033,
    'SHL': 0x00001033,
    'SHR': 0x00005033,
    'SAR': 0x40005033,
    'MUL': 0x02000033,
    'DIV': 0x02004033,
    'REM': 0x02006033,
}


# Inverted-condition B-type opcodes for the skip-taken-over-jalr pattern:
# the skip fires when the P1 condition is FALSE, so the jalr below is the
# taken target.
CONDB_INV_BASE = {
    'BEQ':  0x00001063,  # native BNE -- skip when not equal
    'BNE':  0x00000063,  # native BEQ -- skip when equal
    'BLT':  0x00005063,  # native BGE -- skip when ra >= rb (signed)
    'BLTU': 0x00007063,  # native BGEU -- skip when ra >= rb (unsigned)
}


CONDBZ_INV_BASE = {
    'BEQZ': 0x00001063,
    'BNEZ': 0x00000063,
    'BLTZ': 0x00005063,
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


def rv_r_type(base, rd, ra, rb):
    d = NAT[rd]
    a = NAT[ra]
    b = NAT[rb]
    return le32(base | (b << 20) | (a << 15) | (d << 7))


def rv_i_type(base, rd, ra, imm12):
    d = NAT[rd]
    a = NAT[ra]
    return le32(base | ((imm12 & 0xFFF) << 20) | (a << 15) | (d << 7))


def rv_s_type(base, rs, ra, imm12):
    s = NAT[rs]
    a = NAT[ra]
    imm = imm12 & 0xFFF
    # arithmetic-shift the 12-bit signed value: bits 11:5 -> [31:25],
    # bits 4:0 -> [11:7]. We only need the unsigned 12-bit pattern here
    # because the m1pp encoder uses (>> imm 5) on the masked value.
    hi = (imm >> 5) & 0x7F
    lo = imm & 0x1F
    return le32(base | (hi << 25) | (s << 20) | (a << 15) | (lo << 7))


def rv_b_type_skip8(base, ra, rb):
    # Hardcoded +8 branch: imm = 8, encoded with imm[4:1]=4, imm[11]=0,
    # imm[10:5]=0, imm[12]=0. The combined [11:7] field becomes
    # (imm[4:1] << 1) | imm[11] = 8.
    a = NAT[ra]
    b = NAT[rb]
    return le32(base | (b << 20) | (a << 15) | (8 << 7))


def rv_addi(rd, ra, imm12):
    return rv_i_type(0x00000013, rd, ra, imm12)


def rv_ld(rd, ra, imm12):
    return rv_i_type(0x00003003, rd, ra, imm12)


def rv_sd(rs, ra, imm12):
    return rv_s_type(0x00003023, rs, ra, imm12)


def rv_lbu(rd, ra, imm12):
    return rv_i_type(0x00004003, rd, ra, imm12)


def rv_sb(rs, ra, imm12):
    return rv_s_type(0x00000023, rs, ra, imm12)


def rv_lwu(rd, ra, imm12):
    return rv_i_type(0x00006003, rd, ra, imm12)


def rv_mov_rr(dst, src):
    return rv_addi(dst, src, 0)


def rv_slli(rd, ra, shamt):
    d = NAT[rd]
    a = NAT[ra]
    return le32(0x00001013 | ((shamt & 0x3F) << 20) | (a << 15) | (d << 7))


def rv_srli(rd, ra, shamt):
    d = NAT[rd]
    a = NAT[ra]
    return le32(0x00005013 | ((shamt & 0x3F) << 20) | (a << 15) | (d << 7))


def rv_srai(rd, ra, shamt):
    d = NAT[rd]
    a = NAT[ra]
    return le32(0x40005013 | ((shamt & 0x3F) << 20) | (a << 15) | (d << 7))


def rv_jalr(rd, rs, imm12):
    d = NAT[rd]
    s = NAT[rs]
    return le32(0x00000067 | ((imm12 & 0xFFF) << 20) | (s << 15) | (d << 7))


def rv_ecall():
    return le32(0x00000073)


def rv_lit64_prefix(rd):
    # auipc rd, 0 ; ld rd, 12(rd) ; jal x0, +12.
    # The 8 bytes that follow in source become the literal.
    d = NAT[rd]
    auipc = 0x00000017 | (d << 7)
    ld = 0x00C03003 | (d << 15) | (d << 7)
    jal = 0x00C0006F
    return le32(auipc) + le32(ld) + le32(jal)


def rv_lit32_prefix(rd):
    # auipc rd, 0 ; lwu rd, 12(rd) ; jal x0, +8.
    # lwu zero-extends a 4-byte literal; enough for stage0 addresses.
    d = NAT[rd]
    auipc = 0x00000017 | (d << 7)
    lwu = 0x00C06003 | (d << 15) | (d << 7)
    jal = 0x0080006F
    return le32(auipc) + le32(lwu) + le32(jal)


def rv_epilogue():
    # Frame teardown shared by ERET, TAIL, TAILR. Mirrors p1_eret/p1_tail
    # in P1-riscv64.M1pp: load saved ra, load saved caller sp into fp,
    # then move fp into sp. The caller appends the actual jalr.
    return rv_ld('ra', 'sp', 0) + rv_ld('fp', 'sp', 8) + rv_mov_rr('sp', 'fp')


def encode_li(_arch, row):
    return rv_lit64_prefix(row.rd)


def encode_la(_arch, row):
    return rv_lit32_prefix(row.rd)


def encode_labr(_arch, _row):
    return rv_lit32_prefix('br')


def encode_mov(_arch, row):
    # Portable sp is the frame-local base, which sits 16 bytes above
    # native sp (the backend's 2-word hidden header occupies the low
    # end of each frame). MOV rd, sp must therefore yield native_sp+16.
    if row.rs == 'sp':
        return rv_addi(row.rd, 'sp', 16)
    return rv_mov_rr(row.rd, row.rs)


def encode_rrr(_arch, row):
    return rv_r_type(RRR_BASE[row.op], row.rd, row.ra, row.rb)


def encode_addi(_arch, row):
    return rv_addi(row.rd, row.ra, row.imm)


def encode_logi(_arch, row):
    base = {
        'ANDI': 0x00007013,
        'ORI':  0x00006013,
    }[row.op]
    return rv_i_type(base, row.rd, row.ra, row.imm)


def encode_shifti(_arch, row):
    if row.op == 'SHLI':
        return rv_slli(row.rd, row.ra, row.imm)
    if row.op == 'SHRI':
        return rv_srli(row.rd, row.ra, row.imm)
    if row.op == 'SARI':
        return rv_srai(row.rd, row.ra, row.imm)
    raise ValueError(f'unknown shift op: {row.op}')


def encode_mem(_arch, row):
    # Portable sp points to the frame-local base; the 2-word hidden header
    # at native_sp+0/+8 is not portable-addressable. Shift sp-relative
    # offsets past the header.
    off = row.off + 16 if row.rn == 'sp' else row.off
    if row.op == 'LD':
        return rv_ld(row.rt, row.rn, off)
    if row.op == 'ST':
        return rv_sd(row.rt, row.rn, off)
    if row.op == 'LB':
        return rv_lbu(row.rt, row.rn, off)
    if row.op == 'SB':
        return rv_sb(row.rt, row.rn, off)
    raise ValueError(f'unknown mem op: {row.op}')


def encode_ldarg(_arch, row):
    # LDARG loads the saved caller sp from [sp+8] (the hidden header
    # slot), then indexes the incoming stack-arg area off it. Slot 0 is
    # at caller_sp+16 because the native call instruction does not push
    # a return address on riscv64 -- the +16 matches the aarch64 layout
    # by convention for stage0 frame uniformity.
    return rv_ld('scratch', 'sp', 8) + rv_ld(row.rd, 'scratch', 16 + 8 * row.slot)


def encode_branch_reg(_arch, row):
    if row.kind == 'BR':
        return rv_jalr('zero', row.rs, 0)
    if row.kind == 'CALLR':
        return rv_jalr('ra', row.rs, 0)
    if row.kind == 'TAILR':
        return rv_epilogue() + rv_jalr('zero', row.rs, 0)
    raise ValueError(f'unknown branch-reg kind: {row.kind}')


def encode_condb(_arch, row):
    return rv_b_type_skip8(CONDB_INV_BASE[row.op], row.ra, row.rb) + rv_jalr('zero', 'br', 0)


def encode_condbz(_arch, row):
    return rv_b_type_skip8(CONDBZ_INV_BASE[row.op], row.ra, 'zero') + rv_jalr('zero', 'br', 0)


def encode_enter(arch, row):
    frame_bytes = round_up(arch.stack_align, 2 * arch.word_bytes + row.size)
    return (
        rv_addi('sp', 'sp', -frame_bytes)
        + rv_sd('ra', 'sp', 0)
        + rv_addi('fp', 'sp', frame_bytes)
        + rv_sd('fp', 'sp', 8)
    )


def encode_nullary(_arch, row):
    if row.kind == 'B':
        return rv_jalr('zero', 'br', 0)
    if row.kind == 'CALL':
        return rv_jalr('ra', 'br', 0)
    if row.kind == 'RET':
        return rv_jalr('zero', 'ra', 0)
    if row.kind == 'ERET':
        return rv_epilogue() + rv_jalr('zero', 'ra', 0)
    if row.kind == 'TAIL':
        return rv_epilogue() + rv_jalr('zero', 'br', 0)
    if row.kind == 'SYSCALL':
        # P1: a0=number, a1..a3,t0,s0,s1 = args 0..5.
        # Linux riscv64: a7=number, a0..a5 = args 0..5, return in a0.
        # SYSCALL clobbers only P1 a0; restore a1/a2/a3 after ecall.
        return ''.join([
            rv_mov_rr('save0', 'a1'),
            rv_mov_rr('save1', 'a2'),
            rv_mov_rr('save2', 'a3'),
            rv_mov_rr('a7', 'a0'),
            rv_mov_rr('a0', 'save0'),
            rv_mov_rr('a1', 'save1'),
            rv_mov_rr('a2', 'save2'),
            rv_mov_rr('a3', 't0'),
            rv_mov_rr('a4', 's0'),
            rv_mov_rr('a5', 's1'),
            rv_ecall(),
            rv_mov_rr('a1', 'save0'),
            rv_mov_rr('a2', 'save1'),
            rv_mov_rr('a3', 'save2'),
        ])
    raise ValueError(f'unknown nullary kind: {row.kind}')


def rv_start_stub():
    # Backend-owned :_start stub per docs/P1.md §Program Entry. Linux
    # riscv64 puts argc at [sp] and argv starting at [sp+8]; load argc
    # into a0, compute &argv[0] into a1, call p1_main under the one-word
    # direct-result convention, then issue sys_exit. Mirrors %p1_entry
    # in p1/P1-riscv64.M1pp.
    #
    # Raw hex outside DEFINE bodies must be single-quoted so bootstrap
    # M0 treats it as a literal byte run rather than a token.
    def q(hex_bytes):
        return f"'{hex_bytes}'"
    return [
        ':_start',
        q(rv_ld('a0', 'sp', 0)),
        q(rv_addi('a1', 'sp', 8)),
        q(rv_lit32_prefix('br')),
        '&p1_main',
        q(rv_jalr('ra', 'br', 0)),
        q(rv_addi('a7', 'zero', 93)),
        q(rv_ecall()),
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
    name='riscv64',
    word_bytes=8,
    stack_align=16,
    syscall_numbers=SYSCALL_NUMBERS,
    encoders=ENCODERS,
    start_stub=rv_start_stub,
)
