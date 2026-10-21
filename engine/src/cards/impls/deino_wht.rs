//! Deino (WHT 65): Body Slam - 20; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Darkness Fang - 50.
//!
//! The coin is flipped after the damage (Litten TEF 32).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DeinoWHT",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
