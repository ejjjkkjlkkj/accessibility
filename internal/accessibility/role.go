package accessibility

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
