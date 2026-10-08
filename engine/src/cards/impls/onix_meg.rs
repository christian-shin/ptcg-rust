//! Onix (M1L / MEG 70): Bind — 30; flip a coin, if heads the opponent's
//! Active Pokémon is now Paralyzed. Strength — 100.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Onix@Onix M1L",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed], Cause::Ability))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
