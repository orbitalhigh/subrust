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
    byte,
    le32,
    round_up,
)


# ---- Native register numbers --------------------------------------------
#
# Backend-private mapping from P1 register names to native amd64 regnums.
# `br` is the hidden branch-target reg (r15). `scratch` is the per-expansion
# scratch reg (r9). rax/rbp are also used internally (retaddr spill, rcx /
# rdx save slots) and are not P1-visible.

NAT = {
    'a0': 7,    # rdi
    'a1': 6,    # rsi
    'a2': 2,    # rdx
    'a3': 1,    # rcx
    't0': 10,   # r10
    't1': 11,   # r11
    't2': 8,    # r8
    's0': 3,    # rbx
    's1': 12,   # r12
    's2': 13,   # r13
    's3': 14,   # r14
    'sp': 4,    # rsp
    'br': 15,   # r15
    'scratch': 9,  # r9
    'rax': 0,
    'rcx': 1,
    'rdx': 2,
    'rbx': 3,
    'rsp': 4,
    'rbp': 5,
    'rsi': 6,
    'rdi': 7,
    'r8': 8,
    'r9': 9,
    'r10': 10,
    'r11': 11,
    'r12': 12,
    'r13': 13,
    'r14': 14,
    'r15': 15,
}


SYSCALL_NUMBERS = {
    'SYS_READ': 0,
    'SYS_WRITE': 1,
    'SYS_CLOSE': 3,
    'SYS_OPENAT': 257,
    'SYS_EXIT': 60,
    'SYS_CLONE': 56,
    'SYS_EXECVE': 59,
    'SYS_WAITID': 247,
}


# ---- REX / ModRM helpers ------------------------------------------------

def amd_rex_b_short(r):
    # Optional one-byte REX.B (no W) prefix used by push/pop/jmp r/call r/
    # mov r,imm32 when the target reg is r8-r15. Returns '' for low regs.
    if NAT[r] >= 8:
        return byte(0x41)
    return ''


def amd_rex_wb(r):
    # REX.W=1, B=(r>>3) to extend ModRM.rm / SIB.base.
    return byte(0x48 | ((NAT[r] >> 3) & 1))


def amd_rex_wrb(rg, rm):
    # REX.W=1, R=(rg>>3), B=(rm>>3). Used whenever a ModRM.reg field is in
    # use together with a ModRM.rm field.
    return byte(0x48 | (((NAT[rg] >> 3) & 1) << 2) | ((NAT[rm] >> 3) & 1))


def amd_modrm_rr(rg, rm):
    return byte(0xC0 | ((NAT[rg] & 7) << 3) | (NAT[rm] & 7))


def amd_modrm_ext_r(ext, rm):
    return byte(0xC0 | ((ext & 7) << 3) | (NAT[rm] & 7))


# ---- Memory-addressing ModRM (+ SIB + disp) ----------------------------
#
# [base + disp] with `reg` in ModRM.reg. Bases whose low 3 bits are 100 -
# rsp and r12 - must go through a SIB byte; all others use the plain
# encoding. disp selects mod=1 (disp8) when it fits in [-128,127], else
# mod=2 (disp32). We never emit mod=0 / no-disp; the extra byte is fine.

def amd_modrm_disp(reg, base, disp):
    use_sib = (NAT[base] & 7) == 4
    use_disp8 = -128 <= disp <= 127
    reg_lo = NAT[reg] & 7
    if use_sib:
        if use_disp8:
            return byte(0x44 | (reg_lo << 3)) + byte(0x24) + byte(disp)
        return byte(0x84 | (reg_lo << 3)) + byte(0x24) + le32(disp)
    base_lo = NAT[base] & 7
    if use_disp8:
        return byte(0x40 | (reg_lo << 3) | base_lo) + byte(disp)
    return byte(0x80 | (reg_lo << 3) | base_lo) + le32(disp)


# ---- Register / arithmetic primitives ----------------------------------

def amd_mov_rr(dst, src):
    # mov dst, src  --  REX.WRB 89 /r  (source in ModRM.reg, dest in rm)
    return amd_rex_wrb(src, dst) + byte(0x89) + amd_modrm_rr(src, dst)


def amd_alu_rr(opcode, dst, src):
    # ADD/SUB/AND/OR/XOR dst, src  --  REX.WRB <op> /r (src in reg, dst in rm)
    return amd_rex_wrb(src, dst) + byte(opcode) + amd_modrm_rr(src, dst)


def amd_alu_ri8(ext, dst, imm):
    # op dst, imm8 -- REX.WB 83 /ext ib
    return amd_rex_wb(dst) + byte(0x83) + amd_modrm_ext_r(ext, dst) + byte(imm)


