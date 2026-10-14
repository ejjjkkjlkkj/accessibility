import Foundation

@main
struct ScreenReaderCoreTests {
    static func expect(_ condition: @autoclosure () -> Bool, _ message: String) {
        if !condition() { fatalError(message) }
    }
    static func main() {
        expect(ScreenReaderCore.announce(FocusNode(role: "bouton", name: "Valider", focused: true)) == "bouton: Valider", "focused announcement")
        expect(ScreenReaderCore.announce(FocusNode(role: "bouton", name: "Valider")) == "", "unfocused silence")
        expect(ScreenReaderCore.announce(FocusNode(role: "champ", name: "Volume", value: "80 %", focused: true, disabled: true)) == "champ: Volume, 80 %, désactivé", "value and disabled")

        var q = EventQueue()
        var node = FocusNode(role: "bouton", name: "Valider", focused: true)
        expect(q.push(.focus, node: node), "queue push")
        node.name = "modified"
        let first = q.pop()
        expect(first?.node.name == "Valider", "snapshot ownership")
        expect(q.pop() == nil, "empty queue")
        let long = String(repeating: "x", count: EventQueue.textLimit)
        expect(!q.push(.focus, node: FocusNode(name: long)), "reject oversized field")
        for _ in 0..<EventQueue.capacity { expect(q.push(.stateChanged, node: node), "fill queue") }
        expect(!q.push(.focus, node: node), "reject overflow")
        for _ in 0..<EventQueue.capacity { expect(q.pop() != nil, "drain queue") }
        expect(q.pop() == nil, "empty after drain")
        print("Swift screen-reader core tests passed")
    }
}
