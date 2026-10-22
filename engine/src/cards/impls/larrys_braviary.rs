//! Larry's Braviary (ASC 174): Clutch - 50; during your opponent's next turn, the Defending Pokémon can't retreat. Brave Bird - 120; this Pokémon also does 30 damage to itself.
//!
//! The lasting retreat lock on the Defending Pokémon (Hop's Sandaconda JTG 87) and the self damage (Klang SCR 100).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LarrysBraviary",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) })),
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
