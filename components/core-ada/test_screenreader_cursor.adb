with Ada.Assertions; use Ada.Assertions;
with Screenreader_Cursor; use Screenreader_Cursor;
procedure Test_Screenreader_Cursor is
   State : Cursor;
   Index : Natural := 99;
   Empty : Cursor;
begin
   Initialize (State, 2);
   Assert (not Next (State, Index), "next before first must fail");
   Assert (not State.Has_Item and State.Current = 0, "failed next changed state");
   Assert (not Previous (State, Index), "previous before first must fail");
   Assert (not State.Has_Item and State.Current = 0, "failed previous changed state");
   Assert (First (State, Index) and then Index = 0, "first must select zero");
   Assert (First (State, Index) and then Index = 0, "repeated first must reset zero");
   Assert (Next (State, Index) and then Index = 1, "next must select one");
   Assert (not Next (State, Index), "next must stop at end");
   Assert (State.Has_Item and State.Current = 1, "failed next changed state");
   Assert (Previous (State, Index) and then Index = 0, "previous must select zero");
   Assert (not Previous (State, Index), "previous must stop at start");
   Assert (State.Has_Item and State.Current = 0, "failed previous changed state");
   Initialize (Empty, 0);
   Assert (not First (Empty, Index), "empty list must have no item");
   Assert (not Next (Empty, Index), "Next on empty list must fail");
   Assert (not Previous (Empty, Index), "Previous on empty list must fail");
   Assert (not Empty.Has_Item, "empty list acquired focus");
end Test_Screenreader_Cursor;
