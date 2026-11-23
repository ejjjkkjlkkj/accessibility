// Shell d'intentions. Usage : graine-intent [dossier-racine]
// Chaque phrase produit un plan affiché avant exécution. Lecture seule par défaut.
import Foundation

let rootArg = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "rootfs"
let shell = try ShellV2(rootPath: URL(fileURLWithPath: rootArg).standardizedFileURL.path)
let intent = IntentShell(natural: NaturalShell(shell: shell))

print("Graine (intentions) : parlez-moi. Lecture seule par défaut.")
print("Autorisations : « autoriser ecriture », « autoriser suppression », « lecture-seule ». « quitter » pour sortir.")
var pending: Plan? = nil

while true {
    print("vous> ", terminator: "")
    guard let raw = readLine() else { break }
    let line = raw.trimmingCharacters(in: .whitespaces)
    if line.isEmpty { continue }
    if line == "quitter" { break }

    // Confirmation d'un plan en attente
    if let p = pending, ["oui", "ok", "confirme"].contains(line.lowercased()) {
        pending = nil
        print(intent.execute(p, confirmed: true))
        continue
    }
    pending = nil

    if line.hasPrefix("autoriser ") {
        print(intent.grant(String(line.dropFirst("autoriser ".count))))
        continue
    }
    if line == "lecture-seule" {
        print(intent.grant("lecture-seule"))
        continue
    }

    do {
        let p = try intent.plan(for: line)
        print(intent.describe(p))
        let out = intent.execute(p, confirmed: !p.needsConfirmation)
        if p.needsConfirmation {
            pending = p
        }
        print(out)
    } catch {
        print("\(error)")
    }
}
