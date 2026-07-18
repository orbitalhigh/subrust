    # f_amd64.s — the per-arch (amd64) assembly backend for the BOOT_API
    # IEEE-f64 intrinsics that the sr0i C prototype punts on
    # (die("f_* intrinsics need the assembly backend")).
    #
    # Protocol (same getb/putb byte discipline as the rest of the seed chain):
    #   stdin  = stream of 17-byte records: op:u8, a:u64-LE, b:u64-LE
    #   stdout = 8 bytes per record: result:u64-LE
    # EOF at a record boundary -> exit 0.
    #
    # ops: 0 f_add  1 f_sub  2 f_mul  3 f_div  4 f_rem
    #      5 f_lt   6 f_eq   7 f_from_i  8 f_to_i
    # f_lt/f_eq return 0/1; f_from_i reads a as i64; f_to_i is Rust-saturating.
    #
    # Position-independent: no static data, stack frame only, relative jumps —
    # so the .text bytes can later be wrapped verbatim by the seed ELF header.
    .text
    .globl _start
_start:
    subq $48, %rsp
    movq %rsp, %rbp              # [rbp,rbp+17) record ; [rbp+24,rbp+32) scratch/out

.Lloop:
    xorq %r13, %r13             # got = 0
.Lfill:
    xorl %eax, %eax             # SYS_read
    xorl %edi, %edi             # fd 0
    leaq (%rbp,%r13,1), %rsi
    movq $17, %rdx
    subq %r13, %rdx
    syscall
    testq %rax, %rax
    jle .Lexit                  # <=0 -> EOF/err (partial record => stop)
    addq %rax, %r13
    cmpq $17, %r13
    jb .Lfill

    movzbl 0(%rbp), %eax        # op
    movq 1(%rbp), %xmm0         # a as f64 bits
    movq 9(%rbp), %xmm1         # b as f64 bits
    movq 1(%rbp), %rcx          # a as i64 (for f_from_i)

    cmpl $0, %eax
    je .Ladd
    cmpl $1, %eax
    je .Lsub
    cmpl $2, %eax
    je .Lmul
    cmpl $3, %eax
    je .Ldiv
    cmpl $4, %eax
    je .Lrem
    cmpl $5, %eax
    je .Llt
    cmpl $6, %eax
    je .Leq
    cmpl $7, %eax
    je .Lfromi
    cmpl $8, %eax
    je .Ltoi
    xorq %rax, %rax             # unknown op -> 0
    jmp .Lstore

.Ladd:
    addsd %xmm1, %xmm0
    movq %xmm0, %rax
    jmp .Lstore
.Lsub:
    subsd %xmm1, %xmm0
    movq %xmm0, %rax
    jmp .Lstore
.Lmul:
    mulsd %xmm1, %xmm0
    movq %xmm0, %rax
    jmp .Lstore
.Ldiv:
    divsd %xmm1, %xmm0
    movq %xmm0, %rax
    jmp .Lstore
.Lrem:                          # Rust f64 % == C fmod: remainder, sign of dividend
    movsd %xmm1, 24(%rbp)
    fldl 24(%rbp)              # st0 = b
    movsd %xmm0, 24(%rbp)
    fldl 24(%rbp)              # st0 = a, st1 = b
.Lrem1:
    fprem
    fnstsw %ax
    testb $0x04, %ah           # C2 (bit 10) set => reduction incomplete, loop
    jnz .Lrem1
    fstpl 24(%rbp)            # store a%b, pop
    fstp %st(0)              # pop b
    movq 24(%rbp), %rax
    jmp .Lstore
.Llt:
    xorl %eax, %eax
    comisd %xmm1, %xmm0        # compare a(xmm0) with b(xmm1)
    jp .Lstore                # unordered (NaN) -> 0
    setb %al                  # CF=1 => a<b
    jmp .Lstore
.Leq:
    xorl %eax, %eax
    ucomisd %xmm1, %xmm0
    jp .Lstore                # NaN -> not equal
    sete %al                  # ZF=1 => a==b (also +0==-0)
    jmp .Lstore
.Lfromi:
    cvtsi2sdq %rcx, %xmm0      # signed i64 -> f64
    movq %xmm0, %rax
    jmp .Lstore
.Ltoi:                          # Rust `f64 as i64` : truncate then SATURATE
    cvttsd2si %xmm0, %rax
    movabsq $0x8000000000000000, %rcx
    cmpq %rcx, %rax
    jne .Lstore               # in-range result
    ucomisd %xmm0, %xmm0
    jp .Ltoi_zero             # NaN -> 0
    movq %xmm0, %rdx
    testq %rdx, %rdx
    js .Lstore                # negative (or -inf, < -2^63) -> i64::MIN (rax already)
    movabsq $0x7FFFFFFFFFFFFFFF, %rax  # positive overflow / +inf -> i64::MAX
    jmp .Lstore
.Ltoi_zero:
    xorq %rax, %rax
    jmp .Lstore

.Lstore:
    movq %rax, 24(%rbp)
    xorq %r13, %r13
.Lwr:
    movl $1, %eax              # SYS_write
    movl $1, %edi              # fd 1
    leaq 24(%rbp,%r13,1), %rsi
    movq $8, %rdx
    subq %r13, %rdx
    syscall
    testq %rax, %rax
    jle .Lexit
    addq %rax, %r13
    cmpq $8, %r13
    jb .Lwr
    jmp .Lloop

.Lexit:
    movl $60, %eax             # SYS_exit
    xorl %edi, %edi
    syscall
