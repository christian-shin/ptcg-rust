//! Boltund (PFL 31): Electric Run - 70+; flip a coin. If heads, this attack does 70 more damage.
//!
//! A coin before the damage; heads adds 70 (Crustle BLK 52).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Boltund",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(70, Cond::True))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
