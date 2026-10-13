//! Iono's Wattrel (JTG / ASC): Quick Attack - 10+; flip a coin, if heads this attack does 20 more damage.
//!
//! One coin before the damage; heads adds 20 to the attack's damage.
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
