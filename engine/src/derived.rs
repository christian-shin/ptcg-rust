//! The derived layer (events design section 6): the facts continuous effects decide, per Pokémon slot
//! and per player, read by execution and legality alike.
//!
//! **Batch 1 (this skeleton):** every read delegates to today's check effect, exactly as its callers
//! ran it before (`hp` runs `CheckHp`, `provided_energy` runs `CheckProvidedEnergy`, ...), so behavior
//! and traces are unchanged. The fact storage, the dirty flag and the lazy [`Derived::rebuild`] are in
//! place; nothing is stored yet.
//!
//! **Batch 8:** `rebuild` computes the facts from the declarations in force and the reads answer from
//! the storage (no effect dispatch); the `Check*` effects go away.
//!
//! **Invalidation.** The facts change only after events that can change a continuous effect or its
//! inputs: a card entering or leaving play, evolving or devolving, an attach (Energy, Tool), a Stadium
//! change, an Ability lock change, the turn changing ([`INVALIDATING_KINDS`], checked after each
//! dispatch in `reduce_effect`). A game copy starts stale.

use crate::effects::{k, mask, Cost, Effect, EnergyMap, KindMask, ResistanceV, SlotRef, WeaknessV};
use crate::game::{Game, R};
use crate::list::*;
use crate::state::{AttackRef, SlotId, MAX_CARDS, MAX_SLOTS};
use crate::types::CardType;

/// The facts of one Pokémon slot (`None`: not derived yet).
#[derive(Clone, Copy, Debug, Default)]
pub struct SlotFacts {
    /// Remaining-HP base: the Pokémon's HP with every modifier (`CheckHp`).
    pub hp: Option<i32>,
    /// Its types (`CheckPokemonType`).
    pub types: Option<SVec<CardType, 4>>,
    /// Its Weakness and Resistance (`CheckPokemonStats`).
    pub weakness: Option<SVec<WeaknessV, 4>>,
    pub resistance: Option<SVec<ResistanceV, 4>>,
}

/// The facts of one player.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerFacts {
    /// The Active Pokémon's Retreat Cost (`CheckRetreatCost`).
    pub retreat_cost: Option<Cost>,
    /// The attacks the Active Pokémon may use, and those copied from the Bench (`CheckPokemonAttacks`).
    pub attacks: Option<crate::engine::turn::CheckedAttacks>,
}

/// Per game: the derived facts and whether they are stale.
#[derive(Clone, Copy, Debug)]
pub struct Derived {
    dirty: bool,
    pub slots: [[SlotFacts; MAX_SLOTS]; 2],
    pub players: [PlayerFacts; 2],
    /// Per card id: the Energy types an attached card provides, one entry per unit
    /// (`CheckProvidedEnergy`; APR C-01).
    pub provides: [Option<SVec<CardType, 4>>; MAX_CARDS],
}

/// The effect kinds after which the facts may have changed (the events of the design's list as today's
/// effects carry them): enter or leave play and attach (MOVE_CARDS, ENTER_PLAY, ATTACH_ENERGY,
/// ATTACH_POKEMON_TOOL, DISCARD_CARDS, KNOCK_OUT), evolve, devolve and swap (EVOLVE, DEVOLVE, SWAP), Active changes
/// (MOVED_TO_ACTIVE, MOVED_FROM_ACTIVE_TO_BENCH), the Stadium (PLAY_STADIUM), the turn (BEGIN_TURN,
/// END_TURN), and the state check where Ability locks are re-stamped (CHECK_TABLE_STATE; `lock_sync`
/// runs after the same kinds).
pub const INVALIDATING_KINDS: KindMask = mask(&[
    k::MOVE_CARDS,
    k::ENTER_PLAY,
    k::ATTACH_ENERGY,
    k::ATTACH_POKEMON_TOOL,
    k::DISCARD_CARDS,
    k::KNOCK_OUT,
    k::EVOLVE,
    k::DEVOLVE,
    k::SWAP,
    k::MOVED_TO_ACTIVE,
    k::MOVED_FROM_ACTIVE_TO_BENCH,
    k::PLAY_STADIUM,
    k::BEGIN_TURN,
    k::END_TURN,
    k::CHECK_TABLE_STATE,
]);

impl Derived {
    pub fn new() -> Derived {
        Derived { dirty: true, slots: [[SlotFacts::default(); MAX_SLOTS]; 2], players: [PlayerFacts::default(); 2], provides: [None; MAX_CARDS] }
    }

