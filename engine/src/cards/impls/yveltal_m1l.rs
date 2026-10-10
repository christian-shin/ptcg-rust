//! Yveltal (M1L / MEG 88): Clutch — 20, the Defending Pokémon can't retreat
//! during your opponent's next turn. Dark Feather — 110.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yveltal@MEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
