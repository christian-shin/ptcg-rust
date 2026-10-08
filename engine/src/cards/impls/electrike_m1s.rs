//! Electrike (M1S): Thunder Jolt — 30; this Pokémon also does 10 damage to
//! itself (THIS_POKEMON_DOES_DAMAGE_TO_ITSELF).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Electrike@MEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(self_damage(10)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
