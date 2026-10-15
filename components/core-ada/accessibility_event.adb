with Interfaces;

package body Accessibility_Event is
   use type Interfaces.Unsigned_64;

   function State_Flags_Valid (State_Flags : Interfaces.Unsigned_64) return Boolean is
      Focused   : constant Boolean := (State_Flags and State_Focused) /= 0;
      Focusable : constant Boolean := (State_Flags and State_Focusable) /= 0;
      Enabled   : constant Boolean := (State_Flags and State_Enabled) /= 0;
      Disabled  : constant Boolean := (State_Flags and State_Disabled) /= 0;
   begin
      if (State_Flags and (not State_Known_Mask)) /= 0 then
         return False;
      end if;
      if Enabled and Disabled then
         return False;
      end if;
      return (not Focused) or else (Focusable and Enabled and (not Disabled));
   end State_Flags_Valid;

   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name_Length : Natural;
                      State_Flags : Interfaces.Unsigned_64 := 0) return Boolean is
   begin
      return Abi_Version = 1
        and then Name_Length <= 4096
        and then State_Flags_Valid (State_Flags);
   end Is_Valid;
end Accessibility_Event;
