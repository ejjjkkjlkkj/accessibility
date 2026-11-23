// Analyse HTML sans dépendance : lexeur tolérant puis arbre DOM simplifié.

final class Node {
    enum Kind {
        case document
        case element(String)
        case text(String)
    }

    let kind: Kind
    var attrs: [String: String] = [:]
    var children: [Node] = []

    init(_ kind: Kind) { self.kind = kind }

    var tag: String? {
        if case .element(let t) = kind { return t }
        return nil
    }

    var isText: Bool {
        if case .text = kind { return true }
        return false
    }

    var ownText: String {
        if case .text(let t) = kind { return t }
        return ""
    }

    func attr(_ name: String) -> String? { attrs[name] }
}

enum Token {
    case open(String, [String: String], selfClosing: Bool)
    case close(String)
    case text(String)
}

let voidTags: Set<String> = ["area", "base", "br", "col", "embed", "hr", "img", "input",
                             "link", "meta", "param", "source", "track", "wbr"]
let rawTextTags: Set<String> = ["script", "style"]

// MARK: - Entités

private let namedEntities: [String: String] = [
    "amp": "&", "lt": "<", "gt": ">", "quot": "\"", "apos": "'", "nbsp": "\u{00A0}",
    "copy": "©", "reg": "®", "hellip": "…", "mdash": "—", "ndash": "–",
    "laquo": "«", "raquo": "»", "lsquo": "‘", "rsquo": "’", "ldquo": "“", "rdquo": "”",
    "eacute": "é", "egrave": "è", "ecirc": "ê", "agrave": "à", "acirc": "â",
    "ccedil": "ç", "ocirc": "ô", "ucirc": "û", "ugrave": "ù", "iuml": "ï",
    "euro": "€", "deg": "°", "times": "×", "middot": "·", "bull": "•",
]

func decodeEntities(_ s: String) -> String {
    guard s.contains("&") else { return s }
    let it = Array(s.unicodeScalars)
    var out = ""
    var i = 0
    while i < it.count {
        if it[i] == "&" {
            let end = min(it.count, i + 12)
            var k = i + 1
            while k < end && it[k] != ";" { k += 1 }
            if k < end {
                var name = ""
                name.unicodeScalars.append(contentsOf: it[(i + 1)..<k])
                if let rep = resolveEntity(name) {
                    out += rep
                    i = k + 1
                    continue
                }
            }
        }
        out.unicodeScalars.append(it[i])
        i += 1
    }
    return out
}

private func resolveEntity(_ name: String) -> String? {
    if name.hasPrefix("#x") || name.hasPrefix("#X") {
        guard let v = UInt32(name.dropFirst(2), radix: 16), let u = Unicode.Scalar(v) else { return nil }
        return String(Character(u))
    }
    if name.hasPrefix("#") {
        guard let v = UInt32(name.dropFirst(1)), let u = Unicode.Scalar(v) else { return nil }
        return String(Character(u))
    }
    return namedEntities[name]
}

// MARK: - Lexeur

private func isSpace(_ c: Unicode.Scalar) -> Bool {
    c == " " || c == "\n" || c == "\t" || c == "\r" || c == "\u{0C}"
}

private func isAlpha(_ c: Unicode.Scalar) -> Bool {
    (c >= "a" && c <= "z") || (c >= "A" && c <= "Z")
}

private func lowerScalar(_ c: Unicode.Scalar) -> Unicode.Scalar {
    if c >= "A" && c <= "Z" { return Unicode.Scalar(c.value + 32)! }
    return c
}

private func matchesAt(_ c: [Unicode.Scalar], _ at: Int, _ pat: String, caseInsensitive: Bool = false) -> Bool {
    var k = at
    for p in pat.unicodeScalars {
        guard k < c.count else { return false }
        let a = caseInsensitive ? lowerScalar(c[k]) : c[k]
        if a != p { return false }
        k += 1
    }
    return true
}

private func findPattern(_ c: [Unicode.Scalar], from: Int, _ pat: String) -> Int? {
    var k = from
    while k < c.count {
        if matchesAt(c, k, pat) { return k }
        k += 1
    }
    return nil
}

private func readName(_ c: [Unicode.Scalar], _ from: Int) -> (String, Int) {
    var name = ""
    var j = from
    while j < c.count, !isSpace(c[j]), c[j] != ">", c[j] != "/" {
        name.unicodeScalars.append(lowerScalar(c[j]))
        j += 1
    }
    return (name, j)
}

