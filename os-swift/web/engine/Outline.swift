// Transforme l'arbre HTML en liste d'éléments accessibles (ce qu'un lecteur d'écran annoncerait).

enum ItemKind: Equatable {
    case heading(Int)
    case link
    case button
    case field(String)     // type de champ lisible, ex. "texte", "mot de passe"
    case image
    case listItem
    case paragraph
    case region(String)    // repère : navigation, contenu principal, formulaire...

    /// Libellé français utilisé par les tests et l'affichage texte.
    var label: String {
        switch self {
        case .heading(let n): return "titre \(n)"
        case .link: return "lien"
        case .button: return "bouton"
        case .field(let t): return "champ \(t)"
        case .image: return "image"
        case .listItem: return "élément de liste"
        case .paragraph: return "paragraphe"
        case .region: return "repère"
        }
    }
}

struct Item: Equatable {
    let kind: ItemKind
    let text: String
    let href: String?

    init(_ kind: ItemKind, _ text: String, href: String? = nil) {
        self.kind = kind
        self.text = text
        self.href = href
    }

    var isLink: Bool { kind == .link }
    var isHeading: Bool {
        if case .heading = kind { return true }
        return false
    }
}

let skipTags: Set<String> = ["head", "script", "style", "noscript", "template", "title",
                             "meta", "link", "svg", "iframe", "object"]

let blockTags: Set<String> = ["p", "div", "section", "article", "blockquote", "pre", "address",
                              "dl", "dt", "dd", "tr", "table", "ul", "ol", "li", "hr", "figure",
                              "figcaption", "header", "footer", "main", "nav", "aside", "body",
                              "html", "center", "fieldset", "legend", "caption", "thead", "tbody",
                              "tfoot", "td", "th", "h1", "h2", "h3", "h4", "h5", "h6"]

let landmarks: [String: String] = [
    "nav": "navigation", "main": "contenu principal", "header": "en-tête",
    "footer": "pied de page", "form": "formulaire", "aside": "complémentaire",
]

let fieldNames: [String: String] = [
    "text": "texte", "email": "courriel", "password": "mot de passe", "search": "recherche",
    "number": "nombre", "tel": "téléphone", "url": "adresse", "date": "date",
    "checkbox": "case à cocher", "radio": "bouton radio", "file": "fichier",
]

func normalize(_ s: String) -> String {
    s.split(whereSeparator: { $0.isWhitespace }).joined(separator: " ")
}

func isHidden(_ n: Node) -> Bool {
    if n.attr("hidden") != nil || n.attr("aria-hidden") == "true" { return true }
    if let style = n.attr("style")?.lowercased() {
        let compact = String(style.filter { $0 != " " })
        if compact.contains("display:none") { return true }
    }
    return false
}

func headingLevel(_ tag: String) -> Int? {
    guard tag.count == 2, tag.hasPrefix("h"), let d = Int(tag.dropFirst()), (1...6).contains(d) else {
        return nil
    }
    return d
}

/// Texte visible d'un nœud (sans scripts, styles, éléments cachés).
func collectText(_ n: Node) -> String {
    if n.isText { return n.ownText }
    guard let tag = n.tag else { return n.children.map(collectText).joined() }
    if skipTags.contains(tag) || isHidden(n) { return "" }
    if tag == "img" { return " " + (n.attr("alt") ?? "") + " " }
    if tag == "br" { return " " }
    let inner = n.children.map(collectText).joined()
    return blockTags.contains(tag) ? " " + inner + " " : inner
}

func pageTitle(_ doc: Node) -> String {
    func find(_ n: Node) -> String? {
        if n.tag == "title" { return normalize(n.children.map { $0.ownText }.joined()) }
        for c in n.children { if let t = find(c) { return t } }
        return nil
    }
    let t = find(doc) ?? ""
    return t.isEmpty ? "sans titre" : t
}

