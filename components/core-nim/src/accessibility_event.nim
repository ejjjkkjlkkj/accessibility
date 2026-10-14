import strutils

type
  EventRole* = enum
    erUnknown, erWindow, erButton, erTextField, erHeading
  AccessibilityEvent* = object
    abiVersion*: uint32
    role*: EventRole
    name*: string

proc isValid*(event: AccessibilityEvent): bool =
  event.abiVersion == 1'u32 and
    event.role != erUnknown and
    event.name.len <= 256

when isMainModule:
  doAssert AccessibilityEvent(abiVersion: 1, role: erButton, name: "Save").isValid()
  doAssert not AccessibilityEvent(abiVersion: 2, role: erButton, name: "Save").isValid()
  doAssert not AccessibilityEvent(abiVersion: 1, role: erButton, name: repeat('x', 257)).isValid()
