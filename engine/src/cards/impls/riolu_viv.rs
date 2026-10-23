//! Riolu (VIV): Best Punch - 30; flip a coin, if tails this attack does nothing (the damage is set to 0 on tails).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RioluVIV",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { tails: &[Step::new(attack_does_nothing(Cond::True))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
