//! Croconaw (TEF): Reverse Thrust — 30; switch this Pokémon with 1 of your
//! Benched Pokémon.
//!
//! Rule: the switch is part of the attack's effect, after its damage, and
//! happens whether or not damage was dealt (prevented, or 0 after
//! Resistance). It is a ChangeActive (Switch, APR C-03) by the attack, done to
//! this Pokémon: it goes to the Bench and loses its Special Conditions and the
//! effects of attacks on it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Croconaw",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(switch_self()),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
