//! Litten (TEF): Fake Out — 10; flip a coin, if heads the opponent's Active
//! Pokémon is now Paralyzed.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Litten",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
