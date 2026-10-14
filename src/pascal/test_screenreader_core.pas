program TestScreenReaderCore;
{$mode objfpc}{$H+}
uses SysUtils, ScreenReaderCore;
var N: TAccessibleNode; Q: TEventQueue; E, O: TScreenReaderEvent;
begin
  N.Role := 'bouton'; N.Name := 'Valider'; N.Value := ''; N.Focused := True; N.Disabled := False;
  if Announce(N) <> 'bouton: Valider' then Halt(1);
  N.Focused := False; if Announce(N) <> '' then Halt(2);
  QueueInit(Q); E.Kind := ekFocus; E.Node := N;
  if not QueuePush(Q,E) then Halt(3);
  if not QueuePop(Q,O) or (O.Kind <> ekFocus) then Halt(4);
  if QueuePop(Q,O) then Halt(5);
  WriteLn('PASS: Pascal portable core checks');
end.
