package accessibility

import (
	"strings"
	"testing"
)

func TestEventContractV1(t *testing.T) {
	valid := Event{
		ABIVersion: 1,
		Role:       RoleButton,
		StateFlags: StateFocusable | StateEnabled | StateFocused,
		SourceID:   42,
		Name:       "Continue",
	}
	if !valid.Valid() {
		t.Fatal("valid semantic event rejected")
	}
	for _, event := range []Event{
		{ABIVersion: 2, Role: RoleButton, Name: "Continue"},
		{ABIVersion: 1, Role: Role(18), Name: "Continue"},
		{ABIVersion: 1, Role: RoleButton, StateFlags: StateFocused, Name: "Continue"},
		{ABIVersion: 1, Role: RoleButton, Name: string([]byte{0xff})},
		{ABIVersion: 1, Role: RoleButton, Name: "bad\x00name"},
		{ABIVersion: 1, Role: RoleButton, Name: strings.Repeat("a", 4097)},
	} {
		if event.Valid() {
			t.Fatalf("invalid event accepted: %#v", event)
		}
	}
	maxName := Event{ABIVersion: 1, Role: RoleButton, Name: strings.Repeat("a", 4096)}
	if !maxName.Valid() {
		t.Fatal("valid 4096-byte UTF-8 event name rejected")
	}
}

func TestSemanticStateFlagsV1(t *testing.T) {
	validMasks := map[StateFlags]bool{
		0: true, StateFocusable: true, StateEnabled: true,
		StateFocusable | StateEnabled:                true,
		StateFocused | StateFocusable | StateEnabled: true,
		StateDisabled: true, StateFocusable | StateDisabled: true,
	}
	for raw := uint64(0); raw <= 255; raw++ {
		flags := StateFlags(raw)
		expected := validMasks[flags]
		if actual := flags.Valid(); actual != expected {
			t.Fatalf("flags=%#x: got %v, want %v", flags, actual, expected)
		}
	}
	valid := StateFocusable | StateEnabled | StateFocused
	if !valid.Valid() {
		t.Fatal("valid focused target rejected")
	}
	for _, flags := range []StateFlags{
		StateFocused,
		StateEnabled | StateDisabled,
		StateFlags(1 << 63),
	} {
		if flags.Valid() {
			t.Fatalf("invalid flags accepted: %#x", flags)
		}
	}
}

func TestCanonicalRoleCodesV1(t *testing.T) {
	roles := []Role{RoleUnknown, RoleApplication, RoleWindow, RoleGroup, RoleText, RoleButton, RoleCheckbox, RoleRadio, RoleTextbox, RoleLink, RoleHeading, RoleList, RoleListItem, RoleTable, RoleRow, RoleCell, RoleMenu, RoleMenuItem}
	for want, got := range roles {
		if int(got) != want {
			t.Fatalf("role at code %d has value %d", want, got)
		}
	}
	if !RoleUnknown.Valid() || !RoleMenuItem.Valid() || Role(18).Valid() {
		t.Fatal("role code boundary validation failed")
	}
}
