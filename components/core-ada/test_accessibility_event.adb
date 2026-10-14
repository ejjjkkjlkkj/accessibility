with Ada.Assertions; use Ada.Assertions;
with Accessibility_Event; use Accessibility_Event;

procedure Test_Accessibility_Event is
begin
   Assert (Is_Valid (1, Button_Role, 4), "valid event rejected");
   Assert (not Is_Valid (2, Button_Role, 4), "wrong ABI accepted");
   Assert (not Is_Valid (1, Button_Role, 4097), "oversized name accepted");
   Assert (not Is_Valid (1, Unknown_Role, 0), "unknown role accepted");
   Assert (Event_Role'Pos (Button_Role) = 5, "button role code mismatch");
   Assert (Event_Role'Pos (Menu_Item_Role) = 17, "last role code mismatch");
end Test_Accessibility_Event;
