import screenreader_core

doAssert announce(FocusNode(role: "bouton", name: "Valider", focused: true)) == "bouton: Valider"
doAssert announce(FocusNode(role: "bouton", name: "Valider", focused: false)) == ""
doAssert announce(FocusNode(role: "champ", name: "Volume", value: "80 %", focused: true, disabled: true)) == "champ: Volume, 80 %, désactivé"
echo "Nim screen-reader core tests passed"
