# Release readiness

**Status: NOT RELEASED.** This file is a gate checklist, not a release attestation.

## Required before a tagged release

- [ ] All required GitHub Actions workflows pass on the exact commit selected for release.
- [ ] Review the integration diff against `main`; do not merge a divergent branch wholesale.
- [ ] Decide and document the repository license and verify third-party license notices.
- [ ] Record exact compiler/toolchain versions and reproducible build commands for each shipped artifact.
- [ ] Produce checksums and release notes for every published artifact.
- [ ] Keep implementation status honest: a portable core or stub is not an operating-system screen reader.
- [ ] Record UEFI/QEMU results separately from physical-hardware evidence.
- [ ] Do not claim audio output, OS accessibility integration, or physical boot without direct test evidence.

## Evidence boundaries

- Hosted CI validates only the commands and targets it actually executes.
- QEMU/virtual machines are simulation evidence, not physical hardware proof.
- No firmware flashing, NVRAM writes, or automatic physical reboot are part of release automation.
- Any failing or queued required check blocks the release; a cancelled run is not a pass.
