//! Seismitoad (BLK 21): Round — 70×; this attack does 70 damage for each of your Pokémon in play that has the Round attack. Hyper
//! Voice — 160.
//!
//! The damage is 70 times the Pokémon of yours in play (Active and Bench) whose top card prints an attack named Round
//! (APR B-02), Seismitoad itself included.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SeismitoadBLK",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Mul(&Num::InPlayCount(Who::Me, PlayScope::All, Pred::HasAttackNamed("Round")), &Num::Lit(70))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
