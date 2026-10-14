package body Accessibility_Event is
   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name_Length : Natural) return Boolean is
   begin
      return Abi_Version = 1
        and then Role /= Unknown_Role
        and then Name_Length <= 256;
   end Is_Valid;
end Accessibility_Event;
