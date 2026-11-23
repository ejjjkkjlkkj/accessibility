import Foundation
// Scénarios du chargeur : démarrage normal, noyau falsifié, clé kernel absente -> secours.

let fm = FileManager.default
let base = "tests/scenario"
let bs = String(UnicodeScalar(92))              // caractère antislash, sans échappement
let kernelName = bs + "graine" + bs + "noyau.bin"
let fallbackName = bs + "secours" + bs + "env.bin"

func fresh(_ name: String) throws -> String {
    let dir = base + "/" + name
    try? fm.removeItem(atPath: dir)
    try fm.createDirectory(atPath: dir + "/journal", withIntermediateDirectories: true)
    for rel in ["graine/noyau.bin", "secours/env.bin"] {
        let dst = dir + "/" + rel
        try fm.createDirectory(atPath: (dst as NSString).deletingLastPathComponent, withIntermediateDirectories: true)
        try fm.copyItem(atPath: "tests/boot/" + rel, toPath: dst)
    }
    try fm.copyItem(atPath: "tests/boot.gbs", toPath: dir + "/boot.gbs")
    try fm.copyItem(atPath: "tests/manifest.gbs", toPath: dir + "/manifeste.gbs")
    return dir
}

var failures = 0
func check(_ name: String, _ got: BootOutcome, _ want: BootOutcome) {
    let ok = got == want
    if !ok { failures += 1 }
    print((ok ? "OK    " : "ECHEC ") + name + " -> \(got)")
}

// 1. Démarrage normal
let d1 = try fresh("normal")
check("démarrage normal", BootLoader(root: d1).boot(), .kernel(kernelName))

// 2. Noyau falsifié : la vérification échoue, retour sur secours
let d2 = try fresh("falsifie")
try Data([0x00]).write(to: URL(fileURLWithPath: d2 + "/graine/noyau.bin"))
check("noyau falsifié -> secours", BootLoader(root: d2).boot(), .fallback(fallbackName))

// 3. Clé kernel absente de boot.gbs : retour sur secours
let d3 = try fresh("sans_noyau")
let cfgText = try String(contentsOfFile: "tests/boot.gbs", encoding: .utf8)
let cfgNoKernel = cfgText.replacingOccurrences(of: "kernel ", with: "absent ")
try cfgNoKernel.write(toFile: d3 + "/boot.gbs", atomically: true, encoding: .utf8)
check("clé kernel absente -> secours", BootLoader(root: d3).boot(), .fallback(fallbackName))

// 4. Journal du démarrage normal
let log = try String(contentsOfFile: d1 + "/journal/boot.log", encoding: .utf8)
print("journal (normal) :\n" + log)
print(failures == 0 ? "CHARGEUR VERIFIE" : "CHARGEUR ECHEC (\(failures))")
