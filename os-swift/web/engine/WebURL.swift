// Adresses web (http, https) sans dépendance système : lecture, résolution des liens relatifs.

struct WebURL: Equatable {
    let scheme: String   // "http" ou "https"
    let host: String
    let path: String     // commence par "/", peut contenir "?requête", sans fragment

    var string: String { "\(scheme)://\(host)\(path)" }

    /// Adresse absolue http(s). Ajoute "https://" si aucun schéma n'est donné.
    init?(_ raw: String) {
        var text = trimSpaces(raw)
        if !text.contains("://") { text = "https://" + text }
        guard let sep = text.firstIndex(of: ":") else { return nil }
        let scheme = text[text.startIndex..<sep].lowercased()
        guard scheme == "http" || scheme == "https" else { return nil }
        let rest = text[text.index(sep, offsetBy: 3)...]
        self.init(scheme: scheme, rest: rest)
    }

    private init?(scheme: String, rest: Substring) {
        let withoutFragment = rest.prefix { $0 != "#" }
        let slash = withoutFragment.firstIndex(of: "/")
        let question = withoutFragment.firstIndex(of: "?")
        let hostEnd = [slash, question].compactMap { $0 }.min() ?? withoutFragment.endIndex
        let host = withoutFragment[withoutFragment.startIndex..<hostEnd].lowercased()
        guard !host.isEmpty else { return nil }
        var path = String(withoutFragment[hostEnd...])
        if path.isEmpty { path = "/" }
        if path.hasPrefix("?") { path = "/" + path }
        self.scheme = scheme
        self.host = host
        self.path = normalizePath(path)
    }

    /// Résout un href trouvé dans la page `self`. Retourne nil pour les schémas non web (mailto:, javascript:...)
    /// et pour les liens qui pointent seulement vers un fragment de la page courante.
    func resolve(_ href: String) -> WebURL? {
        let h = trimSpaces(href)
        if h.isEmpty || h.hasPrefix("#") { return nil }
        if h.contains("://") { return WebURL(h) }
        if h.hasPrefix("//") { return WebURL(scheme + ":" + h) }
        // Schéma inconnu (mailto:, javascript:, tel:) : pas géré par ce navigateur.
        if let colon = h.firstIndex(of: ":"), !h[h.startIndex..<colon].contains("/") { return nil }
        let withoutFragment = String(h.prefix { $0 != "#" })
        if withoutFragment.hasPrefix("/") {
            return WebURL(scheme: scheme, rest: Substring(host + withoutFragment))
        }
        let pagePath = String(path.prefix(while: { $0 != "?" }))
        if withoutFragment.hasPrefix("?") {
            let full: String = host + pagePath + withoutFragment
            return WebURL(scheme: scheme, rest: Substring(full))
        }
        // Chemin relatif : on part du dossier de la page courante.
        var folder = "/"
        if let slash = pagePath.lastIndex(of: "/") {
            folder = String(pagePath[pagePath.startIndex...slash])
        }
        let full: String = host + folder + withoutFragment
        return WebURL(scheme: scheme, rest: Substring(full))
    }
}

private func trimSpaces(_ s: String) -> String {
    var chars = Substring(s)
    while let f = chars.first, f.isWhitespace { chars = chars.dropFirst() }
    while let l = chars.last, l.isWhitespace { chars = chars.dropLast() }
    return String(chars)
}

/// Supprime les segments "." et ".." d'un chemin, en gardant la requête.
private func normalizePath(_ path: String) -> String {
    let queryStart = path.firstIndex(of: "?")
    let pathPart = queryStart.map { String(path[path.startIndex..<$0]) } ?? path
    let query = queryStart.map { String(path[$0...]) } ?? ""
    var segments: [Substring] = []
    let parts = pathPart.split(separator: "/", omittingEmptySubsequences: false)
    for (i, seg) in parts.enumerated() {
        if seg == "." { continue }
        if seg == ".." {
            if !segments.isEmpty { segments.removeLast() }
            continue
        }
        if seg.isEmpty && i != 0 && i != parts.count - 1 { continue }
        segments.append(seg)
    }
    var result = segments.joined(separator: "/")
    if !result.hasPrefix("/") { result = "/" + result }
    if pathPart.hasSuffix("/") && !result.hasSuffix("/") { result += "/" }
    return result + query
}
