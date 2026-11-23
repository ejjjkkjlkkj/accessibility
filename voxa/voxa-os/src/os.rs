//! Plateforme de l'OS pour le lecteur d'ecran `silicon-core` : parole (provisoire : texte + secours sonore), alimentation (ACPI S5, reinitialisation par le controleur clavier),
//! informations materiel (SMBIOS copie avant la sortie), lecture du BIOS (HII copie avant la sortie), clavier 8042 par scrutation.
use crate::{console, cpu};
use silicon_core::menu::{Key, MenuApp, Platform, Power};
use silicon_core::sysinfo::{smbios_lines, Lines};
use silicon_core::Out;
use voxa_kernel::hw::{kbd_poll, speaker, Kbd, Key as TKey};
use voxa_agent::{normaliser, Action, Agent, Hote, Texte};
use core::fmt::Write as _;

pub struct Os { pub gui: crate::gui::Etat, voix: Option<crate::hda::Hda>, voix_erreur: u8, pub smbios: &'static [u8], pub hii: &'static [u8], pub pm1a: u32, pub s5: Option<u16>, pub memoire_mo: u64, pub pci: usize, pub audio: usize, pub tables: [[u8; 4]; 24], pub ntables: usize, pub injection: [Key; 8], pub ninj: usize, file: [TKey; 32], tete: usize, n_file: usize, kbd: Kbd }

const MAX_SAMPLES: usize = 90_000;
#[repr(C, align(128))]
struct Stereo([i16; MAX_SAMPLES * 2]);
static mut STEREO: Stereo = Stereo([0; MAX_SAMPLES * 2]);
static mut MONO: [i16; MAX_SAMPLES] = [0; MAX_SAMPLES];
static mut WORK: voxa_speech::synth::Work = voxa_speech::synth::Work::new();
fn stall_us(us: usize) { cpu::delay_us(us.min(u32::MAX as usize) as u32); }

fn num(o: &mut Out, mut n: u64) { let mut t = [0u8; 20]; let mut i = t.len(); if n == 0 { i -= 1; t[i] = b'0'; } while n > 0 { i -= 1; t[i] = b'0' + (n % 10) as u8; n /= 10; } o.push(core::str::from_utf8(&t[i..]).unwrap_or("0")); }

