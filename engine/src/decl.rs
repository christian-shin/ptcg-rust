//! The declaration index (batch 8 plan section 3.2): per game, built once at `start` (the card set is fixed), a set of
//! card ids per reader family, a bit per card whose spec declares something of the family. A reader takes the set,
//! keeps the cards in place for their declaration's origin (`passive::locate` over the card location, `State::at`) and
//! orders them by the propagation order (player, zone order, index in the zone, then the rank class: [`ordered`]), so
//! it visits the 0-5 cards that can matter instead of every card with a handler, in every zone.
//!
//! `PTCG_VERIFY_CACHE=1` checks every ordered list against the broadcast (`Game::propagation_order_fresh`).

use crate::game::Game;
use crate::list::*;
use crate::spec::passive::{Modifier, Passive};
use crate::state::{ListRef, State};

/// The effect kinds no card handles when the effect is made (batch 8 C5): their declarations (locks, preventions,
/// permissions, re-flips, the triggers after the event) are read by their routines and by `run::after_event` through
/// the per-kind sets (`kind_listeners`), so `Game::reduce_effect` reaches no card for them (unit test
/// `unhandled_kinds_have_no_handler`).
pub const UNHANDLED_KINDS: crate::effects::KindMask = crate::effects::mask(&[
    crate::effects::k::DEVOLVE,
    crate::effects::k::SWAP,
    crate::effects::k::ATTACH,
    crate::effects::k::MOVE_ENERGY,
    crate::effects::k::MOVE_TOOL,
    crate::effects::k::GAIN_CONDITION,
    crate::effects::k::REMOVE_CONDITION,
    crate::effects::k::HEAL,
    crate::effects::k::COIN_FLIP,
    crate::effects::k::CHANGE_ACTIVE,
    crate::effects::k::PLACE_COUNTERS,
    crate::effects::k::MOVE_COUNTERS_EVENT,
    crate::effects::k::KNOCK_OUT,
    crate::effects::k::LEAVE_PLAY,
    crate::effects::k::TAKE_PRIZES,
    crate::effects::k::APPLY_EFFECT,
    crate::effects::k::DISCARD,
    crate::effects::k::PUT_INTO_HAND,
    crate::effects::k::PUT_INTO_DECK,
    crate::effects::k::DRAW,
    crate::effects::k::RETREAT,
    crate::effects::k::RETREAT_START,
    crate::effects::k::USE_POWER,
    crate::effects::k::BEFORE_DOING_DAMAGE,
    crate::effects::k::BEGIN_TURN,
    // Batch 8 C5b: the end of the turn (the once-per-turn marks are its consequence, its triggers run after it), the
    // Checkup (the Checkup routine reads the "more damage counters" declarations), entering play and evolving (the
    // once-per-turn marks reset in their reducer), using an attack (the first-turn permission is read by its checks).
    crate::effects::k::END_TURN,
    crate::effects::k::BETWEEN_TURNS,
    crate::effects::k::ENTER_PLAY,
    crate::effects::k::EVOLVE,
    crate::effects::k::USE_ATTACK,
    // Batch 9 C7b: Damage (the triggers over it are read after the event, in the attack's E-03 window) and the end of an
    // attack's effects (trigger-only).
    crate::effects::k::DAMAGE,
    crate::effects::k::AFTER_ATTACK_TRIGGERS,
]);

