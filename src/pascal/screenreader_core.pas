unit screenreader_core;
{$mode objfpc}{$H+}
interface
uses SysUtils;
type
  TAccessibleNode = record
    Role, Name, Value: string;
    Focused, Disabled: Boolean;
  end;
  TEventKind = (ekFocus, ekValueChanged, ekStateChanged);
  TScreenReaderEvent = record
    Kind: TEventKind;
    Node: TAccessibleNode;
  end;
  TEventQueue = record
    Items: array[0..15] of TScreenReaderEvent;
    Head, Count: SizeInt;
  end;
function Announce(const N: TAccessibleNode): string;
procedure QueueInit(var Q: TEventQueue);
function QueuePush(var Q: TEventQueue; const E: TScreenReaderEvent): Boolean;
function QueuePop(var Q: TEventQueue; out E: TScreenReaderEvent): Boolean;
implementation
function Announce(const N: TAccessibleNode): string;
begin
  if not N.Focused then Exit('');
  Result := N.Role;
  if Result = '' then Result := 'élément';
  if N.Name <> '' then Result := Result + ': ' + N.Name;
  if N.Value <> '' then Result := Result + ', ' + N.Value;
  if N.Disabled then Result := Result + ', désactivé';
end;
procedure QueueInit(var Q: TEventQueue);
begin Q.Head := 0; Q.Count := 0; end;
function QueuePush(var Q: TEventQueue; const E: TScreenReaderEvent): Boolean;
var I: SizeInt;
begin
  if Q.Count = Length(Q.Items) then Exit(False);
  I := (Q.Head + Q.Count) mod Length(Q.Items);
  Q.Items[I] := E; Inc(Q.Count); Result := True;
end;
function QueuePop(var Q: TEventQueue; out E: TScreenReaderEvent): Boolean;
begin
  if Q.Count = 0 then Exit(False);
  E := Q.Items[Q.Head]; Q.Head := (Q.Head + 1) mod Length(Q.Items);
  Dec(Q.Count); Result := True;
end;
end.
