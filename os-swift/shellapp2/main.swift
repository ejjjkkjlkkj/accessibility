// Shell graine v2. Usage : graine-shell2 [dossier-racine]
import Foundation

let rootArg = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "rootfs"
let shell = try ShellV2(rootPath: URL(fileURLWithPath: rootArg).standardizedFileURL.path)
print("Shell graine v2. Tapez « aide » pour commencer, « quitter » pour sortir.")
while true {
    print(shell.prompt, terminator: "")
    guard let line = readLine() else { break }
    let t = line.trimmingCharacters(in: .whitespaces)
    if t == "quitter" || t == "exit" { break }
    if t.hasPrefix("cd ") || t == "cd" {
        do { try shell.changeDirectory(String(t.dropFirst(2)).trimmingCharacters(in: .whitespaces).isEmpty ? "/" : String(t.dropFirst(3))) }
        catch { print("Erreur : \(error)") }
        continue
    }
    let out = shell.execute(t)
    if !out.isEmpty { print(out) }
}