impl Os {
    #[allow(clippy::too_many_arguments)]
    pub fn nouveau(smbios: &'static [u8], hii: &'static [u8], pm1a: u32, s5: Option<u16>, memoire_mo: u64, pci: usize, audio: usize, tables: [[u8; 4]; 24], ntables: usize) -> Os {
        Os { gui: crate::gui::Etat::nouveau(), voix: None, voix_erreur: 0, smbios, hii, pm1a, s5, memoire_mo, pci, audio, tables, ntables, injection: [Key::Enter; 8], ninj: 0, file: [TKey::Enter; 32], tete: 0, n_file: 0, kbd: Kbd::default() }
    }
    /// Initialise la voix : pilote HD Audio direct (E/S et memoire), SANS aucun service UEFI. Le resultat est dit honnetement : initialise ne veut pas dire audible.
    pub fn init_voix(&mut self) {
        match crate::hda::Hda::init(stall_us) {
            Ok(h) => { crate::log!("[audio] controleur HD Audio initialise apres la sortie des services UEFI (codec {:#x}, sortie {}, convertisseur {}).", h.vendor, h.pin, h.dac); self.voix = Some(h); }
            Err(e) => { self.voix_erreur = e; crate::log!("[audio] voix indisponible : code d erreur du pilote {} (1 adresse, 2 codec, 3 files, 4 chemin de sortie, 5 aucun controleur).", e); }
        }
    }
    /// Synthetise `text` (francais) puis le joue par DMA. true = tout a ete consomme par le DMA (ne prouve pas que le son est entendu).
    fn parler_audio(&mut self, text: &str) -> bool {
        use voxa_speech::phon::{phon_texte, utf8_to_u16, PhonOut}; use voxa_speech::synth::parler;
        let Some(h) = self.voix.as_mut() else { return false };
        // Ligne de diagnostic EXACTE (UTF-8, avec accents) : le texte reellement donne a la synthese ; les lignes « > » sont repliees en ASCII pour l ecran.
        console::serial_byte(b'='); console::serial_byte(b' '); for b in text.bytes() { console::serial_byte(b); } console::serial_byte(13); console::serial_byte(10);
        let mut src = [0u16; 256]; let n = utf8_to_u16(text, &mut src);
        let (mut d, mut ok) = (0usize, true); let mut o = PhonOut::new();
        while d < n {
            let nd = phon_texte(&src[..n], d, &mut o); d = if nd > d { nd } else { d + 1 };
            if o.len() == 0 { continue; }
            let (mono, stereo, work) = unsafe { (&mut *core::ptr::addr_of_mut!(MONO), &mut *core::ptr::addr_of_mut!(STEREO), &mut *core::ptr::addr_of_mut!(WORK)) };
            let r = parler(work, o.as_str(), mono);
            if r.samples == 0 { continue; }
            let frames = (r.samples + 31) / 32 * 32;
            for k in 0..frames { let s = if k < r.samples { mono[k] } else { 0 }; stereo.0[2 * k] = s; stereo.0[2 * k + 1] = s; }
            let lu = h.jouer(stereo.0.as_ptr() as u64, (frames * 4) as u32);
            if !lu { ok = false; crate::log!("[audio] ECHEC de lecture DMA"); }
            if r.truncated { ok = false; }
        }
        ok
    }
    pub fn assistant_boucle(&mut self) -> bool {
        let mut agent = Agent::nouveau(); let mut ligne = [0u8; 120]; let mut n = 0usize;
        crate::gui::dessiner_assistant(self);
        loop {
            let mut k = 0usize;
            while let Some(t) = self.touche_suivante() {
                k += 1;
                match t {
                    TKey::Escape => return true,
                    TKey::Backspace => { if n > 0 { n -= 1; while n > 0 && ligne[n] & 0xC0 == 0x80 { n -= 1; } self.speak("efface", true); } }
                    TKey::Char(c) => { let mut b = [0u8; 4]; let s = c.encode_utf8(&mut b); if n + s.len() <= ligne.len() { ligne[n..n + s.len()].copy_from_slice(s.as_bytes()); n += s.len(); self.speak(s, true); } }
                    TKey::Enter => {
                        let texte = core::str::from_utf8(&ligne[..n]).unwrap_or("").to_owned_buf(); n = 0;
                        { let mut o = Texte::vide(); let _ = write!(o, "Vous avez dit : {}", texte.as_str()); self.speak(o.as_str(), true); }
                        let r = agent.tour(texte.as_str(), self); self.gui.reponse(r.texte.as_str()); self.speak(r.texte.as_str(), false);
                        match r.action {
                            Action::Quitter => return true,
                            Action::Eteindre => self.power(Power::Shutdown),
                            Action::Redemarrer => self.power(Power::Restart),
                            Action::Bip => { speaker(&mut cpu::RealPorts, 880); cpu::delay_us(300_000); speaker(&mut cpu::RealPorts, 0); }
                            Action::OuvrirBios => { self.injection = [Key::Home, Key::Down, Key::Down, Key::Down, Key::Down, Key::Down, Key::Enter, Key::Enter]; self.ninj = 7; return true; }
                            Action::Rien => {}
                        }
                    }
                    _ => {}
                }
                { let m = n.min(self.gui.a_entree.len()); self.gui.a_entree[..m].copy_from_slice(&ligne[..m]); self.gui.a_n = m; }
                crate::gui::dessiner_assistant(self);
            }
            if k == 0 { cpu::delay_us(500); }
        }
    }
    /// Prochaine touche, dans l'ordre d'arrivee : d'abord la file, sinon le materiel (les touches lues en meme temps sont rangees dans la file, jamais perdues).
    pub fn touche_suivante(&mut self) -> Option<TKey> {
        if self.n_file == 0 {
            let mut t = [TKey::Enter; 16]; let k = kbd_poll(&mut cpu::RealPorts, &mut self.kbd, &mut t);
            for x in t[..k].iter().copied() { if self.n_file < 32 { self.file[(self.tete + self.n_file) % 32] = x; self.n_file += 1; } }
        }
        if self.n_file == 0 { return None; }
        let x = self.file[self.tete]; self.tete = (self.tete + 1) % 32; self.n_file -= 1; Some(x)
    }
}

