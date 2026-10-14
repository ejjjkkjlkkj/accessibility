-- Entry-point sketch only; GNAT runtime and boot handoff remain unvalidated.
procedure Kernel_Main is
   pragma Export (C, Kernel_Main, "kernel_main");
begin
   loop
      null;
   end loop;
end Kernel_Main;
