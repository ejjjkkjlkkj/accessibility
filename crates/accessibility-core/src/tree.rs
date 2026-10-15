//! Platform-neutral semantic accessibility tree primitives.
//! No OS queries, allocation, input injection, or speech output occur here.
//! Call `validate_tree` before using navigation on externally supplied nodes.

use crate::Role;

pub const STATE_FOCUSED: u64 = 1 << 0;
pub const STATE_FOCUSABLE: u64 = 1 << 1;
pub const STATE_ENABLED: u64 = 1 << 2;
pub const STATE_DISABLED: u64 = 1 << 3;
pub const MAX_NODE_NAME_UTF8_BYTES: usize = 4096;
/// Bound validation work for externally supplied trees in no_std environments.
pub const MAX_ACCESSIBLE_TREE_NODES: usize = 512;
/// Maximum number of parent links accepted between a node and the root.
pub const MAX_ACCESSIBLE_TREE_DEPTH: usize = 64;

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

/// Validate unique IDs, exactly one root, parent references, bounded acyclic ancestry,
/// and the shared UTF-8 name limit. Empty, oversized, or excessively deep trees are invalid.
/// Bounds keep validation finite for untrusted input without heap allocation.
pub fn validate_tree(nodes: &[AccessibleNode<'_>]) -> bool {
    if nodes.is_empty() || nodes.len() > MAX_ACCESSIBLE_TREE_NODES {
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
        if node.parent_id.is_none() {
            roots += 1;
        } else if node.parent_id == Some(node.id) {
            return false;
        }
    }
    if roots != 1 {
        return false;
    }

    // Parent lookup is linear, so explicitly bound both node count and ancestry depth.
    // No recursion or heap allocation is required.
    for node in nodes {
        let mut parent_id = node.parent_id;
        let mut depth = 0usize;
        while let Some(id) = parent_id {
            let parent = match nodes.iter().find(|candidate| candidate.id == id) {
                Some(value) => value,
                None => return false,
            };
            depth += 1;
            if depth > MAX_ACCESSIBLE_TREE_DEPTH {
                return false;
            }
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
    fn rejects_tree_above_node_limit() {
        let mut nodes = [AccessibleNode::new(0, None, Role::Window, 0, "root");
            MAX_ACCESSIBLE_TREE_NODES + 1];
        for (index, node) in nodes.iter_mut().enumerate() {
            *node = if index == 0 {
                AccessibleNode::new(0, None, Role::Window, 0, "root")
            } else {
                AccessibleNode::new(index as u64, Some(0), Role::Group, 0, "")
            };
        }
        assert_eq!(nodes.len(), MAX_ACCESSIBLE_TREE_NODES + 1);
        assert!(!validate_tree(&nodes));
    }

    #[test]
    fn rejects_tree_above_depth_limit() {
        let mut nodes = [AccessibleNode::new(0, None, Role::Window, 0, "root");
            MAX_ACCESSIBLE_TREE_DEPTH + 2];
        for (index, node) in nodes.iter_mut().enumerate() {
            *node = if index == 0 {
                AccessibleNode::new(0, None, Role::Window, 0, "root")
            } else {
                AccessibleNode::new(index as u64, Some(index as u64 - 1), Role::Group, 0, "")
            };
        }
        assert!(!validate_tree(&nodes));
    }
}
