//! Skarmory (DRI 141): Roost - heal 50 damage from this Pokémon. During your next turn, this Pokémon can't retreat. Metal Claw - 60.
//!
//! A heal on this Pokémon, then a lasting retreat lock whose target is the attacker (Slakoth SSP's Take It Easy).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SkarmoryDRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(50)),
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_RETREAT, LockUntil::YourNextTurn)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