impl Platform for Os {
    /// Ecrit le texte (serie + ecran) puis le dit par la voix. false si la voix est indisponible ou a echoue : le coeur passe alors au secours sonore (bips).
    fn speak(&mut self, text: &str, _interrupt: bool) -> bool { use core::fmt::Write; self.gui.dire(text); let _ = console::Sortie.write_str("> "); let _ = console::Sortie.write_str(text); let _ = console::Sortie.write_str("\n"); self.parler_audio(text) }
    fn power(&mut self, p: Power) {
        match p {
            Power::Restart => { crate::log!("Redemarrage demande par l'utilisateur (controleur clavier)."); cpu::outb(0x64, 0xFE); }
            Power::Shutdown => {
                crate::log!("Extinction demandee par l'utilisateur (ACPI S5).");
                if let (true, Some(t)) = (self.pm1a != 0, self.s5) { cpu::outw(self.pm1a as u16, (t << 10) | (1 << 13)); }
                else { crate::log!("Extinction ACPI impossible (registre ou type S5 introuvable) : maintenez le bouton d'alimentation."); }
            }
        }
        cpu::halt_forever();
    }
    fn assistant(&mut self) -> bool { self.gui.assistant = true; self.gui.a_n = 0; self.gui.a_rep_n = 0; let r = self.assistant_boucle(); self.gui.assistant = false; r }
    /// Le test du son est une PHRASE dite par la voix (chaine audio complete) ; le haut-parleur du PC ne sert qu'en secours si la voix echoue.
    fn test_sound(&mut self) -> bool {
        if self.parler_audio("Ceci est un test du son.") { return true; }
        speaker(&mut cpu::RealPorts, 880); cpu::delay_us(300_000); speaker(&mut cpu::RealPorts, 0); false
    }
    fn hardware_summary(&self, out: &mut Out) { out.push("Voxa OS, memoire "); num(out, self.memoire_mo); out.push(" Mo, "); num(out, self.pci as u64); out.push(" peripheriques PCI."); }
    fn system_lines(&self, lines: &mut Lines) {
        let mut brand = [0u8; 48];
        for (i, leaf) in (0x8000_0002u32..=0x8000_0004).enumerate() { let r = core::arch::x86_64::__cpuid(leaf); for (j, w) in [r.eax, r.ebx, r.ecx, r.edx].iter().enumerate() { brand[i * 16 + j * 4..i * 16 + j * 4 + 4].copy_from_slice(&w.to_le_bytes()); } }
        let cpu_nom = core::str::from_utf8(&brand).unwrap_or("").trim_matches(|c: char| c == ' ' || c == '\0');
        if self.smbios.is_empty() { let mut o = Out::new(); o.push("Processeur : "); o.push(cpu_nom); lines.add(o.as_str()); lines.add("Tables SMBIOS indisponibles."); } else { smbios_lines(self.smbios, cpu_nom, lines); }
        let mut o = Out::new(); o.push("Memoire utilisable : "); num(&mut o, self.memoire_mo); o.push(" Mo."); lines.add(o.as_str());
        let mut o = Out::new(); o.push("Bus PCI : "); num(&mut o, self.pci as u64); o.push(" peripheriques, dont "); num(&mut o, self.audio as u64); o.push(" controleurs audio."); lines.add(o.as_str());
    }
    fn beep(&mut self, freq: u32, ms: u32) -> bool { if freq == 0 { cpu::delay_us(ms * 1000); } else { speaker(&mut cpu::RealPorts, freq); cpu::delay_us(ms * 1000); speaker(&mut cpu::RealPorts, 0); } true }
    fn bios_count(&mut self) -> usize { if self.hii.is_empty() { 0 } else { voxa_hii::nb_lignes(self.hii) } }
    fn bios_line(&mut self, n: usize, out: &mut Out) -> Option<u16> {
        let l = voxa_hii::ligne(self.hii, n)?;
        out.push(l.genre.mot()); out.push(" : ");
        if l.texte.est_vide() { out.push("sans titre"); } else { out.push(core::str::from_utf8(l.texte.as_bytes()).unwrap_or("?")); }
        if !l.valeur.est_vide() { out.push(" = "); out.push(core::str::from_utf8(l.valeur.as_bytes()).unwrap_or("?")); }
        if !l.aide.est_vide() { out.push(". "); out.push(core::str::from_utf8(l.aide.as_bytes()).unwrap_or("?")); }
        Some(l.cible)
    }
    fn bios_page_line(&mut self, page: u16) -> Option<usize> { voxa_hii::indice_page(self.hii, page) }
}


