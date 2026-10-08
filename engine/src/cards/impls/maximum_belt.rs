//! Maximum Belt (TEF, ACE SPEC tool): the holder's attacks do 50 more damage
//! to the opponent's Active Pokémon ex.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MaximumBelt",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 50,
            attacker: SlotPred::Holder,
            target: SlotPred::Tag(crate::types::tag::POKEMON_EX_LOWER),
            needs_damage: true,
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