def amd_alu_ri32(ext, dst, imm):
    # op dst, imm32 -- REX.WB 81 /ext id
    return amd_rex_wb(dst) + byte(0x81) + amd_modrm_ext_r(ext, dst) + le32(imm)


def amd_shift_ri8(ext, dst, imm):
    # shift dst, imm8 -- REX.WB C1 /ext ib  (SHL=4, SHR=5, SAR=7)
    return (amd_rex_wb(dst) + byte(0xC1) + amd_modrm_ext_r(ext, dst)
            + byte(imm & 0x3F))


def amd_shift_cl(ext, dst):
    # shift dst, cl -- REX.WB D3 /ext
    return amd_rex_wb(dst) + byte(0xD3) + amd_modrm_ext_r(ext, dst)


def amd_imul_rr(dst, src):
    # imul dst, src  --  REX.WRB 0F AF /r  (dst in reg, src in rm)
    return (amd_rex_wrb(dst, src) + byte(0x0F) + byte(0xAF)
            + amd_modrm_rr(dst, src))


def amd_idiv_r(src):
    # idiv src  --  REX.WB F7 /7
    return amd_rex_wb(src) + byte(0xF7) + amd_modrm_ext_r(7, src)


def amd_cqo():
    # cqo -- 48 99 (sign-extend rax into rdx:rax)
    return byte(0x48) + byte(0x99)


def amd_push(r):
    return amd_rex_b_short(r) + byte(0x50 | (NAT[r] & 7))


def amd_pop(r):
    return amd_rex_b_short(r) + byte(0x58 | (NAT[r] & 7))


def amd_mov_imm32_prefix(rd):
    # mov r32, imm32  --  [REX.B] B8+r  (caller appends 4-byte literal).
    # Result is zero-extended into the full 64-bit register.
    return amd_rex_b_short(rd) + byte(0xB8 | (NAT[rd] & 7))


def amd_mov_imm64_prefix(rd):
    # mov r64, imm64  --  REX.W[.B] B8+r  (caller appends 8-byte literal).
    return amd_rex_wb(rd) + byte(0xB8 | (NAT[rd] & 7))


# ---- Memory ops --------------------------------------------------------

def amd_mem_LD(rt, rn, off):
    # mov rT, [rN + off]  --  REX.WRB 8B /r  modrm-with-disp
    return amd_rex_wrb(rt, rn) + byte(0x8B) + amd_modrm_disp(rt, rn, off)


def amd_mem_ST(rt, rn, off):
    # mov [rN + off], rT  --  REX.WRB 89 /r
    return amd_rex_wrb(rt, rn) + byte(0x89) + amd_modrm_disp(rt, rn, off)


def amd_mem_SB(rt, rn, off):
    # mov [rN + off], rT8 -- REX.WRB 88 /r (REX.W forces dil/sil/bpl/spl
    # byte-view encoding when the low byte of those regs is needed).
    return amd_rex_wrb(rt, rn) + byte(0x88) + amd_modrm_disp(rt, rn, off)


def amd_mem_LB(rt, rn, off):
    # movzx rT, byte ptr [rN + off]  --  REX.WRB 0F B6 /r
    return (amd_rex_wrb(rt, rn) + byte(0x0F) + byte(0xB6)
            + amd_modrm_disp(rt, rn, off))


# ---- Control-flow primitives -------------------------------------------

def amd_jmp_r(r):
    # jmp r/m64 -- [REX.B] FF /4. 2 bytes for low regs, 3 bytes for r8-r15.
    return amd_rex_b_short(r) + byte(0xFF) + byte(0xE0 | (NAT[r] & 7))


def amd_call_r(r):
    # call r/m64 -- [REX.B] FF /2.
    return amd_rex_b_short(r) + byte(0xFF) + byte(0xD0 | (NAT[r] & 7))


def amd_ret():
    return byte(0xC3)


def amd_syscall():
    return byte(0x0F) + byte(0x05)


def amd_cmp_rr(ra, rb):
    # cmp rA, rB -- REX.WRB 39 /r (rB in reg, rA in rm).
    return amd_rex_wrb(rb, ra) + byte(0x39) + amd_modrm_rr(rb, ra)


def amd_test_rr(ra, rb):
    return amd_rex_wrb(rb, ra) + byte(0x85) + amd_modrm_rr(rb, ra)


