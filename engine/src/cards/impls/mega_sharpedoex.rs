//! Mega Sharpedo ex (M2 / PFL 61): Greedy Fang — 70, draw 2 cards. Hungry
//! Jaws — 120+, 150 more if this Pokémon has any damage counters on it.
//!
//! Twinleaf fixed in phase 4b: Greedy Fang drew only 1 card (DRAW_CARDS 2
//! now: up to 2, MOVE_CARDS count without sourceCard), and the Hungry Jaws
//! branch tested attack index 0 after a block that always returned, so it
//! never applied; it is now on attack 1.
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
