//! Braviary (BLK 78): Aerial Ace - 40+; flip a coin. If heads, this attack does 40 more damage. Speed Wing - 130.
//!
//! A coin before the damage; heads adds 40 (Crustle BLK 52).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Braviary",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(40, Cond::True))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
