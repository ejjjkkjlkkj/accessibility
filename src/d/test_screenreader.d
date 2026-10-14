module test_screenreader;
import screenreader;

unittest {
    assert(announce(Node("bouton", "Valider", "", true, false)) == "bouton: Valider");
    assert(announce(Node("bouton", "Valider", "", false, false)) == "");
    assert(announce(Node("champ", "Volume", "80 %", true, true)) == "champ: Volume, 80 %, désactivé");
}
unittest {
    EventQueue q;
    assert(q.push(EventKind.focus, Node("bouton", "Valider", "", true, false)));
    Event e;
    assert(q.pop(e) && e.kind == EventKind.focus);
    assert(e.name[0 .. e.nameLength] == "Valider");
    assert(!q.pop(e));
    foreach (i; 0 .. QueueCapacity) assert(q.push(EventKind.stateChanged, Node.init));
    assert(!q.push(EventKind.focus, Node.init));
    foreach (i; 0 .. QueueCapacity) assert(q.pop(e));
    assert(!q.pop(e));
    auto longName = new char[TextCapacity];
    assert(!q.push(EventKind.focus, Node("", longName.idup, "", true, false)));
}
