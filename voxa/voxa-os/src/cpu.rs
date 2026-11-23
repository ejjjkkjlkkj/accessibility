//! Acces machine de bas niveau : ports d'E/S, lecture de la memoire physique, arret du processeur. Seul module avec du code `unsafe` d'assembleur.
use core::arch::asm;
use voxa_kernel::hw::Ports;

#[inline(always)] pub fn outb(port: u16, v: u8) { unsafe { asm!("out dx, al", in("dx") port, in("al") v, options(nomem, nostack, preserves_flags)); } }
#[inline(always)] pub fn inb(port: u16) -> u8 { let v: u8; unsafe { asm!("in al, dx", in("dx") port, out("al") v, options(nomem, nostack, preserves_flags)); } v }
#[inline(always)] pub fn outl(port: u16, v: u32) { unsafe { asm!("out dx, eax", in("dx") port, in("eax") v, options(nomem, nostack, preserves_flags)); } }
#[inline(always)] pub fn inl(port: u16) -> u32 { let v: u32; unsafe { asm!("in eax, dx", in("dx") port, out("eax") v, options(nomem, nostack, preserves_flags)); } v }

/// Ports reels pour les pilotes de `voxa-kernel`.
pub struct RealPorts;
impl Ports for RealPorts { fn inb(&mut self, p: u16) -> u8 { inb(p) } fn outb(&mut self, p: u16, v: u8) { outb(p, v) } fn inl(&mut self, p: u16) -> u32 { inl(p) } fn outl(&mut self, p: u16, v: u32) { outl(p, v) } }

pub unsafe fn rd8(a: u64) -> u8 { core::ptr::read_volatile(a as *const u8) }
pub unsafe fn rd16(a: u64) -> u16 { core::ptr::read_volatile(a as *const u16) }
pub unsafe fn rd32(a: u64) -> u32 { core::ptr::read_volatile(a as *const u32) }
pub unsafe fn rd64(a: u64) -> u64 { core::ptr::read_volatile(a as *const u64) }

/// Valeur actuelle du pointeur de pile (pour reserver la pile courante dans l'allocateur de pages).
pub fn rsp() -> u64 { let v: u64; unsafe { asm!("mov {}, rsp", out(reg) v, options(nomem, nostack, preserves_flags)); } v }

/// Arret definitif du processeur (interruptions masquees). Ne revient jamais.
pub fn halt_forever() -> ! { loop { unsafe { asm!("cli", "hlt", options(nomem, nostack)); } } }

#[inline(always)] pub fn outw(port: u16, v: u16) { unsafe { asm!("out dx, ax", in("dx") port, in("ax") v, options(nomem, nostack, preserves_flags)); } }

/// Delai approximatif en microsecondes : on compte les bascules du bit 4 du port 0x61 (horloge de rafraichissement, ~15,085 us).
/// Borne de securite : si le bit ne bascule pas (materiel different), on abandonne apres un nombre fini de lectures au lieu de bloquer.
pub fn delay_us(us: u32) {
    let cibles = (us as u64 * 1000 / 15085) + 1; let mut dernier = inb(0x61) & 0x10; let mut n = 0u64; let mut lectures = 0u64;
    while n < cibles { let v = inb(0x61) & 0x10; if v != dernier { dernier = v; n += 1; lectures = 0; } else { lectures += 1; if lectures > 400_000 { return; } } }
}
