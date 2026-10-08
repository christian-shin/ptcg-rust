//! Zorua (SFA 31): Stampede — 10. Double Scratch — flip 2 coins, 20 damage
//! for each heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zorua@SFA",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { mode: CoinMode::Fixed(2), then: &[Step::new(damage_is(Num::Mul(&Num::Heads, &Num::Lit(20))))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
