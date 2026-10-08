//! Ethan's Pichu (DRI): Sparking Draw — 30; draw a card (AFTER_ATTACK →
//! DRAW_CARDS(player, 1)).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansPichu",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
