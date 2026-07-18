# boot2 — Make-driven bootstrap pipeline.
#
# The bootN.sh scripts under boot/ are the canonical builders. This
# Makefile is dependency tracking: every target is a real output path,
# and the rule body invokes the right bootN.sh (or prep-src.sh /
# prep-musl.sh) script with the appropriate ARCH and DRIVER.
#
# Output layout: build/<arch>/<driver>/boot{0..6}/<artifacts>.
#   <arch>   ∈ {aarch64, amd64, riscv64}
#   <driver> ∈ {podman, seed}
#
# Path-based builds:
#   make build/aarch64/podman/boot6/Image
#   make build/amd64/seed/boot6/kernel.elf
#   make build/riscv64/podman/boot1/M1pp        # only prep-src + boot0 + boot1
#
# Convenience entrypoints (default ARCH=aarch64, DRIVER=podman):
#   make all                  build boot6 kernel for ARCH × DRIVER
#   make test SUITE=<name>    run a test suite (see SUITE list at bottom)
#   make image                build the per-arch container image used by tests
#   make cloc                 line counts for the bootstrap sources
#   make clean                rm -rf build/
#
# DRIVER=seed bootN stages depend on the podman-built boot6 kernel
# (build/<arch>/podman/boot6/<kernel-name>) since the seed driver runs
# scheme1 inside seed-kernel under QEMU. The seed-side dependency on
# the podman side is encoded in the rules.

# ── Config ───────────────────────────────────────────────────────────────

ARCH    ?= aarch64
DRIVER  ?= podman

ALL_ARCHES  := aarch64 amd64 riscv64
ALL_DRIVERS := podman seed

ifeq ($(filter $(ARCH),$(ALL_ARCHES)),)
  $(error ARCH '$(ARCH)' not supported — use one of $(ALL_ARCHES))
endif
ifeq ($(filter $(DRIVER),$(ALL_DRIVERS)),)
  $(error DRIVER '$(DRIVER)' not supported — use one of $(ALL_DRIVERS))
endif

# Per-arch metadata mirrored from boot/lib-arch.sh.
PLATFORM_aarch64    := linux/arm64
PLATFORM_amd64      := linux/amd64
PLATFORM_riscv64    := linux/riscv64

KERNEL_NAME_aarch64 := Image
KERNEL_NAME_amd64   := kernel.elf
KERNEL_NAME_riscv64 := kernel.elf

MUSL_ARCH_aarch64   := aarch64
MUSL_ARCH_amd64     := x86_64
MUSL_ARCH_riscv64   := riscv64

TCC_PKG := tcc-0.9.26-1147-gee75a10c

OUT_DIR := build/$(ARCH)/$(DRIVER)

.SUFFIXES:

.PHONY: all help clean cloc src package release

# ── Top-level targets ────────────────────────────────────────────────────

all: build/$(ARCH)/$(DRIVER)/boot6/$(KERNEL_NAME_$(ARCH))

# Prepare the canonical source tree (prep-src + prep-musl) for ARCH.
src: build/$(ARCH)/src/musl/.stamp

help:
	@echo 'Targets (default ARCH=$(ARCH) DRIVER=$(DRIVER)):'
	@echo '  make all                                 build boot6 kernel'
	@echo '  make src                                 prep canonical src/ tree (incl. musl)'
	@echo '  make package                             quick: package boot2-<arch>-<rev>.tar.gz from current build'
	@echo '  make release                             validated: clean-rebuild x2 + verify, mint to dist/'
	@echo '  make build/<arch>/<driver>/boot6/<kn>    full chain (kn = Image | kernel.elf)'
	@echo '  make build/<arch>/<driver>/bootN/<file>  any single artifact'
	@echo '  make test SUITE=<suite>                  test suite (NAMES=<filter> optional)'
	@echo '  make image                               per-arch tests container image'
	@echo '  make cloc                                line counts for bootstrap sources'
	@echo '  make clean                               rm -rf build/'

clean:
	rm -rf build/

