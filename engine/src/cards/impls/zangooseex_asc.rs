//! Zangoose ex (ASC 167): Spike Draw - 20; draw 2 cards. Wild Scissors - 180; this Pokémon also does 30 damage to itself.
//!
//! Spike Draw is one Draw event of 2 (cause: this attack) after the damage; Wild Scissors' self damage is one Damage event
//! on the attacker (Arrokuda DRI 62; APR B-09).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ZangooseEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