# ---- P1 register-register op lowering ----------------------------------
#
# For ADD/SUB/AND/OR/XOR we honor rD==rB aliasing -- the naive
# `mov rD,rA ; op rD,rB` would clobber rB before the op reads it. Route rB
# through the scratch reg when that aliasing shows up.

ALU_OPCODE = {
    'ADD': 0x01,
    'SUB': 0x29,
    'AND': 0x21,
    'OR': 0x09,
    'XOR': 0x31,
}


def amd_rrr_simple(opcode, rd, ra, rb):
    if NAT[rd] == NAT[rb]:
        return (amd_mov_rr('scratch', rb)
                + amd_mov_rr(rd, ra)
                + amd_alu_rr(opcode, rd, 'scratch'))
    return amd_mov_rr(rd, ra) + amd_alu_rr(opcode, rd, rb)


def amd_rrr_MUL(rd, ra, rb):
    if NAT[rd] == NAT[rb]:
        return (amd_mov_rr('scratch', rb)
                + amd_mov_rr(rd, ra)
                + amd_imul_rr(rd, 'scratch'))
    return amd_mov_rr(rd, ra) + amd_imul_rr(rd, rb)


# DIV / REM clobber rax and rdx natively. rax is not a P1 register, so we
# clobber it freely; rdx IS P1 a2, so we stash it to rbp (also outside the
# P1 mapping) for the lifetime of the op. Aliasing-safety plan mirrors the
# M1pp comments verbatim.

def amd_rrr_DIV(rd, ra, rb):
    return ''.join([
        amd_mov_rr('rbp', 'rdx'),
        amd_mov_rr('scratch', rb),
        amd_mov_rr('rax', ra),
        amd_cqo(),
        amd_idiv_r('scratch'),
        amd_mov_rr('rdx', 'rbp'),
        amd_mov_rr(rd, 'rax'),
    ])


def amd_rrr_REM(rd, ra, rb):
    return ''.join([
        amd_mov_rr('rbp', 'rdx'),
        amd_mov_rr('scratch', rb),
        amd_mov_rr('rax', ra),
        amd_cqo(),
        amd_idiv_r('scratch'),
        amd_mov_rr('rax', 'rdx'),
        amd_mov_rr('rdx', 'rbp'),
        amd_mov_rr(rd, 'rax'),
    ])


# SHL / SHR / SAR with reg count. x86 reads the count from CL only, so
# staging goes through rcx -- which IS P1 a3. Save rcx to rbp for the
# duration. Ordering matches the M1pp comments.

def amd_rrr_shift(ext, rd, ra, rb):
    return ''.join([
        amd_mov_rr('rbp', 'rcx'),
        amd_mov_rr('scratch', ra),
        amd_mov_rr('rcx', rb),
        amd_shift_cl(ext, 'scratch'),
        amd_mov_rr('rcx', 'rbp'),
        amd_mov_rr(rd, 'scratch'),
    ])


# ---- Encoders ----------------------------------------------------------

def encode_li(_arch, row):
    return amd_mov_imm64_prefix(row.rd)


def encode_la(_arch, row):
    return amd_mov_imm32_prefix(row.rd)


def encode_labr(_arch, _row):
    return amd_mov_imm32_prefix('br')


def encode_mov(_arch, row):
    # Portable sp is the frame-local base, which is 16 bytes above native
    # rsp. Reading sp into a register yields native_rsp + 16, so emit
    # `mov rd, rsp ; add rd, 16` for the sp-source case.
    if row.rs == 'sp':
        return amd_mov_rr(row.rd, 'sp') + amd_alu_ri8(0, row.rd, 16)
    return amd_mov_rr(row.rd, row.rs)


def encode_rrr(_arch, row):
    if row.op == 'MUL':
        return amd_rrr_MUL(row.rd, row.ra, row.rb)
    if row.op == 'DIV':
        return amd_rrr_DIV(row.rd, row.ra, row.rb)
    if row.op == 'REM':
        return amd_rrr_REM(row.rd, row.ra, row.rb)
    if row.op == 'SHL':
        return amd_rrr_shift(4, row.rd, row.ra, row.rb)
    if row.op == 'SHR':
        return amd_rrr_shift(5, row.rd, row.ra, row.rb)
    if row.op == 'SAR':
        return amd_rrr_shift(7, row.rd, row.ra, row.rb)
    return amd_rrr_simple(ALU_OPCODE[row.op], row.rd, row.ra, row.rb)


def encode_addi(_arch, row):
    head = amd_mov_rr(row.rd, row.ra)
    if -128 <= row.imm <= 127:
        return head + amd_alu_ri8(0, row.rd, row.imm)
    return head + amd_alu_ri32(0, row.rd, row.imm)


