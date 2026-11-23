// Lance le shell sur la racine "rootfs". Usage : graine-shell [dossier-racine]
import Foundation

let rootArg = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "rootfs"
let rootURL = URL(fileURLWithPath: rootArg).standardizedFileURL
let shell = Shell(rootPath: rootURL.path)

print("Shell graine. Tapez help pour la liste, exit pour quitter.")
while true {
    print(shell.prompt, terminator: "")
    guard let line = readLine() else { break }
    let out = shell.execute(line: line.trimmingCharacters(in: .whitespaces))
    if out == "__EXIT__" { break }
    if !out.isEmpty { print(out) }
}
