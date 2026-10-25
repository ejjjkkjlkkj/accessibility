#!/usr/bin/env python3
"""
Assembleur, désassembleur et émulateur de référence Tier-0 64-bit pour ZERO-ISA.
Ne possède aucune dépendance tierce (bibliothèque standard Python 3 uniquement).
Toutes les instructions standard sont codées en mots de 64-bit (8 octets).
"""
from __future__ import annotations

import struct
from typing import List, Tuple


class ZeroCpu:
    def __init__(self, memory_size: int = 1024 * 1024) -> None:
        self.gpr: List[int] = [0] * 32  # r0-r31 (64-bit)
        self.tensor: List[List[float]] = [[0.0] * 16 for _ in range(32)]  # t0-t31 (512-bit / 16x FP32)
        self.hac: List[int] = [0] * 8  # a0-a7 (64-bit)
        self.csr: List[int] = [0] * 64  # csr0-csr63 (64-bit)
        self.pc: int = 0  # 64-bit PC
        self.memory: bytearray = bytearray(memory_size)
        self.halted: bool = False
        self.hac_events: List[Tuple[int, int, int]] = []
        self.audio_log: List[Tuple[int, int]] = []

    def read_gpr(self, reg: int) -> int:
        return 0 if reg == 0 else self.gpr[reg] & 0xFFFFFFFFFFFFFFFF

    def write_gpr(self, reg: int, val: int) -> None:
        if reg != 0:
            self.gpr[reg] = val & 0xFFFFFFFFFFFFFFFF

    def read_mem_u64(self, addr: int) -> int:
        if addr < 0 or addr + 8 > len(self.memory):
            raise IndexError(f"Memory access out of bounds: 0x{addr:X}")
        return struct.unpack_from("<Q", self.memory, addr)[0]

    def write_mem_u64(self, addr: int, val: int) -> None:
        if addr < 0 or addr + 8 > len(self.memory):
            raise IndexError(f"Memory access out of bounds: 0x{addr:X}")
        struct.pack_into("<Q", self.memory, addr, val & 0xFFFFFFFFFFFFFFFF)


def assemble_instruction(mnemonic: str, *args) -> bytes:
    """Encode une instruction ZERO-ISA 64-bit en binaire 64-bit (8 octets)."""
    mnemonic = mnemonic.upper()

    # Opcodes Standard 64-bit (Mot unique de 64 bits)
    if mnemonic == "ADD":
        rd, rs1, rs2 = args
        code = (0x01 << 58) | ((rd & 0x1F) << 53) | ((rs1 & 0x1F) << 48) | ((rs2 & 0x1F) << 43)
        return struct.pack("<Q", code)
    elif mnemonic == "SUB":
        rd, rs1, rs2 = args
        code = (0x02 << 58) | ((rd & 0x1F) << 53) | ((rs1 & 0x1F) << 48) | ((rs2 & 0x1F) << 43)
        return struct.pack("<Q", code)
    elif mnemonic == "MUL":
        rd, rs1, rs2 = args
        code = (0x03 << 58) | ((rd & 0x1F) << 53) | ((rs1 & 0x1F) << 48) | ((rs2 & 0x1F) << 43)
        return struct.pack("<Q", code)
    elif mnemonic == "ADDI":
        rd, rs1, imm = args
        code = (0x04 << 58) | ((rd & 0x1F) << 53) | ((rs1 & 0x1F) << 48) | (imm & 0xFFFFFFFFFFFF)
        return struct.pack("<Q", code)
    elif mnemonic == "LD":
        rd, rs1, imm = args
        code = (0x05 << 58) | ((rd & 0x1F) << 53) | ((rs1 & 0x1F) << 48) | (imm & 0xFFFFFFFFFFFF)
        return struct.pack("<Q", code)
    elif mnemonic == "ST":
        rs1, rs2, imm = args
        code = (0x06 << 58) | ((rs1 & 0x1F) << 53) | ((rs2 & 0x1F) << 48) | (imm & 0xFFFFFFFFFFFF)
        return struct.pack("<Q", code)
    elif mnemonic == "JMP":
        target = args[0]
        code = (0x07 << 58) | (target & 0x03FFFFFFFFFFFFFF)
        return struct.pack("<Q", code)
    elif mnemonic == "BEQ":
        rs1, rs2, offset = args
        code = (0x08 << 58) | ((rs1 & 0x1F) << 53) | ((rs2 & 0x1F) << 48) | (offset & 0xFFFFFFFFFFFF)
        return struct.pack("<Q", code)
    elif mnemonic == "HALT":
        code = (0x3F << 58)
        return struct.pack("<Q", code)

    # Opcodes Canal Accessibilité Matérielle (HAC 64-bit)
    elif mnemonic == "HAC_EMIT_EVENT":
        ev_type, role, state = args
        code = (0x20 << 58) | ((ev_type & 0xF) << 54) | ((role & 0x7F) << 47) | (state & 0x7FFFFFFFFFFF)
        return struct.pack("<Q", code)
    elif mnemonic == "HAC_SPEAK_AUDIO":
        buf_reg, len_reg = args
        code = (0x21 << 58) | ((buf_reg & 0x1F) << 53) | ((len_reg & 0x1F) << 48)
        return struct.pack("<Q", code)

    # Opcodes Tenseurs / IA Gigantesque (64-bit)
    elif mnemonic == "TMATMUL":
        td, ts1, ts2 = args
        code = (0x30 << 58) | ((td & 0x1F) << 53) | ((ts1 & 0x1F) << 48) | ((ts2 & 0x1F) << 43)
        return struct.pack("<Q", code)
    elif mnemonic == "TVGELU":
        td, ts1 = args
        code = (0x31 << 58) | ((td & 0x1F) << 53) | ((ts1 & 0x1F) << 48)
        return struct.pack("<Q", code)
    elif mnemonic == "TVATTEN":
        td, ts1, ts2, ts3 = args
        code = (0x32 << 58) | ((td & 0x1F) << 53) | ((ts1 & 0x1F) << 48) | ((ts2 & 0x1F) << 43) | ((ts3 & 0x1F) << 38)
        return struct.pack("<Q", code)

    raise ValueError(f"Unknown instruction mnemonic: {mnemonic}")


