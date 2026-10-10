//! Zekrom ex (SV11B / BLK 34): Slash — 50. Voltage Burst — 130+; 50 more
//! for each Prize card your opponent has taken; this Pokémon does 30 damage
//! to itself.
//!
//! The bonus is 50 times 6 minus the opponent's Prizes left. The recoil is one Damage event on the attacker with the
//! attack as its cause, after the main damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zekromex",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Sub(&Num::Lit(6), &Num::PrizesLeft(Who::Opp)), &Num::Lit(50)), when: Cond::True })),
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
