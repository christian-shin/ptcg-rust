//! Magmar (JTG 20): Searing Flame - 30; flip a coin. If heads, your opponent's Active Pokémon is now Burned.
//!
//! Coin after the damage; heads inflicts Burned (Deino WHT 65).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Magmar",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Burned]))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
