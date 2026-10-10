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
/// effects carry them): enter or leave play, attach and move attached cards (ENTER_PLAY, ATTACH,
/// MOVE_ENERGY, MOVE_TOOL, KNOCK_OUT), every card move (the batch 7 events DISCARD, PUT_INTO_HAND,
/// PUT_INTO_DECK, DRAW: today's timing, batch 8 narrows it), evolve, devolve and swap (EVOLVE, DEVOLVE, SWAP), Active changes
/// (CHANGE_ACTIVE), the Stadium and Tool plays (`play_trainer` settles after them), the turn (BEGIN_TURN,
/// END_TURN), the state check where Ability locks are re-stamped (CHECK_TABLE_STATE; `lock_sync`
/// runs after the same kinds), the events batch 4 events a continuous effect can read: a Special Condition
/// gained or removed (Gutsy Swing's cost reads it) and healing (remaining HP), and the events batch 6 ones: damage
/// and damage counters placed or moved (remaining HP), a Pokémon leaving play, an effect put on a Pokémon or a
/// player (ApplyEffect). A CoinFlip changes no fact.
pub const INVALIDATING_KINDS: KindMask = mask(&[
    k::DISCARD,
    k::PUT_INTO_HAND,
    k::PUT_INTO_DECK,
    k::DRAW,
    k::ENTER_PLAY,
    k::ATTACH,
    k::MOVE_ENERGY,
    k::MOVE_TOOL,
    k::KNOCK_OUT,
    k::EVOLVE,
    k::DEVOLVE,
    k::SWAP,
    k::CHANGE_ACTIVE,
    k::BEGIN_TURN,
    k::END_TURN,
    k::CHECK_TABLE_STATE,
    k::GAIN_CONDITION,
    k::REMOVE_CONDITION,
    k::HEAL,
    k::DAMAGE,
    k::PLACE_COUNTERS,
    k::MOVE_COUNTERS_EVENT,
    k::LEAVE_PLAY,
    k::APPLY_EFFECT,
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

/// Is the event prevented by a `Prevent` declaration in play (`passive::event_prevented`) or by one an attack left on
/// the Pokémon (`passive::lasting_prevented`: "during your opponent's next turn, prevent all ... done to this Pokémon")? The event
/// routines (`engine::condition`, `engine::change_active`) ask it after the locks.
#[inline]
pub fn event_prevented(g: &mut Game, v: &crate::spec::event::EventView) -> R<bool> {
    g.derived.fresh();
    if crate::spec::passive::lasting_prevented(g, v)? {
        return Ok(true);
    }
    crate::spec::passive::event_prevented(g, v)
}

