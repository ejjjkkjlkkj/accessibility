// Table des interruptions (IDT) : toute exception ou interruption inattendue arrive ici.
// - Exceptions 0..31 : affiche "EXCEPTION nn ERR ... RIP ..." sur un bandeau rouge, puis s'arrete.
//   L'ecran reste donc lisible au lieu de redemarrer ou de rester noir.
// - Interruptions 32..255 : ignorees (iretq), pour ne pas bloquer le PC.
use core::arch::{asm, global_asm};

use crate::screen::font;

// 256 stubs de taille fixe (16 octets). Chaque stub pousse un code d'erreur (0 si absent),
// puis le numero de vecteur, puis saute a idt_common.
global_asm!(
    r#"
    .altmacro
    .macro ISTUB n
    .p2align 4
    .globl idt_stub_\n
    idt_stub_\n:
    .if (\n == 8) || (\n == 10) || (\n == 11) || (\n == 12) || (\n == 13) || (\n == 14) || (\n == 17) || (\n == 21) || (\n == 29) || (\n == 30)
    .else
    push 0
    .endif
    push \n
    jmp idt_common
    .endm
    .set i, 0
    .rept 256
    ISTUB %i
    .set i, i + 1
    .endr

    .p2align 4
    idt_common:
    mov rax, [rsp]
    cmp rax, 32
    jae 2f
    mov rdi, [rsp]
    mov rsi, [rsp+8]
    mov rdx, [rsp+16]
    mov qword ptr [rip + idt_saved_sp], rsp
    and rsp, -16
    call idt_exc_rust
    test rax, rax
    jz 1f
    # Recuperation : rax = nouvelle adresse de retour (RIP). On reprend apres l'instruction fautive.
    mov rsp, qword ptr [rip + idt_saved_sp]
    mov qword ptr [rsp + 16], rax
    add rsp, 16
    iretq
    1:
    hlt
    jmp 1b
    2:
    add rsp, 16
    iretq
    "#
);

unsafe extern "C" {
    static idt_stub_0: u8;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Gate {
    lo: u16,
    sel: u16,
    ist: u8,
    attr: u8,
    mid: u16,
    hi: u32,
    zero: u32,
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: u64,
}

static mut IDT: [Gate; 256] = [Gate { lo: 0, sel: 0, ist: 0, attr: 0, mid: 0, hi: 0, zero: 0 }; 256];
// Ecran pour afficher une exception : framebuffer, longueur, stride, largeur, hauteur.
static mut SCR: (usize, usize, usize, usize, usize) = (0, 0, 0, 0, 0);

pub fn set_screen(info: (usize, usize, usize, usize, usize)) {
    // SAFETY: ecriture unique avant toute interruption utile ; pas de reference vers SCR.
    unsafe { SCR = info };
}

/// Installe la table des interruptions. A appeler avant tout le reste.
///
/// # Safety
/// Modifie les registres systeme (lidt). Le segment de code courant est utilise pour les portes.
pub unsafe fn install() {
    let mut cs: u16 = 0;
    // SAFETY: lecture du registre de segment CS.
    unsafe { asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags)) };
    let base = &raw const idt_stub_0 as usize as u64;
    let idt = &raw mut IDT;
    for i in 0..256usize {
        let h = base + (i as u64) * 16;
        // SAFETY: ecriture dans la table statique, index < 256.
        unsafe {
            (*idt)[i] = Gate {
                lo: h as u16,
                sel: cs,
                ist: 0,
                attr: 0x8E, // porte d'interruption, presente, DPL 0
                mid: (h >> 16) as u16,
                hi: (h >> 32) as u32,
                zero: 0,
            };
        }
    }
    let idtr = Idtr { limit: (core::mem::size_of::<Gate>() * 256 - 1) as u16, base: idt as usize as u64 };
    // SAFETY: lidt charge la table ; idtr reste valide le temps de l'instruction.
    unsafe { asm!("lidt [{}]", in(reg) &idtr as *const Idtr, options(readonly, nostack, preserves_flags)) };
}

fn hex(out: &mut [u8], v: u64, digits: usize) -> usize {
    let h = b"0123456789ABCDEF";
    let mut p = 0;
    for i in (0..digits).rev() {
        if p < out.len() {
            out[p] = h[((v >> (4 * i)) & 0xF) as usize];
            p += 1;
        }
    }
    p
}

