//! Eiscue (MEG 44): Freezing Headbutt - 20; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed.
//! Tackle - 90.
//!
//! The coin is flipped after the damage (Litten TEF 32).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Eiscue",
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
