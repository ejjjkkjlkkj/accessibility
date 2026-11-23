import Foundation
let p = "tests/boot/graine/noyau.bin"
print("exists:", FileManager.default.fileExists(atPath: p))
print("cwd:", FileManager.default.currentDirectoryPath)
print("read:", (try? Data(contentsOf: URL(fileURLWithPath: p)))?.count ?? -1)
