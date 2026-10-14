#![no_std]

pub mod screenreader;

pub const ABI_VERSION: u32 = 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role { Unknown = 0, Application = 1, Window = 2, Group = 3, Text = 4, Button = 5, Checkbox = 6, Radio = 7, Textbox = 8, Link = 9, Heading = 10, List = 11, ListItem = 12, Table = 13, Row = 14, Cell = 15, Menu = 16, MenuItem = 17 }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessibleEvent<'a> {
    pub abi_version: u32, pub role: Role, pub state_flags: u64, pub source_id: u64, pub name: &'a str,
}
impl<'a> AccessibleEvent<'a> {
    pub const fn new(role: Role, source_id: u64, name: &'a str) -> Self {
        Self { abi_version: ABI_VERSION, role, state_flags: 0, source_id, name }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_has_stable_version_and_values() {
        let event = AccessibleEvent::new(Role::Button, 7, "Continue");
        assert_eq!(event.abi_version, ABI_VERSION); assert_eq!(event.role, Role::Button);\n        assert_eq!(Role::Button as u32, 5); assert_eq!(Role::MenuItem as u32, 17);
        assert_eq!(event.source_id, 7); assert_eq!(event.name, "Continue"); assert_eq!(event.state_flags, 0);
    }
}
