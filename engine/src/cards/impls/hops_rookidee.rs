//! Hop's Rookidee (JTG 133): Intimidating Stare - during your opponent's next turn, attacks used by the Defending Pokémon do 20 less damage (before applying Weakness and Resistance). Peck - 20.
//!
//! Arms the lasting `DealsLessDamage(20)` on the Defending Pokémon (Buneary MEG 107).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsRookidee",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DealsLessDamage(20) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
