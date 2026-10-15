# DARK / WHITE / GREY — Security workflow

**Current status: CONTRACT_ONLY_NOT_AUTONOMOUS.** This document defines roles and gates. It does not claim that autonomous AI agents are connected or running.

## Scope

DARK performs authorized adversarial testing against this repository, locally generated fixtures, and test systems explicitly approved by the owner. Third-party targets, credential theft, persistence, destructive actions, and unapproved firmware/NVRAM/BIOS changes are out of scope.

## Roles

- **DARK**: threat modeling, code review, safe local fuzzing, boundary testing, binary metadata review, dependency/provenance analysis, reproducible vulnerability reports.
- **WHITE**: remediation, least privilege, hardening, regression fixes.
- **GREY**: protocol and format research, reverse engineering of authorized artifacts, source/version/provenance recording.
- **QA**: repeatable tests, static checks, regression tests and accurate environment labels.
- **INDEPENDENT_AUDIT**: challenge evidence and verify critical fixes; never self-certifies its own work.
- **ORCHESTRATOR**: route tasks and block release when a mandatory gate fails.

## Finding contract

Every finding must record: unique ID; severity and rationale; affected path/component; tested commit; scope authorization; exact reproduction steps; expected and observed behavior; logs or artifact hashes; remediation; regression test; retest result; reviewer; status.

Never report a vulnerability as fixed solely because a patch was proposed. Reproduce the original failure where safe, test the patch, and run regression tests.

## Status vocabulary

- `PASS`: specified check ran and passed, with command and evidence.
- `FAIL`: check ran and failed.
- `PARTIAL`: only part of the required scope was checked.
- `BLOCKED`: prerequisites or safe test environment unavailable.
- `NOT_VALIDATED`: no adequate validation has occurred.
- `UNPROVEN`: a claim lacks sufficient evidence.

## Mandatory release gates

1. Validate the machine-readable agent contract.
2. Run deterministic repository checks and tests.
3. Review findings and remediation evidence.
4. Require independent review for critical/high-severity findings.
5. Block release on unresolved critical/high findings, failed mandatory checks, missing evidence, or scope violations.
6. Record physical hardware validation separately. Hosted CI, static analysis, and emulation are not proof of physical-device behavior.

## Safe fuzzing defaults

Use local fixtures only, bounded inputs and timeouts, no network access, and a temporary workspace. Preserve crashing inputs as regression fixtures only after checking that they contain no secrets or personal data.

## Current implementation boundary

The manifest is a role and policy contract, not an agent runtime. The CI validates the contract and source syntax; it does not execute AI agents, prove the absence of vulnerabilities, or validate physical hardware.

AERL is a local structured evidence-ledger prototype. Its hash chain detects record-level changes and ordering breaks; a separately protected checkpoint is needed to detect whole-ledger replacement or tail truncation. It is not a tamper-proof store, does not sign authorship, and does not replace OS logging infrastructure. See docs/AERL.md.
