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
    pub const N: usize = 8;
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
        _ => return None,
    })
}

/// Per game: the cards of each family (a bit per card id).
#[derive(Clone, Copy, Debug)]
pub struct DeclIndex {
    pub sets: [u128; fam::N],
}

impl DeclIndex {
    pub const fn new() -> DeclIndex {
        DeclIndex { sets: [0; fam::N] }
    }

    /// The index of the game's cards (`Game::start`, after the decks are in).
    pub fn build(g: &Game) -> DeclIndex {
        let mut ix = DeclIndex::new();
        for c in 0..g.st.n_cards {
            for ps in passives(g, c) {
                if let Some(f) = modifier_family(&ps.modifier) {
                    ix.sets[f] |= 1u128 << c;
                }
            }
        }
        ix
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
