//! Silicon Screen Reader : cœur indépendant du système.
//! Contrat : rien ici ne dépend de Windows, d'UI Automation, du cloud ni d'une allocation dynamique.
//! Un adaptateur (UEFI, Voxa OS, Windows…) remplit l'arbre ; le cœur décrit, navigue et met en file les annonces.
//! Statut : testé par des tests unitaires sur l'hôte de développement ; jamais exécuté sur la machine cible.
#![cfg_attr(not(test), no_std)]

pub mod menu;
pub mod sysinfo;
pub mod braille;
pub mod edit;

pub const MAX_NODES: usize = 256;
pub const MAX_TEXT: usize = 64;
pub const MAX_CHANGES: usize = 32;
pub const MAX_QUEUE: usize = 16;
/// Capacité d une annonce (octets UTF-8). Les noms de nœuds restent limités à MAX_TEXT ; une annonce peut décrire plusieurs éléments.
pub const MAX_ANN: usize = 192;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role { Unknown, Window, Menu, MenuItem, Button, CheckBox, Edit, List, ListItem, Table, Cell, Dialog, Label, Status,
    // rôles ajoutés (liste établie par nous, en comparant avec les ~136 rôles du modèle d accessibilité de WebKit et les rôles ARIA courants)
    Link, Heading, Slider, Tab, TabList, Tree, TreeItem, ProgressBar, Alert, Toolbar, ComboBox, RadioButton, Image, Separator, Group, Row, ColumnHeader, Document, Paragraph }

impl Role {
    pub fn name_fr(self) -> &'static str {
        match self {
            Role::Unknown => "élément de type inconnu", Role::Window => "fenêtre", Role::Menu => "menu", Role::MenuItem => "élément de menu",
            Role::Button => "bouton", Role::CheckBox => "case à cocher", Role::Edit => "champ de saisie", Role::List => "liste",
            Role::ListItem => "élément de liste", Role::Table => "tableau", Role::Cell => "cellule", Role::Dialog => "boîte de dialogue",
            Role::Label => "texte", Role::Status => "état",
            Role::Link => "lien", Role::Heading => "titre", Role::Slider => "curseur", Role::Tab => "onglet", Role::TabList => "liste d onglets", Role::Tree => "arborescence",
            Role::TreeItem => "élément d arborescence", Role::ProgressBar => "barre de progression", Role::Alert => "alerte", Role::Toolbar => "barre d outils",
            Role::ComboBox => "liste déroulante", Role::RadioButton => "bouton radio", Role::Image => "image", Role::Separator => "séparateur", Role::Group => "groupe",
            Role::Row => "ligne", Role::ColumnHeader => "en-tête de colonne", Role::Document => "document", Role::Paragraph => "paragraphe",
        }
    }
    /// Rôle qui contient d autres éléments et dont on annonce l entrée et la sortie.
    pub fn is_container(self) -> bool { matches!(self, Role::Menu | Role::List | Role::Table | Role::Tree | Role::TabList | Role::Toolbar | Role::Group | Role::Dialog | Role::Row) }
    /// Un élément qui reçoit le focus de navigation.
    pub fn focusable(self) -> bool { matches!(self, Role::MenuItem | Role::Button | Role::CheckBox | Role::Edit | Role::ListItem | Role::Cell | Role::Link | Role::Slider | Role::Tab | Role::TreeItem | Role::ComboBox | Role::RadioButton) }
}

pub mod state {
    pub const DISABLED: u16 = 1; pub const CHECKED: u16 = 2; pub const SELECTED: u16 = 4; pub const EXPANDED: u16 = 8;
    pub const COLLAPSED: u16 = 16; pub const READONLY: u16 = 32; pub const BUSY: u16 = 64; pub const MIXED: u16 = 128;
}
pub mod action {
    pub const ACTIVATE: u8 = 1; pub const TOGGLE: u8 = 2; pub const EXPAND: u8 = 4; pub const SET_VALUE: u8 = 8; pub const INCREMENT: u8 = 16; pub const DECREMENT: u8 = 32;
}

