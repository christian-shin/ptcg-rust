//! Pawmot (PFL 34): Voltaic Fist — 130; you may have this Pokémon also do 60
//! damage to itself and make your opponent's Active Pokémon Paralyzed.
//!
//! Rule: a may-choice (WANT_TO_USE_ABILITY); on yes the attacking Pokémon takes
//! the damage, then GainCondition(Paralyzed) on the opponent's Active with the
//! attack's cause.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PawmotPFLPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(self_damage(60)), Step::new(inflict(&[SpecialCondition::Paralyzed]))], no: &[] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
