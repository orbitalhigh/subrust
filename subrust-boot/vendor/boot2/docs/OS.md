# Minimal OS contract

The boot2 bootstrap depends on a small, well-bounded set of OS
capabilities. This document specifies that contract so a minimal OS
kernel can be implemented (and verified) against it. The rest of the
chain — `M0`, `hex2`, `cc.scm`, `tcc-boot2`, libc — assumes nothing
beyond what's listed here.

The "shell" here is scheme1 evaluating a driver `.scm` over the
process-management and file-I/O layer in
[`scheme1/prelude.scm`](../scheme1/prelude.scm) (see lines 493–696).
It's not a POSIX shell: it spawns and waits, opens files, reads, and
writes. It does **not** pipe, redirect, or `cd`. Bootstrap steps
compose through files (`catm`-style), not pipelines.

Two tiers:

- **Tier 1 — toolchain.** Enough to run `cc.scm` and `tcc-boot2` and
  to compile/link a static ELF. Eight syscalls.
- **Tier 2 — driver.** Adds spawn-and-wait so a scheme1 driver can
  invoke tcc-boot2 (and other compiled binaries) as subprocesses.
  Three more syscalls.

Anything past Tier 2 (threads, signals beyond default-action, mmap,
dynamic linking, sockets, timers, locale, IEEE-754 math, pipes,
redirection, working-directory state) is out of scope. See
[§Out of scope](#out-of-scope) for the explicit non-list.

## Targets

Three architectures, identical contract. P1-64 only (LP64).

| arch    | platform        | syscall instr    | arg regs                    | nr reg | ret reg |
|---------|-----------------|------------------|-----------------------------|--------|---------|
| amd64   | Linux x86-64    | `syscall`        | rdi rsi rdx r10 r8 r9       | rax    | rax     |
| aarch64 | Linux ARM64     | `svc #0`         | x0 x1 x2 x3 x4 x5           | x8     | x0      |
| riscv64 | Linux RISC-V 64 | `ecall`          | a0 a1 a2 a3 a4 a5           | a7     | a0      |

These are the native Linux ABIs; the per-arch shims in
`P1/P1-{aarch64,amd64,riscv64}.M1pp` (`%macro p1_syscall`, lines
~520–930) marshal P1 registers into them. Any kernel that implements
these three ABIs verbatim can host the chain.

## Platform layers

A compliant platform owes the chain four things:

1. **ISA execution** — a CPU (or emulator) that runs the target
   user-mode instruction stream `M0`/`hex2` emit.
2. **Image loader** — reads a static ELF, maps `PT_LOAD` segments,
   lays out the initial stack, transfers control to `e_entry`.
3. **Address space and syscall trap** — a per-process virtual memory
   with a movable program break, plus a trap handler that decodes the
   per-arch syscall ABI from §Targets and dispatches.
4. **Syscall implementations** — the 8 Tier-1 / +3 Tier-2 behaviors,
   backed by a byte-addressable persistent store for the file-related
   ones.

The remaining sections specify each layer. "Implementing the
contract" means all four; readers chasing only the syscall tables
will miss layers 1–3.

## Layer 1 — ISA execution

The chain emits **integer-only, user-mode** code for the chosen arch:

- Integer arithmetic, load/store, branches/calls.
- The syscall trap instruction from §Targets.
- **No FPU.** `HAVE_FLOAT` is off through libc; `cc.scm` rejects
  `0.0` literals. The kernel needs no FP save/restore beyond what
  the platform demands (single-process here, so moot).
- **No SIMD, no atomics.** Single-threaded; no shared memory.
- **One arch per image.** No multi-arch fat ELFs.

A platform that can run static integer-only Linux user binaries on
the named arch already satisfies this layer.

## Layer 2 — Image loader

### ELF format

- **ET_EXEC, static.** No `PT_INTERP`, no dynamic linker. tcc-boot2's
  output and every host artefact are statically linked.
- **`PT_LOAD` segments only.** Permissions from `p_flags` (R/W/X bits).
  No `PT_GNU_STACK`, no `PT_NOTE` parsing, no `PT_TLS`.
- **Entry at `e_entry`.** No `_start` indirection required from the
  kernel; the loader's job is to transfer control to `e_entry` with
  the stack laid out below and to return execution to userspace.

The `ELF.hex2` file in this repo emits exactly this shape (one
`PT_LOAD`, `e_entry` set, no PHDR self-reference).

### Initial stack

Standard Linux SysV layout. The kernel must place at the initial
stack pointer, low to high:

```
sp + 0           argc                       (word)
sp + 8           argv[0]                    (pointer)
                 ...
                 argv[argc-1]
                 NULL                       (argv terminator)
                 envp[0]
                 ...
                 NULL                       (envp terminator)
                 [argv/envp string bytes follow, anywhere in image]
```

`__libc_init` (`vendor/mes-libc/boot2-syscall.c`) walks past argv's
NULL to find `environ`. **auxv is not required** — nothing in the
chain reads it.

## Layer 3 — Address space and syscall trap

### Memory model

- **One contiguous heap, grown via `brk`.** The kernel exposes a
  per-process program break; `sys_brk(0)` returns it, `sys_brk(addr)`
  sets it (POSIX/Linux semantics). `linux/malloc.c` is a free-list
  allocator on top — no `mmap` required.
- **No shared memory, no per-thread state.** Single-threaded
  processes only.
- **Pages must be readable/writable/executable as their `p_flags`
  request.** No W^X enforcement complications: tcc-boot2 doesn't JIT;
  every page is either RX (text) or RW (data/bss/stack/heap).

### Syscall ABI

Trap instruction, argument registers, syscall-number register, and
return register are listed per arch in §Targets. Syscall numbers
default to the standard Linux-on-`uname-m` values used by the per-arch
P1 macros (e.g. `read=63` on aarch64, `read=0` on amd64). A
fresh-write OS may renumber, but only at the cost of also rewriting
the per-arch `p1_sys_*` macros in `P1/P1-{aarch64,amd64,riscv64}.M1pp`.

Error returns follow the standard Linux convention: a non-negative
result on success or a negative errno value in the return register.
See [§Error convention](#error-convention).

## Layer 4 — Syscalls

### Tier 1 — toolchain (8 calls)

Wired in `P1/P1pp.P1pp:986-1055`.

| name      | linux nr (aa64 / amd64 / riscv64) | semantics                                            |
|-----------|-----------------------------------|------------------------------------------------------|
| read      | 63 / 0   / 63                     | `ssize_t read(fd, buf, len)`                         |
| write     | 64 / 1   / 64                     | `ssize_t write(fd, buf, len)`                        |
| openat    | 56 / 257 / 56                     | called as `openat(AT_FDCWD=-100, path, flags, mode)` |
| close     | 57 / 3   / 57                     | `int close(fd)`                                      |
| lseek     | 62 / 8   / 62                     | `off_t lseek(fd, off, whence)`                       |
| brk       | 214 / 12 / 214                    | `void *brk(addr)`; `addr=0` returns current break    |
| unlinkat  | 35 / 263 / 35                     | called as `unlinkat(AT_FDCWD=-100, path, 0)`         |
| exit_group| 93 / 60  / 93                     | `void exit(status)`; never returns                   |

Everything in `docs/LIBC.txt`'s "syscall-using" column reduces to
exactly these eight (`fopen → openat`, `fseek → lseek`, `malloc/
realloc/free → brk`, `__assert_fail / abort / exit → exit_group`,
etc.).

#### Filesystem semantics

A flat, byte-addressable file abstraction with POSIX read/write
semantics:

- Regular files have a length and an in-file byte offset per fd.
- `O_RDONLY | O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND` flags
  honored; no `O_NONBLOCK`, no `O_DIRECT`.
- Mode bits on `openat(O_CREAT)`: only the user-rwx bits need
  honoring; group/other and setuid bits can be ignored.
- `lseek` whences: `SEEK_SET=0`, `SEEK_CUR=1`, `SEEK_END=2`.
- `unlinkat(AT_FDCWD, path, 0)` removes a regular file.

Not required: `stat`, `fstat`, directory iteration, symlinks, hard
links, file modes beyond a usable subset, mtime, ownership. A
hierarchical filesystem in any rich sense is not required either —
flat directory plus `/` separators is enough; tcc-boot2 reads files
by literal path strings the build emits.

The chain opens 3 fd kinds: source files (read), output files
(write+create+trunc), and the inherited stdin/stdout/stderr (0/1/2).
No pipes are used at any tier.

#### Termination

- **`exit_group`.** Exit status is the low byte of the argument. No
  `atexit`, no destructors.
- **No signal-handler installation required.** Default actions
  (SIGSEGV → terminate, SIGPIPE → terminate, etc.) are sufficient.
  The chain installs zero handlers; `boot2-syscall.c` stubs `raise`
  to ENOSYS.

### Tier 2 — driver (+3 calls)

Per-arch macros already exist in `P1/P1-*.M1pp`. The scheme1 prelude's
`spawn` / `run` / `wait` / `exit` are built directly on these
(`scheme1/prelude.scm:520-537`).

| name    | linux nr (aa64 / amd64 / riscv64) | driver role                               |
|---------|-----------------------------------|-------------------------------------------|
| clone   | 220 / 56  / 220                   | spawn child; called bare (no flags arg in the prelude — kernel must accept clone-as-fork with SIGCHLD) |
| execve  | 221 / 59  / 221                   | image swap; takes `(prog, argv)` — no envp arg in the prelude wrapper, so the kernel-side execve must accept a NULL/empty envp without erroring |
| waitid  | 95  / 247 / 95                    | reap child; called as `waitid(P_PID=1, pid, info, WEXITED=4)` — info[8]=si_code, info[24]=si_status (`scheme1/prelude.scm:497-506`) |

#### Process lifecycle

- **Image swap via `execve`.** Replaces the calling process's memory
  map; on success, control returns at the new image's `e_entry`.
- **Spawn via `clone`** with `fork()` semantics: new address space
  (no `CLONE_VM`), new fd table, parent/child return distinguished by
  return value (0 in child, child-pid in parent). The scheme1 prelude
  calls `(sys-clone)` with no arguments — the P1pp wrapper supplies
  `SIGCHLD` as the only flag. The `fork()` syscall itself is not
  required.
- **Reap via `waitid`.** Only `WEXITED` (=4) is used. Job control
  flags are not needed.

Notably **not** required at Tier 2:

- `dup3` / `dup2`, `pipe` / `pipe2` — no fd plumbing between
  processes. Children inherit stdin/stdout/stderr (0/1/2) from the
  parent and that's the entire fd contract.
- `chdir`, `getcwd` — no working-directory manipulation. All paths
  the driver passes to children are absolute or relative to the
  starting cwd.
- `getpid`, `getppid`, `setpgid`, `tcsetpgrp` — no job control.

If a future driver needs redirection (say, capturing tcc-boot2's
stderr into a file), the right move is to grow the prelude to use
`dup3` and add the syscall here; until then it's not in the contract.

### Error convention

- Every syscall returns either a non-negative result or a negative
  errno value in the return register. No errno TLS variable in the
  kernel/userspace contract — the value lives in the return register.
  The libc errno layer (`vendor/mes-libc/boot2-syscall.c`) negates
  and stores into a single global `errno` int.
- Errno numbers: standard Linux constants (`EBADF=9`, `ENOENT=2`,
  `EFAULT=14`, …). The libc layer maps them through `strerror` lookup
  tables vendored from mes.

## Out of scope

Explicitly **not** required by the chain. Trying to implement these
adds complexity without enabling any chain step:

- **Threading.** `clone` with `CLONE_VM`/`CLONE_THREAD`, futexes,
  TLS. The chain is single-threaded; `errno` is one int global.
- **mmap / munmap / mprotect.** `linux/malloc.c` is brk-only.
  Anonymous and file mmap are unused.
- **Signals beyond default-action.** No `rt_sigaction`,
  `rt_sigprocmask`, `rt_sigreturn`. Default termination on SIGSEGV/
  SIGPIPE/etc. is sufficient.
- **Dynamic linking.** No `PT_INTERP`, no `ld.so`. All binaries
  static.
- **IEEE-754 math.** `HAVE_FLOAT` is off through the entire libc;
  `0.0` literals are even rejected by cc.scm. The kernel needs no
  FPU save/restore beyond what the platform demands at context switch
  (and we're single-process anyway, so that's moot).
- **Sockets, IPC primitives beyond pipes, timers, RNG, /proc, /sys,
  ptrace, namespaces, cgroups.**
- **Filesystem features:** stat-family, directory listing, symlinks,
  hard links, mode/owner semantics beyond user-rwx, mtime,
  cross-device rename.
- **auxv at process entry.** Not consumed.
- **Locale, wide chars, IDN, Unicode normalization.** Bytes are
  bytes.

## Verification

A minimal-OS implementation is compliant when:

1. **Tier 1 acceptance:** `make tcc-boot2 ARCH=<a>` runs to
   completion on it (parses + assembles + links via the chain),
   and `make test SUITE=cc-libc ARCH=<a>` passes.
2. **Tier 2 acceptance:** a scheme1 driver (scheme1 binary + a
   `.scm` over `prelude.scm`'s `spawn`/`run`/`wait` and file-port
   layer) can invoke `tcc-boot2` on a `.c` source, wait for it to
   exit, and read the resulting ELF back from disk.

Both acceptance suites run end-to-end in the boot2 tree; an OS
reaching Tier 2 needs no boot2-side changes.
