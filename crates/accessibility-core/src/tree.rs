//! Platform-neutral semantic accessibility tree primitives.
//! No OS queries, allocation, input injection, or speech output occur here.
//! Call `validate_tree` before using navigation on externally supplied nodes.

use crate::Role;

pub const STATE_FOCUSED: u64 = 1 << 0;
pub const STATE_FOCUSABLE: u64 = 1 << 1;
pub const STATE_ENABLED: u64 = 1 << 2;
pub const STATE_DISABLED: u64 = 1 << 3;
/// Bit mask of state flags defined by this ABI revision.
pub const KNOWN_STATE_FLAGS: u64 = STATE_FOCUSED | STATE_FOCUSABLE | STATE_ENABLED | STATE_DISABLED;
pub const MAX_NODE_NAME_UTF8_BYTES: usize = 4096;
/// Hard bound to limit quadratic validation work on untrusted node slices.
pub const MAX_ACCESSIBLE_NODES: usize = 1024;

/// Reject undefined or contradictory state combinations in a single semantic event.
pub const fn state_flags_are_valid(state_flags: u64) -> bool {
    if state_flags & !KNOWN_STATE_FLAGS != 0 {
        return false;
    }
    let enabled = state_flags & STATE_ENABLED != 0;
    let disabled = state_flags & STATE_DISABLED != 0;
    if enabled && disabled {
        return false;
    }
    let focused = state_flags & STATE_FOCUSED != 0;
    !focused || (
        state_flags & STATE_FOCUSABLE != 0
            && enabled
            && !disabled
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessibleNode<'a> {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub role: Role,
    pub state_flags: u64,
    pub name: &'a str,
}

impl<'a> AccessibleNode<'a> {
    pub const fn new(
        id: u64,
        parent_id: Option<u64>,
        role: Role,
        state_flags: u64,
        name: &'a str,
    ) -> Self {
        Self { id, parent_id, role, state_flags, name }
    }

    pub const fn is_focus_target(&self) -> bool {
        (self.state_flags & STATE_FOCUSABLE) != 0
            && (self.state_flags & STATE_ENABLED) != 0
            && (self.state_flags & STATE_DISABLED) == 0
    }
}

/// Validate unique IDs, exactly one root, parent references, acyclic ancestry,
/// and the shared UTF-8 name limit. An empty slice is not a valid tree.
pub fn validate_tree(nodes: &[AccessibleNode<'_>]) -> bool {
    if nodes.is_empty() || nodes.len() > MAX_ACCESSIBLE_NODES {
        return false;
    }

    let mut roots = 0usize;
    let mut focused_nodes = 0usize;
    for (index, node) in nodes.iter().enumerate() {
        if node.name.len() > MAX_NODE_NAME_UTF8_BYTES || node.name.contains('\0') {
            return false;
        }
        if !state_flags_are_valid(node.state_flags) {
            return false;
        }
        if (node.state_flags & STATE_FOCUSED) != 0 {
            focused_nodes += 1;
        }
        for other in nodes.iter().skip(index + 1) {
            if node.id == other.id {
                return false;
            }
        }
        if node.parent_id.is_none() {
            roots += 1;
        } else if node.parent_id == Some(node.id) {
            return false;
        }
    }
    if roots != 1 || focused_nodes > 1 {
        return false;
    }

    // Every ancestry chain must terminate at the single root. The depth bound
    // also rejects cycles without recursion or heap allocation.
    for node in nodes {
        let mut parent_id = node.parent_id;
        let mut depth = 0usize;
        while let Some(id) = parent_id {
            let parent = match nodes.iter().find(|candidate| candidate.id == id) {
                Some(value) => value,
                None => return false,
            };
            depth += 1;
            if depth >= nodes.len() && parent.parent_id.is_some() {
                return false;
            }
            parent_id = parent.parent_id;
        }
    }
    true
}

/// Return the next enabled, focusable node in slice order. No wraparound.
pub fn next_focusable(nodes: &[AccessibleNode<'_>], after_id: Option<u64>) -> Option<u64> {
    let start = match after_id {
        None => 0,
        Some(id) => nodes.iter().position(|node| node.id == id)?.saturating_add(1),
    };
    nodes.iter().skip(start).find(|node| node.is_focus_target()).map(|node| node.id)
}

/// Return the previous enabled, focusable node in slice order. No wraparound.
pub fn previous_focusable(nodes: &[AccessibleNode<'_>], before_id: Option<u64>) -> Option<u64> {
    let end = match before_id {
        None => nodes.len(),
        Some(id) => nodes.iter().position(|node| node.id == id)?,
    };
    nodes.iter().take(end).rev().find(|node| node.is_focus_target()).map(|node| node.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    const TARGET: u64 = STATE_FOCUSABLE | STATE_ENABLED;

    #[test]
    fn validates_tree_and_bounded_focus_navigation() {
        let nodes = [
            AccessibleNode::new(1, None, Role::Window, 0, "Main"),
            AccessibleNode::new(2, Some(1), Role::Text, 0, "Title"),
            AccessibleNode::new(3, Some(1), Role::Button, TARGET, "Continue"),
            AccessibleNode::new(4, Some(1), Role::Button, STATE_FOCUSABLE | STATE_DISABLED, "Disabled"),
            AccessibleNode::new(5, Some(1), Role::Link, TARGET, "Help"),
        ];
        assert!(validate_tree(&nodes));
        assert_eq!(next_focusable(&nodes, None), Some(3));
        assert_eq!(next_focusable(&nodes, Some(3)), Some(5));
        assert_eq!(next_focusable(&nodes, Some(5)), None);
        assert_eq!(previous_focusable(&nodes, Some(5)), Some(3));
        assert_eq!(previous_focusable(&nodes, Some(3)), None);
        assert_eq!(next_focusable(&nodes, Some(999)), None);
    }

    #[test]
    fn rejects_empty_duplicate_or_missing_parent_trees() {
        assert!(!validate_tree(&[]));
        let duplicate = [
            AccessibleNode::new(1, None, Role::Window, 0, ""),
            AccessibleNode::new(1, None, Role::Button, TARGET, "x"),
        ];
        assert!(!validate_tree(&duplicate));
        let missing_parent = [
            AccessibleNode::new(1, None, Role::Window, 0, ""),
            AccessibleNode::new(2, Some(99), Role::Button, TARGET, "x"),
        ];
        assert!(!validate_tree(&missing_parent));
    }

    #[test]
    fn rejects_cycles_and_oversize_names() {
        let cycle = [
            AccessibleNode::new(1, None, Role::Window, 0, ""),
            AccessibleNode::new(2, Some(3), Role::Group, 0, ""),
            AccessibleNode::new(3, Some(2), Role::Group, 0, ""),
        ];
        assert!(!validate_tree(&cycle));
        let long_name = "x".repeat(MAX_NODE_NAME_UTF8_BYTES + 1);
        let oversized = [AccessibleNode::new(1, None, Role::Window, 0, &long_name)];
        assert!(!validate_tree(&oversized));
    }

    #[test]
    fn rejects_names_containing_nul() {
        let nodes = [
            AccessibleNode::new(1, None, Role::Window, 0, "Main"),
            AccessibleNode::new(2, Some(1), Role::Button, TARGET, "Save\0malformed"),
        ];
        assert!(!validate_tree(&nodes));
    }

    #[test]
    fn exhaustively_checks_all_low_byte_state_flag_combinations() {
        for flags in 0u64..=255 {
            let expected = matches!(flags, 0 | 2 | 4 | 6 | 7 | 8 | 10);
            assert_eq!(state_flags_are_valid(flags), expected, "flags={flags:#x}");
        }
    }

    #[test]
    fn rejects_trees_over_the_node_limit() {
        let nodes: Vec<_> = (0..=MAX_ACCESSIBLE_NODES)
            .map(|index| {
                if index == 0 {
                    AccessibleNode::new(1, None, Role::Window, 0, "Main")
                } else {
                    AccessibleNode::new(
                        (index as u64) + 1,
                        Some(1),
                        Role::Group,
                        0,
                        "Node",
                    )
                }
            })
            .collect();
        assert_eq!(nodes.len(), MAX_ACCESSIBLE_NODES + 1);
        assert!(!validate_tree(&nodes));
    }

    #[test]
    fn accepts_one_valid_focused_target() {
        let nodes = [
            AccessibleNode::new(1, None, Role::Window, 0, "Main"),
            AccessibleNode::new(
                2,
                Some(1),
                Role::Button,
                STATE_FOCUSABLE | STATE_ENABLED | STATE_FOCUSED,
                "Continue",
            ),
        ];
        assert!(validate_tree(&nodes));
    }

    #[test]
    fn rejects_contradictory_unknown_and_multiple_focus_states() {
        let enabled_and_disabled = [
            AccessibleNode::new(1, None, Role::Window, 0, "Main"),
            AccessibleNode::new(2, Some(1), Role::Button, TARGET | STATE_DISABLED, "Bad"),
        ];
        assert!(!validate_tree(&enabled_and_disabled));

        let focused_but_not_focusable = [
            AccessibleNode::new(1, None, Role::Window, 0, "Main"),
            AccessibleNode::new(2, Some(1), Role::Button, STATE_FOCUSED, "Bad"),
        ];
        assert!(!validate_tree(&focused_but_not_focusable));

        let multiple_focused = [
            AccessibleNode::new(1, None, Role::Window, 0, "Main"),
            AccessibleNode::new(
                2,
                Some(1),
                Role::Button,
                TARGET | STATE_FOCUSED,
                "First",
            ),
            AccessibleNode::new(
                3,
                Some(1),
                Role::Button,
                TARGET | STATE_FOCUSED,
                "Second",
            ),
        ];
        assert!(!validate_tree(&multiple_focused));

        let unknown_bits = [
            AccessibleNode::new(1, None, Role::Window, 1 << 63, "Main"),
        ];
        assert!(!validate_tree(&unknown_bits));
    }
}
