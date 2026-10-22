//! Spinarak (POR 1): Gooey Thread - 10; during your opponent's next turn, the Defending Pokémon can't retreat.
//!
//! A lasting retreat lock on the Defending Pokémon (Bulbasaur M1L 1's Bind Down).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpinarakPOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
