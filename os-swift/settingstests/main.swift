import Foundation
var failures = 0
func check(_ ok: Bool, _ name: String) { print((ok ? "PASS " : "FAIL ") + name); if !ok { failures += 1 } }

let path = "settingstests/settings.txt"
try? FileManager.default.removeItem(atPath: path)
let e = SettingsEngine(path: path)

check(e.get("voice") == "off", "defaults are safe: voice off")
check(e.get("confirm-destructive") == "on", "defaults are safe: confirmation on")
check(e.search("voice").map { $0.key }.contains("voice"), "search finds voice")
check(e.search("voice speed").map { $0.key } == ["voice-speed"], "search needs every word")
check(e.settings(in: .accessibility).first?.key == "voice", "accessibility is listed first")

_ = try? e.set("voice", "on")
check(e.get("voice") == "on", "set changes a value")
check((try? e.set("voice", "loud")) == nil, "invalid value is refused")
check((try? e.set("nope", "on")) == nil, "unknown key is refused")
_ = e.undo()
check(e.get("voice") == "off", "undo restores the previous value")
check(e.log.count == 2, "undo itself is logged")

_ = try? e.cycle("voice-speed")
check(e.get("voice-speed") == "slow", "cycle moves to next value")
e.reset(.accessibility)
check(e.get("voice-speed") == "normal", "reset restores category defaults")
check(e.get("confirm-destructive") == "on", "reset of one category leaves others alone")

_ = try? e.set("confirm-destructive", "off")
e.reset()
check(e.get("confirm-destructive") == "on", "reset all restores everything")

// Persistence: a new engine reads the saved file
_ = try? e.set("braille", "off")
let again = SettingsEngine(path: path)
check(again.get("braille") == "off", "values persist across restarts")
print(failures == 0 ? "ALL SETTINGS TESTS PASSED" : "\(failures) FAILED")
