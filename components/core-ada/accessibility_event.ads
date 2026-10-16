package Accessibility_Event is
   type Event_Role is (Unknown_Role, Window_Role, Button_Role,
                       Text_Field_Role, Heading_Role);
   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name_Length : Natural) return Boolean;
end Accessibility_Event;
