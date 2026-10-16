import strutils

type
  EventRole* = enum
    erUnknown = 0, erApplication = 1, erWindow = 2, erGroup = 3, erText = 4,
    erButton = 5, erCheckbox = 6, erRadio = 7, erTextbox = 8, erLink = 9,
    erHeading = 10, erList = 11, erListItem = 12, erTable = 13,
    erRow = 14, erCell = 15, erMenu = 16, erMenuItem = 17
  AccessibilityEvent* = object
    abiVersion*: uint32
    role*: EventRole
    name*: string

proc isValid*(event: AccessibilityEvent): bool =
  event.abiVersion == 1'u32 and
    event.name.len <= 4096 and '\0' notin event.name

when isMainModule:
  doAssert AccessibilityEvent(abiVersion: 1, role: erButton, name: "Save").isValid()
  doAssert not AccessibilityEvent(abiVersion: 2, role: erButton, name: "Save").isValid()
  doAssert not AccessibilityEvent(abiVersion: 1, role: erButton, name: repeat('x', 4097)).isValid()
  doAssert not AccessibilityEvent(abiVersion: 1, role: erButton, name: "bad\0name").isValid()
