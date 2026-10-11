//! Mega Sharpedo ex (M2 / PFL 61): Greedy Fang - 70; draw 2 cards. Hungry Jaws - 120+; 150 more damage if this Pokémon has any
//! damage counters on it.
//!
//! Greedy Fang is one Draw of up to 2 cards after the damage (a Draw event carrying the cards drawn, D3).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaSharpedoex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(150), when: Cond::Slot(MY_ACTIVE, SlotPred::Damaged) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
