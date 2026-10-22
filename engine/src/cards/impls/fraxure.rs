//! Fraxure (BLK 69): Bite - 30. Boundless Power - 90; during your next turn, this Pokémon can't use attacks.
//!
//! A lasting `CANT_ATTACK` lock on this Pokémon until the end of my next turn (Bouffalant SCR).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Fraxure",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
