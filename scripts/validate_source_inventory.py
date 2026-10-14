#!/usr/bin/env python3
"""Check source inventory for language tracks; this is not build or runtime proof."""
from __future__ import annotations

import sys
from pathlib import Path

TRACKS = {
    "C": {
        "core": "src/c/screen_reader_core.c",
        "uefi": "src/c/uefi/AccessibleUefiHello.c",
        "os": "src/c/os/README.md",
        "kernel": "src/c/kernel/kernel.c",
    },
    "Rust": {
        "core": "src/rust/src/lib.rs",
        "uefi": "src/rust-uefi/src/main.rs",
        "os": "src/rust/os/README.md",
        "kernel": "src/rust/kernel/kernel.rs",
    },
    "Go": {
        "core": "src/go/screenreader.go",
        "uefi": "src/go/uefi/README.md",
        "os": "src/go/os/README.md",
        "kernel": "src/go/kernel/kernel.go",
    },
    "D": {
        "core": "src/d/screenreader.d",
        "uefi": "src/d/uefi/README.md",
        "os": "src/d/os/README.md",
        "kernel": "src/d/kernel/kernel.d",
    },
    "Ada": {
        "core": "src/ada/screen_reader_core.adb",
        "uefi": "src/ada/uefi/README.md",
        "os": "src/ada/os/README.md",
        "kernel": "src/ada/kernel/kernel_main.adb",
    },
    "Zig": {
        "core": "src/zig/screenreader.zig",
        "uefi": "src/zig/uefi/README.md",
        "os": "src/zig/os/README.md",
        "kernel": "src/zig/kernel/kernel.zig",
    },
    "Nim": {
        "core": "src/nim/screenreader_core.nim",
        "uefi": "src/nim/uefi/README.md",
        "os": "src/nim/os/README.md",
        "kernel": "src/nim/kernel/kernel.nim",
    },
    "Swift": {
        "core": "src/swift/ScreenReaderCore.swift",
        "uefi": "src/swift/uefi/README.md",
        "os": "src/swift/os/README.md",
        "kernel": "src/swift/kernel/kernel.swift",
    },
    "C++": {
        "core": "src/cpp/screenreader_core.hpp",
        "tests": "src/cpp/test_screenreader_core.cpp",
        "uefi": "src/cpp/uefi/README.md",
        "kernel": "src/cpp/kernel/README.md",
    },
    "Free Pascal": {
        "core": "src/pascal/screenreader_core.pas",
        "tests": "src/pascal/test_screenreader_core.pas",
        "uefi": "src/pascal/uefi/README.md",
        "kernel": "src/pascal/kernel/README.md",
    },
    "Odin": {
        "core": "src/odin/screenreader_core.odin",
        "uefi": "src/odin/uefi/README.md",
        "kernel": "src/odin/kernel/README.md",
    },
}


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    missing: list[str] = []
    for language, components in TRACKS.items():
        for component, relative_path in components.items():
            if not (root / relative_path).is_file():
                missing.append(f"{language}/{component}: {relative_path}")
    if missing:
        print("FAIL: source inventory incomplete")
        for item in missing:
            print(f"  - {item}")
        return 1
    print(f"PASS: inventory contains {len(TRACKS)} language tracks and {sum(map(len, TRACKS.values()))} expected source/documentation files.")
    print("LIMITATION: file presence does not prove compilation, UEFI execution, OS boot, screen-reader behavior, or physical audio.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