/// The reader families (C3a: the derived facts' modifiers).
pub mod fam {
    /// `HpBonus`, `HpMod`.
    pub const HP: usize = 0;
    /// `TypeOverride`.
    pub const TYPE: usize = 1;
    /// `WeaknessOverride`.
    pub const STATS: usize = 2;
    /// `ProvidesEnergy`, `ProvidesEnergyBoost`.
    pub const PROVIDES: usize = 3;
    /// `AttackCost`.
    pub const ATTACK_COST: usize = 4;
    /// `RetreatCost`.
    pub const RETREAT_COST: usize = 5;
    /// `GrantAttacks`, `BenchAttacks`.
    pub const GRANT_ATTACKS: usize = 6;
    /// `Prevent` with `PreventWhat::ToolEffects` ("Pokémon Tools have no effect": Jamming Tower).
    pub const TOOLS_OFF: usize = 7;
    /// What the state check's table-state step reads: `BenchSize`, `Recover`, `AttachGuard`, and a cause-free `Prevent`
    /// over a Special Condition (the affected Pokémon recovers when it comes into force, id289).
    pub const TABLE_STATE: usize = 8;
    /// `PrizeAdjust` (read by the KnockOut routine).
    pub const PRIZE: usize = 9;
    /// `DamageDealt` at the Deal stage (the Damage routine's step 2).
    pub const DAMAGE_DEALT: usize = 10;
    /// `DamageTaken`, `SurviveOnTen` (the Damage routine's step 5).
    pub const DAMAGE_TAKEN: usize = 11;
    /// `CheckupDamage` (read by the Checkup routine).
    pub const CHECKUP_DAMAGE: usize = 12;
    /// `AttackFlags` with `first_turn` ("if you go first, this Pokémon can attack during your first turn": the UseAttack's
    /// checks read it).
    pub const FIRST_TURN: usize = 13;
    pub const N: usize = 14;
}

/// The family a passive's modifier is read in, if it is one of the index's.
pub const fn modifier_family(m: &Modifier) -> Option<usize> {
    Some(match m {
        Modifier::HpBonus(_) | Modifier::HpMod(_) => fam::HP,
        Modifier::TypeOverride(_) => fam::TYPE,
        Modifier::WeaknessOverride(_) => fam::STATS,
        Modifier::ProvidesEnergy(_) | Modifier::ProvidesEnergyBoost(_) => fam::PROVIDES,
        Modifier::AttackCost(_) => fam::ATTACK_COST,
        Modifier::RetreatCost(_) => fam::RETREAT_COST,
        Modifier::GrantAttacks(_) | Modifier::BenchAttacks(_) => fam::GRANT_ATTACKS,
        Modifier::Prevent(p) if matches!(p.what, crate::spec::passive::PreventWhat::ToolEffects) => fam::TOOLS_OFF,
        Modifier::Prevent(p) if crate::spec::passive::prevention_recovers(p) => fam::TABLE_STATE,
        Modifier::BenchSize(_) | Modifier::Recover(_) | Modifier::AttachGuard(_) => fam::TABLE_STATE,
        Modifier::PrizeAdjust(_) => fam::PRIZE,
        Modifier::DamageDealt(d) if matches!(d.stage, crate::spec::passive::DamageStage::Deal) => fam::DAMAGE_DEALT,
        Modifier::DamageTaken(_) | Modifier::SurviveOnTen(_) => fam::DAMAGE_TAKEN,
        Modifier::CheckupDamage(_) => fam::CHECKUP_DAMAGE,
        Modifier::AttackFlags(a) if matches!(a.first_turn, Some(crate::spec::passive::FirstTurnAttack::ThisPokemon)) => fam::FIRST_TURN,
        _ => return None,
    })
}

/// The most effect kinds a per-kind set is kept for: the kinds some card of the database reacts to (its handler mask:
/// the events its declarations are over, the dispatched effects its program or passives read).
pub const KIND_SLOTS: usize = 96;
const NO_SLOT: u8 = u8::MAX;

/// Effect kind -> its slot in `DeclIndex::kinds` (`NO_SLOT`: no card of the database reacts to it). Read-only after the
/// first use.
fn kind_slots() -> &'static [u8; 256] {
    static T: std::sync::OnceLock<[u8; 256]> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        let mut t = [NO_SLOT; 256];
        let mut n = 0usize;
        for imp in crate::cards::registry::IMPLS.iter() {
            for k in 0..256u32 {
                if imp.mask.has(k) && t[k as usize] == NO_SLOT {
                    assert!(n < KIND_SLOTS, "more than {KIND_SLOTS} effect kinds with card handlers");
                    t[k as usize] = n as u8;
                    n += 1;
                }
            }
        }
        t
    })
}

