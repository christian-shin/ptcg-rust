//! Postwick (JTG): the attacks of Hop's Pokémon (both players') do 30 more
//! damage to the opponent's Active Pokémon (before W/R).
//!
//! Twinleaf: DealDamageEffect only; block probe on the target's owner; the
//! target must be the attacker-opponent's Active; the source slot's Pokémon
//! needs the HOPS tag. Not limited to attacks with printed damage.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Postwick",
    passives: &[
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, attacker: SlotPred::Tag(tag::HOPS), side: Side::Any, ..DamageDealtSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
