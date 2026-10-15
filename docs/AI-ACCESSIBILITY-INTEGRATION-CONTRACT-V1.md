# AI–Accessibility Integration Contract v1

Status: CONTRACT DEFINED; IMPLEMENTATION NOT VALIDATED.

The screen reader and boot path must work without AI or network access. AI is optional and local-first. Both repositories must use a versioned request/response format with bounded UTF-8 input and output, explicit status values, and a shared conformance test suite. Invalid, oversized, timed-out, or incompatible responses must fall back to the non-AI path.

Evidence must be labeled HOST_TEST, SIMULATED, or PHYSICAL. QEMU is SIMULATED, not PHYSICAL. Contract documentation alone does not prove runtime integration; until both repositories pass shared tests from clean checkouts, integration remains NOT_VALIDATED.
