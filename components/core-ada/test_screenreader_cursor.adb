with Ada.Assertions; use Ada.Assertions;
with Screenreader_Cursor; use Screenreader_Cursor;
procedure Test_Screenreader_Cursor is
   State : Cursor;
   Index : Natural := 99;
   Empty : Cursor;
begin
   Initialize (State, 2);
   Assert (First (State, Index) and then Index = 0, "first must select zero");
   Assert (Next (State, Index) and then Index = 1, "next must select one");
   Assert (not Next (State, Index), "navigation must not wrap");
   Assert (Previous (State, Index) and then Index = 0, "previous must select zero");
   Assert (not Previous (State, Index), "previous must stop at beginning");
   Initialize (Empty, 0);
   Assert (not First (Empty, Index), "empty list must have no item");
end Test_Screenreader_Cursor;