fn bcd(v: u8, binaire: bool) -> u8 { if binaire { v } else { (v >> 4) * 10 + (v & 15) } }
/// Horloge materielle (CMOS, ports 0x70/0x71) : heure et minute. Lecture seule. L'horloge peut etre en heure universelle selon le systeme qui l'a reglee.
fn lire_rtc() -> Option<(u8, u8)> {
    let reg = |r: u8| { cpu::outb(0x70, r); cpu::inb(0x71) };
    for _ in 0..1000 { if reg(0x0A) & 0x80 == 0 { break; } }
    let b = reg(0x0B); let binaire = b & 4 != 0; let (h, m) = (reg(0x04), reg(0x02));
    let pm = h & 0x80 != 0 && b & 2 == 0; let mut hh = bcd(h & 0x7F, binaire); if pm { hh = (hh % 12) + 12; }
    let mm = bcd(m, binaire); if hh > 23 || mm > 59 { None } else { Some((hh, mm)) }
}

impl Hote for Os {
    fn materiel(&mut self, t: &mut Texte) {
        let mut l = Lines::new(); self.system_lines(&mut l); let mut trouve = false;
        for i in 0..l.len() { if let Some(x) = l.get(i) { if x.starts_with("Processeur") || x.starts_with("Machine") { let _ = write!(t, "{} ", x); trouve = true; } } }
        if !trouve { let _ = write!(t, "Informations materiel indisponibles."); }
    }
    fn memoire(&mut self, t: &mut Texte) { let _ = write!(t, "Memoire utilisable : {} Mo.", self.memoire_mo); }
    fn pci(&mut self, t: &mut Texte) { let _ = write!(t, "{} peripheriques PCI.", self.pci); }
    fn audio(&mut self, t: &mut Texte) {
        let etat = if self.voix.is_some() { "La voix est initialisee : le controleur repond, mais seule une ecoute confirme qu'elle est audible." } else { "La voix est indisponible, les bips de secours sont utilises." };
        let _ = write!(t, "{} controleurs audio detectes. {}", self.audio, etat);
    }
    fn acpi(&mut self, t: &mut Texte) { let _ = write!(t, "{} tables ACPI :", self.ntables); for i in 0..self.ntables { let _ = write!(t, " {}", core::str::from_utf8(&self.tables[i]).unwrap_or("?")); } }
    fn heure(&mut self, t: &mut Texte) -> bool { match lire_rtc() { Some((h, m)) => { let _ = write!(t, "Il est {} heures {} d'apres l'horloge materielle.", h, m); true } None => false } }
    fn bios_chercher(&mut self, mots: &[&str], t: &mut Texte) -> usize {
        if self.hii.is_empty() { let _ = write!(t, "Les ecrans du BIOS ne sont pas disponibles."); return 1; }
        let n = voxa_hii::nb_lignes(self.hii); let mut meilleur = 0usize; let mut trouvees = [0usize; 3]; let mut nt = 0usize;
        // une seule traversee du BIOS (ligne(n) repartirait du debut a chaque appel) : on garde les 3 premieres lignes de question qui contiennent le plus de mots demandes
        let _ = n; let mut sc = [0usize; 3];
        voxa_hii::visiter(self.hii, &mut |i, l| {
            if matches!(l.genre, voxa_hii::Genre::Option | voxa_hii::Genre::Page | voxa_hii::Genre::Titre | voxa_hii::Genre::Ensemble) { return true; }
            let nl = normaliser(core::str::from_utf8(l.texte.as_bytes()).unwrap_or("")); let score = mots.iter().filter(|m| nl.as_str().contains(**m)).count();
            if score == 0 { return true; }
            if score > meilleur { meilleur = score; nt = 0; }
            if score == meilleur && nt < 3 { trouvees[nt] = i; sc[nt] = score; nt += 1; }
            true });
        for k in 0..nt { if let Some(l) = voxa_hii::ligne(self.hii, trouvees[k]) { let _ = write!(t, "{} : {}. ", l.genre.mot(), core::str::from_utf8(l.texte.as_bytes()).unwrap_or("?")); let a = l.aide.as_bytes(); let m = a.len().min(70); if m > 0 && k == 0 { let _ = write!(t, "{}. ", core::str::from_utf8(&a[..m]).unwrap_or("")); } } }
        if nt > 0 { let _ = write!(t, "Valeur actuelle non lue : elle est dans la memoire du BIOS."); }
        nt
    }
}

