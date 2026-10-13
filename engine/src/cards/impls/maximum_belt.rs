//! Maximum Belt (TEF, ACE SPEC tool): the holder's attacks do 50 more damage
//! to the opponent's Active Pokémon ex.
//!
//! A Tool `DamageDealt` +50 in the attacker-side pass of the damage calculation: the attacker is the Pokémon holding it, the target the
//! opponent's Active Pokémon ex, and only when the attack does damage (id675). It works while the Tool does, and it adds once to the
//! total (JP FAQ Maximum Belt).
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
