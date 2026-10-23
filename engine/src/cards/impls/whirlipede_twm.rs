//! Whirlipede (TWM 116): Poison Ring - your opponent's Active Pokémon is now Poisoned. During your opponent's next turn,
//! the Defending Pokémon can't retreat. Spinning Attack - 60.
//!
//! Poisoned, then the lasting retreat lock on the Defending Pokémon (Gliscor PFL 50's Poison Ring).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WhirlipedeTWM",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(inflict(&[SpecialCondition::Poisoned])),
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
