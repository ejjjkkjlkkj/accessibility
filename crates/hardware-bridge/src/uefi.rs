//! Safe boundary for a future UEFI PCI I/O Protocol adapter.
//!
//! This module does not call firmware protocols itself. A platform-specific
//! adapter implements `PciConfigReader` using UEFI PCI I/O protocol services;
//! raw ECAM dereferencing and unrestricted port/MMIO access are intentionally
//! absent. Enumeration is bounded to an explicitly supplied bus range.
use crate::{DeviceDescriptor, DeviceOrigin, PciAddress};

pub const PCI_CONFIG_MIN_BYTES: usize = 16;
pub const PCI_VENDOR_ABSENT: u16 = 0xffff;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PciReadError {
    FirmwareError,
    ShortRead,
    InvalidRange,
}

/// Adapter contract: reads PCI configuration bytes through an authorized
/// firmware service. Implementations must not interpret this trait as permission
/// to dereference an ACPI MCFG address directly.
pub trait PciConfigReader {
    fn read_config(
        &mut self,
        address: PciAddress,
        offset: u16,
        output: &mut [u8],
    ) -> Result<(), PciReadError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciFunction {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision_id: u8,
    pub programming_interface: u8,
    pub subclass: u8,
    pub base_class: u8,
    pub header_type: u8,
    pub multifunction: bool,
}

impl PciFunction {
    pub fn read<R: PciConfigReader>(
        reader: &mut R,
        address: PciAddress,
    ) -> Result<Option<Self>, PciReadError> {
        if !address.is_valid() {
            return Err(PciReadError::InvalidRange);
        }
        let mut config = [0u8; PCI_CONFIG_MIN_BYTES];
        reader.read_config(address, 0, &mut config)?;
        let vendor_id = u16::from_le_bytes([config[0], config[1]]);
        if vendor_id == PCI_VENDOR_ABSENT {
            return Ok(None);
        }
        let device_id = u16::from_le_bytes([config[2], config[3]]);
        Ok(Some(Self {
            address,
            vendor_id,
            device_id,
            revision_id: config[8],
            programming_interface: config[9],
            subclass: config[10],
            base_class: config[11],
            header_type: config[14] & 0x7f,
            multifunction: config[14] & 0x80 != 0,
        }))
    }

    pub const fn class_code(&self) -> u32 {
        ((self.base_class as u32) << 16)
            | ((self.subclass as u32) << 8)
            | self.programming_interface as u32
    }

    pub fn descriptor(&self, stable_id: u64) -> DeviceDescriptor<'static> {
        DeviceDescriptor::new(
            stable_id,
            DeviceOrigin::FirmwareReported,
            Some(self.address),
            Some(self.vendor_id),
            Some(self.device_id),
            Some(self.class_code()),
            "UEFI-reported PCI function",
        )
    }
}

/// Iterate every function in a caller-bounded bus range. The adapter is
/// queried through its firmware API; this code never maps physical addresses.
/// Caller supplies a callback to avoid allocation in no_std environments.
pub fn enumerate<R, F>(
    reader: &mut R,
    segment: u16,
    start_bus: u8,
    end_bus: u8,
    mut visit: F,
) -> Result<(), PciReadError>
where
    R: PciConfigReader,
    F: FnMut(PciFunction),
{
    if start_bus > end_bus {
        return Err(PciReadError::InvalidRange);
    }
    for bus in start_bus..=end_bus {
        for device in 0u8..32 {
            let first = PciAddress::new(segment, bus, device, 0);
            let Some(function0) = PciFunction::read(reader, first)? else {
                continue;
            };
            let multi = function0.multifunction;
            visit(function0);
            if multi {
                for function in 1u8..8 {
                    let address = PciAddress::new(segment, bus, device, function);
                    if let Some(found) = PciFunction::read(reader, address)? {
                        visit(found);
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockReader;
    impl PciConfigReader for MockReader {
        fn read_config(&mut self, address: PciAddress, offset: u16, out: &mut [u8]) -> Result<(), PciReadError> {
            out.fill(0);
            if address.bus == 0 && address.device == 1 && address.function == 0 && offset == 0 {
                out[0..2].copy_from_slice(&0x1022u16.to_le_bytes());
                out[2..4].copy_from_slice(&0x145cu16.to_le_bytes());
                out[8] = 0x01;
                out[9] = 0x00;
                out[10] = 0x06;
                out[11] = 0x06;
                out[14] = 0x80; // Multifunction device: scan functions 1..7.
            } else if address.bus == 0 && address.device == 1 && address.function == 3 && offset == 0 {
                out[0..2].copy_from_slice(&0x1234u16.to_le_bytes());
                out[2..4].copy_from_slice(&0x5678u16.to_le_bytes());
            } else if offset == 0 {
                out[0..2].copy_from_slice(&PCI_VENDOR_ABSENT.to_le_bytes());
            }
            Ok(())
        }
    }

    #[test]
    fn parses_firmware_supplied_pci_config_snapshot() {
        let mut reader = MockReader;
        let found = PciFunction::read(&mut reader, PciAddress::new(0, 0, 1, 0)).unwrap().unwrap();
        assert_eq!(found.vendor_id, 0x1022);
        assert_eq!(found.device_id, 0x145c);
        assert_eq!(found.class_code(), 0x060600);
        assert!(found.descriptor(1).is_valid());
    }

    #[test]
    fn firmware_inventory_is_not_mislabeled_as_physical_proof() {
        let mut reader = MockReader;
        let function = PciFunction::read(
            &mut reader,
            PciAddress::new(0, 0, 1, 0),
        ).unwrap().unwrap();
        assert_eq!(function.descriptor(1).origin, DeviceOrigin::FirmwareReported);
    }

    #[test]
    fn absent_vendor_is_not_a_device() {
        let mut reader = MockReader;
        assert_eq!(PciFunction::read(&mut reader, PciAddress::new(0, 0, 2, 0)), Ok(None));
    }

    #[test]
    fn bounded_enumerator_follows_multifunction_header() {
        let mut reader = MockReader;
        let mut functions = 0u8;
        enumerate(&mut reader, 0, 0, 0, |function| {
            functions |= 1u8 << function.address.function;
        }).unwrap();
        assert_eq!(functions, (1u8 << 0) | (1u8 << 3));
    }

    #[test]
    fn rejects_reversed_bus_range_before_scanning() {
        let mut reader = MockReader;
        assert_eq!(enumerate(&mut reader, 0, 4, 3, |_| {}), Err(PciReadError::InvalidRange));
    }

    #[test]
    fn invalid_function_is_rejected_before_reader_call() {
        let mut reader = MockReader;
        assert_eq!(PciFunction::read(&mut reader, PciAddress::new(0, 0, 0, 8)), Err(PciReadError::InvalidRange));
    }
}
