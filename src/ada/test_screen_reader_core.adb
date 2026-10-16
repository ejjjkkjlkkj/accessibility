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
   -- Exercise FIFO preservation while the circular queue wraps around.
   Item.Name (1 .. 1) := "A"; Item.Name_Last := 1;
   for I in 1 .. 12 loop
      if not Push (Queue, Focus, Item) then raise Program_Error with "wrap initial push failed"; end if;
   end loop;
   for I in 1 .. 8 loop
      if not Pop (Queue, Got) or else Got.Item.Name (1 .. Got.Item.Name_Last) /= "A" then
         raise Program_Error with "wrap initial FIFO mismatch";
      end if;
   end loop;
   Item.Name (1 .. 1) := "B"; Item.Name_Last := 1;
   for I in 1 .. 12 loop
      if not Push (Queue, Focus, Item) then raise Program_Error with "wrap push failed"; end if;
   end loop;
   for I in 1 .. 4 loop
      if not Pop (Queue, Got) or else Got.Item.Name (1 .. Got.Item.Name_Last) /= "A" then
         raise Program_Error with "wrap remaining FIFO mismatch";
      end if;
   end loop;
   for I in 1 .. 12 loop
      if not Pop (Queue, Got) or else Got.Item.Name (1 .. Got.Item.Name_Last) /= "B" then
         raise Program_Error with "wrap second FIFO mismatch";
      end if;
   end loop;
   for I in 1 .. Queue_Capacity loop
      if not Push (Queue, State_Changed, Item) then raise Program_Error with "queue fill failed"; end if;
   end loop;
   if Push (Queue, Focus, Item) then raise Program_Error with "queue overflow accepted"; end if;
   for I in 1 .. Queue_Capacity loop
      if not Pop (Queue, Got) then raise Program_Error with "queue drain failed"; end if;
   end loop;
   if Pop (Queue, Got) then raise Program_Error with "queue not empty"; end if;
   Item.Role_Last := 0;
   Item.Name_Last := 0;
   Item.Value_Last := 2;
   Item.Value (1 .. 2) := "42";
   if Announce (Item) /= "élément, 42" then raise Program_Error with "fallback role mismatch"; end if;
   Item.Value_Last := 0;
   if Announce (Item) /= "élément" then raise Program_Error with "empty fields mismatch"; end if;
   Put_Line ("Ada screen-reader core tests passed");
end Test_Screen_Reader_Core;
