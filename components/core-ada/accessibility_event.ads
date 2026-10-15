with Interfaces;

package Accessibility_Event is
   State_Focused    : constant Interfaces.Unsigned_64 := 1;
   State_Focusable  : constant Interfaces.Unsigned_64 := 2;
   State_Enabled    : constant Interfaces.Unsigned_64 := 4;
   State_Disabled   : constant Interfaces.Unsigned_64 := 8;
   State_Known_Mask : constant Interfaces.Unsigned_64 := 15;
   function State_Flags_Valid (State_Flags : Interfaces.Unsigned_64) return Boolean;
   type Event_Role is (Unknown_Role, Application_Role, Window_Role, Group_Role,
                       Text_Role, Button_Role, Checkbox_Role, Radio_Role,
                       Textbox_Role, Link_Role, Heading_Role, List_Role,
                       List_Item_Role, Table_Role, Row_Role, Cell_Role,
                       Menu_Role, Menu_Item_Role);
   for Event_Role use
     (Unknown_Role => 0, Application_Role => 1, Window_Role => 2,
      Group_Role => 3, Text_Role => 4, Button_Role => 5,
      Checkbox_Role => 6, Radio_Role => 7, Textbox_Role => 8,
      Link_Role => 9, Heading_Role => 10, List_Role => 11,
      List_Item_Role => 12, Table_Role => 13, Row_Role => 14,
      Cell_Role => 15, Menu_Role => 16, Menu_Item_Role => 17);
   for Event_Role'Size use 8;
   function Is_Valid_Name (Name : String) return Boolean;
   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name : String;
                      State_Flags : Interfaces.Unsigned_64 := 0) return Boolean;
   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name_Length : Natural;
                      State_Flags : Interfaces.Unsigned_64 := 0) return Boolean;
end Accessibility_Event;
