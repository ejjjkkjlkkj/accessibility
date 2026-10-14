package body Screenreader_Cursor is
   procedure Initialize (State : out Cursor; Count : Natural) is
   begin
      State := (Count => Count, Current => 0, Has_Item => False);
   end Initialize;
   function First (State : in out Cursor; Index : out Natural) return Boolean is
   begin
      if State.Count = 0 then State.Has_Item := False; return False; end if;
      State.Current := 0; State.Has_Item := True; Index := 0; return True;
   end First;
   function Next (State : in out Cursor; Index : out Natural) return Boolean is
   begin
      if not State.Has_Item or else State.Current >= State.Count - 1 then return False; end if;
      State.Current := State.Current + 1; Index := State.Current; return True;
   end Next;
   function Previous (State : in out Cursor; Index : out Natural) return Boolean is
   begin
      if not State.Has_Item or else State.Current = 0 then return False; end if;
      State.Current := State.Current - 1; Index := State.Current; return True;
   end Previous;
end Screenreader_Cursor;
