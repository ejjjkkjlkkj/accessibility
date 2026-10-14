; Architecture-specific support primitives only; not a standalone screen reader or OS.
BITS 64
DEFAULT REL
SECTION .text
GLOBAL adm_halt
GLOBAL adm_read_rflags
adm_halt:
    hlt
    ret
adm_read_rflags:
    pushfq
    pop rax
    ret
