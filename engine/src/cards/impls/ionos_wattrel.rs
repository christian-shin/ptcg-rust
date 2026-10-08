//! Iono's Wattrel (JTG / ASC): Quick Attack - 10+; flip a coin, if heads this
//! attack does 20 more damage.
//!
//! Twinleaf: COIN_FLIP_PROMPT with `effect.damage += 20` in the callback (the
//! attack effect is retained across the flip).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IonosWattrel",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(20, Cond::True))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
