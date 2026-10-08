//! Budew (PRE): Itchy Pollen — during your opponent's next turn, they can't
//! play any Item cards from their hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Budew",
    // Itchy Pollen: during your opponent's next turn, they can't play any Item cards from their hand.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(Locked::Item) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
