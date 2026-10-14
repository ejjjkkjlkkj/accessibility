// Kernel entry sketch only. Swift runtime/stdlib dependencies make freestanding
// deployment unproven until a supported embedded toolchain and startup are established.
@_cdecl("kernel_main")
public func kernel_main() -> Never {
    while true { }
}
