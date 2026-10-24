//! Garganacl (MEG 84): Powerful a-Salt - attacks used by your [F] Pokémon do 30 more damage to your opponent's Active Pokémon (before
//! applying Weakness and Resistance). Hammer In - 130.
//!
//! A `DamageDealt` of 30 for the owner's attacks whose Pokémon is [F] as the game checks it (APR D-07), on the opponent's Active
//! Pokémon, applied before Weakness and Resistance; each Garganacl with a working Ability adds 30 (the text doesn't say "doesn't
//! stack"). It needs damage to add to, like every "does N more damage" Ability.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Garganacl",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, attacker: SlotPred::TypeIs(ct::FIGHTING), needs_damage: true, ..DamageDealtSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
