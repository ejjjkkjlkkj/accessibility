import Foundation
let interp = try NativeModelInterpreter(folder: "models/test")
for s in ["list the files", "go to etc", "delete VERSION", "make a dog"] {
    print("\(s)  ->  \(interp.interpret(s))")
}
