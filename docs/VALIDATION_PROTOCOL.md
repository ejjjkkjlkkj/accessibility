# Reproducible validation protocol

## Evidence record template

Every implementation milestone must record:

- Component ID (for example `C-UEFI-APP` or `Rust-SR-Core`).
- Commit SHA and source path.
- Exact command and toolchain version.
- Host OS, target triple, and whether the run was native, emulated, or physical.
- Exit code, test output, and artifact hash when relevant.
- Result: `PASS`, `FAIL`, `PARTIAL`, `BLOCKED`, or `NOT_VALIDATED`.
- Known limitations and the next test needed.

## Validation levels

1. **Source inspection**: source exists and is reviewed; not proof of compilation.
2. **Compile**: compiler exits successfully; not proof of correct runtime behavior.
3. **Unit tests**: listed tests ran and passed.
4. **Emulator boot/run**: validated only in the named emulator and configuration.
5. **Physical hardware**: validated on the named machine and device path.
6. **Accessibility acceptance**: a blind/low-vision user can operate the specified workflow using keyboard and speech; must be tested separately from build success.

## Minimum initial screen-reader-core cases

- Unfocused nodes produce no announcement.
- Focused nodes announce role and accessible name.
- Non-empty value is included without losing the role/name.
- Disabled state is included.
- Missing role falls back to a generic label.
- Empty name/value are handled.
- Long input has documented truncation/error behavior.
- No speech/audio success is claimed by a text-formatting-only core.

## UEFI and OS distinctions

A UEFI application runs under firmware boot services; it is not automatically a bare-metal OS. A kernel must document its entry contract, memory-map transition, interrupt setup, allocation model, drivers, and boot artifacts. Any dependency on a runtime, garbage collector, C library, or host OS must be identified explicitly.

Physical audio output requires a concrete codec/controller driver and a measured test on actual hardware. Writing samples to a buffer or emulated MMIO is not proof that sound reached a speaker.
