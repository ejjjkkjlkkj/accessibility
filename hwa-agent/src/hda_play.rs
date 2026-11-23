// Sortie audio HDA : test de lecture d'un signal carre de 440 Hz pendant 0,5 s.
// Reference : HDA 1.0 : GCAP (ISS bits 11:8, OSS bits 15:12) ; descripteurs de flux a 0x80 + ISS * 0x20
// (SDnCTL 0 : SRST bit 0, RUN bit 1, IOCE bit 2, numero de flux bits 23:20 ; SDnLPIB +4 ; SDnCBL +8 ;
// SDnLVI +C ; SDnFMT +12h ; SDnBDPL +18h ; SDnBDPU +1Ch) ; verbes : 705 (etat d'alimentation), 701
// (selection de connexion), 707 (controle de broche), 706 (flux et canal du convertisseur),
// 2xxx (format du convertisseur, 0x0011 = 48 kHz, 16 bits, 2 canaux), 3xxx (amplificateur).
// Entree d'un descripteur de tampon (BDL) : adresse 64 bits, longueur 32 bits, bit IOC 32 bits.
use crate::usb_r0::{hda_cmd, Line};

// Tampon DMA : 7 s a 48 kHz, stereo 16 bits (voix et signal de test).
const MAX_FRAMES: usize = 48000 * 7;
const BYTES: usize = MAX_FRAMES * 4;

#[repr(C, align(4096))]
struct Pcm([u8; BYTES]);
static mut PCM: Pcm = Pcm([0; BYTES]);
// Liste de descripteurs de tampon : deux entrees (la spec HDA 3.3.38 exige LVI >= 1).
#[repr(C, align(128))]
struct Bdl([u32; 8]);
static mut BDL: Bdl = Bdl([0; 8]);
// Voix : echantillons mono 16 kHz avant conversion vers 48 kHz stereo.
static mut PAROLE: [i16; 16000 * 7] = [0; 16000 * 7];
// Route trouvee une fois : (convertisseur, broche, adresse du descripteur de flux).
static mut ROUTE: (u32, u32, u64) = (0, 0, 0);

/// Coefficients de l'interpolation 16 kHz -> 48 kHz (sinc fenetree de Hann, 8 prises, coupure
/// 0,45 fois la frequence d'entree). Ligne k = phase k/3 entre deux echantillons ; Q15, somme = 32768.
/// Calcul : h(d) = sinc(2 fc d) * (1 + cos(pi d / 4)) / 2, d = j - 3 - k/3, puis normalisation.
const COEF_SINC: [[i32; 8]; 3] = [
    [412, -1533, 2752, 29506, 2752, -1533, 412, 0],
    [0, 512, -3447, 24872, 13877, -3937, 930, -39],
    [-39, 930, -3937, 13877, 24872, -3447, 512, 0],
];

unsafe fn rd8(a: u64) -> u8 {
    // SAFETY: registre MMIO HDA.
    unsafe { core::ptr::read_volatile(a as *const u8) }
}
unsafe fn rd16(a: u64) -> u16 {
    // SAFETY: registre MMIO HDA.
    unsafe { core::ptr::read_volatile(a as *const u16) }
}
unsafe fn rd32(a: u64) -> u32 {
    // SAFETY: registre MMIO HDA.
    unsafe { core::ptr::read_volatile(a as *const u32) }
}
unsafe fn wr8(a: u64, v: u8) {
    // SAFETY: registre MMIO HDA.
    unsafe { core::ptr::write_volatile(a as *mut u8, v) }
}
unsafe fn wr16(a: u64, v: u16) {
    // SAFETY: registre MMIO HDA.
    unsafe { core::ptr::write_volatile(a as *mut u16, v) }
}
unsafe fn wr32(a: u64, v: u32) {
    // SAFETY: registre MMIO HDA.
    unsafe { core::ptr::write_volatile(a as *mut u32, v) }
}

