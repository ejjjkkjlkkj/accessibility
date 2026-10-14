module screenreader;

/// Normalized focus metadata; native platform adapters are not implemented yet.
struct Node {
    string role;
    string name;
    string value;
    bool focused;
    bool disabled;
}

string announce(const Node n) @safe {
    if (!n.focused) return "";
    string result = n.role.length ? n.role : "élément";
    if (n.name.length) result ~= ": " ~ n.name;
    if (n.value.length) result ~= ", " ~ n.value;
    if (n.disabled) result ~= ", désactivé";
    return result;
}
