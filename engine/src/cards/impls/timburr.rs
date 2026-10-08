//! Timburr (TWM 103): Best Punch — 40; flip a coin, if tails this attack
//! does nothing (`effect.damage = 0`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Timburr",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { tails: &[Step::new(damage_is(Num::Lit(0)))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
