//! Lombre (JTG 36): Aqua Slash - 70; during your next turn, this Pokémon can't attack.
//!
//! The lasting attack lock on this Pokémon until the end of the owner's next turn (Floatzel DRI 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LombreJTG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
