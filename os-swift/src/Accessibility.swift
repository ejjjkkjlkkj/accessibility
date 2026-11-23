// Accessibility from the start: every answer reaches the user as text and, optionally, as speech.
// Voice is an extra channel, never the only one. The keyboard always works.
import Foundation

/// A place where the system tells the user something.
protocol OutputChannel: AnyObject {
    func say(_ text: String)
}

/// Text output: works with any screen reader, terminal or braille display.
final class TextChannel: OutputChannel {
    func say(_ text: String) {
        print(text)
    }
}

/// Speech output through Windows SAPI (System.Speech). Falls back silently to text on failure.
final class SpeechChannel: OutputChannel {
    private(set) var failures = 0

    func say(_ text: String) {
        // Escape single quotes for the PowerShell string literal.
        let safe = text.replacingOccurrences(of: "'", with: "''")
        let script = "Add-Type -AssemblyName System.Speech; " +
                     "(New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('\(safe)')"
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")
        p.arguments = ["-NoProfile", "-NonInteractive", "-Command", script]
        do {
            try p.run()
            p.waitUntilExit()
            if p.terminationStatus != 0 { failures += 1 }
        } catch {
            failures += 1
        }
    }
}

/// Sends each message to every enabled channel. Text is always on.
final class Announcer {
    private let text = TextChannel()
    private var speech: SpeechChannel? = nil

    var voiceOn: Bool { speech != nil }

    func setVoice(_ on: Bool) {
        speech = on ? SpeechChannel() : nil
    }

    func announce(_ message: String) {
        text.say(message)
        speech?.say(message)
    }
}