/// Per game: the cards of each family and of each effect kind (a bit per card id), fixed at `start`.
#[derive(Clone, Copy, Debug)]
pub struct DeclIndex {
    pub sets: [u128; fam::N],
    /// Per effect kind (`kind_slots`): the cards whose handler mask has it, the readers' sources by kind (the triggers
    /// over an event, the locks and preventions over it, the cards a dispatched effect can reach).
    pub kinds: [u128; KIND_SLOTS],
    /// The cards declaring an attacker-side modifier at the Attack stage, the next-turn bonus or Shred: what an `Attack`
    /// effect reaches besides the attacking card's program.
    pub attack_passives: u128,
    /// Per derived fact (`derived::fact`), the state classes its reading depends on in this game (`derived::fact_reads`;
    /// `state::reads::ANY`: anything): what its memo is keyed by.
    pub reads: [u8; crate::derived::fact::N],
    /// The index's identity (a number per build, 0 for none): `Game::copy_from` copies the index only into a game holding
    /// another one (the index is fixed per game; a fork of the same game has it already).
    pub id: u64,
}

/// The next `DeclIndex::id`.
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl DeclIndex {
    pub const fn new() -> DeclIndex {
        DeclIndex { sets: [0; fam::N], kinds: [0; KIND_SLOTS], attack_passives: 0, reads: [crate::state::reads::ANY; crate::derived::fact::N], id: 0 }
    }

    /// The index of the game's cards (`Game::start`, after the decks are in).
    pub fn build(g: &Game) -> DeclIndex {
        use crate::spec::passive::DamageStage;
        let mut ix = DeclIndex::new();
        ix.id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let slots = kind_slots();
        for c in 0..g.st.n_cards {
            for ps in passives(g, c) {
                if let Some(f) = modifier_family(&ps.modifier) {
                    ix.sets[f] |= 1u128 << c;
                }
                let attack_side = match &ps.modifier {
                    Modifier::DamageDealt(d) => d.stage == DamageStage::Attack,
                    Modifier::NextTurnBonus(_) => true,
                    Modifier::AttackFlags(a) => a.shred.is_some(),
                    _ => false,
                };
                if attack_side {
                    ix.attack_passives |= 1u128 << c;
                }
            }
            if let Some(imp) = crate::cards::impl_for(g.st.cards[c as usize].def) {
                for (w, word) in imp.mask.0.iter().enumerate() {
                    let mut b = *word;
                    while b != 0 {
                        let k = w * 64 + b.trailing_zeros() as usize;
                        b &= b - 1;
                        let s = slots[k];
                        if s != NO_SLOT {
                            ix.kinds[s as usize] |= 1u128 << c;
                        }
                    }
                }
            }
        }
        ix.reads = crate::derived::fact_reads(g, &ix.sets);
        ix
    }

    /// The cards whose handler mask has effect kind `kind`.
    #[inline]
    pub fn kind(&self, kind: u32) -> u128 {
        match kind_slots().get(kind as usize) {
            Some(&s) if s != NO_SLOT => self.kinds[s as usize],
            _ => 0,
        }
    }
}

impl Default for DeclIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// The passives of card `c`.
#[inline]
pub fn passives(g: &Game, c: CardId) -> &'static [Passive] {
    passives_st(&g.st, c)
}

#[inline]
fn passives_st(st: &State, c: CardId) -> &'static [Passive] {
    crate::cards::spec_for(st.cards[c as usize].def).map_or(&[], |s| s.passives)
}

/// The cards of family `f`, in any zone, in the propagation order of rank class `class` (for the readers whose
/// declarations check where their card is themselves).
pub fn all(g: &Game, f: usize, class: u16) -> SVec<CardId, 120> {
    all_of(g, g.decls.sets[f], class)
}

/// The cards of `set` (a bit per card id), in any zone of the board, in the propagation order of rank class `class`.
pub fn all_of(g: &Game, set: u128, class: u16) -> SVec<CardId, 120> {
    let mut set = set;
    let mut cards: SVec<CardId, 120> = SVec::new();
    while set != 0 {
        cards.push(set.trailing_zeros() as CardId);
        set &= set - 1;
    }
    if cards.is_empty() {
        return cards;
    }
    ordered(g, cards, class)
}

/// The cards of family `f` with a declaration of the family in place for its origin, in the propagation order of rank
/// class `class` (what the readers visit; a card not in place does nothing for any of them).
pub fn in_place(g: &Game, f: usize, class: u16) -> SVec<CardId, 120> {
    let cards = in_place_cards(&g.st, g.decls.sets[f], f);
    ordered(g, cards, class)
}

