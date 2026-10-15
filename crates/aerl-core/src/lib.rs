#![no_std]
#![forbid(unsafe_code)]

//! Fixed-buffer, no_std, hash-chained evidence ledger core.
//! Storage, timekeeping, signatures, encryption, and I/O belong to the caller.

use sha2::{Digest, Sha256};
use accessibility_core::AccessibleEvent;

pub const FORMAT_MAGIC: [u8; 4] = *b"AERL";
pub const FORMAT_VERSION: u8 = 1;
pub const ACCESSIBILITY_PAYLOAD_MAGIC: [u8; 4] = *b"ACCE";
pub const ACCESSIBILITY_PAYLOAD_VERSION: u8 = 1;
pub const ACCESSIBILITY_PAYLOAD_BYTES: usize = 25;
pub const MAX_LEDGER_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ACTOR_BYTES: usize = 63;
pub const MAX_KIND_BYTES: usize = 80;
pub const MAX_PAYLOAD_BYTES: usize = 4096;
pub const HASH_BYTES: usize = 32;
const PREFIX_BYTES: usize = 58;
const FIXED_RECORD_BYTES: usize = PREFIX_BYTES + HASH_BYTES;
const GENESIS_HASH: [u8; HASH_BYTES] = [0; HASH_BYTES];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Status {
    Info = 0,
    Pass = 1,
    Fail = 2,
    Partial = 3,
    Blocked = 4,
    NotValidated = 5,
    Unproven = 6,
}

impl Status {
    fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Info),
            1 => Some(Self::Pass),
            2 => Some(Self::Fail),
            3 => Some(Self::Partial),
            4 => Some(Self::Blocked),
            5 => Some(Self::NotValidated),
            6 => Some(Self::Unproven),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidUsedLength,
    LedgerTooLarge,
    BufferTooSmall,
    InvalidActor,
    InvalidKind,
    PayloadTooLarge,
    InvalidMagic,
    UnsupportedVersion,
    TruncatedRecord,
    InvalidStatus,
    InvalidUtf8,
    SequenceMismatch,
    SequenceOverflow,
    PreviousHashMismatch,
    HashMismatch,
    CheckpointMismatch,
    InvalidAccessibleEvent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub sequence: u64,
    pub head_hash: [u8; HASH_BYTES],
}

impl Checkpoint {
    pub const fn genesis() -> Self {
        Self { sequence: 0, head_hash: GENESIS_HASH }
    }
}

/// Event metadata. Timestamp is Unix epoch milliseconds supplied by the caller;
/// zero means the platform clock was unavailable or not trusted.
#[derive(Clone, Copy, Debug)]
pub struct Event<'a> {
    pub timestamp_utc_ms: u64,
    pub actor: &'a str,
    pub kind: &'a str,
    pub status: Status,
    /// Bounded opaque bytes. The adapter chooses and validates its encoding.
    pub payload: &'a [u8],
}

fn valid_actor(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= MAX_ACTOR_BYTES
        && bytes[0].is_ascii_alphanumeric()
        && bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || b"._:@/-".contains(byte))
}

fn valid_kind(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= MAX_KIND_BYTES
        && bytes[0].is_ascii_alphabetic()
        && bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || b"_.:-".contains(byte))
}

