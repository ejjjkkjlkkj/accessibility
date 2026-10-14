import screenreader_core
import std/strutils

doAssert announce(FocusNode(role: "bouton", name: "Valider", focused: true)) == "bouton: Valider"
doAssert announce(FocusNode(role: "bouton", name: "Valider", focused: false)) == ""
doAssert announce(FocusNode(role: "champ", name: "Volume", value: "80 %", focused: true, disabled: true)) == "champ: Volume, 80 %, désactivé"

doAssert announce(FocusNode(value: "42", focused: true)) == "élément, 42"
doAssert announce(FocusNode(focused: true)) == "élément"

var q: EventQueue
var ev: SemanticEvent
doAssert q.push(focusEvent, FocusNode(role: "bouton", name: "Valider", focused: true))
doAssert q.pop(ev) and ev.node.name == "Valider" and ev.kind == focusEvent
doAssert not q.pop(ev)
for i in 0 ..< EventQueueCapacity:
  doAssert q.push(stateChangedEvent, FocusNode(name: "ok"))
doAssert not q.push(focusEvent, FocusNode())
for i in 0 ..< EventQueueCapacity: doAssert q.pop(ev)
doAssert not q.pop(ev)
doAssert not q.push(focusEvent, FocusNode(name: repeat('x', EventTextLimit)))
echo "Nim screen-reader core tests passed"

var wrapQ: EventQueue
var wrapEv: SemanticEvent
let a = FocusNode(role: "bouton", name: "A", focused: true)
let b = FocusNode(role: "bouton", name: "B", focused: true)
for i in 0 ..< 12: doAssert wrapQ.push(focusEvent, a)
for i in 0 ..< 8:
  doAssert wrapQ.pop(wrapEv) and wrapEv.node.name == "A"
for i in 0 ..< 12: doAssert wrapQ.push(focusEvent, b)
doAssert wrapQ.len == EventQueueCapacity
for i in 0 ..< 4:
  doAssert wrapQ.pop(wrapEv) and wrapEv.node.name == "A"
for i in 0 ..< 12:
  doAssert wrapQ.pop(wrapEv) and wrapEv.node.name == "B"
doAssert not wrapQ.pop(wrapEv)