/// The cards of `set` (family `f`'s declarers) with a declaration of the family in place for its origin.
#[inline]
fn in_place_cards(st: &State, mut set: u128, f: usize) -> SVec<CardId, 120> {
    let mut cards: SVec<CardId, 120> = SVec::new();
    while set != 0 {
        let c = set.trailing_zeros() as CardId;
        set &= set - 1;
        if passives_st(st, c).iter().any(|ps| modifier_family(&ps.modifier) == Some(f) && crate::spec::passive::locate_st(st, c, ps.origin).is_some()) {
            cards.push(c);
        }
    }
    cards
}

#[inline]
fn in_place_st(st: &State, set: u128, f: usize, class: u16) -> SVec<CardId, 120> {
    ordered_st(st, in_place_cards(st, set, f), class)
}

/// The cards of the game whose spec reacts to effect kind `kind` (its handler mask: the lock sources of an event family
/// are listed under the kinds they forbid, the preventions under the kinds they range over, the triggers under their
/// events), in the propagation order (rank class 0): the readers' sources by kind (batch 8 C5, `DeclIndex::kinds`).
pub fn kind_listeners(g: &Game, kind: u32) -> SVec<CardId, 120> {
    let out = kind_order(g, g.decls.kind(kind), 0);
    if crate::game::verify_cache() && g.kinds_present.has(kind) {
        // The broadcast: every card of the game whose handler mask has the kind, in the propagation order.
        assert_eq!(out.as_slice(), g.propagation_order_slow(0, kind).as_slice(), "decl::kind_listeners differs from the broadcast for kind {kind}");
    }
    out
}

/// The cards of `set` (a bit per card id) in the propagation order of rank class `class`.
#[inline]
pub fn kind_order(g: &Game, set: u128, class: u16) -> SVec<CardId, 120> {
    let out = kind_order_st(&g.st, set, class);
    if crate::game::verify_cache() && set != 0 {
        let slow = g.propagation_order_fresh(class, set);
        assert_eq!(out.as_slice(), slow.as_slice(), "decl::kind_order differs from the propagation order (class {class})");
    }
    out
}

#[inline]
fn kind_order_st(st: &State, mut set: u128, class: u16) -> SVec<CardId, 120> {
    let mut cards: SVec<CardId, 120> = SVec::new();
    while set != 0 {
        cards.push(set.trailing_zeros() as CardId);
        set &= set - 1;
    }
    if cards.len() < 2 {
        // One card: in the order when it is visited at all (in a zone of the board).
        if let Some(&c) = cards.as_slice().first() {
            if prop_key(st, c).is_none() {
                return SVec::new();
            }
        }
        return cards;
    }
    ordered_st(st, cards, class)
}

/// The cards in the propagation order (`Game::propagation_order_fresh`): by player, zone (Stadium, Supporter, the Active
/// Pokémon's cards then its Tools, each Benched Pokémon's cards then Tools, the Prizes in use, hand, deck, discard pile)
/// and index in the zone, then a stable sort by the rank class. A card in the Lost Zone or a staging list isn't visited.
pub fn ordered(g: &Game, cards: SVec<CardId, 120>, class: u16) -> SVec<CardId, 120> {
    let out = ordered_st(&g.st, cards, class);
    if crate::game::verify_cache() {
        let mut set = 0u128;
        for &c in cards.iter() {
            set |= 1u128 << c;
        }
        let slow = g.propagation_order_fresh(class, set);
        assert_eq!(out.as_slice(), slow.as_slice(), "decl::ordered differs from the propagation order (class {class})");
    }
    out
}

#[inline]
fn ordered_st(st: &State, cards: SVec<CardId, 120>, class: u16) -> SVec<CardId, 120> {
    let mut keyed: SVec<(u32, CardId), 120> = SVec::new();
    for &c in cards.iter() {
        if let Some(k) = prop_key(st, c) {
            keyed.push(((crate::game::prop_rank_st(st, class, c) as u32) << 24 | k, c));
        }
    }
    keyed.as_mut_slice().sort_unstable_by_key(|x| x.0);
    let mut out: SVec<CardId, 120> = SVec::new();
    for &(_, c) in keyed.iter() {
        out.push(c);
    }
    out
}