# `package`: package a per-arch tarball from the current build tree.
# Depends on the full chain (boot5 musl + boot6 kernel) so
# OUTPUT_MANIFEST.txt has real artifacts to hash. boot5 is not on the
# `all` dep path (boot6 doesn't link musl), so list it explicitly here.
# Lands at build/<arch>/release/boot2-<arch>-<rev>.tar.gz. Fast: no
# reproducibility or verify check.
package: build/$(ARCH)/$(DRIVER)/boot5/libc.a \
         build/$(ARCH)/$(DRIVER)/boot6/$(KERNEL_NAME_$(ARCH))
	DRIVER=$(DRIVER) tools/mkrelease.sh $(ARCH)

# `release`: the validated path. Cleans, rebuilds + packages twice,
# asserts both tarballs hash-match, extracts one and runs its
# verify.sh, then promotes to dist/. Slow but paranoid; this is the
# only way a tarball lands in dist/.
release:
	DRIVER=$(DRIVER) tools/release.sh $(ARCH)

# ── prep-src + boot0..boot6 chain (rules per arch × driver) ──────────────
#
# The .stamp files are the make-rule pegs. Each rule lists its real
# outputs as additional targets so single-binary builds work — `make
# build/aarch64/podman/boot1/M1pp` resolves to the boot1 grouped target
# and only walks back through prep-src + boot0.
#
# Per-arch source deps for prep-src.sh. The file list mirrors what
# prep-src.sh actually copies, so a vendor/seed change re-triggers the
# canonical-tree build.

PREP_SRC_COMMON_SRCS := \
    bootprep/prep-src.sh boot/lib-arch.sh \
    bootprep/stage1-flatten.sh bootprep/libc-flatten.sh \
    bootprep/boot4-gen-runscm.sh bootprep/boot6-gen-runscm.sh \
    bootprep/assets/boot3-run.scm bootprep/assets/boot-hello.c \
    M1pp/M1pp.P1 hex2pp/hex2pp.P1 \
    P1/P1.M1pp P1/P1pp.P1pp \
    P1/entry-libc.P1pp P1/entry-plain.P1pp P1/elf-end.P1pp \
    catm/catm.P1pp \
    scheme1/scheme1.P1pp scheme1/prelude.scm \
    cc/cc.scm cc/main.scm \
    tcc/cc/mem.c \
    seed-kernel/kernel.c \
    vendor/musl/1.2.5.tar.gz \
    vendor/musl/deletes.txt

