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

   function Is_Valid_Name (Name : String) return Boolean is
      Position : Natural := 1;
      B1, B2, B3, B4 : Natural;

      function Byte_At (Relative_Position : Positive) return Natural is
      begin
         return Character'Pos (Name (Name'First + Relative_Position - 1));
      end Byte_At;
   begin
      if Name'Length > 4096 then
         return False;
      end if;

      while Position <= Name'Length loop
         B1 := Byte_At (Position);
         if B1 = 0 then
            return False;
         elsif B1 <= 16#7F# then
            Position := Position + 1;
         elsif B1 in 16#C2# .. 16#DF# then
            if Name'Length - Position < 1 then return False; end if;
            B2 := Byte_At (Position + 1);
            if B2 not in 16#80# .. 16#BF# then return False; end if;
            Position := Position + 2;
         elsif B1 in 16#E0# .. 16#EF# then
            if Name'Length - Position < 2 then return False; end if;
            B2 := Byte_At (Position + 1);
            B3 := Byte_At (Position + 2);
            if B3 not in 16#80# .. 16#BF# then return False; end if;
            if B1 = 16#E0# then
               if B2 not in 16#A0# .. 16#BF# then return False; end if;
            elsif B1 = 16#ED# then
               if B2 not in 16#80# .. 16#9F# then return False; end if;
            elsif B2 not in 16#80# .. 16#BF# then
               return False;
            end if;
            Position := Position + 3;
         elsif B1 in 16#F0# .. 16#F4# then
            if Name'Length - Position < 3 then return False; end if;
            B2 := Byte_At (Position + 1);
            B3 := Byte_At (Position + 2);
            B4 := Byte_At (Position + 3);
            if B3 not in 16#80# .. 16#BF# or else
               B4 not in 16#80# .. 16#BF#
            then
               return False;
            end if;
            if B1 = 16#F0# then
               if B2 not in 16#90# .. 16#BF# then return False; end if;
            elsif B1 = 16#F4# then
               if B2 not in 16#80# .. 16#8F# then return False; end if;
            elsif B2 not in 16#80# .. 16#BF# then
               return False;
            end if;
            Position := Position + 4;
         else
            return False;
         end if;
      end loop;
      return True;
   end Is_Valid_Name;

   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name : String;
                      State_Flags : Interfaces.Unsigned_64 := 0) return Boolean is
   begin
      return Abi_Version = 1
        and then Is_Valid_Name (Name)
        and then State_Flags_Valid (State_Flags);
   end Is_Valid;

   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name_Length : Natural;
                      State_Flags : Interfaces.Unsigned_64 := 0) return Boolean is
   begin
      return Abi_Version = 1
        and then Name_Length <= 4096
        and then State_Flags_Valid (State_Flags);
   end Is_Valid;
end Accessibility_Event;
