//! Misty's Staryu (DRI 46): Bubble Beam - 20; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed.
//!
//! The coin is flipped after the damage (Deino WHT 65's Body Slam).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MistysStaryu",
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
