module kernel;

// Source-level entry contract only. D runtime/GC startup and linker/boot protocol
// must be solved before this can be treated as a freestanding kernel.
extern(C) void kernel_main() @nogc nothrow {
    for (;;) { }
}
