with Ada.Text_IO; use Ada.Text_IO;
with Screen_Reader_Core; use Screen_Reader_Core;

procedure Test_Screen_Reader_Core is
   Item : Node :=
     (Role => (others => ' '), Role_Last => 6,
      Name => (others => ' '), Name_Last => 7,
      Value => (others => ' '), Value_Last => 0,
      Focused => True, Disabled => False);
begin
   Item.Role (1 .. 6) := "bouton";
   Item.Name (1 .. 7) := "Valider";
   if Announce (Item) /= "bouton: Valider" then
      raise Program_Error with "focused node announcement mismatch";
   end if;
   Item.Focused := False;
   if Announce (Item) /= "" then
      raise Program_Error with "unfocused node must be silent";
   end if;
   Put_Line ("Ada screen-reader core tests passed");
end Test_Screen_Reader_Core;
