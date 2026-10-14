package screenreader

import "testing"

func TestAnnounceFocusedNode(t *testing.T) {
    got := Announce(Node{Role: "bouton", Name: "Valider", Focused: true})
    if got != "bouton: Valider" { t.Fatalf("got %q", got) }
}
func TestAnnounceUnfocusedNode(t *testing.T) {
    if got := Announce(Node{Role: "bouton", Name: "Valider"}); got != "" {
        t.Fatalf("unfocused node announced: %q", got)
    }
}
func TestAnnounceValueAndDisabled(t *testing.T) {
    got := Announce(Node{Role: "champ", Name: "Volume", Value: "80 %", Focused: true, Disabled: true})
    if got != "champ: Volume, 80 %, désactivé" { t.Fatalf("got %q", got) }
}
