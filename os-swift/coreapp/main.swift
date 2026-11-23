import Foundation
let bus = OutputBus()
bus.attach(TextOutput())
let focus = FocusManager(bus: bus)

struct Folder: AccessibleObject {
    let name: String
    let role: Role = .list
    let state: Set<State> = [.enabled]
    let actions: [Action] = [Action(key: "enter", label: "open", destructive: false),
                             Action(key: "delete", label: "move to trash", destructive: true)]
}

focus.focus(Folder(name: "Documents"))
focus.focus(Folder(name: "Reports"))
focus.back()
InterruptControl(bus: bus).stop()
