#![no_std]

use heapless::{Deque, String};

#[derive(Clone, Copy)]
pub struct Node<'a> {
    pub role: &'a str,
    pub name: &'a str,
    pub value: &'a str,
    pub focused: bool,
    pub disabled: bool,
}

/// Returns a compact announcement. This core has no platform API or audio backend.
pub fn announce(node: &Node<'_>) -> Option<String<256>> {
    if !node.focused { return None; }
    let mut out = String::<256>::new();
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    Focus,
    ValueChanged,
    StateChanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub kind: EventKind,
    pub role: String<96>,
    pub name: String<96>,
    pub value: String<96>,
    pub focused: bool,
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueError {
    Full,
    TextTooLong,
}

impl Event {
    fn from_node(kind: EventKind, node: &Node<'_>) -> Result<Self, QueueError> {
        let mut role = String::<96>::new();
        let mut name = String::<96>::new();
        let mut value = String::<96>::new();
        role.push_str(node.role).map_err(|_| QueueError::TextTooLong)?;
        name.push_str(node.name).map_err(|_| QueueError::TextTooLong)?;
        value.push_str(node.value).map_err(|_| QueueError::TextTooLong)?;
        Ok(Self {
            kind, role, name, value,
            focused: node.focused,
            disabled: node.disabled,
        })
    }
}

/// Fixed-capacity, allocation-free event queue; events own bounded text snapshots.
pub struct EventQueue {
    items: Deque<Event, 16>,
}

impl EventQueue {
    pub const fn new() -> Self {
        Self { items: Deque::new() }
    }

    pub fn push(&mut self, kind: EventKind, node: &Node<'_>) -> Result<(), QueueError> {
        if self.items.is_full() { return Err(QueueError::Full); }
        let event = Event::from_node(kind, node)?;
        self.items.push_back(event).map_err(|_| QueueError::Full)
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.items.pop_front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
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

    #[test]
    fn queue_preserves_fifo_and_owns_text_snapshot() {
        let mut queue = EventQueue::new();
        let node = Node { role: "bouton", name: "Valider", value: "", focused: true, disabled: false };
        queue.push(EventKind::Focus, &node).unwrap();
        let first = queue.pop().unwrap();
        assert_eq!(first.kind, EventKind::Focus);
        assert_eq!(first.name.as_str(), "Valider");
        assert!(queue.pop().is_none());
    }

    #[test]
    fn queue_rejects_oversized_text_and_full_capacity() {
        let mut queue = EventQueue::new();
        let long_name = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let too_long = Node { role: "bouton", name: long_name, value: "", focused: true, disabled: false };
        assert_eq!(queue.push(EventKind::Focus, &too_long), Err(QueueError::TextTooLong));
        let node = Node { role: "bouton", name: "OK", value: "", focused: true, disabled: false };
        for _ in 0..16 { queue.push(EventKind::Focus, &node).unwrap(); }
        assert_eq!(queue.push(EventKind::Focus, &node), Err(QueueError::Full));
        assert_eq!(queue.len(), 16);
        for _ in 0..16 { assert!(queue.pop().is_some()); }
        assert!(queue.is_empty());
    }
}
