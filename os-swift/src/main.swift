// Crée la racine "/" de notre système dans le dossier "rootfs".
import Foundation

let here = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let rootURL = here.appendingPathComponent("rootfs", isDirectory: true)

do {
    let builder: FileSystemBuilder = RootFilesystemBuilder()
    try builder.build(at: rootURL)
    print("Racine créée : \(rootURL.path)")
} catch {
    print("Erreur : \(error)")
    exit(1)
}