/// Texte UTF-8 de capacité fixe ; coupe toujours sur une limite de caractère.
#[derive(Clone, Copy)]
pub struct TextN<const N: usize> { buf: [u8; N], len: u8 }
pub type Text = TextN<MAX_TEXT>;
pub type AnnText = TextN<MAX_ANN>;
impl<const N: usize> TextN<N> {
    pub const fn empty() -> TextN<N> { TextN { buf: [0; N], len: 0 } }
    pub fn from(s: &str) -> TextN<N> { let mut t = TextN::<N>::empty(); t.set(s); t }
    pub fn set(&mut self, s: &str) {
        let mut n = 0;
        for (i, c) in s.char_indices() { if i + c.len_utf8() > N || i + c.len_utf8() > 255 { break; } n = i + c.len_utf8(); }
        self.buf[..n].copy_from_slice(&s.as_bytes()[..n]);
        self.len = n as u8;
    }
    pub fn as_str(&self) -> &str { core::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("") }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    /// Écrit un entier décimal (négatif accepté).
    pub fn set_i32(&mut self, v: i32) {
        let mut d = [0u8; 12]; let mut i = 12; let neg = v < 0; let mut n = (v as i64).unsigned_abs();
        if n == 0 { i -= 1; d[i] = b'0'; }
        while n > 0 { i -= 1; d[i] = b'0' + (n % 10) as u8; n /= 10; }
        if neg { i -= 1; d[i] = b'-'; }
        self.set(core::str::from_utf8(&d[i..]).unwrap_or("0"));
    }
}

pub type NodeId = u16;
const NONE: u16 = u16::MAX;

#[derive(Clone, Copy)]
pub struct Node { pub role: Role, pub name: Text, pub value: Text, pub state: u16, pub actions: u8, pub range: Option<(i32, i32, i32, i32)>, parent: u16, first_child: u16, last_child: u16, next: u16 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error { Full, BadId }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeKind { Name, Value, State, Focus }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Change { pub node: NodeId, pub kind: ChangeKind }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionResult { Done, NotAvailable, Disabled, /// Incrément ou décrément refusé : la valeur est déjà à sa limite.
    AtLimit }

pub struct Tree { nodes: [Node; MAX_NODES], count: usize, focus: u16, changes: [Change; MAX_CHANGES], nchg: usize, lost_changes: u32 }

impl Tree {
    pub fn new() -> Tree {
        let n = Node { role: Role::Unknown, name: Text::empty(), value: Text::empty(), state: 0, actions: 0, range: None, parent: NONE, first_child: NONE, last_child: NONE, next: NONE };
        Tree { nodes: [n; MAX_NODES], count: 0, focus: NONE, changes: [Change { node: 0, kind: ChangeKind::Name }; MAX_CHANGES], nchg: 0, lost_changes: 0 }
    }
    pub fn len(&self) -> usize { self.count }
    pub fn node(&self, id: NodeId) -> Option<&Node> { if (id as usize) < self.count { Some(&self.nodes[id as usize]) } else { None } }

    /// Ajoute un nœud sous `parent` (None = racine). Erreur explicite si l'arbre est plein ou si le parent n'existe pas.
    pub fn add(&mut self, parent: Option<NodeId>, role: Role, name: &str, actions: u8) -> Result<NodeId, Error> {
        if let Some(p) = parent { if (p as usize) >= self.count { return Err(Error::BadId); } }
        if self.count >= MAX_NODES { return Err(Error::Full); }
        let id = self.count as u16;
        self.nodes[id as usize] = Node { role, name: Text::from(name), value: Text::empty(), state: 0, actions, range: None, parent: parent.unwrap_or(NONE), first_child: NONE, last_child: NONE, next: NONE };
        self.count += 1;
        if let Some(p) = parent {
            let last = self.nodes[p as usize].last_child;
            if last == NONE { self.nodes[p as usize].first_child = id; } else { self.nodes[last as usize].next = id; }
            self.nodes[p as usize].last_child = id;
        }
        Ok(id)
    }

