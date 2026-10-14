# C++ kernel track

Status: NOT_IMPLEMENTED. The portable C++ event core is not a kernel.

Required gates: freestanding target; explicit linker script and boot protocol; entry assembly; memory map and allocator; exceptions/interrupts policy; constructors/runtime audit; serial diagnostics; input/display drivers; reproducible boot image; host and emulator tests. Physical boot is a separate validation and is never inferred from QEMU.
