//! Platform-neutral semantic accessibility tree primitives.
//! No OS queries, allocation, input injection, or speech output occur here.
//! Call `validate_tree` before using navigation on externally supplied nodes.

use crate::Role;

pub const STATE_FOCUSED: u64 = 1 << 0;
pub const STATE_FOCUSABLE: u64 = 1 << 1;
pub const STATE_ENABLED: u64 = 1 << 2;
pub const STATE_DISABLED: u64 = 1 << 3;
pub const MAX_NODE_NAME_UTF8_BYTES: usize = 4096;
pub const MAX_TREE_NODES: usize = 4096;

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

/// Validate a bounded, parent-before-child tree with unique IDs, exactly one
/// root, valid parent references, and bounded UTF-8 names. Parent-before-child
/// ordering makes cycles impossible without recursion or heap allocation.
pub fn validate_tree(nodes: &[AccessibleNode<'_>]) -> bool {
    if nodes.is_empty() || nodes.len() > MAX_TREE_NODES {
        return false;
    }

    let mut roots = 0usize;
    for (index, node) in nodes.iter().enumerate() {
        if node.name.len() > MAX_NODE_NAME_UTF8_BYTES {
            return false;
        }
        for other in nodes.iter().skip(index + 1) {
            if node.id == other.id {
                return false;
            }
        }
        match node.parent_id {
            None => {
                roots += 1;
                if index != 0 {
                    return false;
                }
            }
            Some(parent_id) => {
                if parent_id == node.id
                    || !nodes[..index].iter().any(|candidate| candidate.id == parent_id)
                {
                    return false;
                }
            }
        }
    }
    roots == 1
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
    fn rejects_forward_parent_references() {
        let nodes = [
            AccessibleNode::new(1, None, Role::Window, 0, "Main"),
            AccessibleNode::new(2, Some(3), Role::Button, TARGET, "Child"),
            AccessibleNode::new(3, Some(1), Role::Group, 0, "Parent"),
        ];
        assert!(!validate_tree(&nodes));
    }

    #[test]
    fn rejects_tree_larger_than_fixed_bound() {
        let nodes = std::vec![
            AccessibleNode::new(1, None, Role::Window, 0, "Main");
            MAX_TREE_NODES + 1
        ];
        assert!(!validate_tree(&nodes));
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
}
