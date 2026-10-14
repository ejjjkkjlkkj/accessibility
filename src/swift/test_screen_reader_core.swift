import Foundation

@main
struct ScreenReaderCoreTests {
    static func expect(_ condition: @autoclosure () -> Bool, _ message: String) {
        if !condition() { fatalError(message) }
    }

    static func main() {
        expect(ScreenReaderCore.announce(FocusNode(role: "bouton", name: "Valider", focused: true)) == "bouton: Valider",
               "focused node announcement mismatch")
        expect(ScreenReaderCore.announce(FocusNode(role: "bouton", name: "Valider")) == "",
               "unfocused node must be silent")
        expect(ScreenReaderCore.announce(FocusNode(role: "champ", name: "Volume", value: "80 %", focused: true, disabled: true)) == "champ: Volume, 80 %, désactivé",
               "value/disabled announcement mismatch")
        print("Swift screen-reader core tests passed")
    }
}