# AND/OR with imm: 83 /ext ib sign-extends imm8 to 64 bits. That works for
# imm in [-128, 127] (and for -1 as a convenient all-ones mask), but breaks
# for positive imms >= 128 -- ANDI with 255 would become AND with
# 0xFFFFFFFFFFFFFFFF. Widen to the imm32 form when imm8 would misencode.
LOGI_EXT = {
    'ANDI': 4,
    'ORI': 1,
}


def encode_logi(_arch, row):
    head = amd_mov_rr(row.rd, row.ra)
    ext = LOGI_EXT[row.op]
    if -128 <= row.imm <= 127:
        return head + amd_alu_ri8(ext, row.rd, row.imm)
    return head + amd_alu_ri32(ext, row.rd, row.imm)


SHIFTI_EXT = {
    'SHLI': 4,
    'SHRI': 5,
    'SARI': 7,
}


def encode_shifti(_arch, row):
    return (amd_mov_rr(row.rd, row.ra)
            + amd_shift_ri8(SHIFTI_EXT[row.op], row.rd, row.imm))


def encode_mem(_arch, row):
    # Portable sp points to the frame-local base; the 16-byte hidden frame
    # header sits at native_rsp+0..15 and is not portable-addressable.
    # Shift sp-relative offsets past the header.
    off = row.off + 16 if row.rn == 'sp' else row.off
    if row.op == 'LD':
        return amd_mem_LD(row.rt, row.rn, off)
    if row.op == 'ST':
        return amd_mem_ST(row.rt, row.rn, off)
    if row.op == 'LB':
        return amd_mem_LB(row.rt, row.rn, off)
    if row.op == 'SB':
        return amd_mem_SB(row.rt, row.rn, off)
    raise ValueError(f'unknown mem op: {row.op}')


def encode_ldarg(_arch, row):
    # Internal callers bypass the +16 sp-base translation: native rsp+8
    # holds the saved caller-sp pointer set up by p1_enter, and the first
    # incoming stack-arg word lives 16 bytes past that.
    return (amd_mem_LD('scratch', 'sp', 8)
            + amd_mem_LD(row.rd, 'scratch', 16 + 8 * row.slot))


def amd_epilogue_prefix():
    # Frame-teardown prefix shared by ERET, TAIL, TAILR. Loads retaddr into
    # scratch (r9), saved caller sp into rax, unwinds rsp, then re-pushes
    # retaddr so a trailing `ret` or `jmp` finds the right top-of-stack
    # layout. (For TAIL/TAILR the trailing op is a jmp, but the retaddr
    # still needs to be back on the stack so the eventual callee `ret`
    # returns to the original caller.)
    return ''.join([
        amd_mem_LD('scratch', 'sp', 0),
        amd_mem_LD('rax', 'sp', 8),
        amd_mov_rr('sp', 'rax'),
        amd_push('scratch'),
    ])


def encode_branch_reg(_arch, row):
    if row.kind == 'BR':
        return amd_jmp_r(row.rs)
    if row.kind == 'CALLR':
        return amd_call_r(row.rs)
    if row.kind == 'TAILR':
        return amd_epilogue_prefix() + amd_jmp_r(row.rs)
    raise ValueError(f'unknown branch-reg kind: {row.kind}')


# Conditional-branch lowering:
#   cmp / test
#   Jcc_inverse +3       -- skip the 3-byte `jmp r15`
#   jmp r15              -- P1 branch-taken path
#
# Invert codes: BEQ->JNE(75), BNE->JE(74), BLT->JGE(7D), BLTU->JAE(73),
# BLTZ->JGE(7D), BEQZ->JNE(75), BNEZ->JE(74). The 0x03 rel8 skips
# `amd_jmp_r(br)` which is 3 bytes (REX.B 41 + FF + E7).
CONDB_INVERT = {
    'BEQ': 0x75,   # JNE
    'BNE': 0x74,   # JE
    'BLT': 0x7D,   # JGE
    'BLTU': 0x73,  # JAE
}

CONDBZ_INVERT = {
    'BEQZ': 0x75,  # JNE
    'BNEZ': 0x74,  # JE
    'BLTZ': 0x7D,  # JGE
}


def encode_condb(_arch, row):
    return (amd_cmp_rr(row.ra, row.rb)
            + byte(CONDB_INVERT[row.op]) + byte(0x03)
            + amd_jmp_r('br'))


def encode_condbz(_arch, row):
    return (amd_test_rr(row.ra, row.ra)
            + byte(CONDBZ_INVERT[row.op]) + byte(0x03)
            + amd_jmp_r('br'))


