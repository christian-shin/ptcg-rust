//! Lilligant (JTG): Sunny Day — attacks used by your [G] and [R] Pokémon do
//! 20 more damage to your opponent's Active Pokémon. Spinning Attack — 60.
//!
//! Twinleaf: reacts to every DealDamageEffect. The lock probe runs for the
//! attacking player first (even when this Lilligant is on the other side),
//! then a CheckPokemonTypeEffect on the attacker's Active is always reduced;
//! +20 when that Active is [G]/[R], the target is the opponent's Active and
//! this Lilligant is in play on the attacker's side.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Lilligant@JTG",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 20,
            attacker: SlotPred::OneOf(&[SlotPred::TypeIs(ct::GRASS), SlotPred::TypeIs(ct::FIRE)]),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