/// Append one frame into caller-owned memory. Existing bytes are verified first.
/// On validation/capacity errors, buffer and used remain unchanged.
pub fn append(buffer: &mut [u8], used: &mut usize, event: Event<'_>) -> Result<Checkpoint, Error> {
    if *used > buffer.len() || *used > MAX_LEDGER_BYTES {
        return Err(Error::InvalidUsedLength);
    }
    if !valid_actor(event.actor) {
        return Err(Error::InvalidActor);
    }
    if !valid_kind(event.kind) {
        return Err(Error::InvalidKind);
    }
    if event.payload.len() > MAX_PAYLOAD_BYTES {
        return Err(Error::PayloadTooLarge);
    }

    let previous = verify(&buffer[..*used])?;
    let sequence = previous.sequence.checked_add(1).ok_or(Error::SequenceOverflow)?;
    let body_len = event.actor.len() + event.kind.len() + event.payload.len();
    let record_len = FIXED_RECORD_BYTES + body_len;
    if record_len > MAX_LEDGER_BYTES.saturating_sub(*used) {
        return Err(Error::LedgerTooLarge);
    }
    if record_len > buffer.len() - *used {
        return Err(Error::BufferTooSmall);
    }

    let start = *used;
    buffer[start..start + 4].copy_from_slice(&FORMAT_MAGIC);
    buffer[start + 4] = FORMAT_VERSION;
    buffer[start + 5..start + 13].copy_from_slice(&sequence.to_le_bytes());
    buffer[start + 13..start + 21].copy_from_slice(&event.timestamp_utc_ms.to_le_bytes());
    buffer[start + 21] = event.status as u8;
    buffer[start + 22] = event.actor.len() as u8;
    buffer[start + 23] = event.kind.len() as u8;
    buffer[start + 24..start + 26].copy_from_slice(&(event.payload.len() as u16).to_le_bytes());
    buffer[start + 26..start + PREFIX_BYTES].copy_from_slice(&previous.head_hash);

    let mut cursor = start + PREFIX_BYTES;
    buffer[cursor..cursor + event.actor.len()].copy_from_slice(event.actor.as_bytes());
    cursor += event.actor.len();
    buffer[cursor..cursor + event.kind.len()].copy_from_slice(event.kind.as_bytes());
    cursor += event.kind.len();
    buffer[cursor..cursor + event.payload.len()].copy_from_slice(event.payload);
    cursor += event.payload.len();

    let digest = Sha256::digest(&buffer[start..cursor]);
    let mut head_hash = [0u8; HASH_BYTES];
    head_hash.copy_from_slice(digest.as_slice());
    buffer[cursor..cursor + HASH_BYTES].copy_from_slice(&head_hash);

    *used = start + record_len;
    Ok(Checkpoint { sequence, head_hash })
}

/// Verify all frames. Keep the returned checkpoint in a separate trust domain
/// if whole-ledger rewriting or tail truncation must be detectable.
pub fn verify(bytes: &[u8]) -> Result<Checkpoint, Error> {
    if bytes.len() > MAX_LEDGER_BYTES {
        return Err(Error::LedgerTooLarge);
    }
    let mut checkpoint = Checkpoint::genesis();
    let mut offset = 0usize;

    while offset < bytes.len() {
        let remaining = bytes.len() - offset;
        if remaining < FIXED_RECORD_BYTES {
            return Err(Error::TruncatedRecord);
        }
        if &bytes[offset..offset + 4] != &FORMAT_MAGIC[..] {
            return Err(Error::InvalidMagic);
        }
        if bytes[offset + 4] != FORMAT_VERSION {
            return Err(Error::UnsupportedVersion);
        }

        let mut raw_sequence = [0u8; 8];
        raw_sequence.copy_from_slice(&bytes[offset + 5..offset + 13]);
        let sequence = u64::from_le_bytes(raw_sequence);
        let expected = checkpoint.sequence.checked_add(1).ok_or(Error::SequenceOverflow)?;
        if sequence != expected {
            return Err(Error::SequenceMismatch);
        }

        let _status = Status::from_byte(bytes[offset + 21]).ok_or(Error::InvalidStatus)?;
        let actor_len = bytes[offset + 22] as usize;
        let kind_len = bytes[offset + 23] as usize;
        let mut raw_payload_len = [0u8; 2];
        raw_payload_len.copy_from_slice(&bytes[offset + 24..offset + 26]);
        let payload_len = u16::from_le_bytes(raw_payload_len) as usize;

        if actor_len == 0 || actor_len > MAX_ACTOR_BYTES {
            return Err(Error::InvalidActor);
        }
        if kind_len == 0 || kind_len > MAX_KIND_BYTES {
            return Err(Error::InvalidKind);
        }
        if payload_len > MAX_PAYLOAD_BYTES {
            return Err(Error::PayloadTooLarge);
        }

        let body_len = actor_len + kind_len + payload_len;
        let record_len = FIXED_RECORD_BYTES + body_len;
        if remaining < record_len {
            return Err(Error::TruncatedRecord);
        }

        let previous_hash = &bytes[offset + 26..offset + PREFIX_BYTES];
        if previous_hash != &checkpoint.head_hash[..] {
            return Err(Error::PreviousHashMismatch);
        }
        let mut cursor = offset + PREFIX_BYTES;
        let actor_bytes = &bytes[cursor..cursor + actor_len];
        cursor += actor_len;
        let kind_bytes = &bytes[cursor..cursor + kind_len];
        cursor += kind_len + payload_len;

        let actor = core::str::from_utf8(actor_bytes).map_err(|_| Error::InvalidUtf8)?;
        let kind = core::str::from_utf8(kind_bytes).map_err(|_| Error::InvalidUtf8)?;
        if !valid_actor(actor) {
            return Err(Error::InvalidActor);
        }
        if !valid_kind(kind) {
            return Err(Error::InvalidKind);
        }

        let digest = Sha256::digest(&bytes[offset..cursor]);
        let mut calculated_hash = [0u8; HASH_BYTES];
        calculated_hash.copy_from_slice(digest.as_slice());
        if calculated_hash.as_slice() != &bytes[cursor..cursor + HASH_BYTES] {
            return Err(Error::HashMismatch);
        }
        checkpoint.head_hash.copy_from_slice(&bytes[cursor..cursor + HASH_BYTES]);
        checkpoint.sequence = sequence;
        offset += record_len;
    }

    Ok(checkpoint)
}

