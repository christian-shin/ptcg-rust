//! Iron Boulder (SCR): Adjusted Horn — 170; if you don't have the same number
//! of cards in your hand as your opponent, this attack does nothing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "IronBoulder",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Lit(0), when: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Ne, Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand))) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
