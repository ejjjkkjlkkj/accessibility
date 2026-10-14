## Portable announcement core; no native accessibility or audio adapter yet.
type
  FocusNode* = object
    role*, name*, value*: string
    focused*, disabled*: bool

proc announce*(node: FocusNode): string =
  if not node.focused:
    return ""
  result = if node.role.len == 0: "élément" else: node.role
  if node.name.len > 0: result.add(": " & node.name)
  if node.value.len > 0: result.add(", " & node.value)
  if node.disabled: result.add(", désactivé")
