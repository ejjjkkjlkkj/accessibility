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

   type Event_Kind is (Focus, Value_Changed, State_Changed);
   Queue_Capacity : constant := 16;
   type Event is record
      Kind : Event_Kind := Focus;
      Item : Node;
   end record;
   type Event_Array is array (Positive range 1 .. Queue_Capacity) of Event;
   type Event_Queue is record
      Items : Event_Array;
      Head  : Positive range 1 .. Queue_Capacity := 1;
      Count : Natural range 0 .. Queue_Capacity := 0;
   end record;

   function Announce (Item : Node) return String;
   function Push (Queue : in out Event_Queue; Kind : Event_Kind; Item : Node) return Boolean;
   function Pop (Queue : in out Event_Queue; Result : out Event) return Boolean;
end Screen_Reader_Core;
