//! Explicit provenance labels for hardware inventory and test claims.
//!
//! These records preserve where a fact came from. They are not cryptographic
//! attestations; callers must not treat them as proof of device identity or
//! physical execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EvidenceSource {
    UefiPciIoProtocol = 1,
    AcpiTableBytes = 2,
    OperatingSystemInventory = 3,
    SyntheticFixture = 4,
    ManualObservation = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EvidenceLevel {
    Parsed = 1,
    StructurallyValidated = 2,
    FirmwareReported = 3,
    PhysicallyObserved = 4,
    SyntheticOnly = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceRecord {
    /// Caller-assigned monotonic record number; not a timestamp.
    pub sequence: u64,
    pub subject_id: u64,
    pub source: EvidenceSource,
    pub level: EvidenceLevel,
    /// Optional caller-supplied time counter. Zero means unknown.
    pub observed_at: u64,
    /// Optional identifier for a separately stored evidence object.
    pub evidence_id: u64,
}

impl ProvenanceRecord {
    pub const fn new(
        sequence: u64,
        subject_id: u64,
        source: EvidenceSource,
        level: EvidenceLevel,
        observed_at: u64,
        evidence_id: u64,
    ) -> Self {
        Self { sequence, subject_id, source, level, observed_at, evidence_id }
    }

    pub const fn is_valid(&self) -> bool {
        self.sequence != 0 && self.subject_id != 0
    }

    /// Physical claims require an explicit physical-observation source/level.
    /// Firmware-reported and synthetic evidence cannot be promoted implicitly.
    pub const fn supports_physical_claim(&self) -> bool {
        matches!(self.source, EvidenceSource::ManualObservation)
            && matches!(self.level, EvidenceLevel::PhysicallyObserved)
            && self.observed_at != 0
            && self.evidence_id != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceError {
    InvalidRecord,
    SequenceNotIncreasing,
}

/// Validate an ordered evidence log without allocating or hashing it.
pub fn validate_log(records: &[ProvenanceRecord]) -> Result<(), ProvenanceError> {
    let mut previous = 0u64;
    for record in records {
        if !record.is_valid() {
            return Err(ProvenanceError::InvalidRecord);
        }
        if record.sequence <= previous {
            return Err(ProvenanceError::SequenceNotIncreasing);
        }
        previous = record.sequence;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordered_log_but_does_not_promote_firmware_to_physical() {
        let records = [
            ProvenanceRecord::new(1, 7, EvidenceSource::AcpiTableBytes, EvidenceLevel::StructurallyValidated, 0, 11),
            ProvenanceRecord::new(2, 7, EvidenceSource::UefiPciIoProtocol, EvidenceLevel::FirmwareReported, 12, 12),
        ];
        assert_eq!(validate_log(&records), Ok(()));
        assert!(!records[1].supports_physical_claim());
    }

    #[test]
    fn rejects_duplicate_or_reversed_sequence() {
        let records = [
            ProvenanceRecord::new(2, 7, EvidenceSource::SyntheticFixture, EvidenceLevel::SyntheticOnly, 0, 0),
            ProvenanceRecord::new(2, 8, EvidenceSource::SyntheticFixture, EvidenceLevel::SyntheticOnly, 0, 0),
        ];
        assert_eq!(validate_log(&records), Err(ProvenanceError::SequenceNotIncreasing));
    }

    #[test]
    fn only_explicit_observation_can_support_physical_claim() {
        let observed = ProvenanceRecord::new(
            1, 7, EvidenceSource::ManualObservation, EvidenceLevel::PhysicallyObserved, 99, 123,
        );
        assert!(observed.supports_physical_claim());
        assert!(!ProvenanceRecord::new(
            2, 7, EvidenceSource::UefiPciIoProtocol, EvidenceLevel::FirmwareReported, 99, 123,
        ).supports_physical_claim());
    }
}
