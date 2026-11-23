// Démo en ligne de commande : pilote le navigateur au clavier, sortie texte sur le bus.
// Commandes : o ADRESSE | tab | shift-tab | enter | h H l L | escape | ctrl-. | q

let bus = OutputBus()
bus.attach(TextOutput())
let browser = Browser(bus: bus, transport: HostTransport())

if let first = CommandLine.arguments.dropFirst().first {
    browser.open(first)
} else {
    bus.emit("Type o ADDRESS to open a page. Keys: tab, shift-tab, enter, h, H, l, L, escape, ctrl-period, q.")
}

while let line = readLine() {
    let parts = line.split(separator: " ", maxSplits: 1).map(String.init)
    let cmd = parts.first ?? ""
    if cmd == "q" { break }
    if cmd == "o" {
        browser.open(parts.count > 1 ? parts[1] : "")
        continue
    }
    if cmd == "ctrl-period" {
        bus.cancelAll()
        bus.emit("Stopped.", priority: .alert)
        continue
    }
    browser.press(cmd)
}
