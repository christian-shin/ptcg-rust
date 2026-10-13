//! N's Reshiram (ASC 154): Powerful Rage - 20 damage for each damage counter on this Pokémon. Virtuous Flame - 170.
//!
//! Powerful Rage sets the main damage to 20 per damage counter on this Pokémon (twice the damage in tens of HP, read before the
//! attack's damage).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsReshiram",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::DamageOn(MY_ACTIVE), &Num::Lit(2)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
