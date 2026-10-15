# Local agent task queue — Accessibility / Bare-Metal OS

Work only on this dedicated feature branch. Inspect source, manifests, and tests before choosing the smallest concrete correctness, accessibility, parser-bounds, or reproducibility improvement. Add a regression test for every behavior change. Preserve the non-AI fallback and keyboard-first accessibility. Label hardware/UEFI behavior NOT_VALIDATED unless there is actual physical evidence.

Never modify `.github/workflows/`, `AGENTS.md`, this task file, secrets, release files, or the agent runner. Do not add credential access, network beacons, persistence, destructive behavior, arbitrary command execution, firmware/NVRAM/partition actions, or reboot logic. Do not claim simulation is physical proof. If no safe, testable change is apparent, return NO_PATCH.
