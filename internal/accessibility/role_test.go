package accessibility

import "testing"

func TestCanonicalRoleCodesV1(t *testing.T) {
	roles := []Role{
		RoleUnknown, RoleApplication, RoleWindow, RoleGroup, RoleText, RoleButton,
		RoleCheckbox, RoleRadio, RoleTextbox, RoleLink, RoleHeading, RoleList,
		RoleListItem, RoleTable, RoleRow, RoleCell, RoleMenu, RoleMenuItem,
	}
	for want, got := range roles {
		if int(got) != want {
			t.Fatalf("role at code %d has value %d", want, got)
		}
	}
	if !RoleUnknown.Valid() || !RoleMenuItem.Valid() || Role(18).Valid() {
		t.Fatal("role code boundary validation failed")
	}
}
