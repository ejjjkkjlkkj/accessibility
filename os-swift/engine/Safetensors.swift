// Lecteur safetensors : en-tête JSON + données brutes. Convertit F32, F16 et BF16 en Float32.
import Foundation

struct SafetensorInfo {
    let name: String
    let dtype: String
    let shape: [Int]
    let start: Int   // offset absolu dans le fichier
    let end: Int
}

final class SafetensorsFile {
    let data: Data
    let tensors: [String: SafetensorInfo]

    init(path: String) throws {
        let d = try Data(contentsOf: URL(fileURLWithPath: path))
        guard d.count > 8 else { throw NSError(domain: "safetensors", code: 1) }
        let headerLen = d.subdata(in: 0..<8).withUnsafeBytes { $0.loadUnaligned(as: UInt64.self) }
        let headerEnd = 8 + Int(headerLen)
        let headerData = d.subdata(in: 8..<headerEnd)
        guard let json = try JSONSerialization.jsonObject(with: headerData) as? [String: Any] else {
            throw NSError(domain: "safetensors", code: 2)
        }
        let base = headerEnd
        var map: [String: SafetensorInfo] = [:]
        for (name, value) in json where name != "__metadata__" {
            guard let entry = value as? [String: Any],
                  let dtype = entry["dtype"] as? String,
                  let shape = entry["shape"] as? [Int],
                  let offs = entry["data_offsets"] as? [Int], offs.count == 2 else { continue }
            map[name] = SafetensorInfo(name: name, dtype: dtype, shape: shape,
                                       start: base + offs[0], end: base + offs[1])
        }
        self.data = d
        self.tensors = map
    }

    /// Lit un tenseur et le convertit en [Float].
    func floats(_ name: String) -> [Float]? {
        guard let info = tensors[name] else { return nil }
        let bytes = data.subdata(in: info.start..<info.end)
        switch info.dtype {
        case "F32":
            return bytes.withUnsafeBytes { Array($0.bindMemory(to: Float.self)) }
        case "BF16":
            // BF16 = 16 bits hauts d'un Float32
            var out = [Float](repeating: 0, count: bytes.count / 2)
            bytes.withUnsafeBytes { raw in
                let u16 = raw.bindMemory(to: UInt16.self)
                for i in 0..<out.count {
                    out[i] = Float(bitPattern: UInt32(u16[i]) << 16)
                }
            }
            return out
        case "F16":
            var out = [Float](repeating: 0, count: bytes.count / 2)
            bytes.withUnsafeBytes { raw in
                let u16 = raw.bindMemory(to: UInt16.self)
                for i in 0..<out.count { out[i] = Float(Float16(bitPattern: u16[i])) }
            }
            return out
        default:
            return nil
        }
    }
}
