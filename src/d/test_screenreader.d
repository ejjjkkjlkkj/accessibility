module test_screenreader;
import screenreader;
import std.exception : assertThrown;

unittest {
    assert(announce(Node("bouton", "Valider", "", true, false)) == "bouton: Valider");
    assert(announce(Node("bouton", "Valider", "", false, false)) == "");
    assert(announce(Node("champ", "Volume", "80 %", true, true)) == "champ: Volume, 80 %, désactivé");
}
