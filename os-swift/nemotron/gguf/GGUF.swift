// Lecteur GGUF sans dépendance (Foundation seulement pour les fichiers).
// Format : en-tête, métadonnées clé/valeur, table des tenseurs, données alignées.
import Foundation

enum GGUFError: Error, CustomStringConvertible {
    case badMagic
    case unsupportedVersion(UInt32)
    case unknownValueType(UInt32)
    case unknownTensorType(UInt32)
    case truncated(String)

    var description: String {
        switch self {
        case .badMagic: return "pas un fichier GGUF (magie absente)"
        case .unsupportedVersion(let v): return "version GGUF non gérée : \(v)"
        case .unknownValueType(let t): return "type de valeur de métadonnée inconnu : \(t)"
        case .unknownTensorType(let t): return "type de tenseur inconnu : \(t)"
        case .truncated(let what): return "fichier tronqué pendant la lecture de \(what)"
        }
    }
}

/// Types de valeurs des métadonnées (spécification GGUF).
enum GGUFValue {
    case u8(UInt8), i8(Int8), u16(UInt16), i16(Int16), u32(UInt32), i32(Int32)
    case f32(Float), bool(Bool), string(String), u64(UInt64), i64(Int64), f64(Double)
    case array([GGUFValue])

    var asInt: Int? {
        switch self {
        case .u8(let v): return Int(v)
        case .i8(let v): return Int(v)
        case .u16(let v): return Int(v)
        case .i16(let v): return Int(v)
        case .u32(let v): return Int(v)
        case .i32(let v): return Int(v)
        case .u64(let v): return Int(v)
        case .i64(let v): return Int(v)
        default: return nil
        }
    }
    var asFloat: Double? {
        switch self {
        case .f32(let v): return Double(v)
        case .f64(let v): return v
        default: return asInt.map(Double.init)
        }
    }
    var asString: String? { if case .string(let s) = self { return s }; return nil }
}

/// Types de tenseurs (identifiants ggml).
enum GGMLType: UInt32 {
    case f32 = 0, f16 = 1, q4_0 = 2, q4_1 = 3, q5_0 = 6, q5_1 = 7, q8_0 = 8, q8_1 = 9
    case q2_K = 10, q3_K = 11, q4_K = 12, q5_K = 13, q6_K = 14, q8_K = 15
    case bf16 = 30

    /// Nombre d'éléments par bloc et octets par bloc.
    var blockInfo: (elements: Int, bytes: Int) {
        switch self {
        case .f32: return (1, 4)
        case .f16, .bf16: return (1, 2)
        case .q4_0: return (32, 18)
        case .q4_1: return (32, 20)
        case .q5_0: return (32, 22)
        case .q5_1: return (32, 24)
        case .q8_0: return (32, 34)
        case .q8_1: return (32, 36)
        case .q2_K: return (256, 84)
        case .q3_K: return (256, 110)
        case .q4_K: return (256, 144)
        case .q5_K: return (256, 176)
        case .q6_K: return (256, 210)
        case .q8_K: return (256, 292)
        }
    }
}

struct TensorInfo {
    let name: String
    /// Dimensions dans l'ordre GGUF (la première varie le plus vite).
    let dims: [Int]
    let type: GGMLType
    /// Décalage depuis le début de la zone de données.
    let offset: UInt64

    var elementCount: Int { dims.reduce(1, *) }
    /// Taille en octets dans le fichier.
    var byteCount: Int {
        let info = type.blockInfo
        return elementCount / info.elements * info.bytes
    }
}

/// Lecteur séquentiel sur un FileHandle, avec vérification de troncature.
struct ByteReader {
    let handle: FileHandle
    var position: UInt64

    init(handle: FileHandle, position: UInt64 = 0) {
        self.handle = handle
        self.position = position
        handle.seek(toFileOffset: position)
    }

    mutating func bytes(_ n: Int, _ what: String) throws -> Data {
        handle.seek(toFileOffset: position)
        let d = handle.readData(ofLength: n)
        guard d.count == n else { throw GGUFError.truncated(what) }
        position += UInt64(n)
        return d
    }

    mutating func u8(_ what: String = "u8") throws -> UInt8 { try bytes(1, what)[0] }
    mutating func u16(_ what: String = "u16") throws -> UInt16 { try bytes(2, what).withUnsafeBytes { $0.loadUnaligned(as: UInt16.self) } }
    mutating func u32(_ what: String = "u32") throws -> UInt32 { try bytes(4, what).withUnsafeBytes { $0.loadUnaligned(as: UInt32.self) } }
    mutating func u64(_ what: String = "u64") throws -> UInt64 { try bytes(8, what).withUnsafeBytes { $0.loadUnaligned(as: UInt64.self) } }