/// Cherche la sortie (convertisseur + broche), l'alimente et la regle. Fait une seule fois.
unsafe fn route(bar: u64, cad: u32) -> Result<(u32, u32, u64), &'static [u8]> {
    // SAFETY: verbes HDA (lecture et configuration) sur un controleur deja sorti de reset.
    unsafe {
        if ROUTE.2 != 0 {
            return Ok(ROUTE);
        }
        // 1. Groupe de fonctions audio
        let root = crate::usb_r0::codec_root(bar, cad).unwrap_or(0);
        let fgs = (root >> 16) & 0xFF;
        let fgc = root & 0xFF;
        let mut fg = 0u32;
        for n in fgs..fgs + fgc {
            if hda_cmd(bar, cad, n, (0xF00 << 8) | 0x05).unwrap_or(0) & 0xFF == 1 {
                fg = n;
                break;
            }
        }
        if fg == 0 {
            return Err(b"AUDIO : codec sans groupe audio (pas de reponse ?)");
        }
        // 2. Broche de sortie et chemin jusqu'a un convertisseur (spec HDA 7.3.4.6 type de noeud,
        //    7.3.3.3 liste de connexion). Entre la broche et le convertisseur il peut y avoir un
        //    melangeur ou un selecteur : le chemin est suivi noeud par noeud, rien n'est suppose.
        //    Ordre d'essai des broches : haut-parleur (1), casque (2), sortie ligne (0).
        let fgn = hda_cmd(bar, cad, fg, (0xF00 << 8) | 0x04).unwrap_or(0);
        let ws = (fgn >> 16) & 0xFF;
        let wc = fgn & 0xFF;
        let mut chemin = [0u32; 5]; // chemin[0] = broche ... chemin[n-1] = convertisseur
        let mut index = [0u32; 5]; // index[k] = rang de chemin[k+1] dans la liste de chemin[k]
        let mut nch = 0usize;
        'broches: for voulu in [1u32, 2, 0] {
            for w in ws..ws + wc {
                let t = (hda_cmd(bar, cad, w, (0xF00 << 8) | 0x09).unwrap_or(0) >> 20) & 0xF;
                if t != 4 {
                    continue;
                }
                let cfg = hda_cmd(bar, cad, w, 0xF1C << 8).unwrap_or(0);
                if (cfg >> 30) & 0x3 == 1 || (cfg >> 20) & 0xF != voulu {
                    continue;
                }
                if hda_cmd(bar, cad, w, (0xF00 << 8) | 0x0C).unwrap_or(0) & (1 << 4) == 0 {
                    continue; // la broche ne sait pas sortir du son
                }
                chemin[0] = w;
                nch = 1;
                // Descente : a chaque noeud, premiere entree qui mene a un convertisseur.
                while nch < 5 {
                    let cur = chemin[nch - 1];
                    let ty = (hda_cmd(bar, cad, cur, (0xF00 << 8) | 0x09).unwrap_or(0) >> 20) & 0xF;
                    if ty == 0 {
                        break 'broches;
                    }
                    let len = hda_cmd(bar, cad, cur, (0xF00 << 8) | 0x0E).unwrap_or(0);
                    if len & 0x80 != 0 || len & 0x7F == 0 {
                        break; // forme longue non geree, ou pas de liste
                    }
                    let n = (len & 0x7F).min(16);
                    let mut choisi = None;
                    let mut premier = None;
                    for k in 0..n {
                        let quad = hda_cmd(bar, cad, cur, (0xF02 << 8) | (k & !3)).unwrap_or(0);
                        let e = (quad >> (8 * (k & 3))) & 0xFF;
                        if e & 0x80 != 0 || e == 0 {
                            continue; // plage de noeuds : non geree
                        }
                        let te = (hda_cmd(bar, cad, e, (0xF00 << 8) | 0x09).unwrap_or(0) >> 20) & 0xF;
                        if te == 0 {
                            choisi = Some((k, e));
                            break;
                        }
                        if premier.is_none() && (te == 2 || te == 3) {
                            premier = Some((k, e));
                        }
                    }
                    match choisi.or(premier) {
                        Some((k, e)) => {
                            index[nch - 1] = k;
                            chemin[nch] = e;
                            nch += 1;
                        }
                        None => break,
                    }
                }
                nch = 0;
            }
        }
        if nch < 2 {
            return Err(b"AUDIO : pas de chemin broche-convertisseur");
        }
        let pin = chemin[0];
        let dac = chemin[nch - 1];
        // 3. Alimentation D0 du groupe et de chaque noeud du chemin (verbe 705), puis lecture de
        //    l'etat (F05) jusqu'a ce que l'etat reel (bits 7:4) soit D0.
        hda_cmd(bar, cad, fg, 0x705 << 8);
        for &n in &chemin[..nch] {
            hda_cmd(bar, cad, n, 0x705 << 8);
        }
        for &n in core::iter::once(&fg).chain(&chemin[..nch]) {
            for _ in 0..50 {
                // Une reponse absente (None) n'est pas un etat D0 : on continue d'attendre.
                if hda_cmd(bar, cad, n, 0xF05 << 8).unwrap_or(0xF0) & 0xF0 == 0 {
                    break;
                }
                crate::usb_r0::attente_tsc(3_000_000);
            }
        }
        // 4. Le long du chemin : selection de l'entree (701) sauf sur un melangeur, ouverture
        //    de l'amplificateur d'entree de cette entree et de l'amplificateur de sortie (verbe 3),
        //    au gain 0 dB lu dans les capacites (parametres 0Dh et 12h, bits 6:0).
        let gain = |n: u32, param: u32| -> u32 {
            let mut c = hda_cmd(bar, cad, n, (0xF00 << 8) | param).unwrap_or(0);
            if c == 0 {
                c = hda_cmd(bar, cad, fg, (0xF00 << 8) | param).unwrap_or(0);
            }
            c & 0x7F
        };
        for k in 0..nch {
            let n = chemin[k];
            let caps = hda_cmd(bar, cad, n, (0xF00 << 8) | 0x09).unwrap_or(0);
            let ty = (caps >> 20) & 0xF;
            if k + 1 < nch {
                if ty != 2 {
                    hda_cmd(bar, cad, n, (0x701 << 8) | index[k]);
                }
                if caps & (1 << 1) != 0 {
                    // amplificateur d'entree : bit 14 entree, bits 13/12 gauche/droite, index bits 11:8
                    hda_cmd(bar, cad, n, (0x3 << 16) | 0x7000 | (index[k] << 8) | gain(n, 0x0D));
                }
            }
            if caps & (1 << 2) != 0 {
                // amplificateur de sortie : bit 15 sortie, bits 13/12 gauche/droite, muet (bit 7) a 0
                hda_cmd(bar, cad, n, (0x3 << 16) | 0xB000 | gain(n, 0x12));
            }
        }
        // 5. Broche en sortie (707, bit 6) ; amplificateur externe (EAPD, verbe 70C bit 1) allume
        //    si la broche le declare (bit 16 des capacites de broche) : sans lui un haut-parleur
        //    reste muet.
        hda_cmd(bar, cad, pin, (0x707 << 8) | 0x40);
        if hda_cmd(bar, cad, pin, (0xF00 << 8) | 0x0C).unwrap_or(0) & (1 << 16) != 0 {
            hda_cmd(bar, cad, pin, (0x70C << 8) | 0x02);
        }
        // 6. Format et flux du convertisseur : 48 kHz 16 bits 2 canaux, flux 1, canal 0
        hda_cmd(bar, cad, dac, (0x2 << 16) | 0x0011);
        hda_cmd(bar, cad, dac, (0x706 << 8) | 0x10);
        // 7. Descripteur de flux de sortie (premier apres les flux d'entree)
        let gcap = rd16(bar);
        let iss = ((gcap >> 8) & 0xF) as u64;
        if (gcap >> 12) & 0xF == 0 {
            return Err(b"AUDIO : pas de flux de sortie");
        }
        ROUTE = (dac, pin, bar + 0x80 + iss * 0x20);
        Ok(ROUTE)
    }
}

