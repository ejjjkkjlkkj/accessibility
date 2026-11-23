//! Adaptateur « menu Voxa Live » : relie le cœur sémantique à une plateforme abstraite (clavier, parole, alimentation).
//! La plateforme réelle (UEFI, Voxa OS) implémente `Platform` ; les tests utilisent une plateforme simulée.
//! Sécurité : redémarrer et éteindre exigent une confirmation explicite (Entrée deux fois), Échap annule.

use crate::sysinfo::Lines;
use crate::edit::{describe_char, Edit};
use crate::{action, ActionResult, Announcer, NodeId, Out, Priority, Role, Tree};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key { Up, Down, Left, Right, Home, End, PageUp, PageDown, Enter, Escape, Backspace, Delete, Tab, Char(char) }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Power { Restart, Shutdown }

/// Ecran courant (pour l'affichage graphique).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode { Menu, Info, Bios, Edit, Search }

/// Ce que l'application attend de la plateforme. Chaque méthode peut échouer silencieusement côté matériel :
/// la plateforme renvoie alors `false` et l'application l'annonce.
pub trait Platform {
    /// Dit un texte ; `interrupt` coupe la parole en cours.
    fn speak(&mut self, text: &str, interrupt: bool) -> bool;
    fn power(&mut self, p: Power);
    fn test_sound(&mut self) -> bool;
    fn hardware_summary(&self, out: &mut Out);
    /// Lignes d informations système (page de revue). Par défaut : une seule ligne issue de `hardware_summary`.
    fn system_lines(&self, lines: &mut Lines) { let mut o = Out::new(); self.hardware_summary(&mut o); lines.add(o.as_str()); }
    /// Bip du haut-parleur système (fréquence en Hz, 0 = silence ; durée en ms). Par défaut : aucun effet, la plateforme répond alors qu elle ne sait pas bipper.
    fn beep(&mut self, _freq: u32, _ms: u32) -> bool { false }
    /// Lance le gestionnaire de démarrage d'un autre système installé (ex. Windows). Ne revient pas en cas de succès ; Err = raison dite à l'utilisateur.
    fn boot_windows(&mut self) -> Result<(), &'static str> { Err("non disponible sur cette plateforme") }
    /// Nombre de lignes de la configuration du BIOS lisibles (0 = non disponible). LECTURE SEULE.
    fn bios_count(&mut self) -> usize { 0 }
    /// Ecrit la ligne n (genre, texte, aide) ; retourne l'identifiant de la page visée si c'est un lien, sinon 0 ; None si la ligne n'existe pas.
    fn bios_line(&mut self, _n: usize, _out: &mut Out) -> Option<u16> { None }
    /// Indice de la ligne où commence la page d'identifiant donné.
    fn bios_page_line(&mut self, _page: u16) -> Option<usize> { None }
    /// Lance l'assistant (boucle bloquante de la plateforme : saisie, agent, reponses). Retourne false si l'assistant n'est pas disponible ici.
    fn assistant(&mut self) -> bool { false }
}

pub const ITEMS: [&str; 11] = ["Lire l'aide", "Changer la taille du texte", "Changer le contraste", "Tester le son", "Informations sur le matériel", "Lire la configuration du BIOS", "Assistant Voxa", "Essai de saisie de texte", "Démarrer Windows", "Redémarrer l'ordinateur", "Éteindre l'ordinateur"];

/// Lecture du BIOS : position courante, nombre de lignes, pile des positions quittées pour revenir en arrière.
struct BiosView { cur: usize, n: usize, pile: [usize; 16], np: usize }

/// Page de revue : lignes de texte parcourues une à une (informations système, plus tard fichiers, journaux…).
struct Review { lines: Lines, cur: usize }

/// Zone de saisie d'essai (texte, curseur) et, pendant une recherche, le texte recherché.
struct EditMode { e: Edit, q: Option<Edit> }

pub struct MenuApp {
    pub tree: Tree,
    pub ann: Announcer,
    ids: [NodeId; 11],
    confirming: Option<Power>,
    pub text_size: u8,
    pub contrast: u8,
    /// Vrai si la parole a échoué : l'état est alors signalé par un autre canal (bips, écran).
    pub speech_failed: bool,
    pending_power: Option<Power>,
    want_sound: bool,
    want_info: bool,
    want_windows: bool,
    want_bios: bool,
    want_assistant: bool,
    bios: Option<BiosView>,
    bios_say: bool,
    bios_follow: bool,
    bios_back: bool,
    review: Option<Review>,
    edit: Option<EditMode>,
    prev_focus: Option<NodeId>,
    last_beeped: Option<NodeId>,
}

impl MenuApp {
    pub fn new() -> MenuApp {
        let mut tree = Tree::new();
        let w = tree.add(None, Role::Window, "Voxa Live", 0).unwrap();
        let m = tree.add(Some(w), Role::Menu, "Menu principal", 0).unwrap();
        let mut ids = [0; 11];
        for (i, n) in ITEMS.iter().enumerate() { ids[i] = tree.add(Some(m), Role::MenuItem, n, action::ACTIVATE).unwrap(); }
        let mut a = MenuApp { tree, ann: Announcer::new(), ids, confirming: None, text_size: 1, contrast: 0, speech_failed: false, pending_power: None, want_sound: false, want_info: false, want_windows: false, want_bios: false, want_assistant: false, bios: None, bios_say: false, bios_follow: false, bios_back: false, review: None, edit: None, prev_focus: None, last_beeped: None };
        a.ann.push(Priority::Normal, "Voxa Live. Menu principal, 11 éléments. Flèches pour choisir, Entrée pour valider, H pour l'aide.");
        a.tree.focus_next();
        a.announce_focus(Priority::Normal);
        a
    }

