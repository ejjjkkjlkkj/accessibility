with Ada.Text_IO; use Ada.Text_IO;
with Screen_Reader_Core; use Screen_Reader_Core;

procedure Test_Screen_Reader_Core is
   Item : Node :=
     (Role => (others => ' '), Role_Last => 6,
      Name => (others => ' '), Name_Last => 7,
      Value => (others => ' '), Value_Last => 0,
      Focused => True, Disabled => False);
   Queue : Event_Queue;
   Got : Event;
begin
   Item.Role (1 .. 6) := "bouton";
   Item.Name (1 .. 7) := "Valider";
   if Announce (Item) /= "bouton: Valider" then raise Program_Error with "focus announcement mismatch"; end if;
   Item.Focused := False;
   if Announce (Item) /= "" then raise Program_Error with "unfocused node must be silent"; end if;
   Item.Focused := True;
   if not Push (Queue, Focus, Item) then raise Program_Error with "queue push failed"; end if;
   Item.Name (1 .. 7) := "Modifie";
   if not Pop (Queue, Got) or else Got.Kind /= Focus or else Got.Item.Name (1 .. 7) /= "Valider" then
      raise Program_Error with "queue must preserve owned FIFO snapshot";
   end if;
   if Pop (Queue, Got) then raise Program_Error with "empty queue returned event"; end if;
   for I in 1 .. Queue_Capacity loop
      if not Push (Queue, State_Changed, Item) then raise Program_Error with "queue fill failed"; end if;
   end loop;
   if Push (Queue, Focus, Item) then raise Program_Error with "queue overflow accepted"; end if;
   for I in 1 .. Queue_Capacity loop
      if not Pop (Queue, Got) then raise Program_Error with "queue drain failed"; end if;
   end loop;
   if Pop (Queue, Got) then raise Program_Error with "queue not empty"; end if;
   Put_Line ("Ada screen-reader core tests passed");
end Test_Screen_Reader_Core;