prep_src_arch_srcs = \
    vendor/seed/$1/hex0-seed \
    vendor/seed/$1/ELF.hex2 \
    vendor/seed/$1/hex0.hex0 vendor/seed/$1/hex1.hex0 vendor/seed/$1/hex2.hex1 \
    vendor/seed/$1/catm.hex2 vendor/seed/$1/M0.hex2 \
    P1/P1-$1.M1 P1/P1-$1.M1pp \
    tcc/libc/$1/start.S tcc/libc/$1/sys_stubs.S \
    $(wildcard seed-kernel/arch/$1/*) \
    $(wildcard seed-kernel/user/*) \
    vendor/musl/generated/$(MUSL_ARCH_$1)/alltypes.h \
    vendor/musl/generated/$(MUSL_ARCH_$1)/syscall.h \
    $(wildcard vendor/musl/skip-$1.txt) \
    $(shell find vendor/musl/overrides -type f 2>/dev/null) \
    $(shell find vendor/mes-libc -type f \( -name '*.c' -o -name '*.h' \) 2>/dev/null) \
    $(wildcard vendor/mes-libc/patches/*.before) \
    $(wildcard vendor/mes-libc/patches/*.after) \
    $(wildcard vendor/tcc/0.9.26.tar.gz)

# DRIVER=seed bootN stages run scheme1 under QEMU using the podman-built
# boot6 kernel. Add that as a make dep so a clean-tree seed build pulls
# in the podman side automatically. Empty for DRIVER=podman.
seed_kernel_dep = $(if $(filter seed,$2),build/$1/podman/boot6/$(KERNEL_NAME_$1))

# Per-arch prep-src rule. Driver-independent.
#
# prep-src.sh produces the full canonical tree, including the filtered
# musl/ subtree (unpack + overrides + deletes + per-arch skip list,
# plus the generated boot5 enumerate / run.scm). musl/.stamp is a
# secondary peg that boot5 depends on — it's just touched after
# src/.stamp, since the musl tree is populated by the same recipe.
define PREP_RULES
build/$1/src/.stamp: $$(PREP_SRC_COMMON_SRCS) $$(call prep_src_arch_srcs,$1)
	bootprep/prep-src.sh $1
	@touch $$@

build/$1/src/musl/.stamp: build/$1/src/.stamp
	@mkdir -p $$(@D) && touch $$@
endef

$(foreach a,$(ALL_ARCHES),$(eval $(call PREP_RULES,$a)))

# Per-(arch, driver) boot0..boot6 rules.
#
# Each stage's .stamp is the recipe peg; each real artifact is declared
# as a target depending on the .stamp with an empty recipe (`;`). We
# avoid grouped targets (`&:`) — GNU make 4.4 does not propagate "must
# remake of prereq" through grouped targets across multi-level chains,
# so a path-based request like `make .../boot6/Image` would skip the
# boot6 rebuild after a deeper change. Single-target rules with stamp-
# anchored declarations propagate correctly.
define BOOT_CHAIN_RULES
# boot0: hex0-seed -> hex2 / M0 / catm
build/$1/$2/boot0/.stamp: \
        build/$1/src/.stamp \
        boot/boot0.sh boot/lib-arch.sh boot/lib-pipeline.sh \
        $$(call seed_kernel_dep,$1,$2)
	DRIVER=$2 boot/boot0.sh $1
	@touch $$@

build/$1/$2/boot0/hex2 build/$1/$2/boot0/M0 build/$1/$2/boot0/catm: \
    build/$1/$2/boot0/.stamp ;

# boot1: M1pp.P1 + hex2pp.P1 -> M1pp + hex2pp
build/$1/$2/boot1/.stamp: \
        build/$1/$2/boot0/.stamp \
        boot/boot1.sh boot/lib-arch.sh boot/lib-pipeline.sh \
        $$(call seed_kernel_dep,$1,$2)
	DRIVER=$2 boot/boot1.sh $1
	@touch $$@

build/$1/$2/boot1/M1pp build/$1/$2/boot1/hex2pp: build/$1/$2/boot1/.stamp ;

# boot2: catm.P1pp + scheme1.P1pp -> catm + scheme1
build/$1/$2/boot2/.stamp: \
        build/$1/$2/boot1/.stamp build/$1/$2/boot0/.stamp \
        boot/boot2.sh boot/lib-arch.sh boot/lib-pipeline.sh \
        $$(call seed_kernel_dep,$1,$2)
	DRIVER=$2 boot/boot2.sh $1
	@touch $$@

build/$1/$2/boot2/catm build/$1/$2/boot2/scheme1: build/$1/$2/boot2/.stamp ;

# boot3: cc.scm-built bootstrap tcc (tcc0)
build/$1/$2/boot3/.stamp: \
        build/$1/$2/boot2/.stamp build/$1/$2/boot1/.stamp \
        boot/boot3.sh boot/lib-arch.sh boot/lib-runscm.sh \
        $$(call seed_kernel_dep,$1,$2)
	DRIVER=$2 boot/boot3.sh $1
	@touch $$@

build/$1/$2/boot3/tcc0 build/$1/$2/boot3/libc.P1pp \
build/$1/$2/boot3/tcc.flat.P1pp: build/$1/$2/boot3/.stamp ;

# boot4: tcc0 -> tcc1 -> tcc2 -> tcc3 self-host chain (+ libc.a, libtcc1.a, hello)
build/$1/$2/boot4/.stamp: \
        build/$1/$2/boot3/.stamp build/$1/$2/boot2/.stamp \
        boot/boot4.sh boot/lib-arch.sh boot/lib-runscm.sh \
        $$(call seed_kernel_dep,$1,$2)
	DRIVER=$2 boot/boot4.sh $1
	@touch $$@

build/$1/$2/boot4/tcc1 build/$1/$2/boot4/tcc2 \
build/$1/$2/boot4/tcc3 build/$1/$2/boot4/hello \
build/$1/$2/boot4/crt1.o build/$1/$2/boot4/libc.a \
build/$1/$2/boot4/libtcc1.a: build/$1/$2/boot4/.stamp ;

# boot5: musl libc + hello (consumes prep-musl output)
build/$1/$2/boot5/.stamp: \
        build/$1/$2/boot4/.stamp build/$1/$2/boot2/.stamp \
        build/$1/src/musl/.stamp \
        boot/boot5.sh boot/lib-arch.sh boot/lib-runscm.sh \
        $$(call seed_kernel_dep,$1,$2)
	DRIVER=$2 boot/boot5.sh $1
	@touch $$@

build/$1/$2/boot5/libc.a build/$1/$2/boot5/crt1.o \
build/$1/$2/boot5/crti.o build/$1/$2/boot5/crtn.o \
build/$1/$2/boot5/hello: build/$1/$2/boot5/.stamp ;

# boot6: seed-kernel ELF/Image, built with boot4's tcc3
build/$1/$2/boot6/.stamp: \
        build/$1/$2/boot4/.stamp build/$1/$2/boot2/.stamp \
        boot/boot6.sh boot/lib-arch.sh boot/lib-runscm.sh \
        $$(call seed_kernel_dep,$1,$2)
	DRIVER=$2 boot/boot6.sh $1
	@touch $$@

build/$1/$2/boot6/$$(KERNEL_NAME_$1): build/$1/$2/boot6/.stamp ;
endef

$(foreach a,$(ALL_ARCHES),$(foreach d,$(ALL_DRIVERS),$(eval $(call BOOT_CHAIN_RULES,$a,$d))))

# ── Top-level catm'd cc bundle (arch-independent) ────────────────────────
#
# scheme1 reads cc as one file: prelude + cc + main concatenated. The
# output is arch-independent, so we keep a single repo-level copy that
# every test invocation reuses regardless of arch. Plain `cat` is byte-
# equivalent to running the bootN catm on these inputs.

build/cc.scm: scheme1/prelude.scm cc/cc.scm cc/main.scm
	mkdir -p $(@D)
	cat $^ > $@

# ── cloc ─────────────────────────────────────────────────────────────────
#
# Default covers every arch; `make cloc ARCH=<a>` narrows the per-arch
# pieces (P1-<a>.M1pp/.M1, vendor/seed/<a>/*) to one arch.

CLOC_SEED_BASES := hex0.hex0 hex1.hex0 hex2.hex1 catm.hex2 M0.hex2 ELF.hex2

ifeq ($(origin ARCH),file)
  CLOC_ARCHES := $(ALL_ARCHES)
else
  CLOC_ARCHES := $(ARCH)
endif

CLOC_FILES := \
    $(foreach a,$(CLOC_ARCHES),$(foreach f,$(CLOC_SEED_BASES),vendor/seed/$(a)/$(f))) \
    $(foreach a,$(CLOC_ARCHES),P1/P1-$(a).M1) \
    M1pp/M1pp.P1 \
    hex2pp/hex2pp.P1 \
    $(foreach a,$(CLOC_ARCHES),vendor/seed/$(a)/ELF.hex2) \
    $(foreach a,$(CLOC_ARCHES),P1/P1-$(a).M1pp) \
    P1/P1.M1pp \
    P1/P1pp.P1pp \
    scheme1/scheme1.P1pp \
    scheme1/prelude.scm \
    cc/cc.scm

cloc:
	@sh tools/count-lines.sh $(CLOC_FILES)

# ── Test infrastructure (suites + their build deps) ──────────────────────
# tests/Makefile owns the test-build rules and the `test` / `image` /
# `tables` targets. ARCH/DRIVER and per-arch metadata above are visible
# to the include. REPO_ROOT signals to tests/Makefile that it is being
# included from the top-level (rather than invoked standalone via
# `make -C tests`).
REPO_ROOT := $(CURDIR)
include tests/Makefile
