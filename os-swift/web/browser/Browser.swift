// Navigateur web du système. Chaque élément de page est un AccessibleObject du noyau :
// il est annoncé par le bus de sortie et se parcourt avec le focus. Aucun appel à l'OS ici :
// le réseau passe par WebTransport, fourni par l'hôte.

struct WebResponse {
    let status: Int
    let contentType: String
    let body: [UInt8]
}

enum WebError: Error {
    case badURL(String)
    case network(String)
    case http(Int)
    case notHTML(String)
}

/// Fournit les pages. Implémenté par l'hôte (aujourd'hui : web/host ; demain : la pile réseau de l'OS).
protocol WebTransport: AnyObject {
    func get(_ url: WebURL) -> Result<WebResponse, WebError>
}

/// Octets -> texte. UTF-8 par défaut, ISO-8859-1 si la page l'annonce.
func decodeBody(_ bytes: [UInt8], contentType: String) -> String {
    if contentType.contains("iso-8859-1") || contentType.contains("latin1") {
        var s = ""
        s.unicodeScalars.append(contentsOf: bytes.map { Unicode.Scalar($0) })
        return s
    }
    return String(decoding: bytes, as: UTF8.self)
}

private func describe(_ e: WebError) -> String {
    switch e {
    case .badURL(let s): return "invalid address \(s)"
    case .network(let s): return "network error \(s)"
    case .http(let c): return "server answered \(c)"
    case .notHTML(let t): return "not an HTML page (\(t))"
    }
}

/// Un élément de page (titre, lien, champ...) vu par le noyau.
final class WebItem: AccessibleObject {
    let item: Item
    weak var browser: Browser?

    init(_ item: Item) { self.item = item }

    var name: String {
        switch item.kind {
        case .field(let type): return "\(item.text) (\(type))"
        default: return item.text
        }
    }

    var role: Role {
        switch item.kind {
        case .heading: return .heading
        case .link: return .link
        case .button: return .button
        case .field: return .field
        case .image: return .image
        case .listItem: return .item
        case .paragraph: return .text
        case .region: return .region
        }
    }

    var state: Set<State> { [.enabled] }

    var actions: [Action] {
        guard item.isLink, item.href != nil else { return [] }
        return [Action(key: "enter", label: "open link", destructive: false)]
    }

    func perform(_ action: Action) -> String {
        guard let href = item.href, let b = browser else { return "" }
        b.open(href)
        return ""
    }
}

/// Une page chargée : conteneur de ses éléments.
final class WebPage: AccessibleObject {
    let title: String
    let items: [WebItem]

    init(title: String, items: [Item]) {
        self.title = title
        self.items = items.map { WebItem($0) }
    }

    var name: String { title }
    var role: Role { .window }
    var state: Set<State> { [.enabled] }
    var actions: [Action] { [] }
    var children: [AccessibleObject] { items.map { $0 as AccessibleObject } }

    func perform(_ action: Action) -> String { "" }
}

final class Browser {
    private let bus: OutputBus
    private let transport: WebTransport
    private(set) var focus: FocusManager
    private var router: KeyRouter
    private(set) var page: WebPage?
    private(set) var currentURL: WebURL?

    init(bus: OutputBus, transport: WebTransport) {
        self.bus = bus
        self.transport = transport
        let f = FocusManager(bus: bus)
        self.focus = f
        self.router = KeyRouter(focus: f, bus: bus)
    }

    /// Ouvre une adresse. Retourne false si la page ne peut pas être chargée.
    @discardableResult
    func open(_ raw: String) -> Bool {
        guard let url = WebURL(raw) else {
            bus.emit("Invalid address: \(raw).", priority: .alert)
            return false
        }
        switch transport.get(url) {
        case .failure(let e):
            bus.emit("Could not load \(url.string): \(describe(e)).", priority: .alert)
            return false
        case .success(let resp):
            let text = decodeBody(resp.body, contentType: resp.contentType.lowercased())
            let doc = HTMLParser.parse(text)
            let items = OutlineBuilder(base: url).build(doc)
            currentURL = url
            install(WebPage(title: pageTitle(doc), items: items))
            return true
        }
    }

    private func install(_ newPage: WebPage) {
        page = newPage
        // Une nouvelle page repart d'un focus neuf : pas de piles d'une page précédente.
        focus = FocusManager(bus: bus)
        router = KeyRouter(focus: focus, bus: bus)
        for it in newPage.items { it.browser = self }
        focus.enter(newPage)
        if let first = newPage.items.first {
            focus.enter(first)
        } else {
            bus.emit("This page has no readable content.")
        }
    }

    /// Touches du navigateur. Tab / Maj-Tab, Entrée, Échap, Ctrl-. sont celles du noyau.
    /// Pour les pages : h / H titre suivant / précédent, l / L lien suivant / précédent.
    func press(_ key: String) {
        switch key {
        case "h": jump(forward: true) { $0.item.isHeading }
        case "H": jump(forward: false) { $0.item.isHeading }
        case "l": jump(forward: true) { $0.item.isLink }
        case "L": jump(forward: false) { $0.item.isLink }
        default: router.press(key)
        }
    }

    private func jump(forward: Bool, _ match: (WebItem) -> Bool) {
        guard let page = page else {
            bus.emit("No page open.", priority: .alert)
            return
        }
        let current = page.items.firstIndex { $0.identity == focus.current?.identity }
        var i = forward ? (current.map { $0 + 1 } ?? 0) : (current.map { $0 - 1 } ?? page.items.count - 1)
        while i >= 0 && i < page.items.count {
            let candidate = page.items[i]
            if match(candidate) {
                if focus.depth >= 2 { focus.jump(to: candidate) } else { focus.enter(candidate) }
                return
            }
            i += forward ? 1 : -1
        }
        bus.emit("Nothing found in that direction.", priority: .alert)
    }
}
