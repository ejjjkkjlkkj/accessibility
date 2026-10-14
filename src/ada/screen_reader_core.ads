package Screen_Reader_Core is
   type Node is record
      Role     : String (1 .. 64);
      Role_Last : Natural range 0 .. 64 := 0;
      Name     : String (1 .. 256);
      Name_Last : Natural range 0 .. 256 := 0;
      Value    : String (1 .. 256);
      Value_Last : Natural range 0 .. 256 := 0;
      Focused  : Boolean := False;
      Disabled : Boolean := False;
   end record;

   -- Pure formatting core; no OS API or audio backend is connected.
   function Announce (Item : Node) return String;
end Screen_Reader_Core;
