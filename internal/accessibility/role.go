package accessibility

import "unicode/utf8"

// Role is the stable logical role identifier defined by docs/ACCESSIBILITY-ROLE-CODES-V1.md.
// It is a wire-level code, not a platform handle or a complete binary ABI.
type Role uint32

const (
	RoleUnknown Role = 0
	RoleApplication Role = 1
	RoleWindow Role = 2
	RoleGroup Role = 3
	RoleText Role = 4
	RoleButton Role = 5
	RoleCheckbox Role = 6
	RoleRadio Role = 7
	RoleTextbox Role = 8
	RoleLink Role = 9
	RoleHeading Role = 10
	RoleList Role = 11
	RoleListItem Role = 12
	RoleTable Role = 13
	RoleRow Role = 14
	RoleCell Role = 15
	RoleMenu Role = 16
	RoleMenuItem Role = 17
)

// Valid reports whether code is assigned by role contract v1.
func (r Role) Valid() bool { return r <= RoleMenuItem }

// StateFlags defines the shared semantic state bitset prototype v1.
type StateFlags uint64

const (
	StateFocused StateFlags = 1 << iota
	StateFocusable
	StateEnabled
	StateDisabled
)

const stateKnownMask StateFlags = StateFocused | StateFocusable | StateEnabled | StateDisabled

// Valid reports whether flags have known bits and a non-contradictory combination.
func (s StateFlags) Valid() bool {
	if s & ^stateKnownMask != 0 {
		return false
	}
	if s&StateEnabled != 0 && s&StateDisabled != 0 {
		return false
	}
	if s&StateFocused != 0 &&
		(s&StateFocusable == 0 || s&StateEnabled == 0 || s&StateDisabled != 0) {
		return false
	}
	return true
}

// Event is the language-level semantic event contract, not a native binary ABI.
type Event struct {
	ABIVersion uint32
	Role       Role
	StateFlags StateFlags
	SourceID   uint64
	Name       string
}

// Valid checks the version, role, state flags, UTF-8 encoding, and byte limit.
func (e Event) Valid() bool {
	return e.ABIVersion == 1 &&
		e.Role.Valid() &&
		e.StateFlags.Valid() &&
		utf8.ValidString(e.Name) &&
		len(e.Name) <= 4096
}
