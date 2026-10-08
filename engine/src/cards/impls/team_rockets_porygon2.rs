//! Team Rocket's Porygon2 (DRI): R Command — 20 damage for each Supporter
//! with "Team Rocket" in its name in your discard pile.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsPorygon2",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Supporter, Pred::NameContains("Team Rocket")])), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