/// Joue les `frames` premieres trames du tampon PCM (48 kHz stereo). Attend la fin en suivant
/// la position du flux (LPIB), avec un delai maximal. Retourne la derniere position lue.
unsafe fn jouer(sd: u64, frames: usize) -> u32 {
    // SAFETY: registres du flux de sortie ; tampons statiques alignes.
    unsafe {
        // Longueur multiple de 256 octets, coupee en deux tampons egaux.
        let octets = ((frames.min(MAX_FRAMES) * 4) & !0xFF).max(512);
        let moitie = octets / 2;
        let addr = core::ptr::addr_of!(PCM) as u64;
        let bdl = core::ptr::addr_of_mut!(BDL) as u64;
        let b = &mut *(bdl as *mut [u32; 8]);
        b[0] = addr as u32;
        b[1] = (addr >> 32) as u32;
        b[2] = moitie as u32;
        b[3] = 0;
        b[4] = (addr + moitie as u64) as u32;
        b[5] = ((addr + moitie as u64) >> 32) as u32;
        b[6] = moitie as u32;
        b[7] = 0;
        // Reinitialisation du flux, puis configuration
        wr8(sd, 1);
        let mut n = 0;
        while rd8(sd) & 1 == 0 && n < 1_000_000 {
            n += 1;
        }
        wr8(sd, 0);
        n = 0;
        while rd8(sd) & 1 != 0 && n < 1_000_000 {
            n += 1;
        }
        wr32(sd + 0x18, bdl as u32);
        wr32(sd + 0x1C, (bdl >> 32) as u32);
        wr32(sd + 0x08, octets as u32); // CBL
        wr16(sd + 0x0C, 1); // LVI : deux entrees
        wr16(sd + 0x12, 0x0011); // FMT
        // Lancement : flux 1 (SDnCTL bits 23:20) et RUN. Aucune interruption demandee.
        wr32(sd, (1 << 20) | (1 << 1));
        // Attente : environ 1 ms par tour ; fin quand la position revient au debut (tampon
        // cyclique) ou atteint la fin ; abandon si rien n'avance pendant 200 tours.
        // Pas de limite de temps fixe : la duree depend de la frequence du TSC (inconnue ici).
        // La fin est detectee par la position (LPIB) ; l'arret sur blocage est fait par le compteur `fige`.
        let limite = u32::MAX;
        let mut dernier = 0u32;
        let mut fige = 0u32;
        for _ in 0..limite {
            crate::usb_r0::attente_tsc(3_000_000);
            let lpib = rd32(sd + 0x04);
            if lpib < dernier || lpib as usize >= octets - 192 {
                break;
            }
            if lpib == dernier {
                fige += 1;
                if fige > 200 {
                    break;
                }
            } else {
                fige = 0;
            }
            dernier = lpib;
        }
        wr32(sd, 0);
        wr8(sd + 0x03, 0x1C); // efface les drapeaux d'etat du flux (ecriture de 1)
        dernier
    }
}