enum Tokenizer {
    static func tokenize(_ html: String) -> [Token] {
        let c = Array(html.unicodeScalars)
        let n = c.count
        var out: [Token] = []
        var text = ""
        var i = 0

        func flush() {
            if !text.isEmpty {
                out.append(.text(decodeEntities(text)))
                text = ""
            }
        }

        while i < n {
            let ch = c[i]
            guard ch == "<" else {
                text.unicodeScalars.append(ch)
                i += 1
                continue
            }
            if matchesAt(c, i, "<!--") {
                flush()
                if let e = findPattern(c, from: i + 4, "-->") { i = e + 3 } else { i = n }
                continue
            }
            if i + 1 < n && (c[i + 1] == "!" || c[i + 1] == "?") {
                flush()
                if let e = findPattern(c, from: i + 1, ">") { i = e + 1 } else { i = n }
                continue
            }
            if i + 2 < n && c[i + 1] == "/" && isAlpha(c[i + 2]) {
                flush()
                let (name, after) = readName(c, i + 2)
                var j = after
                while j < n && c[j] != ">" { j += 1 }
                out.append(.close(name))
                i = min(j + 1, n)
                continue
            }
            if i + 1 < n && isAlpha(c[i + 1]) {
                flush()
                let (name, after) = readName(c, i + 1)
                var attrs: [String: String] = [:]
                var selfClosing = false
                var j = after
                while j < n {
                    if isSpace(c[j]) { j += 1; continue }
                    if c[j] == ">" { j += 1; break }
                    if c[j] == "/" { selfClosing = true; j += 1; continue }
                    var an = ""
                    while j < n, !isSpace(c[j]), c[j] != "=", c[j] != ">", c[j] != "/" {
                        an.unicodeScalars.append(lowerScalar(c[j]))
                        j += 1
                    }
                    while j < n && isSpace(c[j]) { j += 1 }
                    var av = ""
                    if j < n && c[j] == "=" {
                        j += 1
                        while j < n && isSpace(c[j]) { j += 1 }
                        if j < n && (c[j] == "\"" || c[j] == "'") {
                            let q = c[j]
                            j += 1
                            while j < n && c[j] != q {
                                av.unicodeScalars.append(c[j])
                                j += 1
                            }
                            j += 1
                        } else {
                            while j < n && !isSpace(c[j]) && c[j] != ">" {
                                av.unicodeScalars.append(c[j])
                                j += 1
                            }
                        }
                    }
                    if !an.isEmpty && attrs[an] == nil {
                        attrs[an] = decodeEntities(av)
                    }
                }
                out.append(.open(name, attrs, selfClosing: selfClosing || voidTags.contains(name)))
                i = min(j, n)
                if rawTextTags.contains(name) && !selfClosing {
                    var close = i
                    while close < n {
                        if c[close] == "<" && close + 1 < n && c[close + 1] == "/"
                            && matchesAt(c, close + 2, name, caseInsensitive: true) {
                            break
                        }
                        close += 1
                    }
                    var raw = ""
                    raw.unicodeScalars.append(contentsOf: c[i..<close])
                    if !raw.isEmpty { out.append(.text(raw)) }
                    i = close
                }
                continue
            }
            text.unicodeScalars.append(ch)
            i += 1
        }
        flush()
        return out
    }
}

// MARK: - Arbre

enum HTMLParser {
    private static let blockStarters: Set<String> = [
        "p", "div", "ul", "ol", "h1", "h2", "h3", "h4", "h5", "h6", "table", "section",
        "article", "header", "footer", "nav", "main", "blockquote", "pre", "form", "hr",
        "address", "dl", "aside", "fieldset",
    ]

    static func parse(_ html: String) -> Node {
        let doc = Node(.document)
        var stack: [Node] = [doc]
        for tok in Tokenizer.tokenize(html) {
            switch tok {
            case .text(let t):
                let node = Node(.text(t))
                stack[stack.count - 1].children.append(node)
            case .open(let name, let attrs, let selfClosing):
                closeImplicit(name, &stack)
                let node = Node(.element(name))
                node.attrs = attrs
                stack[stack.count - 1].children.append(node)
                if !selfClosing { stack.append(node) }
            case .close(let name):
                if let idx = stack.lastIndex(where: { $0.tag == name }), idx > 0 {
                    stack.removeSubrange(idx...)
                }
            }
        }
        return doc
    }

    /// Ferme les éléments qu'un nouvel élément ferme implicitement (p, li, option, td...).
    private static func closeImplicit(_ name: String, _ stack: inout [Node]) {
        if name == "li" {
            if let idx = stack.lastIndex(where: { $0.tag == "li" }),
               !stack[idx...].contains(where: { $0.tag == "ul" || $0.tag == "ol" }) {
                stack.removeSubrange(idx...)
            }
            return
        }
        if blockStarters.contains(name), stack.count > 1, stack[stack.count - 1].tag == "p" {
            stack.removeLast()
        }
        if name == "td" || name == "th" || name == "tr" {
            while stack.count > 1, let t = stack[stack.count - 1].tag, ["td", "th"].contains(t) || (name == "tr" && t == "tr") {
                stack.removeLast()
            }
        }
        if name == "option", stack.count > 1, stack[stack.count - 1].tag == "option" {
            stack.removeLast()
        }
    }
}
