//! Bounded, allocation-free parsers for ACPI table bytes.
//!
//! The caller must obtain bytes from a trusted, mapped firmware table region.
//! This module only parses a byte slice: it never scans physical memory, maps
//! addresses, or performs PCI/MMIO/port I/O. A parsed MCFG window is metadata,
//! not authorization to access the address range.

pub const SDT_HEADER_LEN: usize = 36;
pub const RSDP_V1_LEN: usize = 20;
pub const RSDP_V2_MIN_LEN: usize = 36;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rsdp<'a> {
    pub revision: u8,
    pub rsdt_address: u32,
    pub xsdt_address: Option<u64>,
    pub length: usize,
    pub bytes: &'a [u8],
}

/// Parse an ACPI Root System Description Pointer from caller-supplied bytes.
/// The caller is responsible for obtaining the candidate from the UEFI
/// configuration table or another trusted source; this function does not scan
/// physical memory.
pub fn parse_rsdp(bytes: &[u8]) -> Result<Rsdp<'_>, AcpiError> {
    if bytes.len() < RSDP_V1_LEN {
        return Err(AcpiError::Truncated);
    }
    if &bytes[0..8] != b"RSD PTR " {
        return Err(AcpiError::BadSignature);
    }
    if bytes[..RSDP_V1_LEN].iter().fold(0u8, |sum, b| sum.wrapping_add(*b)) != 0 {
        return Err(AcpiError::BadChecksum);
    }
    let revision = bytes[15];
    let rsdt_address = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    if revision < 2 {
        return Ok(Rsdp { revision, rsdt_address, xsdt_address: None, length: RSDP_V1_LEN, bytes: &bytes[..RSDP_V1_LEN] });
    }
    if bytes.len() < RSDP_V2_MIN_LEN {
        return Err(AcpiError::Truncated);
    }
    let length = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as usize;
    if length < RSDP_V2_MIN_LEN || length > bytes.len() {
        return Err(AcpiError::InvalidLength);
    }
    let extended = &bytes[..length];
    if extended.iter().fold(0u8, |sum, b| sum.wrapping_add(*b)) != 0 {
        return Err(AcpiError::BadChecksum);
    }
    let xsdt_address = u64::from_le_bytes([
        bytes[24], bytes[25], bytes[26], bytes[27],
        bytes[28], bytes[29], bytes[30], bytes[31],
    ]);
    Ok(Rsdp {
        revision,
        rsdt_address,
        xsdt_address: Some(xsdt_address),
        length,
        bytes: extended,
    })
}

pub const MCFG_HEADER_LEN: usize = 44;
pub const MCFG_ALLOCATION_LEN: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpiError {
    Truncated,
    BadSignature,
    InvalidLength,
    BadChecksum,
    InvalidPciRange,
    AddressOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SdtHeader<'a> {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub bytes: &'a [u8],
}

