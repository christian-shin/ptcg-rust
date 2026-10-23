//! Ariados (POR 2): Poison Ring - 50; your opponent's Active Pokémon is now Poisoned. During your opponent's next turn, that
//! Pokémon can't retreat.
//!
//! Poisoned, then the lasting retreat lock on the Defending Pokémon (Gliscor PFL 50, Dusknoir's Shadow Bind).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ariados",
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
