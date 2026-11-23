// Brique partagee : voix Windows locale (SAPI via COM, sans dependance). Utilisee par `parle` et `lecteur`.
#![allow(dead_code)]
#[repr(C)] struct Guid(u32, u16, u16, [u8; 8]);
#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(r: *mut u8, f: u32) -> i32;
    fn CoCreateInstance(c: *const Guid, o: *mut u8, ctx: u32, i: *const Guid, out: *mut *mut *mut usize) -> i32;
}
pub struct Voice(*mut *mut usize);
type Speak = extern "system" fn(*mut *mut usize, *const u16, u32, *mut u32) -> i32;
type Wait = extern "system" fn(*mut *mut usize, u32) -> i32;
type SetI32 = extern "system" fn(*mut *mut usize, i32) -> i32;
type SetU16 = extern "system" fn(*mut *mut usize, u16) -> i32;
type Release = extern "system" fn(*mut *mut usize) -> u32;
const SPF_ASYNC: u32 = 1; const SPF_PURGE: u32 = 2; const SPF_NOT_XML: u32 = 16;
impl Voice {
    pub fn new() -> Result<Voice, String> {
        let clsid = Guid(0x96749377, 0x3391, 0x11D2, [0x9E, 0xE3, 0x00, 0xC0, 0x4F, 0x79, 0x73, 0x96]);
        let iid = Guid(0x6C44DF74, 0x72B9, 0x4992, [0xA1, 0xEC, 0xEF, 0x99, 0x6E, 0x04, 0x22, 0xD4]);
        unsafe {
            CoInitializeEx(std::ptr::null_mut(), 0);
            let mut v: *mut *mut usize = std::ptr::null_mut();
            let hr = CoCreateInstance(&clsid, std::ptr::null_mut(), 23, &iid, &mut v);
            if hr != 0 || v.is_null() { return Err(format!("voix SAPI indisponible (0x{:08X})", hr as u32)); }
            Ok(Voice(v))
        }
    }
    fn slot<T: Copy>(&self, i: usize) -> T { unsafe { std::mem::transmute_copy(&*(*self.0).add(i)) } }
    /// vitesse -10 (lent) a 10 (rapide)
    pub fn rate(&self, r: i32) -> i32 { let f: SetI32 = self.slot(28); f(self.0, r.clamp(-10, 10)) }
    /// volume 0 a 100
    pub fn volume(&self, v: u16) -> i32 { let f: SetU16 = self.slot(30); f(self.0, v.min(100)) }
    /// dit le texte ; `urgent` interrompt la parole en cours
    pub fn say(&self, text: &str, urgent: bool) -> i32 {
        let w: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let f: Speak = self.slot(20);
        f(self.0, w.as_ptr(), SPF_ASYNC | SPF_NOT_XML | if urgent { SPF_PURGE } else { 0 }, std::ptr::null_mut())
    }
    /// coupe immediatement la parole
    pub fn stop(&self) -> i32 { self.say("", true) }
    pub fn wait(&self, ms: u32) -> i32 { let f: Wait = self.slot(32); f(self.0, ms) }
}
impl Drop for Voice { fn drop(&mut self) { let f: Release = self.slot(2); f(self.0); } }
