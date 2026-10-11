//! Ethan's Pichu (DRI): Zapping Draw — 30; draw a card (one Draw event of the attack's effect, after the damage).
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
