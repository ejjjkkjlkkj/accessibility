import unittest
from ZERO.zero_isa_reference import ZeroCpu, assemble_instruction, step_cpu


class TestZeroIsa(unittest.TestCase):
    def test_r0_is_always_zero(self):
        cpu = ZeroCpu()
        cpu.write_gpr(0, 0x12345678)
        self.assertEqual(cpu.read_gpr(0), 0)

    def test_arithmetic_instructions(self):
        cpu = ZeroCpu()
        # ADDI r1, r0, 10
        # ADDI r2, r0, 20
        # ADD  r3, r1, r2
        # MUL  r4, r3, r2
        # HALT
        prog = (
            assemble_instruction("ADDI", 1, 0, 10)
            + assemble_instruction("ADDI", 2, 0, 20)
            + assemble_instruction("ADD", 3, 1, 2)
            + assemble_instruction("MUL", 4, 3, 2)
            + assemble_instruction("HALT")
        )
        cpu.memory[0 : len(prog)] = prog

        while not cpu.halted:
            step_cpu(cpu)

        self.assertEqual(cpu.read_gpr(1), 10)
        self.assertEqual(cpu.read_gpr(2), 20)
        self.assertEqual(cpu.read_gpr(3), 30)
        self.assertEqual(cpu.read_gpr(4), 600)

    def test_memory_load_store(self):
        cpu = ZeroCpu()
        # ADDI r1, r0, 0x100
        # ADDI r2, r0, 0x42
        # ST   r1, r2, 0
        # LD   r3, r1, 0
        # HALT
        prog = (
            assemble_instruction("ADDI", 1, 0, 0x100)
            + assemble_instruction("ADDI", 2, 0, 0x42)
            + assemble_instruction("ST", 1, 2, 0)
            + assemble_instruction("LD", 3, 1, 0)
            + assemble_instruction("HALT")
        )
        cpu.memory[0 : len(prog)] = prog

        while not cpu.halted:
            step_cpu(cpu)

        self.assertEqual(cpu.read_gpr(3), 0x42)

    def test_hardware_accessibility_channel(self):
        cpu = ZeroCpu()
        # HAC_EMIT_EVENT 1 (FocusChanged), 5 (Button), 1 (Focused)
        # HALT
        prog = assemble_instruction("HAC_EMIT_EVENT", 1, 5, 1) + assemble_instruction("HALT")
        cpu.memory[0 : len(prog)] = prog

        while not cpu.halted:
            step_cpu(cpu)

        self.assertEqual(len(cpu.hac_events), 1)
        self.assertEqual(cpu.hac_events[0], (1, 5, 1))
        self.assertEqual(cpu.hac[0], 5)  # Role
        self.assertEqual(cpu.hac[1], 1)  # State
        self.assertEqual(cpu.hac[2], 1)  # Event Type

    def test_tensor_matmul(self):
        cpu = ZeroCpu()
        for i in range(16):
            cpu.tensor[1][i] = 2.0
            cpu.tensor[2][i] = 3.0

        # TMATMUL t3, t1, t2 (64-bit instruction)
        # HALT
        prog = assemble_instruction("TMATMUL", 3, 1, 2) + assemble_instruction("HALT")
        cpu.memory[0 : len(prog)] = prog

        while not cpu.halted:
            step_cpu(cpu)

        for i in range(16):
            self.assertEqual(cpu.tensor[3][i], 6.0)


if __name__ == "__main__":
    unittest.main()
