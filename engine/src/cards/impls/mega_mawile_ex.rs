//! Mega Mawile ex (M1L / MEG 94): Gobble Down - 80x for each Prize card you have taken. Huge Bite - 260; if the opponent's Active
//! Pokémon already has damage counters on it, this attack does 30 damage instead.
//!
//! Gobble Down sets the damage to 80 times the Prize cards taken (6 minus the Prize cards left). Huge Bite sets the base damage
//! to 30 when the Defending Pokémon already has damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaMawileEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::Sub(&Num::Lit(6), &Num::PrizesLeft(Who::Me)), &Num::Lit(80)), when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Lit(30), when: Cond::Slot(OPP_ACTIVE, SlotPred::Damaged) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
