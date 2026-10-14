//! Platform-independent, allocation-free navigation for a screen-reader item list.
//! This module does not perform OS accessibility queries or produce audio.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cursor { count: u32, current: u32, has_item: bool }
impl Cursor {
 pub const fn new(count: u32) -> Self { Self { count, current: 0, has_item: false } }
 pub fn first(&mut self) -> Option<u32> {
  if self.count == 0 { self.has_item = false; return None; }
  self.current = 0; self.has_item = true; Some(0)
 }
 pub fn next(&mut self) -> Option<u32> {
  if !self.has_item || self.current >= self.count.saturating_sub(1) { return None; }
  self.current += 1; Some(self.current)
 }
 pub fn previous(&mut self) -> Option<u32> {
  if !self.has_item || self.current == 0 { return None; }
  self.current -= 1; Some(self.current)
 }
 pub fn current(&self) -> Option<u32> {
  if self.has_item && self.current < self.count { Some(self.current) } else { None }
 }
}
#[cfg(test)]
mod tests {
 use super::Cursor;
 #[test] fn bounded_navigation() {
  let mut c = Cursor::new(2);
  assert_eq!(c.first(), Some(0)); assert_eq!(c.next(), Some(1));
  assert_eq!(c.next(), None); assert_eq!(c.previous(), Some(0)); assert_eq!(c.previous(), None);
  assert_eq!(Cursor::new(0).current(), None);
 }
}
