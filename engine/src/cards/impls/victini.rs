//! Victini (SSP): Victory Cheer — attacks used by your Evolution [R] Pokémon
//! do 10 more damage to your opponent's Active Pokémon. Flare — 30.
//!
//! Twinleaf: on every DealDamageEffect (either player's): bail out if the
//! ability is blocked for the effect's player; count Victini copies among
//! that player's Pokémon; a CheckPokemonTypeEffect on THAT player's Active
//! must contain [R] (not the attacker's slot); the target must be the
//! opponent's Active; the attacking Pokémon's `evolvesFrom !== ''` (an
//! empty source slot counts as an Evolution); then damage += 10 * count.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Victini",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 10,
            attacker: SlotPred::All(&[SlotPred::TypeIs(ct::FIRE), SlotPred::Evolution]),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