/// Append a semantic accessibility event without recording its accessible name.
///
/// The payload is fixed: magic, version, numeric role, state flags, and source ID.
/// Source IDs should be ephemeral and session-scoped.
pub fn append_accessible_event(
    buffer: &mut [u8],
    used: &mut usize,
    timestamp_utc_ms: u64,
    actor: &str,
    status: Status,
    event: &AccessibleEvent<'_>,
) -> Result<Checkpoint, Error> {
    if !event.is_valid() {
        return Err(Error::InvalidAccessibleEvent);
    }
    let mut payload = [0u8; ACCESSIBILITY_PAYLOAD_BYTES];
    payload[0..4].copy_from_slice(&ACCESSIBILITY_PAYLOAD_MAGIC);
    payload[4] = ACCESSIBILITY_PAYLOAD_VERSION;
    payload[5..9].copy_from_slice(&(event.role as u32).to_le_bytes());
    payload[9..17].copy_from_slice(&event.state_flags.to_le_bytes());
    payload[17..25].copy_from_slice(&event.source_id.to_le_bytes());

    append(
        buffer,
        used,
        Event {
            timestamp_utc_ms,
            actor,
            kind: "accessibility.semantic",
            status,
            payload: &payload,
        },
    )
}

/// Verify the current ledger against a checkpoint saved independently.
pub fn verify_checkpoint(bytes: &[u8], expected: &Checkpoint) -> Result<Checkpoint, Error> {
    let actual = verify(bytes)?;
    if actual != *expected {
        return Err(Error::CheckpointMismatch);
    }
    Ok(actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event<'a>(actor: &'a str, kind: &'a str, payload: &'a [u8]) -> Event<'a> {
        Event {
            timestamp_utc_ms: 1_800_000_000_000,
            actor,
            kind,
            status: Status::Pass,
            payload,
        }
    }

    fn first_record_len() -> usize {
        FIXED_RECORD_BYTES + "QA".len() + "test.unit".len() + b"one".len()
    }

    #[test]
    fn binary_format_matches_published_golden_vector() {
        let mut buffer = [0u8; 256];
        let mut used = 0;
        let checkpoint = append(
            &mut buffer,
            &mut used,
            Event {
                timestamp_utc_ms: 0,
                actor: "QA",
                kind: "test",
                status: Status::Pass,
                payload: b"{}",
            },
        )
        .unwrap();

        assert_eq!(used, 98);
        assert_eq!(&buffer[..4], b"AERL");
        assert_eq!(buffer[4], FORMAT_VERSION);
        assert_eq!(checkpoint.sequence, 1);
        assert_eq!(
            checkpoint.head_hash,
            [
                0x38, 0x7c, 0xec, 0x86, 0xf9, 0xcd, 0x32, 0x9e,
                0xf8, 0xc2, 0x0f, 0x7a, 0x9d, 0xdc, 0x0f, 0xda,
                0x8e, 0xaf, 0xa4, 0x63, 0x9d, 0x9d, 0x14, 0x36,
                0xe6, 0xab, 0xcf, 0xec, 0x84, 0x91, 0x64, 0x5f,
            ]
        );
        assert_eq!(verify(&buffer[..used]), Ok(checkpoint));
    }

    #[test]
    fn detects_every_single_bit_mutation_in_a_valid_frame() {
        let mut buffer = [0u8; 256];
        let mut used = 0;
        append(
            &mut buffer,
            &mut used,
            Event {
                timestamp_utc_ms: 1_800_000_000_000,
                actor: "QA",
                kind: "test.unit",
                status: Status::Pass,
                payload: b"mutation-vector",
            },
        )
        .unwrap();
        let checkpoint = verify(&buffer[..used]).unwrap();

        for index in 0..used {
            for bit in 0..8 {
                let mask = 1u8 << bit;
                buffer[index] ^= mask;
                assert!(
                    verify(&buffer[..used]).is_err(),
                    "single-bit mutation unexpectedly accepted at byte {index}, bit {bit}"
                );
                buffer[index] ^= mask;
            }
        }
        assert_eq!(verify(&buffer[..used]), Ok(checkpoint));
    }

    #[test]
    fn semantic_accessibility_event_omits_accessible_name() {
        use accessibility_core::{tree, Role};

        let name = "Private account name";
        let mut accessible = AccessibleEvent::new(Role::Button, 42, name);
        accessible.state_flags = tree::STATE_FOCUSABLE | tree::STATE_ENABLED | tree::STATE_FOCUSED;
        assert!(accessible.is_valid());

        let mut buffer = [0u8; 512];
        let mut used = 0;
        let checkpoint = append_accessible_event(
            &mut buffer,
            &mut used,
            1_800_000_000_000,
            "screenreader",
            Status::Info,
            &accessible,
        )
        .unwrap();

        assert_eq!(verify(&buffer[..used]), Ok(checkpoint));
        assert!(!buffer[..used].windows(name.len()).any(|slice| slice == name.as_bytes()));

        let payload_offset = PREFIX_BYTES + "screenreader".len() + "accessibility.semantic".len();
        assert_eq!(&buffer[payload_offset..payload_offset + 4], &ACCESSIBILITY_PAYLOAD_MAGIC);
        assert_eq!(buffer[payload_offset + 4], ACCESSIBILITY_PAYLOAD_VERSION);
        assert_eq!(
            u32::from_le_bytes(buffer[payload_offset + 5..payload_offset + 9].try_into().unwrap()),
            Role::Button as u32
        );
        assert_eq!(
            u64::from_le_bytes(buffer[payload_offset + 9..payload_offset + 17].try_into().unwrap()),
            accessible.state_flags
        );
        assert_eq!(
            u64::from_le_bytes(buffer[payload_offset + 17..payload_offset + 25].try_into().unwrap()),
            42
        );
    }

    #[test]
    fn rejects_invalid_accessibility_event_before_mutating_buffer() {
        use accessibility_core::Role;

        let long_name = "x".repeat(4097);
        let accessible = AccessibleEvent::new(Role::Button, 42, &long_name);
        let mut buffer = [0u8; 512];
        let mut used = 0;
        assert_eq!(
            append_accessible_event(
                &mut buffer,
                &mut used,
                1,
                "screenreader",
                Status::Info,
                &accessible,
            ),
            Err(Error::InvalidAccessibleEvent)
        );
        assert_eq!(used, 0);
        assert_eq!(verify(&buffer[..used]), Ok(Checkpoint::genesis()));
    }

    #[test]
    fn appends_and_verifies_multiple_frames() {
        let mut buffer = [0u8; 1024];
        let mut used = 0;
        let first = append(&mut buffer, &mut used, event("QA", "test.unit", b"{\"n\":1}")).unwrap();
        let second = append(&mut buffer, &mut used, event("WHITE", "fix.checked", b"{\"ok\":true}")).unwrap();
        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);
        assert_ne!(first.head_hash, second.head_hash);
        assert_eq!(verify(&buffer[..used]), Ok(second));
    }

    #[test]
    fn detects_payload_tampering() {
        let mut buffer = [0u8; 512];
        let mut used = 0;
        append(&mut buffer, &mut used, event("QA", "test.unit", b"{\"n\":1}")).unwrap();
        let payload_offset = PREFIX_BYTES + "QA".len() + "test.unit".len();
        buffer[payload_offset] ^= 1;
        assert_eq!(verify(&buffer[..used]), Err(Error::HashMismatch));
    }

    #[test]
    fn detects_sequence_and_previous_hash_changes() {
        let mut buffer = [0u8; 512];
        let mut used = 0;
        append(&mut buffer, &mut used, event("QA", "test.unit", b"x")).unwrap();
        buffer[5] = 2;
        assert_eq!(verify(&buffer[..used]), Err(Error::SequenceMismatch));

        used = 0;
        append(&mut buffer, &mut used, event("QA", "test.unit", b"x")).unwrap();
        buffer[26] ^= 1;
        assert_eq!(verify(&buffer[..used]), Err(Error::PreviousHashMismatch));
    }

    #[test]
    fn external_checkpoint_detects_truncation() {
        let mut buffer = [0u8; 512];
        let mut used = 0;
        let first = append(&mut buffer, &mut used, event("QA", "test.unit", b"one")).unwrap();
        let final_checkpoint = append(&mut buffer, &mut used, event("QA", "test.unit", b"two")).unwrap();
        assert_eq!(verify_checkpoint(&buffer[..used], &final_checkpoint), Ok(final_checkpoint));
        assert_eq!(verify_checkpoint(&buffer[..first_record_len()], &final_checkpoint), Err(Error::CheckpointMismatch));
        assert_eq!(verify(&buffer[..first_record_len()]), Ok(first));
    }

    #[test]
    fn refuses_partial_and_corrupt_records() {
        let mut buffer = [0u8; 512];
        let mut used = 0;
        append(&mut buffer, &mut used, event("QA", "test.unit", b"payload")).unwrap();
        assert_eq!(verify(&buffer[..used - 1]), Err(Error::TruncatedRecord));
        assert_eq!(verify(&buffer[..used - 4]), Err(Error::TruncatedRecord));
    }

    #[test]
    fn refuses_invalid_identifiers_and_oversized_payload() {
        let mut buffer = [0u8; 8192];
        let mut used = 0;
        assert_eq!(append(&mut buffer, &mut used, event("bad actor", "test.unit", b"x")), Err(Error::InvalidActor));
        assert_eq!(append(&mut buffer, &mut used, event("QA", "bad kind", b"x")), Err(Error::InvalidKind));
        let large = [0u8; MAX_PAYLOAD_BYTES + 1];
        assert_eq!(append(&mut buffer, &mut used, event("QA", "test.unit", &large)), Err(Error::PayloadTooLarge));
        assert_eq!(used, 0);
    }

    #[test]
    fn small_buffer_fails_without_mutating_used() {
        let mut buffer = [0u8; FIXED_RECORD_BYTES];
        let mut used = 0;
        assert_eq!(append(&mut buffer, &mut used, event("QA", "test.unit", b"x")), Err(Error::BufferTooSmall));
        assert_eq!(used, 0);
        assert_eq!(verify(&buffer[..used]), Ok(Checkpoint::genesis()));
    }

    #[test]
    fn rejects_unexpected_magic_and_status() {
        let mut buffer = [0u8; 512];
        let mut used = 0;
        append(&mut buffer, &mut used, event("QA", "test.unit", b"x")).unwrap();
        buffer[0] = b'X';
        assert_eq!(verify(&buffer[..used]), Err(Error::InvalidMagic));

        used = 0;
        append(&mut buffer, &mut used, event("QA", "test.unit", b"x")).unwrap();
        buffer[21] = 255;
        assert_eq!(verify(&buffer[..used]), Err(Error::InvalidStatus));
    }
}