    /// The facts may have changed: the next read rebuilds them.
    #[inline]
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Recompute the facts. Batch 1: the reads delegate to the check effects and nothing is stored (the
    /// facts stay `None`), so there is nothing to recompute.
    pub fn rebuild(&mut self) {
        self.dirty = false;
    }

    /// Rebuild when stale (every read starts here).
    #[inline]
    fn fresh(&mut self) {
        if self.dirty {
            self.rebuild();
        }
    }
}

impl Default for Derived {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// The reads. Batch 1: each runs today's check effect, as its callers did.

/// The HP of the Pokémon in slot `s` of player `p` (`CheckHp`).
#[inline]
pub fn hp(g: &mut Game, p: usize, s: SlotId) -> R<i32> {
    g.derived.fresh();
    crate::engine::check::check_hp(g, p, s)
}

/// The Energy the Pokémon in `source` provides, per attached card (`CheckProvidedEnergy`, asked by `p`).
#[inline]
pub fn provided_energy(g: &mut Game, p: usize, source: SlotRef) -> R<EnergyMap> {
    g.derived.fresh();
    crate::engine::attack::provided_energy_read(g, p, source)
}

/// The types of the Pokémon in `target` (`CheckPokemonType` from its printed types).
pub fn types(g: &mut Game, target: SlotRef) -> R<SVec<CardType, 4>> {
    g.derived.fresh();
    let seed = crate::engine::game_effect::pokemon_types(g, target);
    let (e, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: seed })?;
    Ok(match e {
        Effect::CheckPokemonType { card_types, .. } => card_types,
        _ => SVec::new(),
    })
}

/// The Weakness and Resistance of the Pokémon in `target` (`CheckPokemonStats`).
pub fn weakness_resistance(g: &mut Game, target: SlotRef) -> R<(SVec<WeaknessV, 4>, SVec<ResistanceV, 4>)> {
    g.derived.fresh();
    let (e, _) = g.run_fx(crate::engine::game_effect::stats_effect(g, target))?;
    Ok(match e {
        Effect::CheckPokemonStats { weakness, resistance, .. } => (weakness, resistance),
        _ => (SVec::new(), SVec::new()),
    })
}

/// Player `p`'s Active Pokémon's Retreat Cost (`CheckRetreatCost`).
#[inline]
pub fn retreat_cost(g: &mut Game, p: usize) -> R<Cost> {
    g.derived.fresh();
    crate::engine::retreat::retreat_cost_read(g, p)
}

/// The attacks player `p`'s Active Pokémon may use (`CheckPokemonAttacks`).
#[inline]
pub fn attacks(g: &mut Game, p: usize) -> R<crate::engine::turn::CheckedAttacks> {
    g.derived.fresh();
    crate::engine::turn::read_attack_list(g, p)
}

/// The current cost of `attack` (`CheckAttackCost`).
#[inline]
pub fn attack_cost(g: &mut Game, p: usize, attack: AttackRef) -> R<Cost> {
    g.derived.fresh();
    crate::engine::attack::attack_cost_read(g, p, attack)
}

/// Does `card` (player `p`'s) have no Ability (power `power_index`, or any) now? (The lock pass with
/// take-hold stamps; `prefabs::is_ability_blocked`.)
#[inline]
pub fn has_no_ability(g: &mut Game, p: usize, card: crate::list::CardId, power_index: Option<u8>) -> bool {
    g.derived.fresh();
    crate::prefabs::is_ability_blocked(g, p, card, power_index)
}

/// The lock that forbids a Pokémon event (EnterPlay, Evolve, Devolve, Swap), if any (`passive::event_locked`):
/// the one query the event's routine asks before the event and legality asks for the event a play would
/// produce.
#[inline]
pub fn event_locked(g: &mut Game, v: &crate::spec::event::EventView) -> R<Option<&'static str>> {
    g.derived.fresh();
    crate::spec::passive::event_locked(g, v)
}

/// The in-play or lasting lock that forbids player `p` doing one of `actions` with `card`, if any
/// (`passive::play_locked_as`).
#[inline]
pub fn play_locked(g: &mut Game, p: usize, card: crate::list::CardId, actions: &[crate::spec::passive::LockedAction]) -> Option<&'static str> {
    g.derived.fresh();
    crate::spec::passive::play_locked_as(g, p, card, actions)
}
