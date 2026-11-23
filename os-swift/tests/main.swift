import Foundation
// Test : SHA-256 de référence, configuration, manifeste OK, puis falsification détectée.
let ref = "d1d4f0f2e23ed9cb1a5f5a8aa6e4f2c2b6f3b8d2e6d4a1c9f5e7b3a2d1c0e9f8"
let sample = Data("abc".utf8)
print("sha256(abc) =", SHA256.hex(of: sample))
print("attendu     = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
print(SHA256.hex(of: sample) == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" ? "SHA256 OK" : "SHA256 ECHEC")

let cfg = try BootConfig(text: try String(contentsOfFile: "tests/boot.gbs", encoding: .utf8))
print("noyau =", cfg.kernel ?? "-", "| retour =", cfg.returnTarget ?? "-", "| délai =", cfg.delaySeconds)

let m = try Manifest(text: try String(contentsOfFile: "tests/manifest.gbs", encoding: .utf8))
do { try m.verify(root: "tests/boot"); print("MANIFESTE OK") } catch { print("MANIFESTE ECHEC:", error) }

// Falsification : on modifie un octet, la vérification doit échouer
let p = "tests/boot/secours/env.bin"
let original = FileManager.default.contents(atPath: p)!
var tampered = original; tampered[0] ^= 0x01
try tampered.write(to: URL(fileURLWithPath: p))
do { try m.verify(root: "tests/boot"); print("FALSIFICATION NON DETECTEE") } catch { print("falsification détectée :", error) }
try original.write(to: URL(fileURLWithPath: p))
