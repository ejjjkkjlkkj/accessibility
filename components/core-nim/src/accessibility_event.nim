import strutils
import unicode

const
  STATE_FOCUSED* = 1'u64 shl 0
  STATE_FOCUSABLE* = 1'u64 shl 1
  STATE_ENABLED* = 1'u64 shl 2
  STATE_DISABLED* = 1'u64 shl 3
  STATE_KNOWN_MASK* = 0x0F'u64

proc isValidStateFlags*(flags: uint64): bool =
  if (flags and not STATE_KNOWN_MASK) != 0'u64:
    return false
  if (flags and STATE_ENABLED) != 0'u64 and (flags and STATE_DISABLED) != 0'u64:
    return false
  if (flags and STATE_FOCUSED) != 0'u64 and
      ((flags and STATE_FOCUSABLE) == 0'u64 or
       (flags and STATE_ENABLED) == 0'u64 or
       (flags and STATE_DISABLED) != 0'u64):
    return false
  true

proc isValidName*(name: string): bool =
  name.len <= 4096 and
    name.find('\0') < 0 and
    validateUtf8(name) == -1

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
    stateFlags*: uint64

proc isValid*(event: AccessibilityEvent): bool =
  event.abiVersion == 1'u32 and
    isValidName(event.name) and
    isValidStateFlags(event.stateFlags)

when isMainModule:
  doAssert AccessibilityEvent(abiVersion: 1, role: erButton, name: "Save").isValid()
  doAssert not AccessibilityEvent(abiVersion: 2, role: erButton, name: "Save").isValid()
  doAssert not AccessibilityEvent(abiVersion: 1, role: erButton, name: repeat('x', 4097)).isValid()
  doAssert isValidStateFlags(STATE_FOCUSABLE or STATE_ENABLED or STATE_FOCUSED)
  doAssert not isValidStateFlags(STATE_FOCUSED)
  doAssert not isValidStateFlags(STATE_ENABLED or STATE_DISABLED)
  doAssert not isValidStateFlags(1'u64 shl 63)
  doAssert not isValidName("bad\\0name")
  doAssert not isValidName("\\xC0\\xAF")
  doAssert isValidName("café")