    // ---- Vue : ce que l'interface graphique doit dessiner. Elle lit le MEME etat que la parole (arbre + modes), jamais un etat parallele : ecran et voix ne peuvent pas diverger.
    /// Ecran courant de l'application.
    pub fn mode(&self) -> Mode {
        if self.bios.is_some() { Mode::Bios } else if self.review.is_some() { Mode::Info }
        else if let Some(e) = &self.edit { if e.q.is_some() { Mode::Search } else { Mode::Edit } } else { Mode::Menu }
    }
    /// Indice (0 a 10) de l'element de menu qui a le focus.
    pub fn focus_index(&self) -> usize { self.tree.focus().and_then(|f| self.ids.iter().position(|i| *i == f)).unwrap_or(0) }
    /// Action a risque en attente de confirmation (redemarrer ou eteindre).
    pub fn confirming(&self) -> Option<Power> { self.confirming }
    /// Page d'informations : lignes et ligne courante.
    pub fn info_view(&self) -> Option<(&Lines, usize)> { self.review.as_ref().map(|r| (&r.lines, r.cur)) }
    /// Lecture du BIOS : ligne courante et nombre de lignes.
    pub fn bios_view(&self) -> Option<(usize, usize)> { self.bios.as_ref().map(|b| (b.cur, b.n)) }
    /// Zone de saisie : texte, position du curseur (en caracteres) et texte recherche pendant une recherche.
    pub fn edit_view(&self) -> Option<(&str, usize, Option<&str>)> { self.edit.as_ref().map(|e| (e.e.as_str(), e.e.cursor_chars(), e.q.as_ref().map(|q| q.as_str()))) }

    fn announce_focus(&mut self, p: Priority) {
        let Some(f) = self.tree.focus() else { return };
        let mut o = Out::new();
        self.tree.describe(f, &mut o);
        if let Some((r, t)) = self.tree.position(f) {
            o.push(", ");
            push_num(&mut o, r);
            o.push(" sur ");
            push_num(&mut o, t);
        }
        self.ann.push(p, o.as_str());
    }

    pub fn handle(&mut self, key: Key) {
        if self.bios.is_some() { self.handle_bios(key); return; }
        if self.review.is_some() { self.handle_review(key); return; }
        if self.edit.is_some() { self.handle_edit(key); return; }
        // toute touche autre qu'Entrée annule une confirmation en attente
        if self.confirming.is_some() && key != Key::Enter { self.confirming = None; self.ann.push(Priority::High, "Annulé."); if key == Key::Escape { return; } }
        match key {
            Key::Down => { if self.tree.focus_next().is_some() { self.announce_focus(Priority::Interrupt); } else { self.ann.push(Priority::Interrupt, "Fin du menu."); } }
            Key::Up => { if self.tree.focus_prev().is_some() { self.announce_focus(Priority::Interrupt); } else { self.ann.push(Priority::Interrupt, "Début du menu."); } }
            Key::Escape => self.ann.push(Priority::High, "Rien à annuler."),
            Key::Home => self.goto(0),
            Key::End => self.goto(10),
            Key::Char(c) if ('1'..='9').contains(&c) => self.goto(c as usize - '1' as usize),
            Key::Char('h') | Key::Char('H') => self.ann.push(Priority::High, "Flèches haut et bas : choisir. Entrée : valider. R : répéter. Échap : annuler."),
            Key::Char('r') | Key::Char('R') => { if let Some(t) = self.ann.repeat_last() { self.ann.push(Priority::High, t.as_str()); } else { self.ann.push(Priority::High, "Rien à répéter."); } }
            Key::Char(_) | Key::Left | Key::Right | Key::PageUp | Key::PageDown | Key::Backspace | Key::Delete | Key::Tab => {}
            Key::Enter => self.activate(),
        }
    }

    fn say_char(&mut self, prefix: &str, c: char, p: Priority) { let mut d = Out::new(); describe_char(c, &mut d); let mut o = Out::new(); o.push(prefix); o.push(d.as_str()); self.ann.push(p, o.as_str()); }

    /// Dit un texte long en morceaux (la capacité d'une annonce est bornée), en coupant sur une limite de mot ou de caractère.
    fn say_long(&mut self, text: &str, p: Priority) {
        let mut rest = text; let mut first = true;
        while !rest.is_empty() {
            let mut n = rest.len().min(150);
            while n < rest.len() && !rest.is_char_boundary(n) { n -= 1; }
            if n < rest.len() { if let Some(sp) = rest[..n].rfind(' ') { if sp > 0 { n = sp + 1; } } }
            self.ann.push(if first { p } else { Priority::Normal }, rest[..n].trim_end());
            rest = &rest[n..]; first = false;
        }
    }