/// Joue le signal de test (carre 440 Hz, 0,5 s). Retourne une ligne de resultat.
pub unsafe fn play_test(bar: u64, cad: u32) -> Line {
    let mut l = Line::new();
    // SAFETY: route HDA, tampon statique.
    unsafe {
        let (dac, pin, sd) = match route(bar, cad) {
            Ok(r) => r,
            Err(e) => {
                l.s(e);
                return l;
            }
        };
        let pcm = core::ptr::addr_of_mut!(PCM) as *mut i16;
        for f in 0..24000usize {
            let v: i16 = if (f / 55) % 2 == 0 { 2000 } else { -2000 };
            *pcm.add(f * 2) = v;
            *pcm.add(f * 2 + 1) = v;
        }
        let lpib = jouer(sd, 24000);
        l.s(b"AUDIO LPIB ").h(lpib, 5).s(b" PIN ").h(pin, 2).s(b" DAC ").h(dac, 2);
    }
    l
}

/// Dit un texte francais par la sortie audio (voix a formants de voix.rs). Bloque pendant la
/// parole (7 s au plus par appel). Retourne le nombre de trames jouees, 0 si pas de sortie.
pub unsafe fn parler(bar: u64, cad: u32, texte: &[u8]) -> usize {
    // SAFETY: route HDA, tampons statiques ; un seul fil d'execution.
    unsafe {
        let Ok((_, _, sd)) = route(bar, cad) else { return 0 };
        let parole = &mut *core::ptr::addr_of_mut!(PAROLE);
        let n = crate::voix::dire(texte, parole);
        // 16 kHz mono -> 48 kHz stereo : interpolation par sinc fenetree (8 coefficients par phase,
        // coupure a 7,2 kHz). Les images au-dessus de 8 kHz, audibles avec une interpolation lineaire,
        // sont supprimees : la voix est plus nette.
        let pcm = core::ptr::addr_of_mut!(PCM) as *mut i16;
        for f in 0..n * 3 {
            let (i, k) = (f / 3, f % 3);
            let mut acc: i64 = 0;
            for j in 0..8usize {
                let src = i as isize + j as isize - 3;
                if src >= 0 && (src as usize) < n {
                    acc += COEF_SINC[k][j] as i64 * parole[src as usize] as i64;
                }
            }
            let v = (acc >> 15).clamp(-32768, 32767) as i16;
            *pcm.add(f * 2) = v;
            *pcm.add(f * 2 + 1) = v;
        }
        // 20 ms de silence apres la parole (le flux est arrete pendant ce silence).
        let frames = n * 3;
        let fin = (frames + 960).min(MAX_FRAMES);
        for f in frames..fin {
            *pcm.add(f * 2) = 0;
            *pcm.add(f * 2 + 1) = 0;
        }
        jouer(sd, fin);
        frames
    }
}