/// Ecrit `s` a (x, y) en police 5x7, echelle 3, blanc sur le fond.
unsafe fn text(fb: usize, len: usize, stride: usize, w: usize, h: usize, x0: usize, y0: usize, s: &[u8]) {
    let p = fb as *mut u32;
    // SAFETY: framebuffer GOP ; chaque pixel est verifie contre len, w et h.
    let buf = unsafe { core::slice::from_raw_parts_mut(p, len) };
    let sc = 3usize;
    for (ci, &c) in s.iter().enumerate() {
        let g = font(c);
        for (r, bits) in g.iter().enumerate() {
            for col in 0..5usize {
                let on = bits & (0x10 >> col) != 0;
                for dy in 0..sc {
                    for dx in 0..sc {
                        let x = x0 + (ci * 6 + col) * sc + dx;
                        let y = y0 + r * sc + dy;
                        if x < w && y < h {
                            let idx = y * stride + x;
                            if idx < len {
                                buf[idx] = if on { 0x00FF_FFFF } else { 0x0030_0000 };
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Adresse sauvegardee du pointeur de pile au moment de l'exception (lue par idt_common).
#[unsafe(no_mangle)]
static mut idt_saved_sp: u64 = 0;
/// Vrai pendant une lecture MSR protegee : une #GP est alors une absence du MSR, pas une panne.
static mut MSR_GUARD: bool = false;
/// Mis a vrai si la lecture MSR protegee a provoque une #GP ou #UD.
static mut MSR_FAULT: bool = false;

/// Debut d'une lecture MSR protegee.
pub fn msr_guard_begin() {
    // SAFETY: ecriture simple de drapeaux statiques, pas de reference.
    unsafe {
        MSR_FAULT = false;
        MSR_GUARD = true;
    }
}

/// Fin d'une lecture MSR protegee. Retourne vrai si le MSR n'existe pas (faute).
pub fn msr_guard_end() -> bool {
    // SAFETY: ecriture simple de drapeaux statiques, pas de reference.
    unsafe {
        MSR_GUARD = false;
        MSR_FAULT
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn idt_exc_rust(vec: u64, err: u64, rip: u64) -> u64 {
    // Lecture MSR protegee : #GP (13) ou #UD (6) -> on saute le rdmsr (2 octets : 0F 32) et on continue.
    // SAFETY: lecture de drapeau statique.
    if unsafe { MSR_GUARD } && (vec == 13 || vec == 6) {
        // SAFETY: ecriture de drapeau statique.
        unsafe { MSR_FAULT = true };
        return rip + 2;
    }
    // SAFETY: lecture de SCR, ecrit une seule fois au demarrage.
    let (fb, len, stride, w, h) = unsafe { SCR };
    if fb != 0 && h > 120 {
        let p = fb as *mut u32;
        // SAFETY: bandeau du bas de l'ecran, bornes verifiees.
        let buf = unsafe { core::slice::from_raw_parts_mut(p, len) };
        for y in (h - 120)..h {
            for x in 0..w {
                let idx = y * stride + x;
                if idx < len {
                    buf[idx] = 0x0040_0000;
                }
            }
        }
        let mut line = [b' '; 64];
        let mut n = 0;
        for &c in b"EXCEPTION " {
            line[n] = c;
            n += 1;
        }
        n += hex(&mut line[n..], vec, 2);
        for &c in b" ERR " {
            line[n] = c;
            n += 1;
        }
        n += hex(&mut line[n..], err, 8);
        // SAFETY: texte ecrit dans le bandeau, bornes verifiees par text().
        unsafe { text(fb, len, stride, w, h, 8, h - 110, &line[..n]) };
        let mut l2 = [b' '; 64];
        let mut m = 0;
        for &c in b"RIP " {
            l2[m] = c;
            m += 1;
        }
        m += hex(&mut l2[m..], rip, 16);
        // SAFETY: meme bandeau.
        unsafe { text(fb, len, stride, w, h, 8, h - 70, &l2[..m]) };
        let mut l3 = [b' '; 64];
        let t = b"REDEMARRAGE AUTOMATIQUE DANS 5 S.";
        l3[..t.len()].copy_from_slice(t);
        // SAFETY: meme bandeau.
        unsafe { text(fb, len, stride, w, h, 8, h - 30, &l3[..t.len()]) };
    }
    let _ = vec;
    // Plus d'arret qui demande d'eteindre le PC a la main : le message reste 5 s, puis
    // reinitialisation materielle (port 0xCF9, reset du chipset). Le PC redemarre seul.
    for _ in 0..5_000_000u32 {
        // SAFETY: port 0x80 = port de diagnostic POST, ecriture sans effet ; sert de delai ~1 us.
        unsafe { asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack, preserves_flags)) };
    }
    // SAFETY: ecriture du registre de reset du chipset (0x06 = reset a froid complet).
    unsafe { asm!("out dx, al", in("dx") 0x0CF9u16, in("al") 0x06u8, options(nomem, nostack, preserves_flags)) };
    loop {
        // SAFETY: si le reset n'a pas eu lieu, on reste en attente sans rien modifier.
        unsafe { asm!("hlt", options(nomem, nostack)) };
    }
}