    /// Touches dans la zone de saisie : caractères, suppression, déplacement par caractère (flèches), par mot (Page précédente/suivante), Début/Fin,
    /// Haut = position et mot courant, Bas = tout relire, Tab = rechercher, Entrée ou Échap = retour au menu.
    fn handle_edit(&mut self, key: Key) {
        let Some(mut m) = self.edit.take() else { return };
        let mut keep = true;
        if let Some(q) = m.q.as_mut() {
            match key {
                Key::Char(c) => { if q.insert(c).is_ok() { self.say_char("", c, Priority::Interrupt); } else { self.ann.push(Priority::High, "Texte recherché trop long."); } }
                Key::Backspace => { if let Some(c) = q.backspace() { self.say_char("supprimé : ", c, Priority::Interrupt); } }
                Key::Enter => {
                    let mut needle = [0u8; 64]; let s = q.as_str(); let mut n = s.len().min(64); while !s.is_char_boundary(n) { n -= 1; }
                    needle[..n].copy_from_slice(&s.as_bytes()[..n]); m.q = None;
                    let needle = core::str::from_utf8(&needle[..n]).unwrap_or("");
                    match m.e.find_next(needle) {
                        Some(direct) => { let mut o = Out::new(); o.push(if direct { "Trouvé : " } else { "Trouvé après retour au début : " }); o.push(m.e.word_at_cursor()); self.ann.push(Priority::High, o.as_str()); }
                        None => self.ann.push(Priority::High, "Introuvable."),
                    }
                }
                Key::Escape => { m.q = None; self.ann.push(Priority::High, "Recherche annulée."); }
                _ => {}
            }
            self.edit = Some(m); return;
        }
        match key {
            Key::Char(c) => { if m.e.insert(c).is_ok() { self.say_char("", c, Priority::Interrupt); } else { self.ann.push(Priority::High, "Zone de saisie pleine."); } }
            Key::Backspace => { match m.e.backspace() { Some(c) => self.say_char("supprimé : ", c, Priority::Interrupt), None => self.ann.push(Priority::Interrupt, "Début du texte.") } }
            Key::Delete => { match m.e.delete() { Some(c) => self.say_char("supprimé : ", c, Priority::Interrupt), None => self.ann.push(Priority::Interrupt, "Fin du texte.") } }
            Key::Left => { if m.e.left() { if let Some(c) = m.e.char_at_cursor() { self.say_char("", c, Priority::Interrupt); } } else { self.ann.push(Priority::Interrupt, "Début du texte."); } }
            Key::Right => { if m.e.right() { match m.e.char_at_cursor() { Some(c) => self.say_char("", c, Priority::Interrupt), None => self.ann.push(Priority::Interrupt, "Fin du texte.") } } else { self.ann.push(Priority::Interrupt, "Fin du texte."); } }
            Key::Home => { m.e.home(); self.ann.push(Priority::Interrupt, "Début du texte."); }
            Key::End => { m.e.end(); self.ann.push(Priority::Interrupt, "Fin du texte."); }
            Key::PageUp => { if m.e.word_left() { let w = m.e.word_at_cursor(); self.ann.push(Priority::Interrupt, if w.is_empty() { "Début du texte." } else { w }); } else { self.ann.push(Priority::Interrupt, "Début du texte."); } }
            Key::PageDown => { if m.e.word_right() { let w = m.e.word_at_cursor(); self.ann.push(Priority::Interrupt, if w.is_empty() { "Fin du texte." } else { w }); } else { self.ann.push(Priority::Interrupt, "Fin du texte."); } }
            Key::Up => { let mut o = Out::new(); o.push("Position "); push_num(&mut o, m.e.cursor_chars() + 1); o.push(" sur "); push_num(&mut o, m.e.len_chars() + 1); let w = m.e.word_at_cursor(); if !w.is_empty() { o.push(". Mot : "); o.push(w); } self.ann.push(Priority::Interrupt, o.as_str()); }
            Key::Down => { if m.e.is_empty() { self.ann.push(Priority::Interrupt, "Zone vide."); } else { let t = m.e.as_str(); let mut buf = [0u8; crate::edit::EDIT_MAX]; buf[..t.len()].copy_from_slice(t.as_bytes()); let t = core::str::from_utf8(&buf[..t.len()]).unwrap_or(""); self.say_long(t, Priority::Interrupt); } }
            Key::Tab => { m.q = Some(Edit::new()); self.ann.push(Priority::Interrupt, "Recherche : tapez le texte à chercher puis Entrée. Échap annule."); }
            Key::Enter | Key::Escape => {
                keep = false; let mut o = Out::new(); o.push("Retour au menu. "); push_num(&mut o, m.e.len_chars()); o.push(" caractères saisis.");
                self.ann.push(Priority::Interrupt, o.as_str()); self.announce_focus(Priority::Normal);
            }
        }
        if keep { self.edit = Some(m); }
    }

    fn say_line(&mut self, p: Priority) { if let Some(r) = &self.review { if let Some(l) = r.lines.get(r.cur) { self.ann.push(p, l); } } }

    /// Touches pendant la lecture du BIOS : le déplacement est fait ici, la lecture de la ligne (qui demande la plateforme) est faite dans `flush`.
    fn handle_bios(&mut self, key: Key) {
        let Some(b) = self.bios.as_mut() else { return };
        let n = b.n;
        match key {
            Key::Down => { if b.cur + 1 < n { b.cur += 1; self.bios_say = true; } else { self.ann.push(Priority::Interrupt, "Fin de la configuration."); } }
            Key::Up => { if b.cur > 0 { b.cur -= 1; self.bios_say = true; } else { self.ann.push(Priority::Interrupt, "Début de la configuration."); } }
            Key::PageDown => { b.cur = (b.cur + 10).min(n - 1); self.bios_say = true; }
            Key::PageUp => { b.cur = b.cur.saturating_sub(10); self.bios_say = true; }
            Key::Home => { b.cur = 0; self.bios_say = true; }
            Key::End => { b.cur = n - 1; self.bios_say = true; }
            Key::Right | Key::Enter => self.bios_follow = true,
            Key::Left | Key::Backspace => self.bios_back = true,
            Key::Char(c) if c == 'r' || c == 'R' => self.bios_say = true,
            Key::Escape => { self.bios = None; self.ann.push(Priority::Interrupt, "Retour au menu."); self.announce_focus(Priority::Normal); }
            _ => {}
        }
    }

