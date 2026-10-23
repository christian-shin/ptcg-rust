//! Team Rocket's Nidoran♀ (DRI 114): Surprise Attack - 30; flip a coin. If tails, this attack does nothing.
//!
//! The flip comes before the damage; tails sets the damage to 0 (Charcadet PBL 11).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsNidoranF",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { tails: &[Step::new(attack_does_nothing(Cond::True))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
