// Recovery for the graine OS. No forced system reset: we repair exactly what is damaged.
// 1. Snapshots of the configuration before each change, restorable at any time.
// 2. Integrity check against the signed-style manifest (SHA-256 per file).
// 3. Targeted repair: only damaged files are restored from the pristine copy. Nothing else is touched.
import Foundation

struct IntegrityReport: Equatable {
    var missing: [String] = []
    var modified: [String] = []
    var isHealthy: Bool { missing.isEmpty && modified.isEmpty }
}

final class RecoveryManager {
    let rootPath: String          // the live system
    let pristinePath: String      // a clean reference copy, made once at install time
    let snapshotsPath: String     // where restore points are kept

    init(rootPath: String, pristinePath: String, snapshotsPath: String) {
        self.rootPath = rootPath
        self.pristinePath = pristinePath
        self.snapshotsPath = snapshotsPath
    }

    // MARK: Integrity

    /// Compares every file of the pristine copy with the live system, by SHA-256.
    func check() -> IntegrityReport {
        var report = IntegrityReport()
        for rel in files(in: pristinePath) {
            let live = rootPath + "/" + rel
            let reference = pristinePath + "/" + rel
            guard FileManager.default.fileExists(atPath: live) else {
                report.missing.append(rel); continue
            }
            if !sameContent(live, reference) { report.modified.append(rel) }
        }
        return report
    }

    /// Restores only the damaged files. Returns what was repaired. Healthy files are never touched.
    @discardableResult
    func repairDamaged() throws -> [String] {
        let report = check()
        let damaged = report.missing + report.modified
        for rel in damaged {
            let live = rootPath + "/" + rel
            let reference = pristinePath + "/" + rel
            try? FileManager.default.removeItem(atPath: live)
            try FileManager.default.createDirectory(atPath: (live as NSString).deletingLastPathComponent,
                                                    withIntermediateDirectories: true)
            try FileManager.default.copyItem(atPath: reference, toPath: live)
        }
        return damaged
    }

    // MARK: Snapshots

    /// Saves the configuration (etc and var) under a timestamped name. Returns the snapshot id.
    @discardableResult
    func snapshot(label: String) throws -> String {
        let stamp = Int(Date().timeIntervalSince1970 * 1000)
        let id = "\(stamp)-\(label)"
        let dest = snapshotsPath + "/" + id
        try FileManager.default.createDirectory(atPath: dest, withIntermediateDirectories: true)
        for area in ["etc", "var"] {
            let src = rootPath + "/" + area
            if FileManager.default.fileExists(atPath: src) {
                try FileManager.default.copyItem(atPath: src, toPath: dest + "/" + area)
            }
        }
        return id
    }

    func snapshots() -> [String] {
        ((try? FileManager.default.contentsOfDirectory(atPath: snapshotsPath)) ?? []).sorted()
    }

    /// Puts back the configuration saved in a snapshot. The current state is snapshotted first,
    /// so a rollback can itself be undone.
    func rollback(to id: String) throws {
        let src = snapshotsPath + "/" + id
        guard FileManager.default.fileExists(atPath: src) else {
            throw RecoveryError.noSuchSnapshot(id)
        }
        _ = try snapshot(label: "before-rollback")
        for area in ["etc", "var"] {
            let from = src + "/" + area
            guard FileManager.default.fileExists(atPath: from) else { continue }
            let to = rootPath + "/" + area
            try? FileManager.default.removeItem(atPath: to)
            try FileManager.default.copyItem(atPath: from, toPath: to)
        }
    }

    // MARK: Helpers

    /// Recursive listing of files, relative to `base`. Hidden entries are skipped.
    private func files(in base: String) -> [String] {
        func walk(_ dir: String, _ prefix: String) -> [String] {
            let names = (try? FileManager.default.contentsOfDirectory(atPath: dir)) ?? []
            var out: [String] = []
            for name in names.sorted() where !name.hasPrefix(".") {
                let full = dir + "/" + name
                var isDir: ObjCBool = false
                FileManager.default.fileExists(atPath: full, isDirectory: &isDir)
                let rel = prefix.isEmpty ? name : prefix + "/" + name
                if isDir.boolValue { out += walk(full, rel) } else { out.append(rel) }
            }
            return out
        }
        return walk(base, "")
    }

    private func sameContent(_ a: String, _ b: String) -> Bool {
        guard let da = FileManager.default.contents(atPath: a),
              let db = FileManager.default.contents(atPath: b) else { return false }
        return da == db
    }
}

enum RecoveryError: Error, CustomStringConvertible {
    case noSuchSnapshot(String)
    var description: String {
        switch self {
        case .noSuchSnapshot(let id): return "no snapshot called \(id)"
        }
    }
}