def step_cpu(cpu: ZeroCpu) -> None:
    """Exécute une instruction 64-bit au pointeur PC courant."""
    if cpu.halted:
        return

    dword = struct.unpack_from("<Q", cpu.memory, cpu.pc)[0]
    opcode = (dword >> 58) & 0x3F
    rd = (dword >> 53) & 0x1F
    rs1 = (dword >> 48) & 0x1F
    rs2 = (dword >> 43) & 0x1F

    if opcode == 0x01:  # ADD 64-bit
        cpu.write_gpr(rd, cpu.read_gpr(rs1) + cpu.read_gpr(rs2))
        cpu.pc += 8
    elif opcode == 0x02:  # SUB 64-bit
        cpu.write_gpr(rd, cpu.read_gpr(rs1) - cpu.read_gpr(rs2))
        cpu.pc += 8
    elif opcode == 0x03:  # MUL 64-bit
        cpu.write_gpr(rd, cpu.read_gpr(rs1) * cpu.read_gpr(rs2))
        cpu.pc += 8
    elif opcode == 0x04:  # ADDI 64-bit
        imm = dword & 0xFFFFFFFFFFFF
        if imm & 0x800000000000:
            imm -= 0x1000000000000
        cpu.write_gpr(rd, cpu.read_gpr(rs1) + imm)
        cpu.pc += 8
    elif opcode == 0x05:  # LD 64-bit
        imm = dword & 0xFFFFFFFFFFFF
        if imm & 0x800000000000:
            imm -= 0x1000000000000
        addr = cpu.read_gpr(rs1) + imm
        cpu.write_gpr(rd, cpu.read_mem_u64(addr))
        cpu.pc += 8
    elif opcode == 0x06:  # ST 64-bit
        imm = dword & 0xFFFFFFFFFFFF
        if imm & 0x800000000000:
            imm -= 0x1000000000000
        base_reg = rd
        val_reg = rs1
        addr = cpu.read_gpr(base_reg) + imm
        cpu.write_mem_u64(addr, cpu.read_gpr(val_reg))
        cpu.pc += 8
    elif opcode == 0x07:  # JMP 64-bit
        target = dword & 0x03FFFFFFFFFFFFFF
        cpu.pc += target * 8
    elif opcode == 0x08:  # BEQ 64-bit
        imm = dword & 0xFFFFFFFFFFFF
        if imm & 0x800000000000:
            imm -= 0x1000000000000
        if cpu.read_gpr(rd) == cpu.read_gpr(rs1):
            cpu.pc += imm * 8
        else:
            cpu.pc += 8
    elif opcode == 0x20:  # HAC_EMIT_EVENT 64-bit
        ev_type = (dword >> 54) & 0xF
        role = (dword >> 47) & 0x7F
        state = dword & 0x7FFFFFFFFFFF
        cpu.hac_events.append((ev_type, role, state))
        cpu.hac[0] = role
        cpu.hac[1] = state
        cpu.hac[2] = ev_type
        cpu.pc += 8
    elif opcode == 0x21:  # HAC_SPEAK_AUDIO 64-bit
        buf_reg = rd
        len_reg = rs1
        cpu.audio_log.append((cpu.read_gpr(buf_reg), cpu.read_gpr(len_reg)))
        cpu.pc += 8
    elif opcode == 0x30:  # TMATMUL 64-bit
        td = rd
        ts1 = rs1
        ts2 = rs2
        for i in range(16):
            cpu.tensor[td][i] = cpu.tensor[ts1][i] * cpu.tensor[ts2][i]
        cpu.pc += 8
    elif opcode == 0x31:  # TVGELU 64-bit
        td = rd
        ts1 = rs1
        for i in range(16):
            x = cpu.tensor[ts1][i]
            cpu.tensor[td][i] = x * (0.5 * (1.0 + x * 0.1))
        cpu.pc += 8
    elif opcode == 0x32:  # TVATTEN 64-bit
        td = rd
        ts1 = rs1
        ts2 = rs2
        ts3 = (dword >> 38) & 0x1F
        for i in range(16):
            cpu.tensor[td][i] = cpu.tensor[ts1][i] * cpu.tensor[ts2][i] + cpu.tensor[ts3][i]
        cpu.pc += 8
    elif opcode == 0x3F:  # HALT 64-bit
        cpu.halted = True
    else:
        raise RuntimeError(f"Unknown instruction at PC 0x{cpu.pc:X}: opcode 0x{opcode:X}")
