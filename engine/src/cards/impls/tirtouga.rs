//! Tirtouga (SV11B): Ancient Debris — 30x for each Item card in your
//! opponent's discard pile (`effect.damage = items * 30`). Surf — 80.
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
