#![no_std]
#![no_main]

// Kernel entry contract only. A bootloader handoff, linker script, panic path,
// memory map and target-specific startup are still required.
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    loop {
        // Deliberately no privileged instruction until a target/boot contract is chosen.
        core::hint::spin_loop();
    }
}
