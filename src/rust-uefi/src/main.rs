#![no_main]
#![no_std]

use uefi::prelude::*;

/// First Rust UEFI execution milestone. Console-only; no audio or screen reader.
#[entry]
fn main() -> Status {
    uefi::println!("Accessibility UEFI Rust prototype");
    uefi::println!("Milestone: EFI entry point and text console only.");
    uefi::println!("No semantic UI tree, speech, audio driver, or OS kernel is present.");
    Status::SUCCESS
}
