//! Grubbin (PBL 2): String Shot - 10; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed.
//!
//! Coin after the damage; heads inflicts Paralyzed (Deino WHT 65).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Grubbin",
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
