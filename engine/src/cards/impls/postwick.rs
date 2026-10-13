//! Postwick (JTG 154, Stadium): attacks used by Hop's Pokémon (both yours and your opponent's) do 30 more damage to the
//! opponent's Active Pokémon (before applying Weakness and Resistance).
//!
//! A `DamageDealt` declaration over Hop's attackers (the `HOPS` tag, either player), added only to damage that is above 0: an
//! attack that does nothing or no damage gets no bonus (APR B-01, C-16; as Hop's Choice Band, same wording).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Postwick",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, attacker: SlotPred::Tag(tag::HOPS), side: Side::Any, needs_damage: true, ..DamageDealtSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
