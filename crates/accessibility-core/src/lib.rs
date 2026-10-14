#![no_std]

pub const ABI_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    Unknown = 0,
    Button = 1,
    Text = 2,
    Checkbox = 3,
    Dialog = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessibleEvent<'a> {
    pub abi_version: u32,
    pub role: Role,
    pub state_flags: u64,
    pub source_id: u64,
    pub name: &'a str,
}

impl<'a> AccessibleEvent<'a> {
    pub const fn new(role: Role, source_id: u64, name: &'a str) -> Self {
        Self {
            abi_version: ABI_VERSION,
            role,
            state_flags: 0,
            source_id,
            name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_has_stable_version_and_values() {
        let event = AccessibleEvent::new(Role::Button, 7, "Continue");
        assert_eq!(event.abi_version, ABI_VERSION);
        assert_eq!(event.role, Role::Button);
        assert_eq!(event.source_id, 7);
        assert_eq!(event.name, "Continue");
        assert_eq!(event.state_flags, 0);
    }
}
