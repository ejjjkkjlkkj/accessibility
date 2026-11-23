import Foundation

// Test fixtures
final class Doc: AccessibleObject {
    let name: String
    let role: Role
    var state: Set<State>
    var actions: [Action]
    var children: [AccessibleObject]
    var performed: [String] = []
    init(_ name: String, role: Role = .item, state: Set<State> = [.enabled],
         actions: [Action] = [], children: [AccessibleObject] = []) {
        self.name = name; self.role = role; self.state = state
        self.actions = actions; self.children = children
    }
    func perform(_ action: Action) -> String {
        performed.append(action.label)
        return "\(action.label) done: \(name)"
    }
}

let openAction = Action(key: "enter", label: "open", destructive: false)
let deleteAction = Action(key: "delete", label: "delete", destructive: true)

var failures = 0
func check(_ ok: Bool, _ name: String) {
    print((ok ? "PASS " : "FAIL ") + name)
    if !ok { failures += 1 }
}

// 1. Braille rendering: a -> dots 1 (U+2801), space stays space
check(BrailleOutput.render("a") == "\u{2801}", "braille letter a")
check(BrailleOutput.render("a b") == "\u{2801} \u{2803}", "braille keeps spaces")

// 2. Bus: text and braille both receive every message
let bus = OutputBus()
let braille = BrailleOutput()
var texts: [String] = []
final class Capture: OutputChannel {
    let name = "capture"
    var sink: (String) -> Void
    init(_ s: @escaping (String) -> Void) { sink = s }
    func emit(_ m: Message) { sink(m.text) }
}
bus.attach(Capture { texts.append($0) })
bus.attach(braille)
bus.emit("hello")
check(texts == ["hello"], "bus delivers to text channel")
check(braille.current == BrailleOutput.render("hello"), "bus delivers to braille channel")

// 3. History and review
bus.emit("second"); bus.emit("third")
check(bus.review(last: 2) == ["second", "third"], "review returns last messages")

// 4. Focus: enter announces, next/previous move among siblings and wrap
let a = Doc("A"), b = Doc("B"), c = Doc("C")
let list = Doc("List", role: .list, children: [a, b, c])
let focus = FocusManager(bus: bus)
focus.enter(list)
focus.enter(a)
focus.next()
check(focus.current === b, "next moves to sibling")
focus.next(); focus.next()
check(focus.current === a, "next wraps to first sibling")
focus.previous()
check(focus.current === c, "previous wraps to last sibling")

// 5. Back returns to the parent
focus.back()
check(focus.current === list, "back returns to parent")

// 6. Destructive action refused by default
let target = Doc("File", actions: [deleteAction])
focus.enter(target)
focus.activate(actionKey: "delete")
check(target.performed.isEmpty, "destructive action refused without confirmation")

// 7. Destructive action runs after confirmation
let confirmed = FocusManager(bus: bus, confirm: { _ in true })
confirmed.enter(target)
confirmed.activate(actionKey: "delete")
check(target.performed == ["delete"], "destructive action runs after confirmation")

// 8. Non-destructive action runs without confirmation
let opened = Doc("Report", actions: [openAction])
focus.enter(opened)
focus.activate(actionKey: "enter")
check(opened.performed == ["open"], "safe action runs directly")

// 9. Unknown key is reported, not silent
let before = bus.history.count
focus.activate(actionKey: "zz")
check(bus.history.count > before, "unknown action key is announced")

// 10. Alert priority cancels speech on channels
final class Cancellable: OutputChannel {
    let name = "speech-test"
    var cancelled = 0
    func emit(_ m: Message) {}
    func cancel() { cancelled += 1 }
}
let bus2 = OutputBus()
let sp = Cancellable()
bus2.attach(sp)
bus2.emit("routine", priority: .info)
check(sp.cancelled == 0, "info message does not cancel speech")
bus2.emit("danger", priority: .alert)
check(sp.cancelled == 1, "alert cancels speech before speaking")

// 11. Stop key cancels and announces
let router = KeyRouter(focus: focus, bus: bus2)
router.press("ctrl-period")
check(sp.cancelled > 1, "stop key cancels speech")

// 12. Error state raises an alert line
let broken = Doc("Disk", role: .status, state: [.enabled, .error])
focus.enter(broken)
check(bus.history.last?.priority == .alert, "error state is announced as alert")

print(failures == 0 ? "ALL CORE TESTS PASSED" : "\(failures) TEST(S) FAILED")