    fn record(&mut self, node: NodeId, kind: ChangeKind) {
        if self.nchg < MAX_CHANGES { self.changes[self.nchg] = Change { node, kind }; self.nchg += 1; } else { self.lost_changes += 1; }
    }
    fn slot(&mut self, id: NodeId) -> Result<&mut Node, Error> {
        if (id as usize) < self.count { Ok(&mut self.nodes[id as usize]) } else { Err(Error::BadId) }
    }
    pub fn set_value(&mut self, id: NodeId, v: &str) -> Result<(), Error> { self.slot(id)?.value.set(v); self.record(id, ChangeKind::Value); Ok(()) }
    /// Déclare un élément à valeur numérique bornée (minimum, maximum, pas, valeur) ; le texte de la valeur est tenu à jour.
    pub fn set_range(&mut self, id: NodeId, min: i32, max: i32, step: i32, cur: i32) -> Result<(), Error> {
        if min > max || step <= 0 { return Err(Error::BadId); }
        let cur = cur.clamp(min, max);
        let n = self.slot(id)?; n.range = Some((min, max, step, cur)); n.value.set_i32(cur);
        self.record(id, ChangeKind::Value); Ok(())
    }
    pub fn set_name(&mut self, id: NodeId, v: &str) -> Result<(), Error> { self.slot(id)?.name.set(v); self.record(id, ChangeKind::Name); Ok(()) }
    pub fn set_state(&mut self, id: NodeId, flag: u16, on: bool) -> Result<(), Error> {
        let n = self.slot(id)?;
        let before = n.state;
        if on { n.state |= flag } else { n.state &= !flag }
        let changed = n.state != before;
        if changed { self.record(id, ChangeKind::State); }
        Ok(())
    }
    /// Vide la liste et renvoie (changements copiés, changements perdus). Les pertes sont comptées pour être annoncées, jamais ignorées en silence.
    pub fn drain_changes(&mut self, out: &mut [Change]) -> (usize, u32) {
        let n = self.nchg.min(out.len());
        out[..n].copy_from_slice(&self.changes[..n]);
        let lost = self.lost_changes + (self.nchg - n) as u32;
        self.nchg = 0;
        self.lost_changes = 0;
        (n, lost)
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> { let n = self.node(id)?; if n.parent == NONE { None } else { Some(n.parent) } }
    /// Nombre d enfants directs du nœud.
    pub fn child_count(&self, id: NodeId) -> usize {
        let Some(n) = self.node(id) else { return 0 }; let (mut c, mut k) = (n.first_child, 0);
        while c != NONE { k += 1; c = self.nodes[c as usize].next; } k
    }
    /// Conteneurs quittés et conteneurs franchis quand le focus passe de `from` à `to` (modèle de Fuchsia : on annonce les changements de contexte).
    /// `exited` : du plus profond au plus haut ; `entered` : du plus haut au plus profond. Seuls les rôles de conteneur sont retenus. Renvoie (nb_sortis, nb_entrés).
    pub fn container_change(&self, from: Option<NodeId>, to: NodeId, exited: &mut [NodeId; 6], entered: &mut [NodeId; 6]) -> (usize, usize) {
        let chain = |start: Option<NodeId>| -> ([NodeId; 12], usize) { let mut a = [0u16; 12]; let mut n = 0; let mut cur = start.and_then(|s| self.parent(s)); while let Some(p) = cur { if n < 12 { a[n] = p; n += 1; } cur = self.parent(p); } (a, n) };
        let (fa, fnb) = chain(from); let (ta, tnb) = chain(Some(to));
        let is_cont = |id: NodeId| self.node(id).map_or(false, |n| n.role.is_container());
        // ancêtres communs : suffixe identique des deux chaînes (de la racine vers le bas)
        let mut common = 0; while common < fnb && common < tnb && fa[fnb - 1 - common] == ta[tnb - 1 - common] { common += 1; }
        let (mut ne, mut nn) = (0, 0);
        for i in 0..fnb - common { if is_cont(fa[i]) && ne < 6 { exited[ne] = fa[i]; ne += 1; } }
        for i in (0..tnb - common).rev() { if is_cont(ta[i]) && nn < 6 { entered[nn] = ta[i]; nn += 1; } }
        (ne, nn)
    }

    /// Rang (à partir de 1) et nombre d éléments focalisables parmi les frères du nœud, pour annoncer « 2 sur 7 ».
    pub fn position(&self, id: NodeId) -> Option<(usize, usize)> {
        let n = self.node(id)?;
        if n.parent == NONE { return Some((1, 1)); }
        let (mut rank, mut total) = (0, 0);
        let mut c = self.nodes[n.parent as usize].first_child;
        while c != NONE {
            if self.nodes[c as usize].role.focusable() { total += 1; if c == id { rank = total; } }
            c = self.nodes[c as usize].next;
        }
        if rank == 0 { None } else { Some((rank, total)) }
    }

    // ---- navigation en profondeur d'abord ----
    fn next_in_order(&self, id: u16) -> u16 {
        let n = &self.nodes[id as usize];
        if n.first_child != NONE { return n.first_child; }
        let mut cur = id;
        loop {
            let c = &self.nodes[cur as usize];
            if c.next != NONE { return c.next; }
            if c.parent == NONE { return NONE; }
            cur = c.parent;
        }
    }
    /// Le précédent est cherché par parcours depuis le début : l'arbre est petit et borné (MAX_NODES), ce qui évite un lien « précédent » à maintenir.
    fn prev_in_order(&self, id: u16) -> u16 {
        if self.count == 0 { return NONE; }
        let mut prev = NONE;
        let mut cur = 0u16;
        while cur != NONE && cur != id { prev = cur; cur = self.next_in_order(cur); }
        if cur == id { prev } else { NONE }
    }
    pub fn focus(&self) -> Option<NodeId> { if self.focus == NONE { None } else { Some(self.focus) } }
    pub fn set_focus(&mut self, id: NodeId) -> Result<(), Error> {
        if (id as usize) >= self.count { return Err(Error::BadId); }
        self.focus = id;
        self.record(id, ChangeKind::Focus);
        Ok(())
    }
    fn step(&mut self, forward: bool) -> Option<NodeId> {
        if self.count == 0 { return None; }
        let mut cur = self.focus;
        let mut guard = 0;
        loop {
            cur = if cur == NONE { if forward { 0 } else { return None } } else if forward { self.next_in_order(cur) } else { self.prev_in_order(cur) };
            if cur == NONE { return None; }
            let n = &self.nodes[cur as usize];
            if n.role.focusable() && n.state & state::DISABLED == 0 {
                self.focus = cur;
                self.record(cur, ChangeKind::Focus);
                return Some(cur);
            }
            guard += 1;
            if guard > MAX_NODES { return None; }
        }
    }
    /// Passe à l'élément activable suivant ; None (focus inchangé) en fin de parcours : l'appelant doit l'annoncer.
    pub fn focus_next(&mut self) -> Option<NodeId> { self.step(true) }
    pub fn focus_prev(&mut self) -> Option<NodeId> { self.step(false) }

    /// Exécute une action ; le résultat est toujours renvoyé (jamais d'échec silencieux).
    pub fn perform(&mut self, id: NodeId, act: u8) -> ActionResult {
        let Some(n) = self.node(id) else { return ActionResult::NotAvailable };
        if n.state & state::DISABLED != 0 { return ActionResult::Disabled; }
        if n.actions & act == 0 { return ActionResult::NotAvailable; }
        if act == action::INCREMENT || act == action::DECREMENT {
            let Some((min, max, step, cur)) = n.range else { return ActionResult::NotAvailable };
            let nv = if act == action::INCREMENT { cur.saturating_add(step).min(max) } else { cur.saturating_sub(step).max(min) };
            if nv == cur { return ActionResult::AtLimit; }
            let nn = self.nodes.get_mut(id as usize).unwrap(); nn.range = Some((min, max, step, nv)); nn.value.set_i32(nv);
            self.record(id, ChangeKind::Value); return ActionResult::Done;
        }
        let (checked, expanded) = (n.state & state::CHECKED != 0, n.state & state::EXPANDED != 0);
        if act == action::TOGGLE { let _ = self.set_state(id, state::CHECKED, !checked); }
        if act == action::EXPAND { let _ = self.set_state(id, state::EXPANDED, !expanded); let _ = self.set_state(id, state::COLLAPSED, expanded); }
        ActionResult::Done
    }

    /// Phrase française décrivant un nœud. Les informations absentes sont dites explicitement, jamais inventées.
    pub fn describe(&self, id: NodeId, out: &mut Out) -> bool {
        let Some(n) = self.node(id) else { return false };
        out.clear();
        if n.name.is_empty() { out.push("sans nom"); } else { out.push(n.name.as_str()); }
        out.push(", ");
        out.push(n.role.name_fr());
        if n.state & state::DISABLED != 0 { out.push(", indisponible"); }
        if n.state & state::MIXED != 0 { out.push(", partiellement coché"); } else if n.state & state::CHECKED != 0 { out.push(", coché"); } else if n.actions & action::TOGGLE != 0 { out.push(", non coché"); }
        if n.state & state::SELECTED != 0 { out.push(", sélectionné"); }
        if n.state & state::EXPANDED != 0 { out.push(", développé"); }
        if n.state & state::COLLAPSED != 0 { out.push(", réduit"); }
        if n.state & state::READONLY != 0 { out.push(", lecture seule"); }
        if n.state & state::BUSY != 0 { out.push(", occupé"); }
        if !n.value.is_empty() { out.push(", valeur "); out.push(n.value.as_str()); }
        true
    }
}

/// Tampon de sortie de capacité fixe ; signale la troncature au lieu de la cacher.
pub struct Out { buf: [u8; 192], len: usize, pub truncated: bool }
impl Out {
    pub const fn new() -> Out { Out { buf: [0; 192], len: 0, truncated: false } }
    pub fn clear(&mut self) { self.len = 0; self.truncated = false; }
    pub fn push(&mut self, s: &str) {
        for c in s.chars() {
            let l = c.len_utf8();
            if self.len + l > self.buf.len() { self.truncated = true; return; }
            c.encode_utf8(&mut self.buf[self.len..]);
            self.len += l;
        }
    }
    pub fn as_str(&self) -> &str { core::str::from_utf8(&self.buf[..self.len]).unwrap_or("") }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Priority { Low = 0, Normal = 1, High = 2, Interrupt = 3 }

/// File d'annonces : priorité, interruption, répétition de la dernière annonce.
pub struct Announcer { q: [(Priority, AnnText); MAX_QUEUE], n: usize, last: AnnText, pub interrupt_pending: bool, pub dropped: u32 }
impl Announcer {
    pub fn new() -> Announcer { Announcer { q: [(Priority::Low, AnnText::empty()); MAX_QUEUE], n: 0, last: AnnText::empty(), interrupt_pending: false, dropped: 0 } }
    pub fn push(&mut self, p: Priority, s: &str) {
        if p == Priority::Interrupt {
            // l interruption écarte les annonces ordinaires en attente mais JAMAIS les annonces importantes (erreurs, annulations, confirmations)
            let mut k = 0;
            for i in 0..self.n { if self.q[i].0 >= Priority::High { self.q[k] = self.q[i]; k += 1; } }
            self.n = k;
            self.interrupt_pending = true;
        } else if self.n == MAX_QUEUE {
            // file pleine : on écarte l'annonce la plus ancienne de priorité la plus basse, seulement si la nouvelle vaut davantage
            let mut worst = 0;
            for i in 1..self.n { if self.q[i].0 < self.q[worst].0 { worst = i; } }
            self.dropped += 1;
            if self.q[worst].0 >= p { return; }
            for i in worst..self.n - 1 { self.q[i] = self.q[i + 1]; }
            self.n -= 1;
        }
        self.q[self.n] = (p, AnnText::from(s));
        self.n += 1;
    }
    /// Prochaine annonce : la plus prioritaire, à égalité la plus ancienne.
    pub fn pop(&mut self) -> Option<AnnText> {
        if self.n == 0 { return None; }
        let mut best = 0;
        for i in 1..self.n { if self.q[i].0 > self.q[best].0 { best = i; } }
        let t = self.q[best].1;
        for i in best..self.n - 1 { self.q[i] = self.q[i + 1]; }
        self.n -= 1;
        self.last = t;
        self.interrupt_pending = false;
        Some(t)
    }
    pub fn repeat_last(&self) -> Option<AnnText> { if self.last.is_empty() { None } else { Some(self.last) } }
    pub fn pending(&self) -> usize { self.n }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> (Tree, [NodeId; 7]) {
        let mut t = Tree::new();
        let w = t.add(None, Role::Window, "Voxa Live", 0).unwrap();
        let m = t.add(Some(w), Role::Menu, "Menu principal", 0).unwrap();
        let names = ["Lire l'aide", "Changer la taille du texte", "Changer le contraste", "Tester le son", "Informations sur le matériel", "Redémarrer l'ordinateur", "Éteindre l'ordinateur"];
        let mut ids = [0; 7];
        for (i, n) in names.iter().enumerate() { ids[i] = t.add(Some(m), Role::MenuItem, n, action::ACTIVATE).unwrap(); }
        (t, ids)
    }

