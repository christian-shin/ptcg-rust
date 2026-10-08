//! Beldum (TEF): Dig Claws — 10. Iron Tackle — 50; this Pokémon also does
//! 10 damage to itself (THIS_POKEMON_DOES_DAMAGE_TO_ITSELF: a DealDamageEffect
//! aimed at `effect.source`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Beldum@TEF",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(10)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
