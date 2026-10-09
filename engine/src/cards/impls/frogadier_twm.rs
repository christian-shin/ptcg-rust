//! Frogadier (TWM): Numbing Water — flip a coin, if heads the opponent's
//! Active Pokémon is now Paralyzed (a CoinFlip, then on heads a
//! GainCondition(Paralyzed) with the attack's cause).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Frogadier@TWM",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