/// The card's place in the zone order: player (bit 20), zone (bits 8..19), index in the zone (bits 0..7). Read from the
/// place record (`State::place`: the list and the index), the zone from the list.
fn prop_key(st: &State, c: CardId) -> Option<u32> {
    let i = st.place(c).1;
    let key = |p: u8, zone: u32, list: &[CardId]| -> Option<u32> {
        if crate::game::verify_cache() {
            assert_eq!(list.get(i as usize), Some(&c), "State::pos answered differently from the list for card {c}");
        }
        Some((p as u32) << 20 | zone << 8 | i as u32)
    };
    let pl = |p: u8| &st.players[p as usize];
    let Some(r) = st.locate(c) else {
        let (p, s) = st.tool_spot(c)?;
        let zone = if pl(p as u8).active == s { 3 } else { 5 + 2 * pl(p as u8).bench.position(&s)? as u32 };
        return key(p as u8, zone, pl(p as u8).slots[s as usize].tools().as_slice());
    };
    match r {
        ListRef::Stadium(p) => key(p, 0, pl(p).stadium().as_slice()),
        ListRef::Supporter(p) => key(p, 1, pl(p).supporter().as_slice()),
        ListRef::Slot(p, s) => {
            let zone = if pl(p).active == s { 2 } else { 4 + 2 * pl(p).bench.position(&s)? as u32 };
            key(p, zone, pl(p).slots[s as usize].cards().as_slice())
        }
        ListRef::Prize(p, i) => key(p, 20 + i as u32, pl(p).prizes()[i as usize].as_slice()),
        ListRef::Hand(p) => key(p, 26, pl(p).hand().as_slice()),
        ListRef::Deck(p) => key(p, 27, pl(p).deck().as_slice()),
        ListRef::Discard(p) => key(p, 28, pl(p).discard().as_slice()),
        ListRef::LostZone(_) | ListRef::SlotEnergies(..) | ListRef::Temp(_) => None,
    }
}

// ---------------------------------------------------------------------------
// The order cache (batch 8 C7c; review 8a should-fix 1): the readers' card lists depend only on where the cards are, so
// they are kept per layout generation (`State::layout_gen`) and computed again only after a card moves. `Game` keeps one
// (`Game::order_cache`, not copied by `copy_from`: another line of play may reach the same generation number), legality's
// `Ctx` its own for the decision, which reads the game's warm lists of the same generation first (`warm`). `PTCG_VERIFY_CACHE=1` checks every list against the propagation order computed fresh
// (`Game::decl_in_place`, `decl_ordered`, `decl_kind_listeners`).

/// One cached list: the family read in place (`IN_PLACE`) or a set of cards (`SET`), the rank class, the cards in order.
#[derive(Clone, Copy)]
struct OrderEntry {
    tag: u8,
    class: u16,
    key: u128,
    cards: SVec<CardId, 120>,
}

const SET: u8 = 0;
const IN_PLACE: u8 = 1;

/// The card lists of one layout generation (at most 16; the oldest is replaced).
#[derive(Clone, Copy)]
pub struct OrderCache {
    gen: u32,
    valid: bool,
    next: u8,
    entries: SVec<OrderEntry, 16>,
}

impl Default for OrderCache {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderCache {
    pub fn new() -> OrderCache {
        OrderCache { gen: 0, valid: false, next: 0, entries: SVec::new() }
    }

    /// Forget every list (a restore from a snapshot).
    #[inline]
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Write a copy of `self` to `dst`, moving only the lists kept (none when it is invalid): a copy of the game
    /// (`snapshot.rs`).
    ///
    /// # Safety
    /// `dst` must be valid for writes and properly aligned.
    #[inline]
    pub unsafe fn copy_live_to(&self, dst: *mut Self) {
        use std::ptr::addr_of_mut as m;
        m!((*dst).gen).write(self.gen);
        m!((*dst).valid).write(self.valid);
        m!((*dst).next).write(self.next);
        if self.valid {
            self.entries.copy_live_to(m!((*dst).entries));
        } else {
            m!((*dst).entries).write(SVec::new());
        }
    }

