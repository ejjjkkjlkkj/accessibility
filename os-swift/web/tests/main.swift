// Tests du navigateur intégré au noyau. Réseau simulé : aucun accès internet.

var passes = 0
var failures = 0

func check(_ cond: Bool, _ name: String, _ detail: @autoclosure () -> String = "") {
    if cond {
        passes += 1
        print("PASS \(name)")
    } else {
        failures += 1
        print("FAIL \(name) \(detail())")
    }
}

/// Canal de sortie qui garde les annonces, pour les comparer dans les tests.
final class CaptureChannel: OutputChannel {
    let name = "capture"
    var lines: [String] = []
    func emit(_ message: Message) { lines.append(message.text) }
}

/// Faux réseau : une table adresse -> réponse.
final class FakeTransport: WebTransport {
    var pages: [String: WebResponse] = [:]
    func get(_ url: WebURL) -> Result<WebResponse, WebError> {
        if let r = pages[url.string] { return .success(r) }
        return .failure(.http(404))
    }
}

func html(_ s: String) -> WebResponse {
    WebResponse(status: 200, contentType: "text/html; charset=utf-8", body: Array(s.utf8))
}

func fmtItems(_ items: [Item]) -> [String] {
    items.map { "\($0.kind.label)|\($0.text)|\($0.href ?? "")" }
}

func outline(_ s: String, base: String = "https://example.com/dir/page.html") -> [Item] {
    OutlineBuilder(base: WebURL(base)).build(HTMLParser.parse(s))
}

// MARK: - Adresses

let base = WebURL("https://example.com/dir/page.html")!
check(base.resolve("p2.html")?.string == "https://example.com/dir/p2.html", "lien relatif au dossier")
check(base.resolve("/a")?.string == "https://example.com/a", "lien absolu au chemin")
check(base.resolve("../x/y.html")?.string == "https://example.com/x/y.html", "remontée ../")
check(base.resolve("//cdn.x.org/s.js")?.string == "https://cdn.x.org/s.js", "lien protocole-relatif")
check(base.resolve("?q=2")?.string == "https://example.com/dir/page.html?q=2", "lien de requête seule")
check(base.resolve("mailto:a@b.c") == nil, "mailto ignoré")
check(base.resolve("#haut") == nil, "ancre seule ignorée")
check(WebURL("ftp://x.org/") == nil, "schéma ftp refusé")
check(WebURL("Example.ORG")?.string == "https://example.org/", "sans schéma : https ajouté, hôte en minuscules")
check(decodeBody([0x63, 0xE9], contentType: "text/html; charset=iso-8859-1") == "cé", "ISO-8859-1 décodé")

// MARK: - Plan accessible

let sample = """
<html><head><title>Accueil test</title></head><body>
<h1>Accueil</h1><p>Texte <a href="/b.html">Suivant</a></p><h2>Fin</h2>
</body></html>
"""
check(fmtItems(outline(sample)) == [
    "titre 1|Accueil|",
    "paragraphe|Texte|",
    "lien|Suivant|https://example.com/b.html",
    "titre 2|Fin|",
], "plan de la page de test", "\(fmtItems(outline(sample)))")

// MARK: - Navigation par le noyau

let net = FakeTransport()
net.pages["https://site.test/"] = html(sample)
net.pages["https://site.test/b.html"] = html("<title>Page B</title><h1>Page B</h1>")

let bus = OutputBus()
let capture = CaptureChannel()
bus.attach(capture)
let browser = Browser(bus: bus, transport: net)

check(browser.open("https://site.test/"), "ouverture de la page")
check(capture.lines.contains("window: Accueil test."), "la page est annoncée", "\(capture.lines)")
check(capture.lines.last == "heading: Accueil.", "premier élément annoncé", "\(capture.lines.last ?? "")")

capture.lines = []
browser.press("h")
check(capture.lines.last == "heading: Fin.", "h saute au titre suivant", "\(capture.lines)")

capture.lines = []
browser.press("H")
check(capture.lines.last == "heading: Accueil.", "H revient au titre précédent", "\(capture.lines)")

capture.lines = []
browser.press("l")
check(capture.lines.last?.hasPrefix("link: Suivant.") == true, "l saute au lien", "\(capture.lines)")

capture.lines = []
browser.press("tab")
check(capture.lines.last == "heading: Fin.", "Tab passe à l'élément suivant", "\(capture.lines)")

capture.lines = []
browser.press("shift-tab")
check(capture.lines.last?.hasPrefix("link: Suivant.") == true, "Maj-Tab revient en arrière", "\(capture.lines)")

capture.lines = []
browser.press("enter")
check(browser.currentURL?.string == "https://site.test/b.html", "Entrée suit le lien", "\(String(describing: browser.currentURL))")
check(capture.lines.contains("window: Page B."), "la page liée est annoncée", "\(capture.lines)")

// MARK: - Erreurs

capture.lines = []
check(!browser.open("https://site.test/absente"), "page absente refusée")
check(capture.lines.last == "Could not load https://site.test/absente: server answered 404.",
      "erreur 404 annoncée", "\(capture.lines)")

capture.lines = []
check(!browser.open("ftp://nope"), "adresse non web refusée")
check(capture.lines.last == "Invalid address: ftp://nope.", "adresse invalide annoncée", "\(capture.lines)")

// Titre introuvable dans la page : le plan reste utilisable.
net.pages["https://site.test/vide"] = html("<body></body>")
capture.lines = []
browser.open("https://site.test/vide")
check(capture.lines.contains("This page has no readable content."), "page vide annoncée", "\(capture.lines)")

print("\(passes) PASS, \(failures) FAIL")
if failures == 0 { print("ALL WEB TESTS PASSED") } else { print("WEB TESTS FAILED") }
