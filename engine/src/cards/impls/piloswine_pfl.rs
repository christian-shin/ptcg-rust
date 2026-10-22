//! Piloswine (PFL 24): Rising Lunge - 30+; flip a coin. If heads, this attack does 30 more damage. Frost Smash - 70.
//!
//! The coin's heads add damage before the damage step (Boltund PFL 31).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PiloswinePFL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(30, Cond::True))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
