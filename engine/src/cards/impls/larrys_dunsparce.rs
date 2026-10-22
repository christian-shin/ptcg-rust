//! Larry's Dunsparce (ASC 163): Rising Lunge - 10+; flip a coin. If heads, this attack does 20 more damage.
//!
//! The coin is flipped before the damage; heads adds 20 (Crustle BLK 52).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LarrysDunsparce",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(20, Cond::True))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
