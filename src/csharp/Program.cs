using Adm.Accessibility;

static void Require(bool condition, string label)
{
    if (!condition) throw new Exception("FAIL: " + label);
}
var focused = new AccessibleNode("button", "Confirm", "", true, false);
Require(ScreenReaderCore.Announce(focused) == "button: Confirm", "focused announcement");
Require(ScreenReaderCore.Announce(focused with { Focused = false }) == "", "unfocused node is silent");
Require(ScreenReaderCore.Announce(focused with { Value = "On", Disabled = true }) == "button: Confirm, On, disabled", "value and state");
var queue = new BoundedEventQueue(2);
var a = new AccessibilityEvent(EventKind.Focus, focused with { Name = "A" });
var b = new AccessibilityEvent(EventKind.Focus, focused with { Name = "B" });
Require(queue.TryEnqueue(a) && queue.TryEnqueue(b), "queue accepts capacity");
Require(!queue.TryEnqueue(a), "queue rejects overflow");
Require(queue.TryDequeue(out var first) && first!.Node.Name == "A", "FIFO first");
Require(queue.TryDequeue(out var second) && second!.Node.Name == "B", "FIFO second");
Require(!queue.TryDequeue(out _), "empty queue");
Console.WriteLine("PASS: C# portable core checks (source-level test executable)");