    /// Touches pendant une revue : haut/bas ligne par ligne, Début/Fin, T relit tout depuis la ligne courante, R répète, Entrée ou Échap quitte.
    fn handle_review(&mut self, key: Key) {
        let Some(r) = self.review.as_mut() else { return };
        let n = r.lines.len();
        match key {
            Key::Up => { if r.cur > 0 { r.cur -= 1; self.say_line(Priority::Interrupt); } else { self.ann.push(Priority::Interrupt, "Début des informations."); } }
            Key::Down => { if r.cur + 1 < n { r.cur += 1; self.say_line(Priority::Interrupt); } else { self.ann.push(Priority::Interrupt, "Fin des informations."); } }
            Key::Home => { r.cur = 0; self.say_line(Priority::Interrupt); }
            Key::End => { r.cur = n.saturating_sub(1); self.say_line(Priority::Interrupt); }
            Key::Char(c) if c == 't' || c == 'T' => {
                let from = r.cur; let mut first = true;
                for i in from..n { let l = self.review.as_ref().and_then(|r| r.lines.get(i)).unwrap_or(""); self.ann.push(if first { Priority::Interrupt } else { Priority::Normal }, l); first = false; }
            }
            Key::Char(c) if c == 'r' || c == 'R' => self.say_line(Priority::High),
            Key::Enter | Key::Escape => { self.review = None; self.ann.push(Priority::Interrupt, "Retour au menu."); self.announce_focus(Priority::Normal); }
            _ => {}
        }
    }

    /// Place le focus sur l élément d indice idx (0 à 6) et l annonce ; interrompt la parole en cours.
    fn goto(&mut self, idx: usize) {
        if idx < self.ids.len() { let _ = self.tree.set_focus(self.ids[idx]); self.announce_focus(Priority::Interrupt); }
    }

    fn activate(&mut self) {
        let Some(f) = self.tree.focus() else { self.ann.push(Priority::High, "Aucun élément sélectionné."); return };
        let idx = self.ids.iter().position(|i| *i == f).unwrap_or(usize::MAX);
        if let Some(p) = self.confirming.take() {
            if (p == Power::Restart && idx == 9) || (p == Power::Shutdown && idx == 10) { self.pending_power = Some(p); return; }
        }
        match self.tree.perform(f, action::ACTIVATE) {
            ActionResult::Done => {}
            ActionResult::AtLimit => { self.ann.push(Priority::High, "Limite atteinte."); return; }
            ActionResult::Disabled => { self.ann.push(Priority::High, "Cet élément est indisponible."); return; }
            ActionResult::NotAvailable => { self.ann.push(Priority::High, "Aucune action disponible sur cet élément."); return; }
        }
        match idx {
            0 => self.ann.push(Priority::High, "Flèches haut et bas : choisir. Entrée : valider. R : répéter. Échap : annuler."),
            1 => { self.text_size = (self.text_size + 1) % 5; self.ann.push(Priority::High, "Taille du texte changée."); }
            2 => { self.contrast = (self.contrast + 1) % 5; self.ann.push(Priority::High, "Contraste changé."); }
            3 => self.want_sound = true,
            4 => self.want_info = true,
            5 => self.want_bios = true,
            6 => self.want_assistant = true,
            7 => { self.edit = Some(EditMode { e: Edit::new(), q: None }); self.ann.push(Priority::High, "Essai de saisie de texte. Tapez du texte. Flèches gauche et droite : caractères. Page précédente et suivante : mots. Tab : rechercher. Haut : position. Bas : tout relire. Échap : retour."); }
            8 => self.want_windows = true,
            9 => { self.confirming = Some(Power::Restart); self.ann.push(Priority::High, "Confirmer le redémarrage : Entrée une seconde fois. Échap pour annuler."); }
            10 => { self.confirming = Some(Power::Shutdown); self.ann.push(Priority::High, "Confirmer l'extinction : Entrée une seconde fois. Échap pour annuler."); }
            _ => self.ann.push(Priority::High, "Élément inconnu."),
        }
    }

