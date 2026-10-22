//! Servine (BLK 2): Wrap - 20; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Vine Whip - 60.
//!
//! `Coin { heads: Paralyzed }` after the damage (Litten TEF 32).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ServineBLK",
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
