//! Budew (PRE / ASC): Itchy Pollen — during your opponent's next turn, they can't play any Item cards from their hand.
//!
//! A lasting lock on the opponent over PlayTrainer (an Item played from the hand), armed after the damage; it never binds
//! an Item whose effect is used by another card (id2226).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Budew",
    // Itchy Pollen: during your opponent's next turn, they can't play any Item cards from their hand.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_opponent(&LockDecl::on(PLAY_ITEM_FROM_HAND, "BLOCKED_BY_EFFECT"))) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
