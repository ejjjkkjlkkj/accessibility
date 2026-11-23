//! Interface UEFI minimale (decalages de la specification UEFI 2.x), utilisee UNIQUEMENT avant ExitBootServices.
use crate::console::Fb;
use core::ffi::c_void;

const ST_BOOT: usize = 0x60; const ST_CONFIG_COUNT: usize = 0x68; const ST_CONFIG_TABLE: usize = 0x70;
const BS_GET_MEMORY_MAP: usize = 0x38; const BS_HANDLE_PROTOCOL: usize = 0x98; const BS_EXIT_BOOT_SERVICES: usize = 0xE8; const BS_LOCATE_PROTOCOL: usize = 0x140;
pub const GUID_GOP: [u8; 16] = [0xDE, 0xA9, 0x42, 0x90, 0xDC, 0x23, 0x38, 0x4A, 0x96, 0xFB, 0x7A, 0xDE, 0xD0, 0x80, 0x51, 0x6A];
pub const GUID_ACPI20: [u8; 16] = [0x71, 0xE8, 0x68, 0x88, 0xF1, 0xE4, 0xD3, 0x11, 0xBC, 0x22, 0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81];
pub const GUID_SMBIOS3: [u8; 16] = [0x44, 0x15, 0xFD, 0xF2, 0x94, 0x97, 0x2C, 0x4A, 0x99, 0x2E, 0xE5, 0xBB, 0xCF, 0x20, 0xE3, 0x94];
pub const GUID_LOADED_IMAGE: [u8; 16] = [0xA1, 0x31, 0x1B, 0x5B, 0x62, 0x95, 0xD2, 0x11, 0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B];

unsafe fn field(base: *const u8, off: usize) -> *const u8 { *(base.add(off) as *const *const u8) }
unsafe fn fnptr(base: *const u8, off: usize) -> usize { *(base.add(off) as *const usize) }

pub struct Systeme { pub st: *const u8, pub image: *const c_void }

impl Systeme {
    unsafe fn bs(&self) -> *const u8 { field(self.st, ST_BOOT) }

    /// Cherche une table de configuration par GUID (RSDP ACPI 2.0, SMBIOS 3...). Retourne l'adresse physique.
    pub fn table_config(&self, guid: &[u8; 16]) -> Option<u64> {
        unsafe {
            let n = *(self.st.add(ST_CONFIG_COUNT) as *const usize); let t = field(self.st, ST_CONFIG_TABLE);
            if t.is_null() || n > 4096 { return None; }
            for i in 0..n { let e = t.add(i * 24); if core::slice::from_raw_parts(e, 16) == guid { return Some(*(e.add(16) as *const u64)); } }
            None
        }
    }
    /// Cadre graphique (GOP). None si le firmware n'en donne pas.
    pub fn gop(&self) -> Option<Fb> {
        unsafe {
            let f: extern "efiapi" fn(*const u8, *const c_void, *mut *const u8) -> usize = core::mem::transmute(fnptr(self.bs(), BS_LOCATE_PROTOCOL));
            let mut g: *const u8 = core::ptr::null(); if f(GUID_GOP.as_ptr(), core::ptr::null(), &mut g) != 0 || g.is_null() { return None; }
            let mode = field(g, 0x18); if mode.is_null() { return None; } let info = field(mode, 8); if info.is_null() { return None; }
            let base = *(mode.add(0x18) as *const u64); let w = *(info.add(4) as *const u32); let h = *(info.add(8) as *const u32); let fmt = *(info.add(12) as *const u32); let stride = *(info.add(32) as *const u32);
            if base == 0 || w == 0 || h == 0 || fmt > 1 { return None; }
            Some(Fb { base, w, h, stride: if stride == 0 { w } else { stride }, bgr: fmt == 1 })
        }
    }
    /// Image chargee : (adresse de base, taille) pour la reserver dans l'allocateur de pages.
    pub fn image_chargee(&self) -> Option<(u64, u64)> {
        unsafe {
            let f: extern "efiapi" fn(*const c_void, *const u8, *mut *const u8) -> usize = core::mem::transmute(fnptr(self.bs(), BS_HANDLE_PROTOCOL));
            let mut li: *const u8 = core::ptr::null(); if f(self.image, GUID_LOADED_IMAGE.as_ptr(), &mut li) != 0 || li.is_null() { return None; }
            Some((*(li.add(0x40) as *const u64), *(li.add(0x48) as *const u64)))
        }
    }
    /// Carte memoire dans `buf` : (nombre de descripteurs, taille d'un descripteur, cle). La cle change si la carte change.
    pub fn carte_memoire(&self, buf: &mut [u8]) -> Option<(usize, usize, usize)> {
        unsafe {
            let f: extern "efiapi" fn(*mut usize, *mut u8, *mut usize, *mut usize, *mut u32) -> usize = core::mem::transmute(fnptr(self.bs(), BS_GET_MEMORY_MAP));
            let (mut size, mut key, mut dsz, mut dver) = (buf.len(), 0usize, 0usize, 0u32);
            if f(&mut size, buf.as_mut_ptr(), &mut key, &mut dsz, &mut dver) != 0 || dsz < 40 { return None; }
            Some((size / dsz, dsz, key))
        }
    }
    /// Sortie definitive des services de demarrage. Retourne false si la cle est perimee (la carte a change : il faut la relire).
    pub fn sortir(&self, key: usize) -> bool {
        unsafe { let f: extern "efiapi" fn(*const c_void, usize) -> usize = core::mem::transmute(fnptr(self.bs(), BS_EXIT_BOOT_SERVICES)); f(self.image, key) == 0 }
    }
}