/// Parse an ACPI System Description Table and validate its declared length and
/// 8-bit checksum. The input may contain bytes after the declared table.
pub fn parse_sdt(bytes: &[u8], expected_signature: [u8; 4]) -> Result<SdtHeader<'_>, AcpiError> {
    if bytes.len() < SDT_HEADER_LEN {
        return Err(AcpiError::Truncated);
    }
    if bytes[0..4] != expected_signature {
        return Err(AcpiError::BadSignature);
    }
    let length = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let length_usize = length as usize;
    if length_usize < SDT_HEADER_LEN || length_usize > bytes.len() {
        return Err(AcpiError::InvalidLength);
    }
    let table = &bytes[..length_usize];
    if table.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) != 0 {
        return Err(AcpiError::BadChecksum);
    }
    Ok(SdtHeader {
        signature: [bytes[0], bytes[1], bytes[2], bytes[3]],
        length,
        revision: bytes[8],
        bytes: table,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McfgAllocation {
    pub base_address: u64,
    pub segment_group: u16,
    pub start_bus: u8,
    pub end_bus: u8,
}

impl McfgAllocation {
    /// Validate the allocation's bus range and calculate the exclusive end
    /// for its ECAM window. Each PCI bus occupies 1 MiB in ECAM.
    pub fn ecam_end_exclusive(&self) -> Result<u64, AcpiError> {
        if self.start_bus > self.end_bus {
            return Err(AcpiError::InvalidPciRange);
        }
        let bus_count = (self.end_bus as u64) - (self.start_bus as u64) + 1;
        let length = bus_count.checked_mul(1 << 20).ok_or(AcpiError::AddressOverflow)?;
        self.base_address.checked_add(length).ok_or(AcpiError::AddressOverflow)
    }

    /// Calculate an ECAM address for a device/function/register, without
    /// dereferencing it or accessing hardware.
    pub fn ecam_address(
        &self,
        bus: u8,
        device: u8,
        function: u8,
        register: u16,
    ) -> Result<u64, AcpiError> {
        if bus < self.start_bus || bus > self.end_bus || device >= 32 || function >= 8 || register >= 4096 {
            return Err(AcpiError::InvalidPciRange);
        }
        let bus_offset = ((bus - self.start_bus) as u64)
            .checked_shl(20).ok_or(AcpiError::AddressOverflow)?;
        let device_offset = (device as u64) << 15;
        let function_offset = (function as u64) << 12;
        self.base_address
            .checked_add(bus_offset)
            .and_then(|v| v.checked_add(device_offset))
            .and_then(|v| v.checked_add(function_offset))
            .and_then(|v| v.checked_add(register as u64))
            .ok_or(AcpiError::AddressOverflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McfgTable<'a> {
    pub header: SdtHeader<'a>,
    allocations: &'a [u8],
}

impl<'a> McfgTable<'a> {
    /// Parse a validated MCFG table. Reserved bytes are required to be zero.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AcpiError> {
        let header = parse_sdt(bytes, *b"MCFG")?;
        if header.bytes.len() < MCFG_HEADER_LEN
            || (header.bytes.len() - MCFG_HEADER_LEN) % MCFG_ALLOCATION_LEN != 0
        {
            return Err(AcpiError::InvalidLength);
        }
        if header.bytes[36..44].iter().any(|byte| *byte != 0) {
            return Err(AcpiError::InvalidLength);
        }
        let allocations = &header.bytes[MCFG_HEADER_LEN..];
        let table = Self { header, allocations };
        for index in 0..table.allocation_count() {
            let allocation = table.allocation(index).ok_or(AcpiError::InvalidLength)?;
            allocation.ecam_end_exclusive()?;
            if allocation.base_address & 0xFFFFF != 0 {
                return Err(AcpiError::InvalidPciRange);
            }
        }
        Ok(table)
    }

    pub const fn allocation_count(&self) -> usize {
        self.allocations.len() / MCFG_ALLOCATION_LEN
    }

    pub fn allocation(&self, index: usize) -> Option<McfgAllocation> {
        let start = index.checked_mul(MCFG_ALLOCATION_LEN)?;
        let entry = self.allocations.get(start..start.checked_add(MCFG_ALLOCATION_LEN)?)?;
        Some(McfgAllocation {
            base_address: u64::from_le_bytes([
                entry[0], entry[1], entry[2], entry[3],
                entry[4], entry[5], entry[6], entry[7],
            ]),
            segment_group: u16::from_le_bytes([entry[8], entry[9]]),
            start_bus: entry[10],
            end_bus: entry[11],
        })
    }
}


/// Validated RSDT/XSDT table-address list. Addresses are metadata only; the
/// caller must validate and authorize each table before reading it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RootTable<'a> {
    pub header: SdtHeader<'a>,
    entry_width: usize,
    entries: &'a [u8],
}

impl<'a> RootTable<'a> {
    pub fn parse(bytes: &'a [u8], xsdt: bool) -> Result<Self, AcpiError> {
        let signature = if xsdt { *b"XSDT" } else { *b"RSDT" };
        let header = parse_sdt(bytes, signature)?;
        let entry_width = if xsdt { 8 } else { 4 };
        let payload = &header.bytes[SDT_HEADER_LEN..];
        if payload.len() % entry_width != 0 {
            return Err(AcpiError::InvalidLength);
        }
        Ok(Self { header, entry_width, entries: payload })
    }

    pub const fn entry_count(&self) -> usize {
        self.entries.len() / self.entry_width
    }

    pub fn entry_address(&self, index: usize) -> Option<u64> {
        let start = index.checked_mul(self.entry_width)?;
        let entry = self.entries.get(start..start.checked_add(self.entry_width)?)?;
        Some(if self.entry_width == 8 {
            u64::from_le_bytes([entry[0], entry[1], entry[2], entry[3], entry[4], entry[5], entry[6], entry[7]])
        } else {
            u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]) as u64
        })
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    fn checksum_table(mut bytes: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
        bytes[9] = 0;
        let sum = bytes.iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
        bytes[9] = 0u8.wrapping_sub(sum);
        bytes
    }

    fn make_mcfg(base: u64, segment: u16, start_bus: u8, end_bus: u8) -> std::vec::Vec<u8> {
        let mut b = vec![0u8; MCFG_HEADER_LEN + MCFG_ALLOCATION_LEN];
        b[0..4].copy_from_slice(b"MCFG");
        let len = b.len() as u32;
        b[4..8].copy_from_slice(&len.to_le_bytes());
        b[8] = 1;
        let a = MCFG_HEADER_LEN;
        b[a..a + 8].copy_from_slice(&base.to_le_bytes());
        b[a + 8..a + 10].copy_from_slice(&segment.to_le_bytes());
        b[a + 10] = start_bus;
        b[a + 11] = end_bus;
        checksum_table(b)
    }


    fn set_checksum(bytes: &mut [u8], checksum_index: usize, range_end: usize) {
        bytes[checksum_index] = 0;
        let sum = bytes[..range_end].iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
        bytes[checksum_index] = 0u8.wrapping_sub(sum);
    }


    #[test]
    fn parses_rsdt_and_xsdt_address_entries() {
        let mut rsdt = vec![0u8; SDT_HEADER_LEN + 8];
        rsdt[0..4].copy_from_slice(b"RSDT");
        let length = rsdt.len() as u32;
        rsdt[4..8].copy_from_slice(&length.to_le_bytes());
        rsdt[SDT_HEADER_LEN..SDT_HEADER_LEN + 4].copy_from_slice(&0x1234_5000u32.to_le_bytes());
        rsdt[SDT_HEADER_LEN + 4..].copy_from_slice(&0x5678_9000u32.to_le_bytes());
        let rsdt = checksum_table(rsdt);
        let parsed = RootTable::parse(&rsdt, false).unwrap();
        assert_eq!(parsed.entry_count(), 2);
        assert_eq!(parsed.entry_address(0), Some(0x1234_5000));
        assert_eq!(parsed.entry_address(2), None);

        let mut xsdt = vec![0u8; SDT_HEADER_LEN + 8];
        xsdt[0..4].copy_from_slice(b"XSDT");
        let length = xsdt.len() as u32;
        xsdt[4..8].copy_from_slice(&length.to_le_bytes());
        xsdt[SDT_HEADER_LEN..].copy_from_slice(&0x1234_5678_9abc_def0u64.to_le_bytes());
        let xsdt = checksum_table(xsdt);
        let parsed = RootTable::parse(&xsdt, true).unwrap();
        assert_eq!(parsed.entry_address(0), Some(0x1234_5678_9abc_def0));
    }

    #[test]
    fn rejects_root_table_payload_with_partial_address() {
        let mut bytes = vec![0u8; SDT_HEADER_LEN + 5];
        bytes[0..4].copy_from_slice(b"RSDT");
        let length = bytes.len() as u32;
        bytes[4..8].copy_from_slice(&length.to_le_bytes());
        let bytes = checksum_table(bytes);
        assert_eq!(RootTable::parse(&bytes, false), Err(AcpiError::InvalidLength));
    }

    #[test]
    fn parses_rsdp_v1_and_v2_and_rejects_bad_checksums() {
        let mut v1 = vec![0u8; RSDP_V1_LEN];
        v1[0..8].copy_from_slice(b"RSD PTR ");
        v1[15] = 0;
        v1[16..20].copy_from_slice(&0x1234_5000u32.to_le_bytes());
        set_checksum(&mut v1, 8, RSDP_V1_LEN);
        let parsed = parse_rsdp(&v1).unwrap();
        assert_eq!(parsed.rsdt_address, 0x1234_5000);
        assert_eq!(parsed.xsdt_address, None);

        let mut v2 = vec![0u8; RSDP_V2_MIN_LEN];
        v2[0..8].copy_from_slice(b"RSD PTR ");
        v2[15] = 2;
        v2[16..20].copy_from_slice(&0x1234_5000u32.to_le_bytes());
        v2[20..24].copy_from_slice(&(RSDP_V2_MIN_LEN as u32).to_le_bytes());
        v2[24..32].copy_from_slice(&0x1234_5678_9abc_def0u64.to_le_bytes());
        set_checksum(&mut v2, 8, RSDP_V1_LEN);
        set_checksum(&mut v2, 32, RSDP_V2_MIN_LEN);
        let parsed = parse_rsdp(&v2).unwrap();
        assert_eq!(parsed.xsdt_address, Some(0x1234_5678_9abc_def0));
        assert_eq!(parsed.length, RSDP_V2_MIN_LEN);

        v2[27] ^= 1;
        assert_eq!(parse_rsdp(&v2), Err(AcpiError::BadChecksum));
    }

    #[test]
    fn rejects_invalid_rsdp_signature_and_declared_length() {
        let mut bytes = vec![0u8; RSDP_V2_MIN_LEN];
        bytes[0..8].copy_from_slice(b"BAD PTR ");
        set_checksum(&mut bytes, 8, RSDP_V1_LEN);
        set_checksum(&mut bytes, 32, RSDP_V2_MIN_LEN);
        assert_eq!(parse_rsdp(&bytes), Err(AcpiError::BadSignature));

        bytes[0..8].copy_from_slice(b"RSD PTR ");
        bytes[15] = 2; // Revision 2+ is required to interpret the extended length field.
        bytes[20..24].copy_from_slice(&19u32.to_le_bytes());
        set_checksum(&mut bytes, 8, RSDP_V1_LEN);
        set_checksum(&mut bytes, 32, RSDP_V2_MIN_LEN);
        assert_eq!(parse_rsdp(&bytes), Err(AcpiError::InvalidLength));
    }

    #[test]
    fn parses_sdt_and_rejects_bad_signature_length_and_checksum() {
        let mut b = vec![0u8; SDT_HEADER_LEN];
        b[0..4].copy_from_slice(b"TEST");
        b[4..8].copy_from_slice(&(SDT_HEADER_LEN as u32).to_le_bytes());
        let valid = checksum_table(b);
        assert!(parse_sdt(&valid, *b"TEST").is_ok());
        assert_eq!(parse_sdt(&valid, *b"NOPE"), Err(AcpiError::BadSignature));
        assert_eq!(parse_sdt(&valid[..12], *b"TEST"), Err(AcpiError::Truncated));
        let mut bad_sum = valid.clone();
        bad_sum[10] ^= 1;
        assert_eq!(parse_sdt(&bad_sum, *b"TEST"), Err(AcpiError::BadChecksum));
        let mut bad_len = valid;
        bad_len[4..8].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(parse_sdt(&bad_len, *b"TEST"), Err(AcpiError::InvalidLength));
    }

    #[test]
    fn parses_mcfg_allocation_and_calculates_ecam_address() {
        let bytes = make_mcfg(0xE000_0000, 0, 0, 0x3f);
        let table = McfgTable::parse(&bytes).unwrap();
        assert_eq!(table.allocation_count(), 1);
        let a = table.allocation(0).unwrap();
        assert_eq!(a.segment_group, 0);
        assert_eq!(a.ecam_end_exclusive(), Ok(0xE400_0000));
        assert_eq!(a.ecam_address(0, 0, 0, 0), Ok(0xE000_0000));
        assert_eq!(a.ecam_address(1, 2, 3, 0x10), Ok(0xE011_3010));
        assert_eq!(a.ecam_address(0x40, 0, 0, 0), Err(AcpiError::InvalidPciRange));
    }

    #[test]
    fn rejects_bad_mcfg_alignment_and_bus_ranges() {
        let misaligned = make_mcfg(0xE000_0001, 0, 0, 1);
        assert_eq!(McfgTable::parse(&misaligned), Err(AcpiError::InvalidPciRange));
        let reversed = make_mcfg(0xE000_0000, 0, 5, 4);
        assert_eq!(McfgTable::parse(&reversed), Err(AcpiError::InvalidPciRange));
    }

    #[test]
    fn rejects_truncated_allocation_and_nonzero_reserved_header() {
        let mut short = vec![0u8; MCFG_HEADER_LEN + 3];
        short[0..4].copy_from_slice(b"MCFG");
        let len = short.len() as u32;
        short[4..8].copy_from_slice(&len.to_le_bytes());
        short = checksum_table(short);
        assert_eq!(McfgTable::parse(&short), Err(AcpiError::InvalidLength));

        let mut reserved = make_mcfg(0xE000_0000, 0, 0, 1);
        reserved[36] = 1;
        reserved = checksum_table(reserved);
        assert_eq!(McfgTable::parse(&reserved), Err(AcpiError::InvalidLength));
    }
}
