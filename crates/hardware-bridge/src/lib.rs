#![no_std]

pub mod acpi;
pub mod provenance;
pub mod uefi;
//! Platform-neutral contracts for describing physical and software-defined devices.
//!
//! This crate deliberately performs no PCI configuration access, MMIO, port I/O,
//! DMA, interrupt registration, firmware mutation, or device emulation. A future
//! platform adapter must validate and authorize resources before touching hardware.
//! Passing these data-model tests is not evidence of operation on physical hardware.

pub const HARDWARE_BRIDGE_ABI_VERSION: u32 = 1;
pub const MAX_DEVICE_NAME_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DeviceOrigin {
    PhysicalInventory = 1,
    SoftwareDefined = 2,
    FirmwareReported = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ResourceKind {
    Mmio = 1,
    IoPort = 2,
    Interrupt = 3,
    DmaBuffer = 4,
    SharedMemory = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciAddress {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciAddress {
    pub const fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        Self { segment, bus, device, function }
    }

    pub const fn is_valid(&self) -> bool {
        self.device < 32 && self.function < 8
    }
}

/// A resource range is descriptive only. It does not grant permission to access it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceWindow {
    pub kind: ResourceKind,
    pub base: u64,
    pub length: u64,
}

impl ResourceWindow {
    pub const fn new(kind: ResourceKind, base: u64, length: u64) -> Self {
        Self { kind, base, length }
    }

    /// Checks a non-empty, non-overflowing half-open range [base, base + length).
    pub fn is_valid(&self) -> bool {
        self.length > 0 && self.base.checked_add(self.length).is_some()
    }

    pub fn end_exclusive(&self) -> Option<u64> {
        if self.is_valid() { self.base.checked_add(self.length) } else { None }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        match (self.end_exclusive(), other.end_exclusive()) {
            (Some(a_end), Some(b_end)) => self.base < b_end && other.base < a_end,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceDescriptor<'a> {
    pub abi_version: u32,
    pub id: u64,
    pub origin: DeviceOrigin,
    pub pci: Option<PciAddress>,
    pub vendor_id: Option<u16>,
    pub device_id: Option<u16>,
    pub class_code: Option<u32>,
    pub name: &'a str,
}

impl<'a> DeviceDescriptor<'a> {
    pub const fn new(
        id: u64,
        origin: DeviceOrigin,
        pci: Option<PciAddress>,
        vendor_id: Option<u16>,
        device_id: Option<u16>,
        class_code: Option<u32>,
        name: &'a str,
    ) -> Self {
        Self {
            abi_version: HARDWARE_BRIDGE_ABI_VERSION,
            id,
            origin,
            pci,
            vendor_id,
            device_id,
            class_code,
            name,
        }
    }

    /// Validates metadata only; no hardware is probed and no resource is accessed.
    pub fn is_valid(&self) -> bool {
        self.abi_version == HARDWARE_BRIDGE_ABI_VERSION
            && self.id != 0
            && !self.name.is_empty()
            && self.name.len() <= MAX_DEVICE_NAME_BYTES
            && self.pci.map_or(true, |address| address.is_valid())
            && self.vendor_id.map_or(true, |id| id != 0xffff)
            && self.vendor_id.is_some() == self.device_id.is_some()
    }
}

/// Capability-free inventory record: listing a resource is not authorization to use it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventoryEntry<'a> {
    pub device: DeviceDescriptor<'a>,
    pub resource_count: u8,
}

impl<'a> InventoryEntry<'a> {
    pub const fn new(device: DeviceDescriptor<'a>, resource_count: u8) -> Self {
        Self { device, resource_count }
    }

    pub fn is_valid(&self) -> bool {
        self.device.is_valid()
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    fn physical_device() -> DeviceDescriptor<'static> {
        DeviceDescriptor::new(
            1,
            DeviceOrigin::PhysicalInventory,
            Some(PciAddress::new(0, 2, 3, 0)),
            Some(0x1022),
            Some(0x145c),
            Some(0x060000),
            "PCI host bridge",
        )
    }

    #[test]
    fn accepts_well_formed_physical_inventory_metadata() {
        let device = physical_device();
        assert!(device.is_valid());
        assert_eq!(device.abi_version, HARDWARE_BRIDGE_ABI_VERSION);
        assert_eq!(device.origin, DeviceOrigin::PhysicalInventory);
    }

    #[test]
    fn rejects_invalid_pci_address_and_unmatched_ids() {
        let invalid_address = DeviceDescriptor::new(
            1, DeviceOrigin::PhysicalInventory,
            Some(PciAddress::new(0, 0, 32, 0)),
            Some(0x1022), Some(0x145c), None, "device",
        );
        let unmatched_ids = DeviceDescriptor::new(
            2, DeviceOrigin::PhysicalInventory, None, Some(0x1022), None, None, "device",
        );
        let unknown_vendor = DeviceDescriptor::new(
            3, DeviceOrigin::PhysicalInventory, None, Some(0xffff), Some(1), None, "device",
        );
        assert!(!invalid_address.is_valid());
        assert!(!unmatched_ids.is_valid());
        assert!(!unknown_vendor.is_valid());
    }

    #[test]
    fn rejects_empty_names_and_zero_ids() {
        let no_id = DeviceDescriptor::new(
            0, DeviceOrigin::SoftwareDefined, None, None, None, None, "logical device",
        );
        let no_name = DeviceDescriptor::new(
            1, DeviceOrigin::SoftwareDefined, None, None, None, None, "",
        );
        assert!(!no_id.is_valid());
        assert!(!no_name.is_valid());
    }

    #[test]
    fn validates_resource_ranges_without_touching_hardware() {
        let valid = ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0x100);
        let empty = ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0);
        let overflow = ResourceWindow::new(ResourceKind::Mmio, u64::MAX - 3, 8);
        assert!(valid.is_valid());
        assert_eq!(valid.end_exclusive(), Some(0x1100));
        assert!(!empty.is_valid());
        assert!(!overflow.is_valid());
        assert_eq!(overflow.end_exclusive(), None);
    }

    #[test]
    fn detects_overlaps_and_allows_adjacent_ranges() {
        let first = ResourceWindow::new(ResourceKind::Mmio, 0x1000, 0x100);
        let overlap = ResourceWindow::new(ResourceKind::Mmio, 0x1080, 0x100);
        let adjacent = ResourceWindow::new(ResourceKind::Mmio, 0x1100, 0x100);
        assert!(first.overlaps(&overlap));
        assert!(!first.overlaps(&adjacent));
    }

    #[test]
    fn inventory_validity_does_not_imply_hardware_access() {
        let entry = InventoryEntry::new(physical_device(), 2);
        assert!(entry.is_valid());
        assert_eq!(entry.resource_count, 2);
    }
}
