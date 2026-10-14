# CI findings and repair ledger

## Run observed on 2026-10-14

Workflow run: `38582779417` (Core smoke tests), checked against merge revision `df23e7e178c1f2e3476666281e1181822902b7e3`.

| Job | Observed result | Evidence / follow-up |
|---|---|---|
| C core | PASS | Warnings-as-errors build and test executable completed successfully |
| Go core | PASS | `go test ./...` succeeded |
| D core | PASS | LDC unit tests compiled and executed successfully |
| Ada core | PASS | GNAT build and test executable succeeded, including fixed-capacity queue |
| Nim core | PASS | Nim compile-and-run test succeeded |
| Swift core | PASS | Swift compile-and-run test succeeded |
| Rust core | FAIL | Queue accepted a field of exactly 96 bytes while test expected the common limit to reject it; added explicit pre-check in `Event::from_node`. Retest pending. |
| Zig core | FAIL | Zig 0.14.1 rejected `.empty` initialization for the unmanaged ArrayList type; changed initialization to `.{}`. Retest pending. |
| Rust UEFI build | FAIL | no_std executable lacked `#[panic_handler]`; added a minimal non-returning panic handler. Retest pending. |
| Manifest validation | PASS | All eight language tracks and 24 component records validated. |

## Important scope limits

These jobs are CI evidence for portable host-side cores only. They do not validate native accessibility APIs, speech/audio, EFI boot, a complete OS kernel, QEMU boot, or physical hardware.

## Next required checks

1. Confirm Rust core, Zig core and Rust UEFI CI after the corrective commits.
2. Confirm the freestanding C kernel object job on the latest revision.
3. Add native accessibility adapters and platform-level acceptance tests.
4. Build and boot EFI artifacts in an explicitly configured emulator before considering physical validation.