final class OutlineBuilder {
    private(set) var items: [Item] = []
    private var buf = ""
    private var bufKind: ItemKind = .paragraph
    private var labels: [String: String] = [:]
    private let base: WebURL?

    init(base: WebURL?) { self.base = base }

    func build(_ doc: Node) -> [Item] {
        collectLabels(doc)
        visit(doc)
        flush()
        return items
    }

    private func collectLabels(_ n: Node) {
        if n.tag == "label", let f = n.attr("for") {
            labels[f] = normalize(collectText(n))
        }
        for c in n.children { collectLabels(c) }
    }

    private func flush() {
        let t = normalize(buf)
        buf = ""
        if !t.isEmpty { items.append(Item(bufKind, t)) }
    }

    private func add(_ kind: ItemKind, _ text: String, href: String? = nil) {
        flush()
        items.append(Item(kind, text, href: href))
    }

    private func resolve(_ href: String) -> String? {
        guard let b = base else { return WebURL(href)?.string }
        return b.resolve(href)?.string
    }

    private func controlName(_ n: Node, fallback: String) -> String {
        let t = normalize(collectText(n))
        if !t.isEmpty { return t }
        for key in ["aria-label", "title", "value"] {
            if let v = n.attr(key), !normalize(v).isEmpty { return normalize(v) }
        }
        return fallback
    }

    private func fieldLabel(_ n: Node) -> String {
        if let id = n.attr("id"), let l = labels[id], !l.isEmpty { return l }
        for key in ["aria-label", "placeholder", "title", "name"] {
            if let v = n.attr(key), !normalize(v).isEmpty { return normalize(v) }
        }
        return "sans nom"
    }

    private func visit(_ n: Node) {
        if n.isText {
            buf += n.ownText
            return
        }
        guard let tag = n.tag else {
            for c in n.children { visit(c) }
            return
        }
        if skipTags.contains(tag) || isHidden(n) { return }
        // Un label rattaché à un champ devient le nom du champ, pas un paragraphe.
        if tag == "label", n.attr("for") != nil { return }

        if let level = headingLevel(tag) {
            let t = normalize(collectText(n))
            add(.heading(level), t.isEmpty ? "titre vide" : t)
            return
        }

        switch tag {
        case "a":
            if let href = n.attr("href") {
                // Lien hors web (mailto, fragment) : href nil, il reste annoncé mais ne se suit pas.
                add(.link, controlName(n, fallback: "lien sans nom"), href: resolve(href))
            } else {
                for c in n.children { visit(c) }
            }
        case "button":
            add(.button, controlName(n, fallback: "bouton sans nom"))
        case "input":
            let type = (n.attr("type") ?? "text").lowercased()
            switch type {
            case "hidden":
                return
            case "submit", "button", "reset", "image":
                add(.button, controlName(n, fallback: "bouton sans nom"))
            case "checkbox", "radio":
                let state = n.attr("checked") != nil ? "coché" : "non coché"
                add(.field(fieldNames[type] ?? type), fieldLabel(n) + ", " + state)
            default:
                add(.field(fieldNames[type] ?? "texte"), fieldLabel(n))
            }
        case "textarea":
            add(.field("texte"), fieldLabel(n))
        case "select":
            add(.field("liste déroulante"), fieldLabel(n))
        case "img":
            let alt = normalize(n.attr("alt") ?? "")
            add(.image, alt.isEmpty ? "image sans description" : alt)
        case "li":
            flush()
            let saved = bufKind
            bufKind = .listItem
            for c in n.children { visit(c) }
            flush()
            bufKind = saved
        case "br":
            buf += " "
        default:
            if let land = landmarks[tag] {
                add(.region(land), land)
                for c in n.children { visit(c) }
                flush()
                return
            }
            let block = blockTags.contains(tag)
            if block { flush() }
            for c in n.children { visit(c) }
            if block { flush() }
        }
    }
}
