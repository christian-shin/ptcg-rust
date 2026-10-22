//! Team Rocket's Exeggcute (ASC 77): Focused Wish - 10+; flip a coin. If heads, this attack does 20 more damage.
//!
//! The flip comes before the damage (Crustle BLK 52).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsExeggcute",
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
