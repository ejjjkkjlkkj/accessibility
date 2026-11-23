// Graine du système : crée l'arborescence racine "/" façon Linux, en orienté objet.
import Foundation

/// Un répertoire de l'arborescence avec sa description.
struct RootEntry {
    let path: String
    let purpose: String
}

protocol FileSystemBuilder {
    func build(at root: URL) throws
}

/// Crée la racine "/" : chaque répertoire reçoit un fichier README qui explique son rôle.
final class RootFilesystemBuilder: FileSystemBuilder {
    let entries: [RootEntry]

    init(entries: [RootEntry] = RootFilesystemBuilder.standardLayout) {
        self.entries = entries
    }

    static let standardLayout: [RootEntry] = [
        RootEntry(path: "bin",  purpose: "Programmes essentiels accessibles à tous les utilisateurs."),
        RootEntry(path: "sbin", purpose: "Programmes d'administration du système."),
        RootEntry(path: "etc",  purpose: "Fichiers de configuration du système."),
        RootEntry(path: "dev",  purpose: "Fichiers représentant les périphériques."),
        RootEntry(path: "proc", purpose: "Informations sur les processus en cours."),
        RootEntry(path: "sys",  purpose: "Informations et réglages du noyau et des pilotes."),
        RootEntry(path: "tmp",  purpose: "Fichiers temporaires, effacés au redémarrage."),
        RootEntry(path: "var",  purpose: "Données qui changent : journaux, files d'attente, caches."),
        RootEntry(path: "usr",  purpose: "Programmes et bibliothèques installés pour les utilisateurs."),
        RootEntry(path: "lib",  purpose: "Bibliothèques partagées nécessaires aux programmes de bin et sbin."),
        RootEntry(path: "home", purpose: "Répertoires personnels des utilisateurs."),
        RootEntry(path: "root", purpose: "Répertoire personnel de l'administrateur."),
        RootEntry(path: "boot", purpose: "Fichiers de démarrage du noyau."),
        RootEntry(path: "mnt",  purpose: "Points de montage temporaires pour les systèmes de fichiers."),
        RootEntry(path: "opt",  purpose: "Logiciels optionnels fournis par des tiers."),
    ]

    func build(at root: URL) throws {
        let fm = FileManager.default
        try fm.createDirectory(at: root, withIntermediateDirectories: true)
        for entry in entries {
            let dir = root.appendingPathComponent(entry.path, isDirectory: true)
            try fm.createDirectory(at: dir, withIntermediateDirectories: true)
            let readme = dir.appendingPathComponent("README.txt")
            try "\(entry.purpose)\n".write(to: readme, atomically: true, encoding: .utf8)
        }
        // Fichier de racine : identifie la version de la graine
        try "graine v0.1 : arborescence racine de notre système\n"
            .write(to: root.appendingPathComponent("VERSION"), atomically: true, encoding: .utf8)
    }
}
