from collections import namedtuple


ArchDef = namedtuple(
    'ArchDef',
    'name word_bytes stack_align syscall_numbers encoders start_stub',
)

Banner = namedtuple('Banner', 'text')
Literal = namedtuple('Literal', 'name hex_by_arch')
Nullary = namedtuple('Nullary', 'name kind')
Li = namedtuple('Li', 'name rd')
La = namedtuple('La', 'name rd')
LaBr = namedtuple('LaBr', 'name')
Mov = namedtuple('Mov', 'name rd rs')
Rrr = namedtuple('Rrr', 'name op rd ra rb')
AddI = namedtuple('AddI', 'name rd ra imm')
LogI = namedtuple('LogI', 'name op rd ra imm')
ShiftI = namedtuple('ShiftI', 'name op rd ra imm')
Mem = namedtuple('Mem', 'name op rt rn off')
LdArg = namedtuple('LdArg', 'name rd slot')
BranchReg = namedtuple('BranchReg', 'name kind rs')
CondB = namedtuple('CondB', 'name op ra rb')
CondBZ = namedtuple('CondBZ', 'name op ra')
Enter = namedtuple('Enter', 'name size')


def byte(n):
    return f'{n & 0xFF:02X}'


def le32(n):
    return (n & 0xFFFFFFFF).to_bytes(4, 'little').hex().upper()


def le64(n):
    return (n & 0xFFFFFFFFFFFFFFFF).to_bytes(8, 'little').hex().upper()


def word_hex(word_bytes, n):
    if word_bytes == 4:
        return le32(n)
    if word_bytes == 8:
        return le64(n)
    raise ValueError(f'unsupported word size: {word_bytes}')


def round_up(align, n):
    return ((n + align - 1) // align) * align
