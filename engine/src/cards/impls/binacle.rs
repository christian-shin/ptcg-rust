//! Binacle (POR 42): Double Draw - draw 2 cards. Scratch - 30.
//!
//! Double Draw is one Draw event of 2 cards (cause: this attack), after the damage. A short deck draws what is left: an
//! attack stays usable when its effect can't happen in full (APR D-01).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Binacle",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
