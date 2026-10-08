//! Eevee (SSP): Boosted Evolution — as long as this Pokémon is in the Active
//! Spot, it can evolve during your first turn or the turn you play it.
//! Reckless Charge — 30; this Pokémon also does 10 damage to itself.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SSP.
//! Twinleaf quirks kept: every copy (any zone) adds its
//! EVOLUTIONARY_ADVANTAGE_MARKER to the player of any PlayPokemonEffect and
//! removes it at that player's end of turn (the marker does nothing else).
//!
//! Fixed in phase 4b (R4): Boosted Evolution answers the CheckPokemonPlayedTurnEffect of
//! this Eevee's own slot: when it is its owner's Active Pokémon, still this
//! card (not evolved) and a stub-Ability probe passes, the effect gets
//! `pokemonPlayedTurn = turn - 1` and `canEvolveOnFirstTurn = true` (the
//! PlayPokemonEffect first-turn test honours it). It used to write
//! `player.canEvolve = true` on every CheckTableStateEffect while the active
//! player's Active `cards[0]` was this card (even after it evolved), letting
//! every Pokémon of that player evolve on the first turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Eevee@SSP|PRE",
    // Boosted Evolution: as long as this Pokémon is in the Active Spot, it can evolve during your
    // first turn or the turn you play it.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::AllowEvolve(AllowEvolveSpec { subject: SlotPred::All(&[SlotPred::IsActive, SlotPred::IsThisPokemon]) }),
    }],
    // Reckless Charge: this Pokémon also does 10 damage to itself.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(self_damage(10))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
