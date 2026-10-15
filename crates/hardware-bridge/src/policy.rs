//! Explicit, deny-by-default policy checks for resource requests.
//!
//! A successful evaluation is a software policy decision only. It is not an
//! operating-system handle, UEFI protocol, mapping, or permission to perform I/O.
//! Platform adapters must independently verify platform authorization before
//! accessing a resource.
use crate::{ResourceKind, ResourceWindow};

const KIND_MMIO: u16 = 1 << 0;
const KIND_IO_PORT: u16 = 1 << 1;
const KIND_INTERRUPT: u16 = 1 << 2;
const KIND_DMA: u16 = 1 << 3;
const KIND_SHARED_MEMORY: u16 = 1 << 4;

const fn kind_bit(kind: ResourceKind) -> u16 {
    match kind {
        ResourceKind::Mmio => KIND_MMIO,
        ResourceKind::IoPort => KIND_IO_PORT,
        ResourceKind::Interrupt => KIND_INTERRUPT,
        ResourceKind::DmaBuffer => KIND_DMA,
        ResourceKind::SharedMemory => KIND_SHARED_MEMORY,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    InvalidRange,
    ResourceKindMismatch,
    OutsideDeclaredWindow,
    KindDenied,
    WriteDenied,
    WindowTooLarge,
}

/// Allowlist policy. Construct with deny_all(), then explicitly enable resource
/// kinds and write access. DMA remains disabled unless explicitly enabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourcePolicy {
    allowed_kinds: u16,
    writable_kinds: u16,
    max_window_bytes: u64,
}

impl ResourcePolicy {
    pub const fn deny_all() -> Self {
        Self { allowed_kinds: 0, writable_kinds: 0, max_window_bytes: 0 }
    }

    pub const fn with_max_window_bytes(mut self, maximum: u64) -> Self {
        self.max_window_bytes = maximum;
        self
    }

    pub const fn allow_kind(mut self, kind: ResourceKind, allow_write: bool) -> Self {
        self.allowed_kinds |= kind_bit(kind);
        if allow_write {
            self.writable_kinds |= kind_bit(kind);
        }
        self
    }

    pub fn evaluate(
        &self,
        declared: ResourceWindow,
        requested: ResourceWindow,
        mode: AccessMode,
    ) -> Result<ValidatedResourceRequest, PolicyError> {
        if !declared.is_valid() || !requested.is_valid() {
            return Err(PolicyError::InvalidRange);
        }
        if declared.kind != requested.kind {
            return Err(PolicyError::ResourceKindMismatch);
        }
        let declared_end = declared.end_exclusive().ok_or(PolicyError::InvalidRange)?;
        let requested_end = requested.end_exclusive().ok_or(PolicyError::InvalidRange)?;
        if requested.base < declared.base || requested_end > declared_end {
            return Err(PolicyError::OutsideDeclaredWindow);
        }
        let bit = kind_bit(requested.kind);
        if self.allowed_kinds & bit == 0 {
            return Err(PolicyError::KindDenied);
        }
        if mode == AccessMode::Write && self.writable_kinds & bit == 0 {
            return Err(PolicyError::WriteDenied);
        }
        if requested.length > self.max_window_bytes {
            return Err(PolicyError::WindowTooLarge);
        }
        Ok(ValidatedResourceRequest {
            kind: requested.kind,
            base: requested.base,
            length: requested.length,
            mode,
        })
    }
}

/// Validated request metadata; deliberately contains no pointer or device handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedResourceRequest {
    kind: ResourceKind,
    base: u64,
    length: u64,
    mode: AccessMode,
}

impl ValidatedResourceRequest {
    pub const fn kind(&self) -> ResourceKind { self.kind }
    pub const fn base(&self) -> u64 { self.base }
    pub const fn length(&self) -> u64 { self.length }
    pub const fn mode(&self) -> AccessMode { self.mode }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_all_is_default_and_policy_can_allow_read_only_mmio() {
        let declared = ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0x1000);
        let request = ResourceWindow::new(ResourceKind::Mmio, 0x1100, 0x100);
        assert_eq!(ResourcePolicy::deny_all().evaluate(declared, request, AccessMode::Read), Err(PolicyError::KindDenied));

        let policy = ResourcePolicy::deny_all()
            .with_max_window_bytes(0x200)
            .allow_kind(ResourceKind::Mmio, false);
        let validated = policy.evaluate(declared, request, AccessMode::Read).unwrap();
        assert_eq!(validated.base(), 0x1100);
        assert_eq!(validated.length(), 0x100);
        assert_eq!(validated.mode(), AccessMode::Read);
        assert_eq!(policy.evaluate(declared, request, AccessMode::Write), Err(PolicyError::WriteDenied));
    }

    #[test]
    fn rejects_out_of_window_kind_mismatch_and_oversized_requests() {
        let declared = ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0x100);
        let policy = ResourcePolicy::deny_all()
            .with_max_window_bytes(0x40)
            .allow_kind(ResourceKind::Mmio, true);
        assert_eq!(policy.evaluate(declared, ResourceWindow::new(ResourceKind::Mmio, 0x10f0, 0x20), AccessMode::Read), Err(PolicyError::OutsideDeclaredWindow));
        assert_eq!(policy.evaluate(declared, ResourceWindow::new(ResourceKind::IoPort, 0x1000, 0x10), AccessMode::Read), Err(PolicyError::ResourceKindMismatch));
        assert_eq!(policy.evaluate(declared, ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0x80), AccessMode::Read), Err(PolicyError::WindowTooLarge));
    }

    #[test]
    fn rejects_empty_and_overflowing_ranges() {
        let policy = ResourcePolicy::deny_all().with_max_window_bytes(u64::MAX).allow_kind(ResourceKind::Mmio, true);
        assert_eq!(policy.evaluate(ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0), ResourceWindow::new(ResourceKind::Mmio, 0x1000, 1), AccessMode::Read), Err(PolicyError::InvalidRange));
        assert_eq!(policy.evaluate(ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0x100), ResourceWindow::new(ResourceKind::Mmio, u64::MAX - 2, 8), AccessMode::Read), Err(PolicyError::InvalidRange));
    }

    #[test]
    fn dma_requires_explicit_allowlist_entry() {
        let declared = ResourceWindow::new(ResourceKind::DmaBuffer, 0x8000, 0x1000);
        let request = ResourceWindow::new(ResourceKind::DmaBuffer, 0x8000, 0x100);
        let policy = ResourcePolicy::deny_all().with_max_window_bytes(0x1000);
        assert_eq!(policy.evaluate(declared, request, AccessMode::Read), Err(PolicyError::KindDenied));
        let explicit = policy.allow_kind(ResourceKind::DmaBuffer, false);
        assert!(explicit.evaluate(declared, request, AccessMode::Read).is_ok());
    }
}
