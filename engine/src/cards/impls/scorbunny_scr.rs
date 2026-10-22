//! Scorbunny (SCR 26): Quick Attack - 10+; flip a coin. If heads, this attack does 10 more damage.
//!
//! The flip comes before the damage (Crustle BLK 52).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScorbunnySCR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(10, Cond::True))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
