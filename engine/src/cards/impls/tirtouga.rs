//! Tirtouga (BLK 22): Ancient Seaweed — 30× for each Item card in your opponent's discard pile. Surf — 80.
//!
//! The damage is 30 times the Item cards in the opponent's discard pile, set before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tirtouga",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Opp, Zone::Discard), Pred::Item), &Num::Lit(30)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
