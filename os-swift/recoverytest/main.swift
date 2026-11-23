import Foundation
var failures = 0
func check(_ ok: Bool, _ name: String) { print((ok ? "PASS " : "FAIL ") + name); if !ok { failures += 1 } }
let base = "recoverytest"
let r = RecoveryManager(rootPath: base + "/live", pristinePath: base + "/pristine", snapshotsPath: base + "/snaps")

check(r.check().isHealthy, "fresh system is healthy")

// Damage one file and delete another
try! "voice = CORRUPT\n".write(toFile: base + "/live/etc/settings.txt", atomically: true, encoding: .utf8)
try! FileManager.default.removeItem(atPath: base + "/live/etc/README.txt")
let rep = r.check()
check(rep.modified == ["etc/settings.txt"], "modified file is detected")
check(rep.missing == ["etc/README.txt"], "missing file is detected")

// Targeted repair: only the damaged files come back
try! "user note\n".write(toFile: base + "/live/etc/user-note.txt", atomically: true, encoding: .utf8)
let fixed = try! r.repairDamaged()
check(Set(fixed) == ["etc/settings.txt", "etc/README.txt"], "repair lists only damaged files")
check(r.check().isHealthy, "system is healthy after repair")
check(FileManager.default.fileExists(atPath: base + "/live/etc/user-note.txt"), "user's own file is never touched")

// Snapshot and rollback
try! "voice = on\n".write(toFile: base + "/live/etc/settings.txt", atomically: true, encoding: .utf8)
let id = try! r.snapshot(label: "before-change")
try! "voice = off\n".write(toFile: base + "/live/etc/settings.txt", atomically: true, encoding: .utf8)
try! r.rollback(to: id)
let restored = try! String(contentsOfFile: base + "/live/etc/settings.txt", encoding: .utf8)
check(restored == "voice = on\n", "rollback restores the snapshot")
check(r.snapshots().count >= 2, "rollback first snapshots the current state, so it can be undone")
check((try? r.rollback(to: "missing")) == nil, "unknown snapshot is refused")
print(failures == 0 ? "ALL RECOVERY TESTS PASSED" : "\(failures) FAILED")
