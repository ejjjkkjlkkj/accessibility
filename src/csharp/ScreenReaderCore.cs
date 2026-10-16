namespace Adm.Accessibility;

public sealed record AccessibleNode(
    string Role, string Name, string Value, bool Focused, bool Disabled);

public enum EventKind { Focus, ValueChanged, StateChanged }
public sealed record AccessibilityEvent(EventKind Kind, AccessibleNode Node);

public static class ScreenReaderCore
{
    public static string Announce(AccessibleNode node)
    {
        if (!node.Focused) return string.Empty;
        var role = string.IsNullOrWhiteSpace(node.Role) ? "element" : node.Role;
        var text = role;
        if (!string.IsNullOrEmpty(node.Name)) text += ": " + node.Name;
        if (!string.IsNullOrEmpty(node.Value)) text += ", " + node.Value;
        if (node.Disabled) text += ", disabled";
        return text;
    }
}

public sealed class BoundedEventQueue
{
    private readonly AccessibilityEvent?[] items;
    private int head;
    private int count;
    public BoundedEventQueue(int capacity = 16)
    {
        if (capacity < 1) throw new ArgumentOutOfRangeException(nameof(capacity));
        items = new AccessibilityEvent[capacity];
    }
    public int Count => count;
    public bool TryEnqueue(AccessibilityEvent item)
    {
        if (count == items.Length) return false;
        items[(head + count) % items.Length] = item;
        count++;
        return true;
    }
    public bool TryDequeue(out AccessibilityEvent? item)
    {
        if (count == 0) { item = null; return false; }
        item = items[head];
        items[head] = null;
        head = (head + 1) % items.Length;
        count--;
        return true;
    }
}
