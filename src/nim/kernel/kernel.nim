## Kernel entry sketch only. Nim runtime and startup dependencies must be removed
## or explicitly initialized before this can become a freestanding kernel.
proc kernel_main() {.exportc, cdecl.} =
  while true:
    discard
