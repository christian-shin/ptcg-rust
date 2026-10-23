//! Zweilous (WHT 66): Double Hit - 40×; flip 2 coins. This attack does 40 damage for each heads. Pitch-Black Fangs - 100.
//!
//! `Flips::Count(2)` + `PerHeads::DamageIs(40)` before the damage (Cottonee PRE 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ZweilousWHT",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(2)), per_heads: PerHeads::DamageIs(40), ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
