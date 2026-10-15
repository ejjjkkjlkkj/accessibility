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

   Assert (Is_Valid (1, Button_Role, "Continue"), "valid UTF-8 name rejected");
   Assert (not Is_Valid (1, Button_Role, "bad" & Character'Val (0) & "name"),
           "embedded NUL accepted");
   Assert (not Is_Valid (1, Button_Role,
           Character'Val (16#C0#) & Character'Val (16#AF#)),
           "overlong UTF-8 accepted");
   Assert (not Is_Valid (1, Button_Role,
           Character'Val (16#ED#) & Character'Val (16#A0#) & Character'Val (16#80#)),
           "UTF-8 surrogate accepted");
   Assert (not Is_Valid (1, Button_Role,
           Character'Val (16#F4#) & Character'Val (16#90#) &
           Character'Val (16#80#) & Character'Val (16#80#)),
           "code point above U+10FFFF accepted");
   Assert (not Is_Valid (1, Button_Role,
           Character'Val (16#E2#) & Character'Val (16#82#)),
           "truncated UTF-8 accepted");
   Assert (Is_Valid (1, Button_Role,
           "caf" & Character'Val (16#C3#) & Character'Val (16#A9#)),
           "valid multibyte UTF-8 rejected");
   Assert (Is_Valid (1, Unknown_Role, 0), "unknown role fallback rejected");
   Assert (Event_Role'Pos (Button_Role) = 5, "button role code mismatch");
   Assert (Event_Role'Pos (Menu_Item_Role) = 17, "last role code mismatch");
end Test_Accessibility_Event;
