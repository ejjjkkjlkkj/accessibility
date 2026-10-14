package screenreader

import "strings"

// Node is normalized focus metadata. Native accessibility API adapters are separate.
type Node struct {
    Role, Name, Value string
    Focused, Disabled bool
}

// Announce returns a short announcement, or an empty string if focus is absent.
func Announce(n Node) string {
    if !n.Focused { return "" }
    parts := []string{n.Role}
    if parts[0] == "" { parts[0] = "élément" }
    if n.Name != "" { parts = append(parts, n.Name) }
    if n.Value != "" { parts = append(parts, n.Value) }
    text := strings.Join(parts, ": ")
    if n.Value != "" && n.Name != "" { text = n.Role + ": " + n.Name + ", " + n.Value }
    if n.Disabled { text += ", désactivé" }
    return text
}
