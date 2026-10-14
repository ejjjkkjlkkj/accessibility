package Screenreader_Cursor is
   type Cursor is record
      Count : Natural := 0;
      Current : Natural := 0;
      Has_Item : Boolean := False;
   end record;
   procedure Initialize (State : out Cursor; Count : Natural);
   function First (State : in out Cursor; Index : out Natural) return Boolean;
   function Next (State : in out Cursor; Index : out Natural) return Boolean;
   function Previous (State : in out Cursor; Index : out Natural) return Boolean;
end Screenreader_Cursor;
