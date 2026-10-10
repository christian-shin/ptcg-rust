//! Gurdurr (TWM 104): Knuckle Punch — 20. Superpower — 50; you may do 30
//! more damage. If you do, this Pokémon also does 30 damage to itself.
//!
//! A may-choice before the damage; on yes the main damage is 30 more and the attack does 30 damage to the attacker
//! itself (`self_damage`: one Damage event, the attack as its cause; Weakness, Resistance and the effects on it apply,
//! APR B-09).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gurdurr",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(30), when: Cond::True })), Step::new(self_damage(30))], no: &[] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
