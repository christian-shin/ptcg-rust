//! Dondozo ex (DRI 66): Avenging Billow - 30+; this attack does 10 more damage for each damage counter on this Pokémon.
//! Dynamic Dive - 120+; you may do 120 more damage. If you do, this Pokémon also does 50 damage to itself.
//!
//! Avenging Billow adds the damage on this Pokémon before the attack's damage (Duraludon PRE 69). Dynamic Dive is a may-choice
//! before the damage; on yes the main damage is 120 more and the attack does 50 damage to the attacker itself (Gurdurr TWM 104's
//! Superpower).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DondozoEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageOn(MY_ACTIVE), when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::May(MaySpec { asker: Who::Me, when: Cond::True, yes: &[Step::new(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(120), when: Cond::True })), Step::new(self_damage(50))], no: &[] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