/// Petit tampon pour garder la saisie pendant que l'agent emprunte `self`.
trait VersTampon { fn to_owned_buf(&self) -> Texte; }
impl VersTampon for str { fn to_owned_buf(&self) -> Texte { let mut t = Texte::vide(); let _ = t.write_str(self); t } }

fn touche(k: TKey) -> Key { match k { TKey::Char(c) => Key::Char(c), TKey::Enter => Key::Enter, TKey::Escape => Key::Escape, TKey::Backspace => Key::Backspace, TKey::Tab => Key::Tab, TKey::Delete => Key::Delete, TKey::Up => Key::Up, TKey::Down => Key::Down, TKey::Left => Key::Left, TKey::Right => Key::Right, TKey::Home => Key::Home, TKey::End => Key::End, TKey::PageUp => Key::PageUp, TKey::PageDown => Key::PageDown } }

/// Boucle principale du lecteur d'ecran de l'OS : clavier 8042 par scrutation -> coeur semantique -> annonces. Ne revient jamais.
pub fn boucle(mut os: Os) -> ! {
    crate::log!("Ecrans du BIOS lisibles : {} lignes ({} octets).", if os.hii.is_empty() { 0 } else { voxa_hii::nb_lignes(os.hii) }, os.hii.len());
    let mut app = MenuApp::new();
    app.flush(&mut os);
    crate::gui::dessiner(&app, &mut os);
    loop {
        let mut n = 0usize;
        while let Some(t) = os.touche_suivante() { n += 1; traiter(&mut os, &mut app, t); }
        if os.ninj > 0 { injecter(&mut os, &mut app); crate::gui::dessiner(&app, &mut os); }
        if n == 0 { cpu::delay_us(500); }
    }
}

/// Les touches demandees par l'agent (ex. ouvrir le BIOS) passent AVANT les touches deja tapees : elles sont rejouees des que l'assistant rend la main.
fn injecter(os: &mut Os, app: &mut MenuApp) {
    if os.ninj == 0 { return; }
    let inj = os.injection; let m = os.ninj; os.ninj = 0;
    for k in inj[..m].iter().copied() { app.handle(k); app.flush(os); }
}

/// Une touche : journal (Tab depuis le menu) ou application ; l'ecran est redessine ensuite depuis l'etat de l'application.
fn traiter(os: &mut Os, app: &mut MenuApp, t: TKey) {
    if os.gui.journal {
        match t {
            TKey::Tab | TKey::Escape => { os.gui.journal = false; os.speak("Retour au menu.", true); }
            TKey::Up => os.gui.jdec += 1, TKey::Down => os.gui.jdec = os.gui.jdec.saturating_sub(1),
            TKey::PageUp => os.gui.jdec += 10, TKey::PageDown => os.gui.jdec = os.gui.jdec.saturating_sub(10), TKey::End => os.gui.jdec = 0,
            _ => {}
        }
        crate::gui::dessiner(app, os); return;
    }
    if matches!(t, TKey::Tab) && app.mode() == silicon_core::menu::Mode::Menu {
        os.gui.journal = true; os.gui.jdec = 0; os.speak("Journal du système. Flèches : parcourir. Tab ou Échap : retour.", true);
        crate::gui::dessiner(app, os); return;
    }
    app.handle(touche(t)); app.flush(os); injecter(os, app);
    crate::gui::dessiner(app, os);
}
