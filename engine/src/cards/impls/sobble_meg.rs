//! Sobble (MEG 39): Surprise Attack - 30; flip a coin. If tails, this attack does nothing.
//!
//! The coin is flipped before the damage; tails sets the damage to 0 (Timburr TWM 103).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SobbleMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { tails: &[Step::new(damage_is(Num::Lit(0)))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
