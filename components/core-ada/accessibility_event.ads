package Accessibility_Event is
   type Event_Role is (Unknown_Role, Application_Role, Window_Role, Group_Role,
                       Text_Role, Button_Role, Checkbox_Role, Radio_Role,
                       Textbox_Role, Link_Role, Heading_Role, List_Role,
                       List_Item_Role, Table_Role, Row_Role, Cell_Role,
                       Menu_Role, Menu_Item_Role);
   for Event_Role use
     (Unknown_Role => 0,
      Application_Role => 1,
      Window_Role => 2,
      Group_Role => 3,
      Text_Role => 4,
      Button_Role => 5,
      Checkbox_Role => 6,
      Radio_Role => 7,
      Textbox_Role => 8,
      Link_Role => 9,
      Heading_Role => 10,
      List_Role => 11,
      List_Item_Role => 12,
      Table_Role => 13,
      Row_Role => 14,
      Cell_Role => 15,
      Menu_Role => 16,
      Menu_Item_Role => 17);
   function Is_Valid (Abi_Version : Positive; Role : Event_Role;
                      Name_Length : Natural) return Boolean;
end Accessibility_Event;
