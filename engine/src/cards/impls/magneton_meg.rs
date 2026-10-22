//! Magneton (MEG 46): Thunder Shock - 30; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed.
//!
//! `Coin { heads: Paralyzed }` after the damage (Litten TEF 32).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MagnetonMEG",
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
