/* Freestanding C kernel entry stub. Not bootable until a boot protocol and linker script exist. */
typedef unsigned short u16;
void kernel_main(void) {
    volatile u16 *vga = (volatile u16 *)0xB8000;
    static const char msg[] = "Accessibility kernel: C entry reached";
    for (unsigned i = 0; i < sizeof(msg) - 1; ++i)
        vga[i] = (u16)msg[i] | (u16)0x0700;
    for (;;) { __asm__ volatile ("hlt"); }
}
