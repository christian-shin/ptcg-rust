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
use crate::state::ListRef;

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
    /// `PrizeAdjust`, `PrizeAdjustOnce` (read by the KnockOut routine).
    pub const PRIZE: usize = 9;
    /// `DamageDealt` at the Deal stage (the Damage routine's step 2).
    pub const DAMAGE_DEALT: usize = 10;
    /// `DamageTaken`, `SurviveOnTen` (the Damage routine's step 5).
    pub const DAMAGE_TAKEN: usize = 11;
    pub const N: usize = 12;
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
        Modifier::PrizeAdjust(_) | Modifier::PrizeAdjustOnce(_) => fam::PRIZE,
        Modifier::DamageDealt(d) if matches!(d.stage, crate::spec::passive::DamageStage::Deal) => fam::DAMAGE_DEALT,
        Modifier::DamageTaken(_) | Modifier::SurviveOnTen(_) => fam::DAMAGE_TAKEN,
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
}

impl DeclIndex {
    pub const fn new() -> DeclIndex {
        DeclIndex { sets: [0; fam::N], kinds: [0; KIND_SLOTS], attack_passives: 0 }
    }

    /// The index of the game's cards (`Game::start`, after the decks are in).
    pub fn build(g: &Game) -> DeclIndex {
        use crate::spec::passive::DamageStage;
        let mut ix = DeclIndex::new();
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
    crate::cards::spec_for(g.st.cards[c as usize].def).map_or(&[], |s| s.passives)
}

/// The cards of family `f`, in any zone, in the propagation order of rank class `class` (for the readers whose
/// declarations check where their card is themselves).
pub fn all(g: &Game, f: usize, class: u16) -> SVec<CardId, 120> {
    let mut set = g.decls.sets[f];
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
    let mut set = g.decls.sets[f];
    let mut cards: SVec<CardId, 120> = SVec::new();
    while set != 0 {
        let c = set.trailing_zeros() as CardId;
        set &= set - 1;
        if passives(g, c).iter().any(|ps| modifier_family(&ps.modifier) == Some(f) && crate::spec::passive::locate(g, c, ps.origin).is_some()) {
            cards.push(c);
        }
    }
    ordered(g, cards, class)
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
pub fn kind_order(g: &Game, mut set: u128, class: u16) -> SVec<CardId, 120> {
    let mut cards: SVec<CardId, 120> = SVec::new();
    while set != 0 {
        cards.push(set.trailing_zeros() as CardId);
        set &= set - 1;
    }
    if cards.len() < 2 {
        // One card: in the order when it is visited at all (in a zone of the board).
        if let Some(&c) = cards.as_slice().first() {
            if prop_key(g, c).is_none() {
                return SVec::new();
            }
        }
        if crate::game::verify_cache() && !cards.is_empty() {
            return ordered(g, cards, class);
        }
        return cards;
    }
    ordered(g, cards, class)
}

/// The cards in the propagation order (`Game::propagation_order_fresh`): by player, zone (Stadium, Supporter, the Active
/// Pokémon's cards then its Tools, each Benched Pokémon's cards then Tools, the Prizes in use, hand, deck, discard pile)
/// and index in the zone, then a stable sort by the rank class. A card in the Lost Zone or a staging list isn't visited.
pub fn ordered(g: &Game, cards: SVec<CardId, 120>, class: u16) -> SVec<CardId, 120> {
    let mut keyed: SVec<(u32, CardId), 120> = SVec::new();
    for &c in cards.iter() {
        if let Some(k) = prop_key(g, c) {
            keyed.push(((crate::game::prop_rank(g, class, c) as u32) << 24 | k, c));
        }
    }
    keyed.as_mut_slice().sort_unstable_by_key(|x| x.0);
    let mut out: SVec<CardId, 120> = SVec::new();
    for &(_, c) in keyed.iter() {
        out.push(c);
    }
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

/// The card's place in the zone order: player (bit 20), zone (bits 8..19), index in the zone (bits 0..7).
fn prop_key(g: &Game, c: CardId) -> Option<u32> {
    let key = |p: u8, zone: u32, list: &[CardId]| -> Option<u32> {
        let i = list.iter().position(|x| *x == c)? as u32;
        Some((p as u32) << 20 | zone << 8 | i)
    };
    if let Some((p, s)) = g.st.tool_spot(c) {
        let pl = &g.st.players[p];
        let zone = if pl.active == s { 3 } else { 5 + 2 * pl.bench.position(&s)? as u32 };
        return key(p as u8, zone, pl.slots[s as usize].tools.as_slice());
    }
    let r = g.st.locate(c)?;
    let pl = |p: u8| &g.st.players[p as usize];
    match r {
        ListRef::Stadium(p) => key(p, 0, pl(p).stadium.as_slice()),
        ListRef::Supporter(p) => key(p, 1, pl(p).supporter.as_slice()),
        ListRef::Slot(p, s) => {
            let zone = if pl(p).active == s { 2 } else { 4 + 2 * pl(p).bench.position(&s)? as u32 };
            key(p, zone, pl(p).slots[s as usize].cards.as_slice())
        }
        ListRef::Prize(p, i) => key(p, 20 + i as u32, pl(p).prizes[i as usize].as_slice()),
        ListRef::Hand(p) => key(p, 26, pl(p).hand.as_slice()),
        ListRef::Deck(p) => key(p, 27, pl(p).deck.as_slice()),
        ListRef::Discard(p) => key(p, 28, pl(p).discard.as_slice()),
        ListRef::LostZone(_) | ListRef::SlotEnergies(..) | ListRef::Temp(_) => None,
    }
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
                        // (The Prize adjustments are read by the KnockOut routine, `knockout::prize_adjustments`.)
                        matches!(ps.modifier, Modifier::BlockUse(_) | Modifier::Prevent(_) | Modifier::Permit(_) | Modifier::Reflip(_) | Modifier::PrizeAdjust(_) | Modifier::PrizeAdjustOnce(_)),
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
}