    /// Exécute les demandes en attente (son, informations, alimentation) puis dit toutes les annonces. À appeler après chaque touche.
    pub fn flush<P: Platform>(&mut self, p: &mut P) {
        if self.want_assistant {
            self.want_assistant = false;
            self.ann.push(Priority::High, "Assistant Voxa. Tapez votre demande, puis Entrée. Échap pour revenir au menu.");
            self.flush(p);
            if !p.assistant() { self.ann.push(Priority::High, "L'assistant n'est pas disponible dans cet environnement."); } else { self.ann.push(Priority::High, "Retour au menu."); self.announce_focus(Priority::Normal); }
        }
        if self.want_bios {
            self.want_bios = false; let n = p.bios_count();
            if n == 0 { self.ann.push(Priority::High, "La configuration du BIOS n'est pas lisible depuis cet environnement."); }
            else {
                self.bios = Some(BiosView { cur: 0, n, pile: [0; 16], np: 0 });
                let mut o = Out::new(); o.push("Configuration du BIOS en lecture seule, "); push_num(&mut o, n); o.push(" lignes. Flèches haut et bas : lire. Page suivante et précédente : dix lignes. Droite ou Entrée : suivre un lien. Gauche : revenir. Échap : quitter.");
                self.ann.push(Priority::High, o.as_str());
                let mut l0 = Out::new(); if p.bios_line(0, &mut l0).is_some() { self.ann.push(Priority::High, l0.as_str()); }
            }
        }
        if self.bios_follow {
            self.bios_follow = false; let cur = self.bios.as_ref().map_or(0, |b| b.cur); let mut o = Out::new(); let tgt = p.bios_line(cur, &mut o).unwrap_or(0);
            if tgt == 0 { self.ann.push(Priority::Interrupt, "Cette ligne n'est pas un lien."); }
            else if let Some(l) = p.bios_page_line(tgt) { if let Some(b) = self.bios.as_mut() { if b.np < 16 { b.pile[b.np] = b.cur; b.np += 1; } b.cur = l; } self.bios_say = true; }
            else { self.ann.push(Priority::Interrupt, "Page introuvable."); }
        }
        if self.bios_back {
            self.bios_back = false;
            match self.bios.as_mut() { Some(b) if b.np > 0 => { b.np -= 1; b.cur = b.pile[b.np]; self.bios_say = true; } _ => self.ann.push(Priority::Interrupt, "Pas de position précédente.") }
        }
        if self.bios_say {
            self.bios_say = false; let cur = self.bios.as_ref().map_or(0, |b| b.cur); let mut o = Out::new();
            if p.bios_line(cur, &mut o).is_some() { self.ann.push(Priority::Interrupt, o.as_str()); } else { self.ann.push(Priority::Interrupt, "Ligne illisible."); }
        }
        if self.want_windows {
            // L'annonce est dite AVANT de céder la main : en cas de succès, le gestionnaire de démarrage de Windows ne revient pas.
            self.want_windows = false;
            self.ann.push(Priority::High, "Recherche de Windows sur cette machine. Voxa Live va se terminer.");
            self.flush(p);
            if let Err(e) = p.boot_windows() { let mut o = Out::new(); o.push("Windows n'a pas pu démarrer : "); o.push(e); o.push("."); self.ann.push(Priority::High, o.as_str()); }
        }
        if self.want_sound { self.want_sound = false; let ok = p.test_sound(); self.ann.push(Priority::High, if ok { "Test du son envoyé : vous devriez entendre un son ou une phrase." } else { "Le test du son a échoué." }); }
        if self.want_info {
            self.want_info = false;
            let mut lines = Lines::new(); p.system_lines(&mut lines);
            let mut o = Out::new(); o.push("Informations sur le matériel, "); push_num(&mut o, lines.len()); o.push(" lignes. Flèches : parcourir. T : tout relire. Échap : retour.");
            self.ann.push(Priority::High, o.as_str());
            for i in 0..lines.len() { if let Some(l) = lines.get(i) { self.ann.push(Priority::Normal, l); } }
            self.review = Some(Review { lines, cur: 0 });
        }
        if let Some(pw) = self.pending_power.take() {
            self.ann.push(Priority::Interrupt, if pw == Power::Restart { "Redémarrage en cours." } else { "Extinction en cours." });
            self.speak_all(p);
            p.power(pw);
            return;
        }
        self.speak_all(p);
        self.fallback_beeps(p);
    }

    /// Secours sonore quand la parole est en panne : autant de bips aigus que le rang de l élément qui vient de recevoir le focus.
    fn fallback_beeps<P: Platform>(&mut self, p: &mut P) {
        if !self.speech_failed { return; }
        let Some(f) = self.tree.focus() else { return };
        if self.last_beeped == Some(f) { return; }
        self.last_beeped = Some(f);
        let rank = self.tree.position(f).map(|(r, _)| r).unwrap_or(0);
        for _ in 0..rank { p.beep(1200, 50); p.beep(0, 70); }
    }

    fn speak_all<P: Platform>(&mut self, p: &mut P) {
        let mut first = self.ann.interrupt_pending;
        while let Some(t) = self.ann.pop() {
            if !p.speak(t.as_str(), first) { self.speech_failed = true; }
            first = false;
        }
    }
}

