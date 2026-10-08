//! Varoom (SFA): Rigidify — during your opponent's next turn this Pokémon
//! takes 30 less damage (`damageReductionNextTurn = 30` on the Active).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Varoom",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
