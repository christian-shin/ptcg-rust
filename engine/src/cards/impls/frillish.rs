//! Frillish (WHT / SV11W): Oceanic Gloom - 20; during your opponent's next
//! turn, they can't play any Item cards from their hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Frillish",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(&LockDecl::of(&[LockedAction::PlayItem])) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
