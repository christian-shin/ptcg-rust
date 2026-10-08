//! Cottonee (MEP 18): Collect — draw a card. Ported so Whimsicott ex
//! (SV11W) can be evolved in pool games.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CottoneeMEPPool",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
