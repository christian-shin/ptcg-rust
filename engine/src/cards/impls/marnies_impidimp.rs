//! Marnie's Impidimp (DRI): Filch — draw a card (one Draw event of the attack's effect, after the damage). Corkscrew Punch — 10.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesImpidimp",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
