// Affiche les métadonnées et la table des tenseurs d'un fichier GGUF.
// Usage : gguf-info <fichier.gguf> [--tensors]
import Foundation

let args = CommandLine.arguments
guard args.count >= 2 else {
    print("usage : gguf-info <fichier.gguf> [--tensors]")
    exit(1)
}
let showTensors = args.contains("--tensors")

do {
    let f = try GGUFFile(path: args[1])
    print("version GGUF : \(f.version)")
    print("métadonnées : \(f.metadata.count)")
    print("tenseurs : \(f.tensors.count)")
    print("début des données : octet \(f.dataStart)")

    for key in f.metadata.keys.sorted() {
        let v = f.metadata[key]!
        switch v {
        case .array(let items): print("  \(key) = [tableau de \(items.count)]")
        case .string(let s): print("  \(key) = \"\(s.prefix(80))\"")
        default:
            if let i = v.asInt { print("  \(key) = \(i)") }
            else if let d = v.asFloat { print("  \(key) = \(d)") }
            else if case .bool(let b) = v { print("  \(key) = \(b)") }
        }
    }

    if showTensors {
        var counts: [GGMLType: Int] = [:]
        var bytes = 0
        for t in f.tensors {
            counts[t.type, default: 0] += 1
            bytes += t.byteCount
            print("  \(t.name) \(t.dims) \(t.type) \(t.byteCount) octets")
        }
        print("répartition par type : \(counts.map { "\($0.key)=\($0.value)" }.sorted())")
        print("taille totale des données : \(bytes) octets")
    }
} catch {
    print("erreur : \(error)")
    exit(2)
}
