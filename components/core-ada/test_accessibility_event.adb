with Ada.Assertions; use Ada.Assertions;
with Accessibility_Event; use Accessibility_Event;
with Interfaces; use type Interfaces.Unsigned_64;

procedure Test_Accessibility_Event is
begin
   Assert (Is_Valid (1, Button_Role, 4), "valid event rejected");
   Assert (not Is_Valid (2, Button_Role, 4), "wrong ABI accepted");
   Assert (not Is_Valid (1, Button_Role, 4097), "oversized name accepted");
   Assert (State_Flags_Valid (State_Focusable or State_Enabled or State_Focused),
           "valid focused state rejected");
   Assert (not State_Flags_Valid (State_Focused), "focused non-focusable state accepted");
   Assert (not State_Flags_Valid (State_Enabled or State_Disabled),
           "contradictory enabled/disabled state accepted");
   Assert (not State_Flags_Valid (2 ** 63), "reserved state bit accepted");
   Assert (Is_Valid (1, Unknown_Role, 0), "unknown role fallback rejected");
   Assert (Event_Role'Pos (Button_Role) = 5, "button role code mismatch");
   Assert (Event_Role'Pos (Menu_Item_Role) = 17, "last role code mismatch");
end Test_Accessibility_Event;