    #[test] fn navigation_forward_and_backward() {
        let (mut t, ids) = menu();
        assert_eq!(t.focus(), None);
        assert_eq!(t.focus_next(), Some(ids[0]));
        assert_eq!(t.focus_next(), Some(ids[1]));
        assert_eq!(t.focus_prev(), Some(ids[0]));
        assert_eq!(t.focus_prev(), None, "début de liste : pas de focus fantôme");
        assert_eq!(t.focus(), Some(ids[0]), "le focus reste en place quand on ne peut pas reculer");
    }
    #[test] fn end_of_list_is_reported() {
        let (mut t, ids) = menu();
        t.set_focus(ids[6]).unwrap();
        assert_eq!(t.focus_next(), None);
        assert_eq!(t.focus(), Some(ids[6]));
    }
    #[test] fn disabled_items_are_skipped() {
        let (mut t, ids) = menu();
        t.set_state(ids[1], state::DISABLED, true).unwrap();
        t.set_focus(ids[0]).unwrap();
        assert_eq!(t.focus_next(), Some(ids[2]));
    }
    #[test] fn describe_states_in_french() {
        let mut t = Tree::new();
        let c = t.add(None, Role::CheckBox, "Mode sombre", action::TOGGLE).unwrap();
        let mut o = Out::new();
        t.describe(c, &mut o);
        assert_eq!(o.as_str(), "Mode sombre, case à cocher, non coché");
        assert_eq!(t.perform(c, action::TOGGLE), ActionResult::Done);
        t.describe(c, &mut o);
        assert_eq!(o.as_str(), "Mode sombre, case à cocher, coché");
    }
    #[test] fn extended_roles_are_named_in_french_and_focusable_when_interactive() {
        let mut t = Tree::new();
        let w = t.add(None, Role::Window, "Fenêtre", 0).unwrap();
        let l = t.add(Some(w), Role::Link, "Accueil", action::ACTIVATE).unwrap();
        let h = t.add(Some(w), Role::Heading, "Titre", 0).unwrap();
        let s = t.add(Some(w), Role::Slider, "Volume", action::SET_VALUE).unwrap();
        t.set_value(s, "40 %").unwrap();
        let mut o = Out::new();
        t.describe(l, &mut o); assert_eq!(o.as_str(), "Accueil, lien");
        t.describe(s, &mut o); assert_eq!(o.as_str(), "Volume, curseur, valeur 40 %");
        assert_eq!(t.focus_next(), Some(l), "le lien reçoit le focus");
        assert_eq!(t.focus_next(), Some(s), "le titre (non interactif) est sauté");
        assert!(!Role::Heading.focusable() && Role::Slider.focusable());
        let _ = h;
    }
    #[test] fn range_slider_increment_decrement_with_limits() {
        let mut t = Tree::new();
        let s = t.add(None, Role::Slider, "Volume", action::INCREMENT | action::DECREMENT).unwrap();
        t.set_range(s, 0, 100, 25, 50).unwrap();
        let mut o = Out::new();
        t.describe(s, &mut o); assert_eq!(o.as_str(), "Volume, curseur, valeur 50");
        assert_eq!(t.perform(s, action::INCREMENT), ActionResult::Done); assert_eq!(t.node(s).unwrap().value.as_str(), "75");
        assert_eq!(t.perform(s, action::INCREMENT), ActionResult::Done); assert_eq!(t.node(s).unwrap().value.as_str(), "100");
        assert_eq!(t.perform(s, action::INCREMENT), ActionResult::AtLimit, "limite atteinte : signalée, jamais ignorée");
        for _ in 0..4 { t.perform(s, action::DECREMENT); }
        assert_eq!(t.node(s).unwrap().value.as_str(), "0");
        assert_eq!(t.perform(s, action::DECREMENT), ActionResult::AtLimit);
        let mut buf = [Change { node: 0, kind: ChangeKind::Name }; 16]; let (n, _) = t.drain_changes(&mut buf); assert!(n >= 6, "chaque changement de valeur est suivi");
        assert!(t.set_range(s, 5, 1, 1, 0).is_err() && t.set_range(s, 0, 9, 0, 0).is_err(), "bornes incohérentes refusées");
        let b = t.add(None, Role::Button, "OK", action::ACTIVATE).unwrap();
        assert_eq!(t.perform(b, action::INCREMENT), ActionResult::NotAvailable);
    }
    #[test] fn mixed_state_is_announced() {
        let mut t = Tree::new();
        let c = t.add(None, Role::CheckBox, "Tout sélectionner", action::TOGGLE).unwrap();
        t.set_state(c, state::MIXED, true).unwrap();
        let mut o = Out::new(); t.describe(c, &mut o);
        assert_eq!(o.as_str(), "Tout sélectionner, case à cocher, partiellement coché");
    }
    #[test] fn container_changes_are_computed_like_fuchsia() {
        let mut t = Tree::new();
        let w = t.add(None, Role::Window, "Fenêtre", 0).unwrap();
        let l1 = t.add(Some(w), Role::List, "Fichiers", 0).unwrap();
        let a = t.add(Some(l1), Role::ListItem, "a.txt", 0).unwrap();
        let b = t.add(Some(l1), Role::ListItem, "b.txt", 0).unwrap();
        let g = t.add(Some(w), Role::Group, "Actions", 0).unwrap();
        let ok = t.add(Some(g), Role::Button, "OK", 0).unwrap();
        let (mut ex, mut en) = ([0u16; 6], [0u16; 6]);
        assert_eq!(t.container_change(None, a, &mut ex, &mut en), (0, 1)); assert_eq!(en[0], l1, "premier focus : on entre dans la liste");
        assert_eq!(t.container_change(Some(a), b, &mut ex, &mut en), (0, 0), "même liste : rien à annoncer");
        assert_eq!(t.container_change(Some(b), ok, &mut ex, &mut en), (1, 1)); assert_eq!((ex[0], en[0]), (l1, g), "on quitte la liste et on entre dans le groupe");
        assert_eq!(t.child_count(l1), 2); assert_eq!(t.child_count(a), 0); assert_eq!(t.parent(a), Some(l1)); assert_eq!(t.parent(w), None);
    }
    #[test] fn missing_information_is_explicit() {
        let mut t = Tree::new();
        let x = t.add(None, Role::Unknown, "", 0).unwrap();
        let mut o = Out::new();
        t.describe(x, &mut o);
        assert_eq!(o.as_str(), "sans nom, élément de type inconnu");
    }
    #[test] fn actions_report_their_result() {
        let (mut t, ids) = menu();
        assert_eq!(t.perform(ids[0], action::ACTIVATE), ActionResult::Done);
        assert_eq!(t.perform(ids[0], action::TOGGLE), ActionResult::NotAvailable);
        t.set_state(ids[0], state::DISABLED, true).unwrap();
        assert_eq!(t.perform(ids[0], action::ACTIVATE), ActionResult::Disabled);
        assert_eq!(t.perform(999, action::ACTIVATE), ActionResult::NotAvailable);
    }
    #[test] fn changes_are_tracked_and_losses_reported() {
        let (mut t, ids) = menu();
        t.set_value(ids[0], "x").unwrap();
        t.set_state(ids[0], state::SELECTED, true).unwrap();
        t.set_state(ids[0], state::SELECTED, true).unwrap(); // pas de changement réel : non enregistré
        let mut buf = [Change { node: 0, kind: ChangeKind::Name }; 8];
        let (n, lost) = t.drain_changes(&mut buf);
        assert_eq!((n, lost), (2, 0));
        assert_eq!(buf[0], Change { node: ids[0], kind: ChangeKind::Value });
        for _ in 0..40 { t.set_value(ids[1], "y").unwrap(); }
        let (n, lost) = t.drain_changes(&mut buf);
        assert_eq!(n, 8);
        assert_eq!(lost, 32, "les changements perdus sont comptés, pas ignorés");
    }
    #[test] fn tree_full_and_bad_parent_are_errors() {
        let mut t = Tree::new();
        for _ in 0..MAX_NODES { t.add(None, Role::Label, "a", 0).unwrap(); }
        assert_eq!(t.add(None, Role::Label, "a", 0), Err(Error::Full));
        assert_eq!(t.add(Some(5000), Role::Label, "a", 0), Err(Error::BadId));
    }
    #[test] fn text_truncation_is_char_safe() {
        let long = "é".repeat(100);
        let t = Text::from(&long);
        assert!(t.as_str().chars().all(|c| c == 'é'));
        assert!(t.as_str().len() <= MAX_TEXT);
    }
    #[test] fn announcer_priority_and_interrupt() {
        let mut a = Announcer::new();
        a.push(Priority::Low, "bas");
        a.push(Priority::High, "haut");
        a.push(Priority::Normal, "normal");
        assert_eq!(a.pop().unwrap().as_str(), "haut");
        assert_eq!(a.pop().unwrap().as_str(), "normal");
        a.push(Priority::Normal, "n2");
        a.push(Priority::Interrupt, "urgent");
        assert!(a.interrupt_pending);
        assert_eq!(a.pending(), 1, "l interruption écarte les annonces ordinaires puis s y place");
        assert_eq!(a.pop().unwrap().as_str(), "urgent");
        assert_eq!(a.repeat_last().unwrap().as_str(), "urgent");
        assert!(a.pop().is_none());
    }
    #[test] fn long_announcements_are_not_truncated() {
        let long = "Confirmer l'extinction : Entrée une seconde fois. Échap pour annuler. Toute autre touche annule aussi la demande en cours.";
        assert!(long.len() > MAX_TEXT);
        let mut a = Announcer::new();
        a.push(Priority::Normal, long);
        assert_eq!(a.pop().unwrap().as_str(), long);
    }
    #[test] fn interrupt_keeps_important_announcements() {
        let mut a = Announcer::new();
        a.push(Priority::Normal, "ordinaire");
        a.push(Priority::High, "Annulé.");
        a.push(Priority::Interrupt, "Redémarrer, 6 sur 7");
        assert_eq!(a.pop().unwrap().as_str(), "Redémarrer, 6 sur 7");
        assert_eq!(a.pop().unwrap().as_str(), "Annulé.", "l annonce importante survit à l interruption");
        assert!(a.pop().is_none(), "l annonce ordinaire a bien été écartée");
    }
    #[test] fn announcer_full_queue_counts_drops() {
        let mut a = Announcer::new();
        for _ in 0..MAX_QUEUE { a.push(Priority::Normal, "x"); }
        a.push(Priority::Low, "perdu");
        assert_eq!(a.dropped, 1);
        a.push(Priority::High, "important");
        assert_eq!(a.dropped, 2);
        assert_eq!(a.pop().unwrap().as_str(), "important");
    }
}
