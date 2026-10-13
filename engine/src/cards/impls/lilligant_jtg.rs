//! Lilligant (JTG): Sunny Day - attacks used by your [G] and [R] Pokémon do 20 more damage to your opponent's Active Pokémon.
//! Spinning Attack - 60.
//!
//! `DamageDealt` +20 in the attacker-side pass of the damage calculation: the attacker is a [G] or [R] Pokémon (type as the game
//! checks it, `SlotPred::TypeIs`) of yours, the target the opponent's Active Pokémon. An attack that does no damage gets no bonus
//! (id675: "you still have to do some damage in order to have 10 more done"; the damage calculation isn't run for it; scenario
//! lilligant-jtg-7-sunny-day-adds-nothing-to-an-attack-without-damage). It works from the Bench while the Ability does; copies
//! stack.
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
