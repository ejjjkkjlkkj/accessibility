#![no_std]

#[derive(Clone, Copy)]
pub struct Node<'a> {
    pub role: &'a str,
    pub name: &'a str,
    pub value: &'a str,
    pub focused: bool,
    pub disabled: bool,
}

/// Returns a compact announcement. This core has no platform API or audio backend.
pub fn announce(node: &Node<'_>) -> Option<heapless::String<256>> {
    if !node.focused { return None; }
    let mut out = heapless::String::<256>::new();
    let _ = out.push_str(if node.role.is_empty() { "élément" } else { node.role });
    if !node.name.is_empty() {
        let _ = out.push_str(": ");
        let _ = out.push_str(node.name);
    }
    if !node.value.is_empty() {
        let _ = out.push_str(", ");
        let _ = out.push_str(node.value);
    }
    if node.disabled { let _ = out.push_str(", désactivé"); }
    Some(out)
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_node_is_announced() {
        let node = Node { role: "bouton", name: "Valider", value: "", focused: true, disabled: false };
        assert_eq!(announce(&node).unwrap().as_str(), "bouton: Valider");
    }

    #[test]
    fn unfocused_node_is_silent() {
        let node = Node { role: "bouton", name: "Valider", value: "", focused: false, disabled: false };
        assert!(announce(&node).is_none());
    }

    #[test]
    fn value_and_disabled_state_are_included() {
        let node = Node { role: "champ", name: "Volume", value: "80 %", focused: true, disabled: true };
        assert_eq!(announce(&node).unwrap().as_str(), "champ: Volume, 80 %, désactivé");
    }
}
