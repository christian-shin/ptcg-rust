//! Lileep (SCR 5): Bind Down - 50; during your opponent's next turn, the Defending Pokémon can't retreat.
//!
//! The lasting retreat lock on the Defending Pokémon (Hop's Sandaconda JTG 87).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Lileep",
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
