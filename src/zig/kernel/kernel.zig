// Freestanding kernel entry sketch; no boot protocol, linker script, or drivers yet.
export fn kernel_main() callconv(.c) noreturn {
    while (true) {
        asm volatile ("" ::: "memory");
    }
}