def encode_enter(arch, row):
    # CALL on amd64 pushed the retaddr, so on entry:
    #   rsp = caller_sp - 8
    #   [rsp] = retaddr
    #
    # Standard frame after ENTER:
    #   [sp + 0]                  = retaddr
    #   [sp + 8]                  = saved caller_sp
    #   [sp + 16 .. 16 + size - 1] = portable locals
    #   total frame = round_up(stack_align, 16 + size)
    frame_bytes = round_up(arch.stack_align, 2 * arch.word_bytes + row.size)
    return ''.join([
        amd_pop('scratch'),
        amd_mov_rr('rax', 'sp'),
        amd_alu_ri32(5, 'sp', frame_bytes),
        amd_mem_ST('scratch', 'sp', 0),
        amd_mem_ST('rax', 'sp', 8),
    ])


def encode_nullary(_arch, row):
    if row.kind == 'B':
        return amd_jmp_r('br')
    if row.kind == 'CALL':
        return amd_call_r('br')
    if row.kind == 'RET':
        return amd_ret()
    if row.kind == 'ERET':
        return amd_epilogue_prefix() + amd_ret()
    if row.kind == 'TAIL':
        return amd_epilogue_prefix() + amd_jmp_r('br')
    if row.kind == 'SYSCALL':
        # P1: a0=num, a1..a3,t0,s0,s1 = args 0..5. Linux amd64: rax=num,
        # rdi/rsi/rdx/r10/r8/r9 = args 0..5, return in rax; syscall also
        # clobbers rcx and r11.
        #
        # Push the P1 registers whose native slots get overwritten or
        # syscall-clobbered -- rsi (a1), rdx (a2), rcx (a3), r11 (t1),
        # r8 (t2) -- then shuffle into the native arg slots, issue
        # syscall, restore, and move the return value (rax) into a0
        # (rdi). Stack offsets after the 5 pushes: [rsp+0]=r8,
        # [rsp+8]=r11, [rsp+16]=rcx (a3), [rsp+24]=rdx (a2),
        # [rsp+32]=rsi (a1).
        return ''.join([
            amd_push('rsi'),
            amd_push('rdx'),
            amd_push('rcx'),
            amd_push('r11'),
            amd_push('r8'),
            amd_mov_rr('rax', 'rdi'),
            amd_mem_LD('rdi', 'sp', 32),
            amd_mem_LD('rsi', 'sp', 24),
            amd_mem_LD('rdx', 'sp', 16),
            amd_mov_rr('r8', 'rbx'),
            amd_mov_rr('r9', 'r12'),
            amd_syscall(),
            amd_pop('r8'),
            amd_pop('r11'),
            amd_pop('rcx'),
            amd_pop('rdx'),
            amd_pop('rsi'),
            amd_mov_rr('rdi', 'rax'),
        ])
    raise ValueError(f'unknown nullary kind: {row.kind}')


def amd_start_stub():
    # Backend-owned :_start stub per docs/P1.md §Program Entry. Linux amd64
    # puts argc at [rsp] and argv starting at [rsp+8]. Load argc into a0
    # (rdi), compute &argv[0] into a1 (rsi), call p1_main under the
    # one-word direct-result convention, then issue sys_exit with
    # p1_main's return value in a0 (== rdi). Mirrors the `%p1_entry`
    # macro in p1/P1-amd64.M1pp.
    #
    # Raw hex outside DEFINE bodies must be single-quoted so bootstrap M0
    # treats it as a literal byte run. The bootstrap amd64 M0 has a 256B
    # token buffer, so each quoted run must stay <= 128 hex chars; we
    # split into multiple short lines defensively.
    def q(hex_bytes):
        return f"'{hex_bytes}'"

    load_argc = amd_mem_LD('a0', 'sp', 0)
    compute_argv = amd_mov_rr('a1', 'sp') + amd_alu_ri8(0, 'a1', 8)
    labr_prefix = amd_mov_imm32_prefix('br')
    call_main = amd_call_r('br')
    # mov eax, 60 ; syscall. P1 a0 (rdi) already holds p1_main's return.
    sys_exit = byte(0xB8) + le32(60) + amd_syscall()

    return [
        ':_start',
        q(load_argc),
        q(compute_argv),
        q(labr_prefix),
        '&p1_main',
        q(call_main),
        q(sys_exit),
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
    name='amd64',
    word_bytes=8,
    stack_align=16,
    syscall_numbers=SYSCALL_NUMBERS,
    encoders=ENCODERS,
    start_stub=amd_start_stub,
)
