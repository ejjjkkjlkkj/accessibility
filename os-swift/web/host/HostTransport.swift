// Adaptateur réseau de l'hôte (prototype sur Windows). C'est le seul fichier du navigateur
// qui dépend de Foundation / URLSession. Il sera remplacé par la pile réseau de l'OS.
import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

private final class Outcome {
    var data: Data?
    var response: URLResponse?
    var error: Error?
}

final class HostTransport: WebTransport {
    var timeout: TimeInterval = 30

    func get(_ url: WebURL) -> Result<WebResponse, WebError> {
        guard let u = URL(string: url.string) else { return .failure(.badURL(url.string)) }
        var req = URLRequest(url: u, timeoutInterval: timeout)
        req.setValue("GraineWeb/0.1", forHTTPHeaderField: "User-Agent")
        req.setValue("text/html, text/plain;q=0.5", forHTTPHeaderField: "Accept")

        let box = Outcome()
        let done = DispatchSemaphore(value: 0)
        URLSession.shared.dataTask(with: req) { data, resp, err in
            box.data = data
            box.response = resp
            box.error = err
            done.signal()
        }.resume()
        if done.wait(timeout: .now() + timeout + 5) == .timedOut {
            return .failure(.network("timeout"))
        }
        if let err = box.error { return .failure(.network(err.localizedDescription)) }
        guard let http = box.response as? HTTPURLResponse else {
            return .failure(.network("unexpected response"))
        }
        let type = http.value(forHTTPHeaderField: "Content-Type") ?? ""
        let lower = type.lowercased()
        if !lower.isEmpty && !lower.contains("html") && !lower.contains("text/plain") {
            return .failure(.notHTML(type))
        }
        if http.statusCode >= 400 { return .failure(.http(http.statusCode)) }
        return .success(WebResponse(status: http.statusCode, contentType: type,
                                    body: [UInt8](box.data ?? Data())))
    }
}
