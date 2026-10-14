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
      if not Item.Focused then return ""; end if;
      if Item.Role_Last = 0 then Add ("élément");
      else Add (Item.Role (1 .. Item.Role_Last)); end if;
      if Item.Name_Last > 0 then Add (": "); Add (Item.Name (1 .. Item.Name_Last)); end if;
      if Item.Value_Last > 0 then Add (", "); Add (Item.Value (1 .. Item.Value_Last)); end if;
      if Item.Disabled then Add (", désactivé"); end if;
      return Result (1 .. Last);
   end Announce;

   function Push (Queue : in out Event_Queue; Kind : Event_Kind; Item : Node) return Boolean is
      Tail : Positive;
   begin
      if Queue.Count = Queue_Capacity then return False; end if;
      Tail := ((Queue.Head - 1 + Queue.Count) mod Queue_Capacity) + 1;
      Queue.Items (Tail) := (Kind => Kind, Item => Item);
      Queue.Count := Queue.Count + 1;
      return True;
   end Push;

   function Pop (Queue : in out Event_Queue; Result : out Event) return Boolean is
   begin
      if Queue.Count = 0 then
         Result := (Kind => Focus, Item => (Role => (others => ' '), Role_Last => 0,
                    Name => (others => ' '), Name_Last => 0, Value => (others => ' '),
                    Value_Last => 0, Focused => False, Disabled => False));
         return False;
      end if;
      Result := Queue.Items (Queue.Head);
      Queue.Head := (Queue.Head mod Queue_Capacity) + 1;
      Queue.Count := Queue.Count - 1;
      return True;
   end Pop;
end Screen_Reader_Core;