    /// The bytes [`copy_live_to`](Self::copy_live_to) writes.
    #[inline]
    pub fn live_bytes(&self) -> usize {
        std::mem::size_of::<u32>() + 2 + if self.valid { self.entries.live_bytes() } else { 1 }
    }

    /// The list kept for the layout generation, without touching the cache (`Ctx` reads the game's warm lists).
    #[inline]
    fn peek(&self, layout_gen: u32, tag: u8, key: u128, class: u16) -> Option<SVec<CardId, 120>> {
        if !self.valid || self.gen != layout_gen {
            return None;
        }
        self.entries.iter().find(|e| e.tag == tag && e.key == key && e.class == class).map(|e| e.cards)
    }

    #[inline]
    fn get(&mut self, layout_gen: u32, tag: u8, key: u128, class: u16) -> Option<SVec<CardId, 120>> {
        if !self.valid || self.gen != layout_gen {
            self.valid = true;
            self.gen = layout_gen;
            self.next = 0;
            self.entries.clear();
            return None;
        }
        self.entries.iter().find(|e| e.tag == tag && e.key == key && e.class == class).map(|e| e.cards)
    }

    #[inline]
    fn put(&mut self, tag: u8, key: u128, class: u16, cards: SVec<CardId, 120>) {
        let e = OrderEntry { tag, class, key, cards };
        if self.entries.len() < 16 {
            self.entries.push(e);
        } else {
            self.entries.as_mut_slice()[self.next as usize] = e;
            self.next = (self.next + 1) % 16;
        }
    }
}

/// [`in_place`] through `cache`; `warm` is another cache (the game's, for legality, which reads a `&Game`) whose lists of
/// this layout generation are read when `cache` has none.
#[inline]
pub fn in_place_cached(st: &State, decls: &DeclIndex, cache: &mut OrderCache, warm: Option<&OrderCache>, f: usize, class: u16) -> SVec<CardId, 120> {
    if decls.sets[f] == 0 {
        return SVec::new();
    }
    if let Some(cards) = cache.get(st.layout_gen, IN_PLACE, f as u128, class) {
        return cards;
    }
    if let Some(cards) = warm.and_then(|w| w.peek(st.layout_gen, IN_PLACE, f as u128, class)) {
        cache.put(IN_PLACE, f as u128, class, cards);
        return cards;
    }
    let cards = in_place_st(st, decls.sets[f], f, class);
    cache.put(IN_PLACE, f as u128, class, cards);
    cards
}

/// [`kind_order`] through `cache`: the cards of `set` in the propagation order of rank class `class`.
#[inline]
pub fn ordered_cached(st: &State, cache: &mut OrderCache, warm: Option<&OrderCache>, set: u128, class: u16) -> SVec<CardId, 120> {
    if set & set.wrapping_sub(1) == 0 {
        // No card or one: nothing to sort.
        return kind_order_st(st, set, class);
    }
    if let Some(cards) = cache.get(st.layout_gen, SET, set, class) {
        return cards;
    }
    if let Some(cards) = warm.and_then(|w| w.peek(st.layout_gen, SET, set, class)) {
        cache.put(SET, set, class, cards);
        return cards;
    }
    let cards = kind_order_st(st, set, class);
    cache.put(SET, set, class, cards);
    cards
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::k;
    use crate::spec::passive::{DamageStage, Modifier};
    use crate::spec::trigger::Event;

    /// Every kind some card reacts to has a per-kind set (`kind_slots` fits), and an unhandled kind is in a card's mask
    /// only through declarations the routines and the triggers after the event read: a lock, a prevention, a permission, a
    /// re-flip, an `Event::On` trigger. (Replaces the dispatch index's `listener_table`.)
    #[test]
    fn unhandled_kinds_have_no_handler() {
        let _ = kind_slots();
        for s in crate::cards::registry::SPECS.iter() {
            for ps in s.passives {
                let m = crate::spec::passive::modifier_kinds(&ps.modifier);
                if m.intersects(UNHANDLED_KINDS) {
                    assert!(
                        // (The Prize adjustments are read by the KnockOut routine, `knockout::prize_adjustments`; the "more damage
                        // counters" at Checkup by the Checkup routine, `phase::checkup_damage`.)
                        matches!(ps.modifier, Modifier::BlockUse(_) | Modifier::Prevent(_) | Modifier::Permit(_) | Modifier::Reflip(_) | Modifier::PrizeAdjust(_) | Modifier::CheckupDamage(_)),
                        "{}: a passive handling an unhandled kind",
                        s.class
                    );
                }
            }
            for t in s.triggers {
                if crate::spec::trigger::event_kinds(&t.event).intersects(UNHANDLED_KINDS) {
                    assert!(matches!(t.event, Event::On(_)), "{}: a dispatch trigger over an unhandled kind", s.class);
                }
            }
        }
    }

    /// What an `Attack` effect reaches besides the attacking card's program (`DeclIndex::attack_passives`) is every
    /// passive reacting to the Attack kind (the Attack-stage bonuses, the next-turn bonus, Shred), and a `Power` effect
    /// reaches the Ability locks (`Game::lock_cards`) besides the power's own card.
    #[test]
    fn attack_and_power_handlers_are_listed() {
        for s in crate::cards::registry::SPECS.iter() {
            for ps in s.passives {
                let m = crate::spec::passive::modifier_kinds(&ps.modifier);
                if m.has(k::ATTACK) {
                    let covered = match &ps.modifier {
                        Modifier::DamageDealt(d) => d.stage == DamageStage::Attack,
                        Modifier::NextTurnBonus(_) => true,
                        Modifier::AttackFlags(a) => a.shred.is_some(),
                        _ => false,
                    };
                    assert!(covered, "{}: a passive over Attack effects the index doesn't list", s.class);
                }
                if m.has(k::POWER) {
                    assert!(matches!(ps.modifier, Modifier::AbilityLock(_) | Modifier::ActiveLock(_)), "{}: a passive over Power effects that isn't a lock", s.class);
                }
            }
        }
    }

    /// A `PlayTrainer`, a `UseStadium` and an `AfterAttack` reach only the acting card (the played Trainer, the used Stadium,
    /// the attack's card: `Game::handler_cards`): no other card's passive or dispatch trigger is over them (an `Event::On`
    /// trigger is read after the event, `run::after_event`).
    #[test]
    fn acting_card_kinds_have_no_other_handler() {
        let acting = crate::effects::mask(&[k::PLAY_TRAINER, k::USE_STADIUM, k::AFTER_ATTACK]);
        for s in crate::cards::registry::SPECS.iter() {
            for ps in s.passives {
                if crate::spec::passive::modifier_kinds(&ps.modifier).intersects(acting) {
                    // (The locks, preventions, permissions and re-flips are read by the routines: `derived::event_locked`.)
                    assert!(
                        matches!(ps.modifier, Modifier::BlockUse(_) | Modifier::Prevent(_) | Modifier::Permit(_) | Modifier::Reflip(_)),
                        "{}: a passive over an effect that reaches only its acting card",
                        s.class
                    );
                }
            }
            for t in s.triggers {
                if crate::spec::trigger::event_kinds(&t.event).intersects(acting) {
                    assert!(matches!(t.event, Event::On(_)), "{}: a dispatch trigger over an effect that reaches only its acting card", s.class);
                }
            }
        }
    }

    /// No declaration naming no kind ranges over the turn-structure events (BeginTurn, EndTurn, Checkup) and the end of an
    /// attack (AttackDone): they aren't in the base of a kind-free predicate (`EVENT_KINDS`, `EFFECT_EVENT_KINDS`), and a
    /// trigger reads them only by naming them.
    #[test]
    fn no_kind_free_declaration_ranges_over_the_turn_structure() {
        let turn = crate::effects::mask(&[k::BEGIN_TURN, k::END_TURN, k::BETWEEN_TURNS, k::AFTER_ATTACK_TRIGGERS]);
        assert!(!crate::spec::event::EVENT_KINDS.intersects(turn));
        assert!(!crate::spec::event::EFFECT_EVENT_KINDS.intersects(turn));
        for s in crate::cards::registry::SPECS.iter() {
            for t in s.triggers {
                let Event::On(p) = &t.event;
                if p.effect_kinds().intersects(turn) {
                    assert!(!p.effect_kinds().intersects(crate::spec::event::EVENT_KINDS), "{}: a trigger over a turn-structure event and others", s.class);
                }
            }
        }
    }
}
