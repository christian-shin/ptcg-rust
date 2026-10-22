//! Flaaffy (CRI 28): Disconnect - 40; during your opponent's next turn, they can't play any Item cards from their hand.
//!
//! A lasting lock on the opponent over PlayTrainer of an Item from the hand (Budew).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Flaaffy",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_opponent(&LockDecl::on(PLAY_ITEM_FROM_HAND, "BLOCKED_BY_EFFECT"))) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
