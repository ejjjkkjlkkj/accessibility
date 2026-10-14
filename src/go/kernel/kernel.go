// Package main is a host/runtime-dependent kernel feasibility stub.
// Standard Go toolchains require runtime initialization; this is NOT a freestanding kernel.
package main

// KernelMain documents the intended handoff symbol for a future specialized toolchain.
func KernelMain() { for {} }
