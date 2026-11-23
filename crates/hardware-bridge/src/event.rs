//! Bounded event transport from hardware inventory to an accessibility runtime.
//!
//! Events are descriptive and contain no device handles or hardware capabilities.
//! The consumer decides how to render/speak each message. Queue overflow is
//! reported to the producer instead of silently discarding an event.
use crate::provenance::EvidenceSource;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EventKind {
    DeviceDiscovered = 1,
    DeviceRemoved = 2,
    DeviceStateChanged = 3,
    InventoryWarning = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EventSeverity {
    Information = 1,
    Warning = 2,
    Error = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessibilityEvent {
    pub sequence: u64,
    pub device_id: u64,
    pub kind: EventKind,
    pub severity: EventSeverity,
    pub source: EvidenceSource,
    /// Static, already-sanitized text. Dynamic device names must be bounded
    /// and sanitized by the platform adapter before becoming an event.
    pub message: &'static str,
}

impl AccessibilityEvent {
    pub const fn new(
        sequence: u64,
        device_id: u64,
        kind: EventKind,
        severity: EventSeverity,
        source: EvidenceSource,
        message: &'static str,
    ) -> Self {
        Self { sequence, device_id, kind, severity, source, message }
    }

    pub const fn is_valid(&self) -> bool {
        self.sequence != 0 && !self.message.is_empty()
            && (self.kind as u8) >= 1
            && (self.severity as u8) >= 1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueError {
    Full,
    InvalidEvent,
}

/// Fixed-capacity FIFO for no_std environments; no heap allocation.
pub struct EventQueue<const N: usize> {
    slots: [Option<AccessibilityEvent>; N],
    head: usize,
    len: usize,
}

impl<const N: usize> EventQueue<N> {
    pub const fn new() -> Self {
        Self { slots: [None; N], head: 0, len: 0 }
    }

    pub const fn capacity(&self) -> usize { N }
    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
    pub const fn is_full(&self) -> bool { self.len == N }

    pub fn push(&mut self, event: AccessibilityEvent) -> Result<(), QueueError> {
        if !event.is_valid() {
            return Err(QueueError::InvalidEvent);
        }
        if self.len == N {
            return Err(QueueError::Full);
        }
        let index = (self.head + self.len) % N.max(1);
        self.slots[index] = Some(event);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<AccessibilityEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.slots[self.head].take();
        self.head = (self.head + 1) % N.max(1);
        self.len -= 1;
        event
    }
}

impl<const N: usize> Default for EventQueue<N> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(sequence: u64, message: &'static str) -> AccessibilityEvent {
        AccessibilityEvent::new(
            sequence, 0x100, EventKind::DeviceDiscovered, EventSeverity::Information,
            EvidenceSource::UefiPciIoProtocol, message,
        )
    }

    #[test]
    fn fifo_preserves_order_and_reuses_slots() {
        let mut queue = EventQueue::<2>::new();
        assert_eq!(queue.push(event(1, "Device detected")), Ok(()));
        assert_eq!(queue.push(event(2, "Audio controller detected")), Ok(()));
        assert!(queue.is_full());
        assert_eq!(queue.pop().unwrap().sequence, 1);
        assert_eq!(queue.push(event(3, "Keyboard controller detected")), Ok(()));
        assert_eq!(queue.pop().unwrap().sequence, 2);
        assert_eq!(queue.pop().unwrap().sequence, 3);
        assert!(queue.is_empty());
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn full_queue_reports_backpressure_without_dropping_events() {
        let mut queue = EventQueue::<1>::new();
        assert_eq!(queue.push(event(1, "First")), Ok(()));
        assert_eq!(queue.push(event(2, "Second")), Err(QueueError::Full));
        assert_eq!(queue.pop().unwrap().message, "First");
    }

    #[test]
    fn rejects_invalid_event() {
        let mut queue = EventQueue::<2>::new();
        assert_eq!(queue.push(event(0, "invalid sequence")), Err(QueueError::InvalidEvent));
        assert_eq!(queue.push(event(1, "")), Err(QueueError::InvalidEvent));
    }

    #[test]
    fn zero_capacity_queue_fails_safely() {
        let mut queue = EventQueue::<0>::new();
        assert_eq!(queue.push(event(1, "No capacity")), Err(QueueError::Full));
        assert_eq!(queue.pop(), None);
    }
}
