with Ada.Assertions; use Ada.Assertions;
with Accessibility_Event; use Accessibility_Event;

procedure Test_Accessibility_Event is
begin
   Assert (Is_Valid (1, Button_Role, 4), "valid event rejected");
   Assert (not Is_Valid (2, Button_Role, 4), "wrong ABI accepted");
   Assert (not Is_Valid (1, Button_Role, 257), "oversized name accepted");
   Assert (not Is_Valid (1, Unknown_Role, 0), "unknown role accepted");
end Test_Accessibility_Event;
