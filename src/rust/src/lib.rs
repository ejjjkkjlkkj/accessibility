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
