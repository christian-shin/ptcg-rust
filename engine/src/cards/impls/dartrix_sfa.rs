//! Dartrix (SFA): United Wings — 20× the number of Pokémon in your discard
//! pile with the United Wings attack. Cutting Wind — 30.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dartrix@SFA",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::HasAttackNamed("United Wings")), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
