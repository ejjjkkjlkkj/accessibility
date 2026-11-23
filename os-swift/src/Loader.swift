// Chargeur GBS en Swift : lit boot.gbs, vérifie le manifeste, lance le noyau, retour automatique.
import Foundation

/// Lancement d'un fichier de démarrage. La v0.1 le simule ; le matériel réel viendra avec le noyau graine.
protocol KernelLauncher {
    /// Retourne true si le fichier a démarré et répondu dans le délai.
    func launch(image: String, timeout: Int) -> Bool
}

/// Lanceur de test : réussit si le fichier existe et n'est pas marqué comme cassé.
final class SimulatedLauncher: KernelLauncher {
    func launch(image: String, timeout: Int) -> Bool {
        FileManager.default.fileExists(atPath: image)
    }
}

enum BootStep: String {
    case config, manifest, verify, kernel, fallback, done
}

/// Journal de démarrage : une ligne par étape, conservée entre démarrages.
final class BootJournal {
    private let path: String
    private(set) var lines: [String] = []

    init(path: String) {
        self.path = path
    }

    func record(_ step: BootStep, _ status: String, _ detail: String = "") {
        let line = "\(step.rawValue) \(status)" + (detail.isEmpty ? "" : " : \(detail)")
        lines.append(line)
        let existing = (try? String(contentsOfFile: path, encoding: .utf8)) ?? ""
        try? (existing + line + "\n").write(toFile: path, atomically: true, encoding: .utf8)
    }
}

enum BootOutcome: Equatable {
    case kernel(String)
    case fallback(String)
    case failed(String)
}

final class BootLoader {
    let root: String
    let launcher: KernelLauncher
    let journal: BootJournal

    init(root: String, launcher: KernelLauncher = SimulatedLauncher()) {
        self.root = root
        self.launcher = launcher
        self.journal = BootJournal(path: root + "/journal/boot.log")
    }

    private func path(_ rel: String) -> String {
        root + "/" + rel.replacingOccurrences(of: "\\", with: "/")
    }

    /// Chaîne de démarrage : noyau, puis retour automatique vers `secours` en cas d'échec.
    func boot() -> BootOutcome {
        // 1. Configuration
        guard let cfgText = try? String(contentsOfFile: path("boot.gbs"), encoding: .utf8),
              let cfg = try? BootConfig(text: cfgText) else {
            journal.record(.config, "échec", "boot.gbs illisible")
            return .failed("configuration illisible")
        }
        journal.record(.config, "ok")

        // 2. Manifeste et empreintes
        do {
            let m = try Manifest(text: String(contentsOfFile: path("manifeste.gbs"), encoding: .utf8))
            journal.record(.manifest, "ok", "\(m.entries.count) fichiers")
            try m.verify(root: root)
            journal.record(.verify, "ok")
        } catch {
            journal.record(.verify, "échec", "\(error)")
            return fallbackOrFail(cfg, reason: "vérification : \(error)")
        }

        // 3. Noyau
        guard let kernel = cfg.kernel else {
            return fallbackOrFail(cfg, reason: "clé kernel absente")
        }
        if launcher.launch(image: path(kernel), timeout: cfg.delaySeconds) {
            journal.record(.kernel, "ok", kernel)
            journal.record(.done, "ok")
            return .kernel(kernel)
        }
        journal.record(.kernel, "échec", "pas de réponse après \(cfg.delaySeconds) s")
        return fallbackOrFail(cfg, reason: "noyau sans réponse")
    }

    private func fallbackOrFail(_ cfg: BootConfig, reason: String) -> BootOutcome {
        guard let fb = cfg.fallback else {
            journal.record(.fallback, "échec", "aucun secours configuré")
            return .failed(reason)
        }
        if launcher.launch(image: path(fb), timeout: cfg.delaySeconds) {
            journal.record(.fallback, "ok", "\(fb) (\(reason))")
            return .fallback(fb)
        }
        journal.record(.fallback, "échec", "secours introuvable (\(reason))")
        return .failed(reason)
    }
}
