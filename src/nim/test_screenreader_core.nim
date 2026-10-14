import screenreader_core

doAssert announce(FocusNode(role: "bouton", name: "Valider", focused: true)) == "bouton: Valider"
doAssert announce(FocusNode(role: "bouton", name: "Valider", focused: false)) == ""
doAssert announce(FocusNode(role: "champ", name: "Volume", value: "80 %", focused: true, disabled: true)) == "champ: Volume, 80 %, désactivé"

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