pub const GUID_HII_DATABASE: [u8; 16] = [0x72, 0xC1, 0x9F, 0xEF, 0xB2, 0xA1, 0x93, 0x46, 0xB3, 0x27, 0x6D, 0x32, 0xFC, 0x41, 0x60, 0x42];
const BS_ALLOCATE_POOL: usize = 0x40;

impl Systeme {
    /// Copie la table SMBIOS 3 dans `dst` (le firmware peut la placer dans une zone reutilisable apres la sortie). Retourne le nombre d'octets copies.
    pub fn copier_smbios3(&self, dst: &mut [u8]) -> usize {
        unsafe {
            let Some(entry) = self.table_config(&GUID_SMBIOS3) else { return 0 }; let e = entry as *const u8;
            if core::slice::from_raw_parts(e, 5) != b"_SM3_" { return 0; }
            let size = *(e.add(0x0C) as *const u32) as usize; let addr = *(e.add(0x10) as *const u64) as *const u8;
            if addr.is_null() || size == 0 || size > dst.len() { return 0; }
            core::ptr::copy_nonoverlapping(addr, dst.as_mut_ptr(), size); size
        }
    }
    /// Demande au firmware l'export de ses listes de paquets HII (ecrans de configuration du BIOS) dans une zone allouee. LECTURE SEULE, avant la sortie des services.
    pub fn exporter_hii(&self) -> Option<(u64, usize)> {
        unsafe {
            let loc: extern "efiapi" fn(*const u8, *const c_void, *mut *const u8) -> usize = core::mem::transmute(fnptr(self.bs(), BS_LOCATE_PROTOCOL));
            let mut db: *const u8 = core::ptr::null(); if loc(GUID_HII_DATABASE.as_ptr(), core::ptr::null(), &mut db) != 0 || db.is_null() { return None; }
            let export: extern "efiapi" fn(*const u8, *const c_void, *mut usize, *mut u8) -> usize = core::mem::transmute(fnptr(db, 0x20));
            let mut size = 0usize; export(db, core::ptr::null(), &mut size, core::ptr::null_mut()); if size == 0 || size > (32 << 20) { return None; }
            let alloc: extern "efiapi" fn(u32, usize, *mut *mut u8) -> usize = core::mem::transmute(fnptr(self.bs(), BS_ALLOCATE_POOL));
            let mut buf: *mut u8 = core::ptr::null_mut(); if alloc(2, size, &mut buf) != 0 || buf.is_null() { return None; }
            if export(db, core::ptr::null(), &mut size, buf) != 0 { return None; }
            Some((buf as u64, size))
        }
    }
}