    mutating func string(_ what: String = "chaîne") throws -> String {
        let n = Int(try u64(what))
        let d = try bytes(n, what)
        return String(decoding: d, as: UTF8.self)
    }
}

/// Fichier GGUF chargé : métadonnées, tenseurs, et position des données.
final class GGUFFile {
    let path: String
    let version: UInt32
    private(set) var metadata: [String: GGUFValue] = [:]
    private(set) var tensors: [TensorInfo] = []
    /// Octet du fichier où commence la zone de données des tenseurs.
    private(set) var dataStart: UInt64 = 0
    let handle: FileHandle

    init(path: String) throws {
        self.path = path
        guard let h = FileHandle(forReadingAtPath: path) else {
            throw GGUFError.truncated("ouverture de \(path)")
        }
        self.handle = h
        var r = ByteReader(handle: h)

        let magic = try r.bytes(4, "magie")
        guard magic == Data("GGUF".utf8) else { throw GGUFError.badMagic }
        version = try r.u32("version")
        guard version == 2 || version == 3 else { throw GGUFError.unsupportedVersion(version) }

        let tensorCount = try r.u64("nombre de tenseurs")
        let kvCount = try r.u64("nombre de métadonnées")

        for _ in 0..<kvCount {
            let key = try r.string("clé de métadonnée")
            let value = try Self.readValue(&r, type: try r.u32("type de valeur"))
            metadata[key] = value
        }

        for _ in 0..<tensorCount {
            let name = try r.string("nom de tenseur")
            let nDims = Int(try r.u32("nombre de dimensions"))
            var dims: [Int] = []
            for _ in 0..<nDims { dims.append(Int(try r.u64("dimension"))) }
            let rawType = try r.u32("type de tenseur")
            guard let type = GGMLType(rawType: rawType) else { throw GGUFError.unknownTensorType(rawType) }
            let offset = try r.u64("décalage de tenseur")
            tensors.append(TensorInfo(name: name, dims: dims, type: type, offset: offset))
        }

        let alignment = metadata["general.alignment"]?.asInt ?? 32
        let end = r.position
        let aligned = (end + UInt64(alignment) - 1) / UInt64(alignment) * UInt64(alignment)
        dataStart = aligned
    }

    /// Lit une valeur de métadonnée selon son type.
    static func readValue(_ r: inout ByteReader, type: UInt32) throws -> GGUFValue {
        switch type {
        case 0: return .u8(try r.u8())
        case 1: return .i8(Int8(bitPattern: try r.u8()))
        case 2: return .u16(try r.u16())
        case 3: return .i16(Int16(bitPattern: try r.u16()))
        case 4: return .u32(try r.u32())
        case 5: return .i32(Int32(bitPattern: try r.u32()))
        case 6: return .f32(Float(bitPattern: try r.u32()))
        case 7: return .bool(try r.u8() != 0)
        case 8: return .string(try r.string())
        case 9:
            let elemType = try r.u32()
            let count = Int(try r.u64())
            var items: [GGUFValue] = []
            items.reserveCapacity(count)
            for _ in 0..<count { items.append(try readValue(&r, type: elemType)) }
            return .array(items)
        case 10: return .u64(try r.u64())
        case 11: return .i64(Int64(bitPattern: try r.u64()))
        case 12: return .f64(Double(bitPattern: try r.u64()))
        default: throw GGUFError.unknownValueType(type)
        }
    }

    func tensor(named name: String) -> TensorInfo? { tensors.first { $0.name == name } }

    /// Lit les octets bruts d'un tenseur, tels qu'ils sont dans le fichier.
    func rawBytes(of t: TensorInfo) throws -> Data {
        handle.seek(toFileOffset: dataStart + t.offset)
        let d = handle.readData(ofLength: t.byteCount)
        guard d.count == t.byteCount else { throw GGUFError.truncated("données de \(t.name)") }
        return d
    }
}

extension GGMLType {
    init?(rawType: UInt32) {
        // Valeurs non contiguës : on les déclare explicitement.
        let table: [UInt32: GGMLType] = [
            0: .f32, 1: .f16, 2: .q4_0, 3: .q4_1, 6: .q5_0, 7: .q5_1, 8: .q8_0, 9: .q8_1,
            10: .q2_K, 11: .q3_K, 12: .q4_K, 13: .q5_K, 14: .q6_K, 15: .q8_K, 30: .bf16,
        ]
        guard let t = table[rawType] else { return nil }
        self = t
    }
}
