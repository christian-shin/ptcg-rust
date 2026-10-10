//! Annihilape (SSP): Tantrum — 130; this Pokémon is now Confused. Destined
//! Fight — both Active Pokémon are Knocked Out.
//!
//! Tantrum's Confusion is a Special Condition event on this Pokémon with the attack as its cause. Destined Fight is two
//! `Op::KnockOut`s, one KnockOut event per Active Pokémon by an effect: each asks the preventions when the effect runs
//! (Mist Energy stops the Defending Pokémon's Knock Out and Annihilape is still Knocked Out: id2427, JP FAQ Annihilape
//! SSP 100), and the ones not prevented wait for the state check with every other Knock Out (D1), the player whose turn
//! is next taking their Prizes first (id2239).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Annihilape@SSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Confused]), gate: Gate::None, when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::KnockOut(KnockOutSpec { target: MY_ACTIVE, when: Cond::True })),
                Step::after_damage(Op::KnockOut(KnockOutSpec { target: OPP_ACTIVE, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
