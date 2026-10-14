## Portable announcement core; no native accessibility or audio adapter yet.
const EventQueueCapacity* = 16
const EventTextLimit* = 96

type
  FocusNode* = object
    role*, name*, value*: string
    focused*, disabled*: bool
  EventKind* = enum
    focusEvent, valueChangedEvent, stateChangedEvent
  SemanticEvent* = object
    kind*: EventKind
    node*: FocusNode
  EventQueue* = object
    items: array[EventQueueCapacity, SemanticEvent]
    head, count: int

proc announce*(node: FocusNode): string =
  if not node.focused: return ""
  result = if node.role.len == 0: "élément" else: node.role
  if node.name.len > 0: result.add(": " & node.name)
  if node.value.len > 0: result.add(", " & node.value)
  if node.disabled: result.add(", désactivé")

proc len*(q: EventQueue): int = q.count
proc push*(q: var EventQueue, kind: EventKind, node: FocusNode): bool =
  if q.count >= EventQueueCapacity or node.role.len >= EventTextLimit or
     node.name.len >= EventTextLimit or node.value.len >= EventTextLimit: return false
  let tail = (q.head + q.count) mod EventQueueCapacity
  q.items[tail] = SemanticEvent(kind: kind, node: node)
  inc q.count
  true
proc pop*(q: var EventQueue, event: var SemanticEvent): bool =
  if q.count == 0: return false
  event = q.items[q.head]
  q.items[q.head] = default(SemanticEvent)
  q.head = (q.head + 1) mod EventQueueCapacity
  dec q.count
  true
