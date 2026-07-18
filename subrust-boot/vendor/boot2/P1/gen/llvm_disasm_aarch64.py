#!/usr/bin/env python3
"""Annotate p1_aarch64.M1 DEFINE rows with llvm-mc disassembly.

Reads generated DEFINE lines from p1_aarch64.M1, disassembles code-bearing rows
with llvm-mc, and prints the DEFINE name beside the native aarch64 mnemonic
sequence. Literal data rows such as syscall-number constants are labeled as data
instead of being treated as instructions.
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


DEFINE_RE = re.compile(r'^DEFINE\s+(\S+)\s+([0-9A-Fa-f]+)\s*$')


def repo_root():
    return Path(__file__).resolve().parent.parent


def default_input_path():
    return repo_root() / 'build' / 'p1' / 'aarch64' / 'p1_aarch64.M1'


def ensure_generated(path: Path):
    if path.exists():
        return
    gen = repo_root() / 'p1' / 'p1_gen.py'
    proc = subprocess.run(
        [sys.executable, str(gen), '--arch', 'aarch64', str(path.parent.parent)],
        check=True,
        cwd=repo_root(),
        capture_output=True,
        text=True,
    )
    if proc.stderr:
        sys.stderr.write(proc.stderr)


def parse_rows(path: Path):
    rows = []
    for line in path.read_text().splitlines():
        match = DEFINE_RE.match(line)
        if not match:
            continue
        name, hex_bytes = match.groups()
        rows.append((name, hex_bytes.upper()))
    return rows


def is_data_row(name: str):
    return name.startswith('sys_')


def disassemble_code_rows(rows, llvm_mc):
    code_rows = [(name, hex_bytes) for name, hex_bytes in rows if not is_data_row(name)]
    if not code_rows:
        return {}

    payload = '\n'.join(hex_bytes for _, hex_bytes in code_rows) + '\n'
    proc = subprocess.run(
        [llvm_mc, '--disassemble', '--hex', '--arch=aarch64'],
        input=payload,
        text=True,
        capture_output=True,
        check=True,
    )
    inst_lines = [line.strip() for line in proc.stdout.splitlines() if line.strip()]

    out = {}
    index = 0
    for name, hex_bytes in code_rows:
        words = len(hex_bytes) // 8
        out[name] = inst_lines[index:index + words]
        index += words

    if index != len(inst_lines):
        raise RuntimeError(
            f'llvm output row split mismatch: consumed {index}, got {len(inst_lines)}'
        )
    return out


def format_rows(rows, disasm_by_name, show_bytes):
    name_width = max(len(name) for name, _ in rows) if rows else 0
    out = []
    for name, hex_bytes in rows:
        if is_data_row(name):
            rhs = f'data 0x{hex_bytes}'
            out.append(f'{name:<{name_width}}  {rhs}')
            continue

        insns = disasm_by_name.get(name, [])
        if not insns:
            out.append(f'{name:<{name_width}}  <no disassembly>')
            continue

        prefix = name.ljust(name_width)
        byte_col = f'  {hex_bytes}' if show_bytes else ''
        out.append(f'{prefix}{byte_col}  {insns[0]}')
        for insn in insns[1:]:
            spacer = ' ' * name_width
            if show_bytes:
                spacer += '  ' + ' ' * len(hex_bytes)
            out.append(f'{spacer}  {insn}')
    return '\n'.join(out)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        'input',
        nargs='?',
        default=str(default_input_path()),
        help='path to p1_aarch64.M1',
    )
    parser.add_argument(
        '--llvm-mc',
        default=os.environ.get('LLVM_MC', 'llvm-mc'),
        help='path to llvm-mc',
    )
    parser.add_argument(
        '--grep',
        default='',
        help='only include DEFINE names containing this substring',
    )
    parser.add_argument(
        '--limit',
        type=int,
        default=0,
        help='maximum number of DEFINE rows to print (0 = all)',
    )
    parser.add_argument(
        '--show-bytes',
        action='store_true',
        help='include raw DEFINE bytes next to the name',
    )
    args = parser.parse_args()

    path = Path(args.input)
    ensure_generated(path)
    rows = parse_rows(path)
    if args.grep:
        rows = [(name, hex_bytes) for name, hex_bytes in rows if args.grep in name]
    if args.limit:
        rows = rows[:args.limit]

    disasm_by_name = disassemble_code_rows(rows, args.llvm_mc)
    print(format_rows(rows, disasm_by_name, args.show_bytes))


if __name__ == '__main__':
    main()
