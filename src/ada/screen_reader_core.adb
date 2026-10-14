package body Screen_Reader_Core is
   function Announce (Item : Node) return String is
      Result : String (1 .. 640);
      Last   : Natural := 0;
      procedure Add (S : String) is
      begin
         if S'Length > 0 and then Last + S'Length <= Result'Length then
            Result (Last + 1 .. Last + S'Length) := S;
            Last := Last + S'Length;
         end if;
      end Add;
   begin
      if not Item.Focused then
         return "";
      end if;
      if Item.Role_Last = 0 then Add ("élément");
      else Add (Item.Role (1 .. Item.Role_Last)); end if;
      if Item.Name_Last > 0 then
         Add (": "); Add (Item.Name (1 .. Item.Name_Last));
      end if;
      if Item.Value_Last > 0 then
         Add (", "); Add (Item.Value (1 .. Item.Value_Last));
      end if;
      if Item.Disabled then Add (", désactivé"); end if;
      return Result (1 .. Last);
   end Announce;
end Screen_Reader_Core;
