// Allocateur global de l'agent : une zone statique (bss) que l'on ne fait qu'avancer.
//
// Pourquoi : l'allocateur UEFI (boot services) n'existe plus apres ExitBootServices. Avec lui,
// toute allocation ou liberation apres la sortie fait planter l'agent. Cette zone, elle, reste
// utilisable apres la sortie.
//
// Consequence : la memoire liberee n'est pas reutilisee. La zone est dimensionnee pour une seule
// execution de l'agent (rapport, tampons de lecture, sortie de la synthese vocale).
use core::alloc::{GlobalAlloc, Layout};

/// Taille de la zone : 192 Mio.
pub const TAILLE: usize = 192 << 20;

#[repr(C, align(4096))]
struct Zone([u8; TAILLE]);

static mut ZONE: Zone = Zone([0u8; TAILLE]);
/// Octets deja attribues dans la zone (depuis son debut).
static mut SUITE: usize = 0;

pub struct Arena;

// SAFETY: l'agent s'execute sur un seul fil ; la zone n'est jamais rendue, donc aucun pointeur
// retourne ne peut etre reutilise par erreur.
unsafe impl GlobalAlloc for Arena {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        // SAFETY: acces a la zone statique et au compteur, sans concurrence.
        unsafe {
            let base = core::ptr::addr_of_mut!(ZONE) as usize;
            let align = l.align();
            let debut = (base + SUITE + align - 1) & !(align - 1);
            let fin = debut + l.size();
            if fin > base + TAILLE {
                return core::ptr::null_mut();
            }
            SUITE = fin - base;
            debut as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _l: Layout) {}
}