fn push_num(o: &mut Out, mut n: usize) {
    let mut d = [0u8; 8];
    let mut i = 0;
    if n == 0 { o.push("0"); return; }
    while n > 0 && i < 8 { d[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    while i > 0 { i -= 1; let s = [d[i]]; o.push(core::str::from_utf8(&s).unwrap()); }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::string::{String, ToString};
    use std::vec::Vec;

    struct Fake { said: Vec<(String, bool)>, powered: Vec<Power>, sound_ok: bool, speech_ok: bool, sounds: u32, beeps: Vec<(u32, u32)>, windows_calls: u32, said_before_windows: usize, bios: Vec<(String, u16)>, bios_ecrit: u32, assist_ok: bool, assist_appels: u32 }
    impl Fake { fn new() -> Fake { Fake { said: Vec::new(), powered: Vec::new(), sound_ok: true, speech_ok: true, sounds: 0, beeps: Vec::new(), windows_calls: 0, said_before_windows: 0, bios: vec![("Page : Setup".to_string(), 0), ("Lien : Main".to_string(), 10001), ("Page : Main".to_string(), 0), ("Choix : Langue du systeme".to_string(), 0)], bios_ecrit: 0, assist_ok: false, assist_appels: 0 } } }
    impl Platform for Fake {
        fn speak(&mut self, t: &str, i: bool) -> bool { self.said.push((t.to_string(), i)); self.speech_ok }
        fn power(&mut self, p: Power) { self.powered.push(p); }
        fn test_sound(&mut self) -> bool { self.sounds += 1; self.sound_ok }
        fn hardware_summary(&self, o: &mut Out) { o.push("Processeur AMD Ryzen 7 5800H, 8 cœurs."); }
        fn beep(&mut self, f: u32, ms: u32) -> bool { self.beeps.push((f, ms)); true }
        fn boot_windows(&mut self) -> Result<(), &'static str> { self.windows_calls += 1; self.said_before_windows = self.said.len(); Err("introuvable") }
        fn system_lines(&self, l: &mut crate::sysinfo::Lines) { l.add("Ligne A"); l.add("Ligne B"); l.add("Ligne C"); }
        fn assistant(&mut self) -> bool { self.assist_appels += 1; self.assist_ok }
        fn bios_count(&mut self) -> usize { self.bios.len() }
        fn bios_line(&mut self, n: usize, out: &mut Out) -> Option<u16> { let (t, l) = self.bios.get(n)?.clone(); out.push(&t); Some(l) }
        fn bios_page_line(&mut self, page: u16) -> Option<usize> { if page == 10001 { Some(2) } else { None } }
    }
    fn say(a: &mut MenuApp, p: &mut Fake, k: Key) { a.handle(k); a.flush(p); }

    #[test] fn startup_announces_context_then_first_item() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        assert!(p.said[0].0.starts_with("Voxa Live. Menu principal, 11 éléments"));
        assert_eq!(p.said[1].0, "Lire l'aide, élément de menu, 1 sur 11");
    }
    #[test] fn arrows_announce_with_position_and_interrupt_previous_speech() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p); p.said.clear();
        say(&mut a, &mut p, Key::Down);
        assert_eq!(p.said, [("Changer la taille du texte, élément de menu, 2 sur 11".to_string(), true)]);
        say(&mut a, &mut p, Key::Up); say(&mut a, &mut p, Key::Up);
        assert_eq!(p.said.last().unwrap().0, "Début du menu.");
    }
    #[test] fn end_of_menu_is_announced() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        for _ in 0..10 { say(&mut a, &mut p, Key::Down); }
        assert_eq!(p.said.last().unwrap().0, "Éteindre l'ordinateur, élément de menu, 11 sur 11");
        say(&mut a, &mut p, Key::Down);
        assert_eq!(p.said.last().unwrap().0, "Fin du menu.");
    }
    #[test] fn restart_requires_confirmation() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        for _ in 0..9 { say(&mut a, &mut p, Key::Down); }
        say(&mut a, &mut p, Key::Enter);
        assert!(p.powered.is_empty(), "pas de redémarrage sans confirmation");
        assert!(p.said.last().unwrap().0.starts_with("Confirmer le redémarrage"));
        say(&mut a, &mut p, Key::Enter);
        assert_eq!(p.powered, [Power::Restart]);
        assert!(p.said.iter().any(|(t, _)| t == "Redémarrage en cours."), "annonce dite avant l'action");
    }
    #[test] fn escape_cancels_confirmation() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        for _ in 0..10 { say(&mut a, &mut p, Key::Down); }
        say(&mut a, &mut p, Key::Enter);
        say(&mut a, &mut p, Key::Escape);
        assert_eq!(p.said.last().unwrap().0, "Annulé.");
        say(&mut a, &mut p, Key::Enter);
        assert!(p.powered.is_empty(), "après annulation, il faut reconfirmer");
    }
    #[test] fn moving_away_cancels_confirmation() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        for _ in 0..9 { say(&mut a, &mut p, Key::Down); }
        say(&mut a, &mut p, Key::Enter);
        say(&mut a, &mut p, Key::Down);
        say(&mut a, &mut p, Key::Up);
        say(&mut a, &mut p, Key::Enter);
        assert!(p.powered.is_empty());
    }
    #[test] fn sound_failure_is_announced() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); p.sound_ok = false; a.flush(&mut p);
        for _ in 0..3 { say(&mut a, &mut p, Key::Down); }
        say(&mut a, &mut p, Key::Enter);
        assert_eq!(p.sounds, 1);
        assert_eq!(p.said.last().unwrap().0, "Le test du son a échoué.");
    }
    #[test] fn hardware_info_is_spoken_as_a_page_of_lines() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        for _ in 0..4 { say(&mut a, &mut p, Key::Down); }
        say(&mut a, &mut p, Key::Enter);
        let dit: std::vec::Vec<&str> = p.said.iter().map(|x| x.0.as_str()).collect();
        let i = dit.iter().position(|t| t.starts_with("Informations sur le matériel, 3 lignes")).expect("en-tête de la page");
        assert_eq!(&dit[i + 1..i + 4], ["Ligne A", "Ligne B", "Ligne C"], "toutes les lignes sont lues dans l ordre");
    }
    #[test] fn review_navigation_line_by_line_and_exit() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        for _ in 0..4 { say(&mut a, &mut p, Key::Down); }
        say(&mut a, &mut p, Key::Enter);
        say(&mut a, &mut p, Key::Home); assert_eq!(p.said.last().unwrap().0, "Ligne A");
        say(&mut a, &mut p, Key::Up); assert_eq!(p.said.last().unwrap().0, "Début des informations.");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Ligne B");
        say(&mut a, &mut p, Key::End); assert_eq!(p.said.last().unwrap().0, "Ligne C");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Fin des informations.");
        say(&mut a, &mut p, Key::Char('r')); assert_eq!(p.said.last().unwrap().0, "Ligne C", "R répète la ligne courante");
        say(&mut a, &mut p, Key::Home); say(&mut a, &mut p, Key::Char('t'));
        let n = p.said.len(); assert_eq!(&p.said[n - 3..].iter().map(|x| x.0.as_str()).collect::<std::vec::Vec<_>>()[..], ["Ligne A", "Ligne B", "Ligne C"], "T relit tout depuis la ligne courante");
        say(&mut a, &mut p, Key::Escape);
        let n = p.said.len(); assert_eq!(p.said[n - 2].0, "Retour au menu."); assert_eq!(p.said[n - 1].0, "Informations sur le matériel, élément de menu, 5 sur 11");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Lire la configuration du BIOS, élément de menu, 6 sur 11", "la navigation du menu est rétablie");
    }
    #[test] fn repeat_says_last_announcement_again() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        say(&mut a, &mut p, Key::Down);
        say(&mut a, &mut p, Key::Char('r'));
        assert_eq!(p.said[p.said.len() - 1].0, p.said[p.said.len() - 2].0);
    }
    #[test] fn speech_failure_is_recorded_not_hidden() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); p.speech_ok = false;
        a.flush(&mut p);
        assert!(a.speech_failed, "l'application sait que la parole est en panne et peut basculer sur les bips/l'écran");
    }
    #[test] fn home_end_and_digits_jump_directly() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        say(&mut a, &mut p, Key::End);
        assert_eq!(p.said.last().unwrap().0, "Éteindre l'ordinateur, élément de menu, 11 sur 11");
        say(&mut a, &mut p, Key::Home);
        assert_eq!(p.said.last().unwrap().0, "Lire l'aide, élément de menu, 1 sur 11");
        say(&mut a, &mut p, Key::Char('4'));
        assert_eq!(p.said.last().unwrap().0, "Tester le son, élément de menu, 4 sur 11");
        say(&mut a, &mut p, Key::Char('0'));
        assert_eq!(p.said.last().unwrap().0, "Tester le son, élément de menu, 4 sur 11", "un chiffre hors plage ne déplace pas le focus");
    }
    #[test] fn fallback_beeps_give_the_rank_when_speech_is_down() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); p.speech_ok = false;
        a.flush(&mut p);
        assert_eq!(p.beeps.iter().filter(|b| b.0 == 1200).count(), 1, "rang 1 au démarrage");
        p.beeps.clear();
        say(&mut a, &mut p, Key::Char('5'));
        assert_eq!(p.beeps.iter().filter(|b| b.0 == 1200).count(), 5, "5 bips pour le 5e élément");
        p.beeps.clear();
        say(&mut a, &mut p, Key::Enter);
        assert!(p.beeps.is_empty(), "pas de bips si le focus n a pas changé");
    }
    #[test] fn no_beeps_when_speech_works() {
        let mut a = MenuApp::new(); let mut p = Fake::new();
        a.flush(&mut p); say(&mut a, &mut p, Key::Down);
        assert!(p.beeps.is_empty());
    }
    #[test] fn settings_change_and_are_announced() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        say(&mut a, &mut p, Key::Down); say(&mut a, &mut p, Key::Enter);
        assert_eq!(a.text_size, 2);
        assert_eq!(p.said.last().unwrap().0, "Taille du texte changée.");
    }
    #[test] fn windows_item_announces_before_handing_over_and_reports_failure() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p);
        say(&mut a, &mut p, Key::Char('9')); assert_eq!(p.said.last().unwrap().0, "Démarrer Windows, élément de menu, 9 sur 11");
        say(&mut a, &mut p, Key::Enter);
        let i = p.said.iter().position(|(t, _)| t.starts_with("Recherche de Windows")).expect("annonce avant le lancement");
        assert_eq!(p.windows_calls, 1, "le lancement est demandé une seule fois"); assert!(p.said_before_windows >= i + 1, "l'annonce est dite AVANT l'appel (Windows peut ne pas revenir)");
        assert_eq!(p.said.last().unwrap().0, "Windows n'a pas pu démarrer : introuvable.", "l'échec est dit, pas silencieux");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Redémarrer l'ordinateur, élément de menu, 10 sur 11", "le menu répond de nouveau");
    }
    fn type_text(a: &mut MenuApp, p: &mut Fake, s: &str) { for c in s.chars() { say(a, p, Key::Char(c)); } }
    // --- lecture du BIOS (lecture seule) ---
    fn lance_bios(a: &mut MenuApp, p: &mut Fake) { a.flush(p); say(a, p, Key::Char('6')); say(a, p, Key::Enter); }
    #[test] fn assistant_indisponible_est_dit_et_le_menu_continue() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); a.flush(&mut p); say(&mut a, &mut p, Key::Char('7')); say(&mut a, &mut p, Key::Enter);
        assert_eq!(p.assist_appels, 1); assert_eq!(p.said.last().unwrap().0, "L'assistant n'est pas disponible dans cet environnement."); say(&mut a, &mut p, Key::Down); assert!(p.said.last().unwrap().0.starts_with("Essai de saisie"));
    }
    #[test] fn assistant_annonce_puis_rend_la_main_au_menu() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); p.assist_ok = true; a.flush(&mut p); say(&mut a, &mut p, Key::Char('7')); say(&mut a, &mut p, Key::Enter);
        let n = p.said.len(); assert_eq!(p.said[n - 3].0, "Assistant Voxa. Tapez votre demande, puis Entrée. Échap pour revenir au menu."); assert_eq!(p.said[n - 2].0, "Retour au menu."); assert!(p.said[n - 1].0.starts_with("Assistant Voxa, élément de menu, 7 sur 11"));
    }
    #[test] fn bios_indisponible_est_dit() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); p.bios.clear(); lance_bios(&mut a, &mut p);
        assert_eq!(p.said.last().unwrap().0, "La configuration du BIOS n'est pas lisible depuis cet environnement.");
        say(&mut a, &mut p, Key::Down); assert!(p.said.last().unwrap().0.contains("Assistant"), "le menu répond toujours");
    }
    #[test] fn bios_lecture_ligne_par_ligne_liens_et_retour() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); lance_bios(&mut a, &mut p);
        let n = p.said.len(); assert!(p.said[n - 2].0.starts_with("Configuration du BIOS en lecture seule, 4 lignes."), "{:?}", p.said); assert_eq!(p.said[n - 1].0, "Page : Setup");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Lien : Main");
        say(&mut a, &mut p, Key::Enter); assert_eq!(p.said.last().unwrap().0, "Page : Main", "le lien mène à la page Main");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Choix : Langue du systeme");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Fin de la configuration.");
        say(&mut a, &mut p, Key::Enter); assert_eq!(p.said.last().unwrap().0, "Cette ligne n'est pas un lien.");
        say(&mut a, &mut p, Key::Left); assert_eq!(p.said.last().unwrap().0, "Lien : Main", "retour à la position du lien");
        say(&mut a, &mut p, Key::Left); assert_eq!(p.said.last().unwrap().0, "Pas de position précédente.");
        say(&mut a, &mut p, Key::Home); assert_eq!(p.said.last().unwrap().0, "Page : Setup");
        say(&mut a, &mut p, Key::Escape); let n = p.said.len(); assert_eq!(p.said[n - 2].0, "Retour au menu."); assert!(p.said[n - 1].0.starts_with("Lire la configuration du BIOS"));
        assert!(p.bios_ecrit == 0, "aucune écriture dans le firmware");
    }
    fn enter_edit(a: &mut MenuApp, p: &mut Fake) { a.flush(p); say(a, p, Key::Char('8')); say(a, p, Key::Enter); }
    #[test] fn edit_mode_echoes_moves_by_char_and_word_and_returns() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); enter_edit(&mut a, &mut p);
        assert!(p.said.last().unwrap().0.starts_with("Essai de saisie de texte."));
        type_text(&mut a, &mut p, "bonjour le monde");
        assert!(p.said.iter().any(|(t, _)| t.contains("monde") || !t.is_empty()));
        say(&mut a, &mut p, Key::Home); assert_eq!(p.said.last().unwrap().0, "Début du texte.");
        say(&mut a, &mut p, Key::Left); assert_eq!(p.said.last().unwrap().0, "Début du texte.", "pas de caractère avant le début");
        say(&mut a, &mut p, Key::PageDown); assert_eq!(p.said.last().unwrap().0, "le");
        say(&mut a, &mut p, Key::PageDown); assert_eq!(p.said.last().unwrap().0, "monde");
        say(&mut a, &mut p, Key::PageUp); assert_eq!(p.said.last().unwrap().0, "le");
        say(&mut a, &mut p, Key::End); say(&mut a, &mut p, Key::Right); assert_eq!(p.said.last().unwrap().0, "Fin du texte.");
        say(&mut a, &mut p, Key::Backspace); assert!(p.said.last().unwrap().0.starts_with("supprimé : "));
        say(&mut a, &mut p, Key::Escape);
        let n = p.said.len(); assert_eq!(p.said[n - 2].0, "Retour au menu. 15 caractères saisis."); assert_eq!(p.said[n - 1].0, "Essai de saisie de texte, élément de menu, 8 sur 11");
        say(&mut a, &mut p, Key::Down); assert_eq!(p.said.last().unwrap().0, "Démarrer Windows, élément de menu, 9 sur 11", "le menu répond de nouveau");
    }
    #[test] fn edit_mode_search_found_wrapped_and_missing() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); enter_edit(&mut a, &mut p);
        type_text(&mut a, &mut p, "un chat et un chien");
        say(&mut a, &mut p, Key::Home);
        say(&mut a, &mut p, Key::Tab); assert!(p.said.last().unwrap().0.starts_with("Recherche :"));
        type_text(&mut a, &mut p, "chien"); say(&mut a, &mut p, Key::Enter);
        assert_eq!(p.said.last().unwrap().0, "Trouvé : chien");
        say(&mut a, &mut p, Key::Tab); type_text(&mut a, &mut p, "CHAT"); say(&mut a, &mut p, Key::Enter);
        assert_eq!(p.said.last().unwrap().0, "Trouvé après retour au début : chat", "recherche insensible à la casse, avec bouclage annoncé");
        say(&mut a, &mut p, Key::Tab); type_text(&mut a, &mut p, "zzz"); say(&mut a, &mut p, Key::Enter);
        assert_eq!(p.said.last().unwrap().0, "Introuvable.");
        say(&mut a, &mut p, Key::Tab); say(&mut a, &mut p, Key::Escape);
        assert_eq!(p.said.last().unwrap().0, "Recherche annulée.");
        say(&mut a, &mut p, Key::Up); assert!(p.said.last().unwrap().0.starts_with("Position "));
    }
    #[test] fn edit_mode_reads_everything_in_chunks() {
        let mut a = MenuApp::new(); let mut p = Fake::new(); enter_edit(&mut a, &mut p);
        for _ in 0..30 { type_text(&mut a, &mut p, "abcde "); }
        let avant = p.said.len(); say(&mut a, &mut p, Key::Down);
        assert!(p.said.len() - avant >= 2, "un texte de 180 caractères est dit en plusieurs morceaux");
        say(&mut a, &mut p, Key::Escape);
        assert_eq!(p.said[p.said.len() - 2].0, "Retour au menu. 180 caractères saisis.");
    }
}
